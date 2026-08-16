use crate::*;
use std::sync::Arc;
use tokio::time::{Duration, interval};

// PERF: reads a fixed key set - good - but two things cost more than they need
// to:
//   - `keys` is built by concatenating `get_all_var_keys()` over every operation
//     plus the operation names, with no `sort()/dedup()`. Operations typically
//     share most of their variables, so the per-tick `MGET` sends the same key
//     many times over. Deduplicating once here shrinks every request.
//   - the tick runs at 500 ms and does a full `MGET` + `build_state` + diff
//     even though the only thing it reacts to is `{sp_id}_replan_trigger`.
//     Suggested: read just that flag (a single `GET`) and only fetch the full
//     planning key set when it is set; or subscribe to a notification on it.
//     Planning is rare and bursty - polling the whole operation model twice a
//     second to discover that nothing is requested is most of this task's cost.
pub async fn planner_ticker(
    sp_id: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(500));
    let log_target = &format!("{}_planner", sp_id);

    log::info!(target: log_target, "Online.");

    // Get only the relevant keys from the state
    log::info!(target: &format!("{}_operation_runner", sp_id), "Online.");
    let mut keys: Vec<String> = model
        .operations
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    // We also need some of the planner vars
    keys.extend(vec![
        format!("{}_planner_information", sp_id),
        format!("{}_planner_state", sp_id),
        format!("{}_plan_state", sp_id),
        format!("{}_plan_current_step", sp_id),
        format!("{}_plan", sp_id),
        format!("{}_plan_id", sp_id),
        format!("{}_replan_trigger", sp_id),
        format!("{}_replanned", sp_id),
        format!("{}_plan_counter", sp_id),
        format!("{}_replan_counter", sp_id),
        format!("{}_replan_counter_total", sp_id),
        format!("{}_current_goal_state", sp_id),
        format!("{}_current_goal_predicate", sp_id),
    ]);

    // And the operation names
    // Maybe we don't even need this if we are not resetting all operations when planning
    // Actually we do need it because the operation planner (bfs needs to access the steate, and the planning is done on the template level)
    keys.extend(
        model
            .operations
            .iter()
            .flat_map(|op| vec![format!("{}", op.name)])
            .collect::<Vec<String>>(),
    );

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    // The operations the planner searches over never change, but every replan
    // has to hand them to a blocking task. Building the `Arc` once here turns
    // that into a refcount bump instead of a deep copy of every operation,
    // transition and predicate per replan.
    let planning_operations = Arc::new(model.operations.clone());

    loop {
        interval.tick().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };
        let old_info = state.get_string_or_default_to_unknown(
            &format!("{}_planner_information", sp_id),
            &log_target,
        );

        let new_state =
            process_planner_tick(sp_id, &planning_operations, &state, &log_target).await;

        let new_info = new_state.get_string_or_default_to_unknown(
            &format!("{}_planner_information", sp_id),
            &log_target,
        );
        if old_info != new_info && !new_info.is_empty() {
            log::info!(target: log_target, "{}", new_info);
        }

        let modified_state = state.get_diff_partial_state_and_add_missing(&new_state);
        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

struct PlannerContext {
    replan_trigger: bool,
    replanned: bool,
    plan_counter: i64,
    // replan_counter: i64,
    // replan_counter_total: i64,
    planner_state: String,
    plan: Vec<String>,
    plan_id: String,
    planner_information: String,
}

async fn process_planner_tick(
    sp_id: &str,
    planning_operations: &Arc<Vec<Operation>>,
    state: &State,
    log_target: &str,
) -> State {
    let mut ctx = PlannerContext {
        replan_trigger: state
            .get_bool_or_default_to_false(&format!("{}_replan_trigger", sp_id), &log_target),
        replanned: state.get_bool_or_default_to_false(&format!("{}_replanned", sp_id), &log_target),
        plan_counter: state
            .get_int_or_default_to_zero(&format!("{}_plan_counter", sp_id), &log_target),
        // replan_counter: state
        //     .get_int_or_default_to_zero(&format!("{}_replan_counter", sp_id), &log_target),
        // replan_counter_total: state
        //     .get_int_or_default_to_zero(&format!("{}_replan_counter_total", sp_id), &log_target),
        planner_state: state
            .get_string_or_default_to_unknown(&format!("{}_planner_state", sp_id), &log_target),
        plan_id: state
            .get_string_or_default_to_unknown(&format!("{}_plan_id", sp_id), &log_target),
        plan: state
            .get_array_or_default_to_empty(&format!("{}_plan", sp_id), &log_target)
            .iter()
            .filter(|val| val.is_string())
            .map(|y| y.to_string())
            .collect(),
        planner_information: state.get_string_or_default_to_unknown(
            &format!("{}_planner_information", sp_id),
            &log_target,
        ),
    };

    let mut new_state = state.clone();
    // let mut state_to_add = State::new();

    if !ctx.replan_trigger {
        // ctx.planner_information = "Planner is not triggered".to_string();
        ctx.replanned = false;
    } else if ctx.replanned {
        ctx.replan_trigger = false;
        ctx.replanned = false;
    } else {
        handle_replan_request(
            &sp_id,
            &mut ctx,
            &mut new_state,
            planning_operations,
            state,
            &log_target,
        )
        .await;
    }

    new_state
        .update(
            &format!("{}_replan_trigger", sp_id),
            ctx.replan_trigger.to_spvalue(),
        )
        .update(&format!("{}_replanned", sp_id), ctx.replanned.to_spvalue())
        .update(
            &format!("{}_plan_counter", sp_id),
            ctx.plan_counter.to_spvalue(),
        )
        // Move this to the goal runner
        // .update(
        //     &format!("{}_replan_counter", sp_id),
        //     ctx.replan_counter.to_spvalue(),
        // )
        // .update(
        //     &format!("{}_replan_counter_total", sp_id),
        //     ctx.replan_counter_total.to_spvalue(),
        // )
        .update(
            &format!("{}_planner_state", sp_id),
            ctx.planner_state.to_spvalue(),
        )
        .update(
            &format!("{}_plan_id", sp_id),
            ctx.plan_id.to_spvalue(),
        )
        .update(&format!("{}_plan", sp_id), ctx.plan.to_spvalue())
        .update(
            &format!("{}_planner_information", sp_id),
            ctx.planner_information.to_spvalue(),
        )
}

// Returns a new state to add containing unique operations ad unique operation meta
//
// DONE: PERF: `bfs_operation_planner(state.clone(), goal, model.operations.clone(), ..)`
// deep-copied the entire state *and* the entire operation model on every replan
// request. The planner takes `&State` and `&[Operation]` now; the operations
// live in an `Arc` built once before the runner loop, so a replan clones the
// state once and nothing else.
//
// DONE: PERF: the call was synchronous and could run for up to `deadline_ms`
// (5000 ms here) inside an async task, blocking that tokio worker for the whole
// time - stalling every other runner scheduled on it, and a very likely cause
// of the "state changes stop happening" symptom during planning. It now runs on
// `tokio::task::spawn_blocking`, so the runtime stays responsive while the
// search runs.
async fn handle_replan_request(
    sp_id: &str,
    ctx: &mut PlannerContext,
    new_state: &mut State,
    planning_operations: &Arc<Vec<Operation>>,
    state: &State,
    log_target: &str
) {
    // *new_state = reset_all_operations(&new_state, &model); // Do we need this?
    ctx.plan = vec![];

    let planner_state = PlannerState::from_str(&ctx.planner_state);
    if planner_state != PlannerState::Ready {
        return;
    }

    // Move this to the goal runner
    // if ctx.replan_counter >= MAX_REPLAN_RETRIES {
    //     ctx.planner_information = "Max allowed replan retries reached.".to_string();
    //     ctx.replan_trigger = false;
    //     return;
    // }

    // ctx.replan_counter += 1;
    // ctx.replan_counter_total += 1;

    let goal = state.extract_goal(&sp_id);

    // `spawn_blocking` needs owned data: the operations are behind an `Arc`
    // built once at startup, so this is a refcount bump, and the state is
    // cloned once per replan - not once per expanded node, as the old
    // by-value signature forced.
    let planning_state = state.clone();
    let operations = Arc::clone(planning_operations);
    let planner_log_target = log_target.to_string();
    let plan_result = match tokio::task::spawn_blocking(move || {
        bfs_operation_planner(
            &planning_state,
            &goal,
            &operations,
            20,
            &planner_log_target,
            5000,
        )
    })
    .await
    {
        Ok(result) => result,
        Err(e) => {
            log::error!(target: log_target, "Planner task failed to run: {e}");
            PlanningResult {
                found: false,
                ..Default::default()
            }
        }
    };

    if !plan_result.found {
        ctx.plan_id = "".to_string();
        ctx.planner_information = format!(
            "Planner triggered but no plan was found.",
            // ctx.replan_counter, MAX_REPLAN_RETRIES
        );
        ctx.planner_state = PlannerState::NotFound.to_string();
        // State::new()
    } else {
        ctx.planner_state = PlannerState::Found.to_string();
        ctx.plan_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
        // ctx.replan_counter = 0;

        if plan_result.length > 0 {
            ctx.replanned = true;
            ctx.plan_counter += 1;
            ctx.plan = plan_result.plan.iter().map(|x| format!("{}_{}", x, nanoid::nanoid!(10, &NANOID_ALPHABET))).collect();
            *new_state = add_operation_state_tracking_variable(&ctx.plan, &new_state, &log_target);
            *new_state = add_operation_meta_tracking_variables(&ctx.plan, &new_state, false, &log_target);
            ctx.planner_information = format!(
                "Got a new plan {}:\n{}",
                ctx.plan_id,
                ctx.plan
                    .iter()
                    .enumerate()
                    .map(|(index, step)| format!("       {} -> {}", index + 1, step))
                    .collect::<Vec<String>>()
                    .join("\n")
            );
            // state_to_add
        } else {
            ctx.planner_information = "We are already in the goal. No action needed.".to_string();
            // State::new()
        }
    }
}
