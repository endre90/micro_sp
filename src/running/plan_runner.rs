use crate::{running::process_operation::OperationProcessingType, *};
use crate::SPConnection;
use std::sync::Arc;
// use crate::SPConnection;
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};

// PERF: this constant does double duty as both the tick period *and* the
// assumed time increment in `process_operation`'s elapsed-time accounting, but
// `sop_runner` ticks at 100 ms and `goal_runner`/`time_runner` at 100 ms too.
// Any change here silently changes operation timeout behaviour. Suggested:
// separate "how often do I poll" from "how much time has passed" by measuring
// the latter with `Instant`/`SystemTime`, which also makes the timeouts correct
// when a tick is skipped because Redis was slow.
pub static OPERAION_RUNNER_TICK_INTERVAL_MS: u64 = 200;

// PERF: third caller of `StateManager::get_full_state` - see the note there;
// with `sop_runner` and `auto_operation_runner` this makes ~20 blocking
// `KEYS *` scans per second. The key set for this runner is derivable from
// `model.operations` `get_all_var_keys()` plus the `{sp_id}_plan*` /
// `{sp_id}_terminated_operations` keys (the planner ticker already builds
// almost exactly this list), so it can move to `get_state_for_keys`.
// PERF: `con.clone()` per tick to hand a second handle to `process_plan_tick`
// - pass `&mut con` instead; `SPConnection` is multiplexed, so the
// clone buys nothing.
pub async fn planned_operation_runner(
    model: &Model,
    // logging_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sp_id = &model.name;
    let log_target = format!("{}_op_runner", sp_id);
    let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));

    // Get only the relevant keys from the state
    log::info!(target: &log_target, "Online.");

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    loop {
        interval.tick().await;

        let state = match StateManager::get_full_state(&mut con).await {
            Some(s) => s,
            None => continue,
        };

        let con_clone = con.clone();
        let new_state = process_plan_tick(
            sp_id,
            con_clone,
            &model,
            &state,
            // logging_tx.clone(),
            &log_target,
        )
        .await;
        let modified_state = state.get_diff_partial_state(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;
    }
}

// PERF: the two `remove_sp_values` calls near the end run unconditionally on
// every tick, even when `terminated_operations` is empty - so every idle tick
// still costs two Redis round trips. `remove_sp_values` returns early on an
// empty slice, but only after the RTT has been paid for the non-empty one and
// after the `terminated_operations_meta` vector has been built. Guard the whole
// block with `if !terminated_operations.is_empty()` (as
// `auto_operation_runner` does) and pipeline the two DELs into one.
// PERF: the tail unconditionally rewrites `_plan_state`, `_plan`,
// `_planner_state`, `_current_goal_state`, `_plan_current_step` and
// `_terminated_operations` with the values it just read, so the diff is
// non-empty on most ticks and an MSET goes out even when nothing happened. Only
// write the fields the tick actually changed.
// PERF: `model.operations.iter().find(|op| op_name.starts_with(&op.name))` is a
// linear scan with a prefix comparison per plan step per tick; a
// `HashMap<&str, &Operation>` built once at startup would make it O(1). Note
// also that `starts_with` makes `op_move` match `op_move_to_b` - a correctness
// hazard as well as a slow path.
// PERF: `let mut new_state = state.clone()` plus `state.get_diff_partial_state(
// &new_state)` in the caller means a full map copy and a full map scan per
// tick; see the dirty-key suggestion on `State::get_diff_partial_state`.
async fn process_plan_tick(
    sp_id: &str,
    mut con: SPConnection,
    model: &Model,
    state: &State,
    // logging_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) -> State {
    let mut new_state = state.clone();
    let planner_state =
        state.get_string_or_default_to_unknown(&format!("{}_planner_state", sp_id), &log_target);

    let goal_state = state
        .get_string_or_default_to_unknown(&format!("{}_current_goal_state", sp_id), &log_target);

    let mut plan_state_str =
        state.get_string_or_default_to_unknown(&format!("{}_plan_state", sp_id), &log_target);
    let mut plan_current_step =
        state.get_int_or_default_to_zero(&format!("{}_plan_current_step", sp_id), &log_target);
    let plan_of_sp_values =
        state.get_array_or_default_to_empty(&format!("{}_plan", sp_id), &log_target);

    let plan: Vec<String> = plan_of_sp_values
        .iter()
        .filter(|val| val.is_string())
        .map(|y| y.to_string())
        .collect();

    let terminated_operations_sp_value = state
        .get_array_or_default_to_empty(&format!("{}_terminated_operations", sp_id), &log_target);

    let terminated_operations: Vec<String> = terminated_operations_sp_value
        .iter()
        .filter(|val| val.is_string())
        .map(|y| y.to_string())
        .collect();

    match PlanState::from_str(&plan_state_str) {
        PlanState::Initial => {
            if planner_state == PlannerState::Found.to_string() {
                plan_state_str = PlanState::Executing.to_string();
                plan_current_step = 0;
            }
            if planner_state == PlannerState::NotFound.to_string() {
                plan_state_str = PlanState::Failed.to_string();
                plan_current_step = 0;
            }
        }
        PlanState::Executing => {
            if let Some(op_name) = plan.get(plan_current_step as usize) {
                match model
                    .operations
                    .iter()
                    .find(|op| op_name.starts_with(&op.name))
                {
                    Some(operation) => {
                        let mut uq_operation = operation.clone();
                        uq_operation.name = op_name.to_owned();
                        new_state = running::process_operation::process_operation(
                            &sp_id,
                            new_state,
                            &uq_operation,
                            OperationProcessingType::Planned,
                            Some(&mut plan_current_step),
                            Some(&mut plan_state_str),
                            // logging_tx,
                            log_target,
                        )
                        .await;

                        // let operation_state = new_state.get_string_or_default_to_unknown(
                        //     &format!("{}", uq_operation.name),
                        //     &log_target,
                        // );
                    }
                    None => {
                        log::error!("Operation '{}' not found in model!", op_name);
                        plan_state_str = PlanState::Failed.to_string();
                    }
                }
            } else {
                plan_state_str = PlanState::Completed.to_string();
            }
        }
        // Maybe I also have to reset all operation here...?
        _ => {
            // new_state = reset_all_operations(&new_state, model);
        }
    }

    let mut terminated_operations_meta = vec![];
    for op in &terminated_operations {
        terminated_operations_meta.push(format!("{}_information", op));
        terminated_operations_meta.push(format!("{}_failure_retry_counter", op));
        terminated_operations_meta.push(format!("{}_timeout_retry_counter", op));
        terminated_operations_meta.push(format!("{}_elapsed_executing_ms", op));
        terminated_operations_meta.push(format!("{}_elapsed_disabled_ms", op));
    }
    StateManager::remove_sp_values(&mut con, &terminated_operations).await;
    StateManager::remove_sp_values(&mut con, &terminated_operations_meta).await;
    // terminated_operations.clear();

    new_state = new_state
        .update(
            &format!("{}_plan_state", sp_id),
            plan_state_str.to_spvalue(),
        )
        .update(&format!("{}_plan", sp_id), plan.to_spvalue())
        .update(
            &format!("{}_planner_state", sp_id),
            planner_state.to_spvalue(),
        )
        .update(
            &format!("{}_current_goal_state", sp_id),
            goal_state.to_spvalue(),
        )
        .update(
            &format!("{}_plan_current_step", sp_id),
            plan_current_step.to_spvalue(),
        ).update(
            &format!("{}_terminated_operations", sp_id),
            Vec::<SPValue>::new().to_spvalue(),
        );

    new_state
}
