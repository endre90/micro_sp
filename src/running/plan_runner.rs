use crate::{running::process_operation::OperationProcessingType, *};
use crate::SPConnection;
use std::sync::Arc;
// use crate::SPConnection;
use tokio::sync::mpsc;

// DONE (correctness): this constant used to do double duty as both the tick
// period *and* the assumed time increment in `process_operation`'s elapsed-time
// accounting - even though `sop_runner` ticks at 100 ms, so its operations aged
// at twice real speed. "How often do I poll" and "how much time has passed" are
// separate now: each runner measures the latter with `Instant` and passes it in,
// which also keeps the counters honest when a tick slips because Redis was slow.
// DONE: PERF: this was the third caller of `StateManager::get_full_state`;
// with `sop_runner` and `auto_operation_runner` that made ~20 blocking
// `KEYS *` scans per second. The key set is `model.operations`
// `get_all_var_keys()` plus the `{sp_id}_plan*` /
// `{sp_id}_terminated_operations` keys (see `running::runner_keys`), plus the
// bookkeeping variables of the steps of the current plan.
//
// Unlike the other two runners this one does not create its operations itself -
// `planner_ticker` writes the plan and the step variables - so it cannot know
// from a local event when the dynamic part of its key set changed. It watches
// `{sp_id}_plan` instead and re-reads once when it changes, which costs one
// extra `MGET` per new plan rather than a keyspace scan per tick.
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
    let mut interval = runner_interval();

    // Get only the relevant keys from the state
    log::info!(target: &log_target, "Online.");

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    let static_keys = plan_runner_static_keys(sp_id, &model);
    let mut keys = static_keys.clone();
    let mut active_plan: Vec<String> = vec![];
    let read_full_state = read_full_state_enabled();
    if read_full_state {
        log::warn!(target: &log_target, "MICRO_SP_READ_FULL_STATE is set: reading the whole keyspace every tick.");
    }

    // Real time between ticks; see the note in `process_operation`.
    let mut tick_clock = TickClock::new();

    loop {
        interval.tick().await;
        let tick_elapsed_ms = tick_clock.elapsed_ms();

        let read = match read_full_state {
            true => StateManager::get_full_state(&mut con).await,
            false => StateManager::get_state_for_keys(&mut con, &keys, &log_target).await,
        };
        let mut state = match read {
            Some(s) => s,
            None => continue,
        };

        // The plan is produced by `planner_ticker`, so a new plan shows up here
        // as a changed `{sp_id}_plan`. Its steps are uniquified operation names
        // whose bookkeeping variables have to be in the key set before
        // `process_plan_tick` reads them - reading a variable that is not in
        // the state panics - so rebuild and re-read once when it changes.
        if !read_full_state {
            let plan = read_plan(&state, sp_id, &log_target);
            if plan != active_plan {
                keys = keys_with_active_operations(&static_keys, &plan);
                active_plan = plan;
                state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
                    Some(s) => s,
                    None => continue,
                };
            }
        }

        let con_clone = con.clone();
        let new_state = process_plan_tick(
            sp_id,
            con_clone,
            &model,
            &state,
            // logging_tx.clone(),
            tick_elapsed_ms,
            &log_target,
        )
        .await;
        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

/// Length of the `_{nanoid}` suffix that `handle_replan_request` appends to an
/// operation name when it instantiates a plan step.
const PLAN_STEP_SUFFIX_LEN: usize = 1 + 10;

/// Find the model operation a plan step was instantiated from.
///
/// DONE (correctness): this was `operations.iter().find(|op| step.starts_with(&op.name))`,
/// which matches *any* operation whose name is a prefix of the step. A model
/// containing both `op_move` and `op_move_to_b` would resolve a
/// `op_move_to_b_A1b2C3d4E5` step to whichever came first in the model - so the
/// plan runner could silently drive the wrong operation's transitions, writing
/// the wrong actions and waiting on the wrong postcondition.
///
/// Plan steps are `{operation name}_{nanoid}` with a fixed-length nanoid, so
/// the step can be resolved exactly: strip the suffix, match the whole name.
/// The prefix form is kept only as a fallback for step names that carry no
/// nanoid suffix, and even then it takes the *longest* match so a shorter name
/// can never shadow a longer one.
fn find_step_operation<'a>(operations: &'a [Operation], step: &str) -> Option<&'a Operation> {
    if step.len() > PLAN_STEP_SUFFIX_LEN {
        let split_at = step.len() - PLAN_STEP_SUFFIX_LEN;
        if step.is_char_boundary(split_at) {
            let (template, suffix) = step.split_at(split_at);
            let looks_like_a_step = suffix.starts_with('_')
                && suffix[1..].chars().all(|c| NANOID_ALPHABET.contains(&c));
            if looks_like_a_step {
                if let Some(operation) = operations.iter().find(|op| op.name == template) {
                    return Some(operation);
                }
            }
        }
    }

    operations
        .iter()
        .filter(|op| step.starts_with(&op.name))
        .max_by_key(|op| op.name.len())
}

/// The current plan as a list of (uniquified) operation names.
///
/// Used both by the tick itself and by the runner loop, which needs the step
/// names to keep its `get_state_for_keys` key set in sync with the plan.
fn read_plan(state: &State, sp_id: &str, log_target: &str) -> Vec<String> {
    state
        .get_array_or_default_to_empty(&format!("{}_plan", sp_id), log_target)
        .iter()
        .filter(|val| val.is_string())
        .map(|y| y.to_string())
        .collect()
}

// DONE: PERF: the two `remove_sp_values` calls near the end ran on every tick.
// They do return early on an empty slice, so an idle tick never actually cost a
// round trip - the correction to the original note - but the meta key list was
// still built every time. The whole block is guarded now. Pipelining the two
// DELs into one is still open.
// DONE: PERF: the tail was a chain of six `.update(..)` calls, i.e. six full
// state-map copies per tick. They are `update_mut` now. Note the original
// claim that this made "an MSET go out even when nothing happened" was wrong:
// five of the six write back the value just read, so they produce no diff -
// measured idle traffic for this runner is zero writes.
// DONE (correctness): the `starts_with` lookup let `op_move` match a
// `op_move_to_b_...` step - see `find_step_operation`.
// PERF (still open): the lookup is a linear scan per plan step per tick. That
// is one scan of the model per tick, so it stays cheap until the model gets
// large; a `HashMap<&str, &Operation>` built once at startup would make it O(1).
// PERF: `let mut new_state = state.clone()` plus `state.get_diff_partial_state(
// &new_state)` in the caller means a full map copy and a full map scan per
// tick; see the dirty-key suggestion on `State::get_diff_partial_state`.
async fn process_plan_tick(
    sp_id: &str,
    mut con: SPConnection,
    model: &Model,
    state: &State,
    // logging_tx: mpsc::Sender<LogMsg>,
    tick_elapsed_ms: i64,
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
    let plan = read_plan(state, sp_id, &log_target);

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
                match find_step_operation(&model.operations, op_name) {
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
                            tick_elapsed_ms,
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

    // Guarded, like `auto_operation_runner` does it: on a tick with nothing
    // terminated there is no key-list building at all. When there is something,
    // both deletes go out in one pipelined round trip.
    if !terminated_operations.is_empty() {
        let mut terminated_operations_meta = vec![];
        for op in &terminated_operations {
            terminated_operations_meta.push(format!("{}_information", op));
            terminated_operations_meta.push(format!("{}_failure_retry_counter", op));
            terminated_operations_meta.push(format!("{}_timeout_retry_counter", op));
            terminated_operations_meta.push(format!("{}_elapsed_executing_ms", op));
            terminated_operations_meta.push(format!("{}_elapsed_disabled_ms", op));
        }
        StateManager::apply(
            &mut con,
            &State::new(),
            &[&terminated_operations, &terminated_operations_meta],
        )
        .await;
    }

    // DONE: PERF: this was a chain of six `.update(..)` calls, each cloning the
    // whole state map - six full copies per tick to write six values. Writing
    // in place costs six map lookups.
    //
    // Most of these write back the value that was just read, so they do not
    // show up in the diff and cost no Redis traffic; they are kept as-is so the
    // tick still has a single obvious place where its outputs are published.
    new_state.update_mut(
        &format!("{}_plan_state", sp_id),
        plan_state_str.to_spvalue(),
    );
    new_state.update_mut(&format!("{}_plan", sp_id), plan.to_spvalue());
    new_state.update_mut(
        &format!("{}_planner_state", sp_id),
        planner_state.to_spvalue(),
    );
    new_state.update_mut(
        &format!("{}_current_goal_state", sp_id),
        goal_state.to_spvalue(),
    );
    new_state.update_mut(
        &format!("{}_plan_current_step", sp_id),
        plan_current_step.to_spvalue(),
    );
    new_state.update_mut(
        &format!("{}_terminated_operations", sp_id),
        Vec::<SPValue>::new().to_spvalue(),
    );

    new_state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(name: &str) -> Operation {
        Operation {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// The bug: `starts_with` let a shorter operation name swallow a step that
    /// belonged to a longer one, in whichever order the model happened to list
    /// them.
    #[test]
    fn a_step_resolves_to_the_operation_it_was_built_from() {
        let operations = vec![op("op_move"), op("op_move_to_b")];
        let reversed = vec![op("op_move_to_b"), op("op_move")];

        for model in [&operations, &reversed] {
            assert_eq!(
                find_step_operation(model, "op_move_to_b_A1b2C3d4E5")
                    .map(|o| o.name.as_str()),
                Some("op_move_to_b"),
                "a step of op_move_to_b must not resolve to op_move"
            );
            assert_eq!(
                find_step_operation(model, "op_move_A1b2C3d4E5").map(|o| o.name.as_str()),
                Some("op_move"),
                "a step of op_move must still resolve to op_move"
            );
        }
    }

    #[test]
    fn an_unknown_step_resolves_to_nothing() {
        let operations = vec![op("op_move")];
        assert!(find_step_operation(&operations, "op_grip_A1b2C3d4E5").is_none());
        assert!(find_step_operation(&operations, "").is_none());
    }

    /// Step names that do not carry a nanoid suffix still resolve by prefix,
    /// but by the longest match rather than the first one listed.
    #[test]
    fn a_step_without_a_nanoid_suffix_takes_the_longest_prefix() {
        let operations = vec![op("op_move"), op("op_move_to_b")];

        assert_eq!(
            find_step_operation(&operations, "op_move_to_b").map(|o| o.name.as_str()),
            Some("op_move_to_b")
        );
        assert_eq!(
            find_step_operation(&operations, "op_move").map(|o| o.name.as_str()),
            Some("op_move")
        );
    }

    /// An operation name that itself ends in something nanoid-shaped must not
    /// confuse the suffix strip.
    #[test]
    fn an_operation_whose_name_looks_suffixed_still_resolves() {
        let operations = vec![op("op_stage_2_A1b2C3d4E5")];
        assert_eq!(
            find_step_operation(&operations, "op_stage_2_A1b2C3d4E5_Z9y8X7w6V5")
                .map(|o| o.name.as_str()),
            Some("op_stage_2_A1b2C3d4E5"),
        );
        // And the bare name, with no step suffix, via the prefix fallback.
        assert_eq!(
            find_step_operation(&operations, "op_stage_2_A1b2C3d4E5").map(|o| o.name.as_str()),
            Some("op_stage_2_A1b2C3d4E5"),
        );
    }
}

/// The plan runner, driven end to end against a real Redis.
///
/// This runner is the executor: it takes the plan `planner_ticker` produced,
/// walks it one step at a time through `process_operation`, and reports back
/// through `{sp_id}_plan_state` - which is what `goal_runner` watches. The plan
/// state machine itself (Initial -> Executing -> Completed / Failed) had no
/// coverage at all; only the step-name resolution above did.
#[cfg(test)]
mod plan_runner_tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    const SP: &str = "sp";
    const TARGET: &str = "test";

    async fn redis() -> (ContainerAsync<Redis>, Arc<ConnectionManager>) {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let manager = Arc::new(ConnectionManager::new().await);
        let mut con = manager.get_connection().await;
        StateManager::flush_state(&mut con).await;
        (container, manager)
    }

    fn key(suffix: &str) -> String {
        format!("{SP}_{suffix}")
    }

    /// A model of two operations that walk `pos` from a to b to c.
    fn model(state: &State) -> Model {
        let hop = |name: &str, from: &str, to: &str| {
            Operation::new(
                name,
                Some(10_000),
                Some(10_000),
                None,
                None,
                false,
                vec![Transition::parse(
                    "start",
                    &format!("var:pos == {from}"),
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    state,
                )],
                vec![Transition::parse(
                    "complete",
                    "true",
                    "true",
                    vec![format!("var:pos <- {to}").as_str()],
                    Vec::<&str>::new(),
                    state,
                )],
                vec![],
                vec![],
                vec![],
                vec![],
            )
        };

        Model::new(
            SP,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![hop("a_to_b", "a", "b"), hop("b_to_c", "b", "c")],
        )
    }

    fn domain() -> State {
        let mut state = State::new();
        state.add_mut(
            SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
            TARGET,
        );
        state
    }

    /// Seed Redis with everything the runner reads, plus a plan of `steps`.
    async fn deploy(manager: &Arc<ConnectionManager>, steps: &[String]) -> Model {
        let domain = domain();
        let model = model(&domain);

        let mut state = generate_runner_state_variables(SP, 0, TARGET);
        state.extend_mut(generate_operation_state_variables(&model, false, TARGET), true);
        state.extend_mut(domain, true);
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key("dashboard_command"), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );
        state = state.update(&key("terminated_operations"), Vec::<SPValue>::new().to_spvalue());
        state = state.update(
            &key("plan"),
            steps.iter().map(|s| s.to_spvalue()).collect::<Vec<SPValue>>().to_spvalue(),
        );
        state = state.update(&key("plan_state"), "initial".to_spvalue());
        state = state.update(&key("plan_current_step"), 0.to_spvalue());

        // The step instances need their own bookkeeping variables, which
        // `planner_ticker` would normally have created.
        let steps: Vec<String> = steps.to_vec();
        state = add_operation_state_tracking_variable(&steps, &state, TARGET);
        state = add_operation_meta_tracking_variables(&steps, &state, false, TARGET);

        let mut con = manager.get_connection().await;
        StateManager::set_state(&mut con, &state).await;
        model
    }

    fn spawn_runner(manager: &Arc<ConnectionManager>, model: Model) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = planned_operation_runner(&model, &manager).await;
        })
    }

    async fn text(con: &mut SPConnection, suffix: &str) -> String {
        match StateManager::get_sp_value(con, &key(suffix)).await {
            Some(SPValue::String(StringOrUnknown::String(s))) => s,
            other => format!("{other:?}"),
        }
    }

    async fn wait_for(con: &mut SPConnection, suffix: &str, expected: &str, ms: u64) -> String {
        let deadline = std::time::Instant::now() + Duration::from_millis(ms);
        let mut last = String::new();
        while std::time::Instant::now() < deadline {
            last = text(con, suffix).await;
            if last == expected {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        last
    }

    fn steps() -> Vec<String> {
        vec![
            "op_a_to_b_AAAAAAAAAA".to_string(),
            "op_b_to_c_BBBBBBBBBB".to_string(),
        ]
    }

    /// The whole executor: a found plan is picked up, both steps run in order,
    /// and the plan reports `completed`.
    #[tokio::test]
    #[serial]
    async fn a_found_plan_is_executed_step_by_step_to_completion() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager, &steps()).await;

        let runner = spawn_runner(&manager, model);
        // This is what `planner_ticker` reports when it finds a plan.
        StateManager::set_sp_value(&mut con, &key("planner_state"), &"found".to_spvalue()).await;

        let plan_state = wait_for(&mut con, "plan_state", "completed", 5000).await;
        runner.abort();

        assert_eq!(plan_state, "completed");
        assert_eq!(
            StateManager::get_sp_value(&mut con, "pos").await,
            Some("c".to_spvalue()),
            "both steps should have run, in order"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("plan_current_step")).await,
            Some(2.to_spvalue()),
            "the cursor should have walked off the end of the plan"
        );
    }

    /// `planner_state == not_found` fails the plan without executing anything -
    /// this is how "there is no route to the goal" reaches the goal runner.
    #[tokio::test]
    #[serial]
    async fn a_plan_that_was_not_found_fails_immediately() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager, &[]).await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &key("planner_state"), &"not_found".to_spvalue()).await;

        let plan_state = wait_for(&mut con, "plan_state", "failed", 3000).await;
        runner.abort();

        assert_eq!(plan_state, "failed");
        assert_eq!(
            StateManager::get_sp_value(&mut con, "pos").await,
            Some("a".to_spvalue()),
            "nothing should have been executed"
        );
    }

    /// An empty plan is trivially complete - the planner says "we are already in
    /// the goal" that way.
    #[tokio::test]
    #[serial]
    async fn an_empty_plan_completes_at_once() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager, &[]).await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &key("planner_state"), &"found".to_spvalue()).await;

        assert_eq!(
            wait_for(&mut con, "plan_state", "completed", 3000).await,
            "completed"
        );
        runner.abort();
    }

    /// A step naming an operation the model does not contain fails the plan
    /// rather than being skipped - a skipped step would mean the plan claims to
    /// have reached the goal without doing the work.
    #[tokio::test]
    #[serial]
    async fn a_step_with_no_matching_operation_fails_the_plan() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager, &["op_does_not_exist_CCCCCCCCCC".to_string()]).await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &key("planner_state"), &"found".to_spvalue()).await;

        assert_eq!(
            wait_for(&mut con, "plan_state", "failed", 3000).await,
            "failed"
        );
        runner.abort();
    }

    /// Pressing stop cancels the running step, and `process_operation` reports
    /// that back as a cancelled plan.
    #[tokio::test]
    #[serial]
    async fn stop_cancels_the_running_plan() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        // A step whose operation can never complete, so it sits in Executing.
        let domain = domain();
        let blocked = Operation::new(
            "stuck",
            Some(10_000),
            Some(10_000),
            None,
            None,
            false,
            vec![Transition::parse(
                "start",
                "var:pos == a",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &domain,
            )],
            vec![Transition::parse(
                "never",
                "false",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &domain,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        let model = Model::new(SP, vec![], vec![], vec![], vec![], vec![blocked]);
        let step = "op_stuck_DDDDDDDDDD".to_string();

        let mut state = generate_runner_state_variables(SP, 0, TARGET);
        state.extend_mut(generate_operation_state_variables(&model, false, TARGET), true);
        state.extend_mut(domain, true);
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&key("dashboard_command"), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );
        state = state.update(&key("terminated_operations"), Vec::<SPValue>::new().to_spvalue());
        state = state.update(&key("plan"), vec![step.to_spvalue()].to_spvalue());
        state = state.update(&key("plan_state"), "initial".to_spvalue());
        state = state.update(&key("plan_current_step"), 0.to_spvalue());
        state = add_operation_state_tracking_variable(&vec![step.clone()], &state, TARGET);
        state = add_operation_meta_tracking_variables(&vec![step.clone()], &state, false, TARGET);
        StateManager::set_state(&mut con, &state).await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &key("planner_state"), &"found".to_spvalue()).await;
        assert_eq!(
            wait_for(&mut con, "plan_state", "executing", 3000).await,
            "executing"
        );

        // The operator presses stop.
        StateManager::set_sp_value(&mut con, &key("dashboard_command"), &"stop".to_spvalue()).await;

        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        let mut step_state = String::new();
        while std::time::Instant::now() < deadline {
            step_state = match StateManager::get_sp_value(&mut con, &step).await {
                Some(SPValue::String(StringOrUnknown::String(s))) => s,
                other => format!("{other:?}"),
            };
            if step_state == "cancelled" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        // The plan state follows on the *next* tick: `process_operation` only
        // reports the cancellation once it re-enters with the operation already
        // in `cancelled`.
        let plan_state = wait_for(&mut con, "plan_state", &PlanState::Cancelled.to_string(), 3000).await;
        runner.abort();

        assert_eq!(step_state, "cancelled", "the running step must be cancelled");
        assert_eq!(
            plan_state,
            PlanState::Cancelled.to_string(),
            "and the plan must report it - even though nothing can read it back, \
             see plan_state_cancelled_does_not_survive_the_round_trip"
        );
    }

    /// An idle runner - no plan, nothing found - must not write.
    #[tokio::test]
    #[serial]
    async fn an_idle_runner_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager, &steps()).await;

        let runner = spawn_runner(&manager, model);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "a plan runner with nothing to execute must not write: {:?}",
            before.get_diff_partial_state(&after)
        );
        assert!(!runner.is_finished());
        runner.abort();
    }
}
