use crate::{running::process_operation::OperationProcessingType, *};
use crate::SPConnection;
use std::sync::Arc;

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
            activity_log::log_state_diff(&log_target, &state, &modified_state);
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

/// Length of the `_{nanoid}` suffix that `handle_replan_request` appends to an
/// operation name when it instantiates a plan step.
const PLAN_STEP_SUFFIX_LEN: usize = 1 + 10;

/// Find the model operation a plan step was instantiated from.
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

    /// Once a step terminates, its five bookkeeping keys must be deleted from
    /// Redis - not just cleared in memory - and `{sp_id}_terminated_operations`
    /// itself must come back empty. A missed delete here would leak a
    /// finished step's retry counters and elapsed-time keys into Redis forever,
    /// and if a later plan happened to reuse the same operation name (same
    /// nanoid alphabet, astronomically unlikely but not impossible) it would
    /// inherit stale counters.
    #[tokio::test]
    #[serial]
    async fn terminated_operations_meta_keys_are_deleted_from_redis() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let domain = domain();
        let model = model(&domain);
        let op = "op_a_to_b_AAAAAAAAAA".to_string();

        let mut state = generate_runner_state_variables(SP, 0, TARGET);
        state.extend_mut(generate_operation_state_variables(&model, false, TARGET), true);
        state.extend_mut(domain, true);
        state = state.update(&key("plan"), Vec::<SPValue>::new().to_spvalue());
        state = state.update(&key("plan_state"), "initial".to_spvalue());
        state = state.update(&key("plan_current_step"), 0.to_spvalue());
        state = state.update(
            &key("terminated_operations"),
            vec![op.to_spvalue()].to_spvalue(),
        );
        state = add_operation_state_tracking_variable(&vec![op.clone()], &state, TARGET);
        state = add_operation_meta_tracking_variables(&vec![op.clone()], &state, false, TARGET);
        StateManager::set_state(&mut con, &state).await;

        // Sanity check: the meta keys actually exist before the tick runs.
        assert!(
            StateManager::get_sp_value(&mut con, &format!("{op}_information")).await.is_some(),
            "test setup should have written the meta keys"
        );

        let tick_con = manager.get_connection().await;
        let _ = process_plan_tick(SP, tick_con, &model, &state, 100, TARGET).await;

        // The tick's own DEL is pipelined but still awaited inside
        // `process_plan_tick`, so no extra wait should be needed - but give it
        // a small grace window since this is a real network round trip.
        tokio::time::sleep(Duration::from_millis(100)).await;

        for suffix in [
            "_information",
            "_failure_retry_counter",
            "_timeout_retry_counter",
            "_elapsed_executing_ms",
            "_elapsed_disabled_ms",
        ] {
            let k = format!("{op}{suffix}");
            assert_eq!(
                StateManager::get_sp_value(&mut con, &k).await,
                None,
                "meta key '{k}' should have been deleted"
            );
        }
        assert_eq!(
            StateManager::get_sp_value(&mut con, &op).await,
            None,
            "the operation's own bare state key should have been deleted too"
        );
    }

    /// `MICRO_SP_READ_FULL_STATE` sends the runner back to a full-keyspace read
    /// every tick instead of its derived key set. This is the escape hatch, so
    /// it has to actually still drive a plan to completion, not just avoid
    /// panicking.
    #[tokio::test]
    #[serial]
    async fn the_full_state_escape_hatch_still_executes_the_plan() {
        struct EnvGuard;
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                unsafe { std::env::remove_var("MICRO_SP_READ_FULL_STATE") };
            }
        }
        unsafe { std::env::set_var("MICRO_SP_READ_FULL_STATE", "true") };
        let _guard = EnvGuard;

        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager, &steps()).await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &key("planner_state"), &"found".to_spvalue()).await;

        let plan_state = wait_for(&mut con, "plan_state", "completed", 5000).await;
        runner.abort();

        assert_eq!(
            plan_state, "completed",
            "the plan must still complete when reading the whole keyspace every tick"
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, "pos").await,
            Some("c".to_spvalue()),
            "both steps should have run under the full-state escape hatch too"
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
