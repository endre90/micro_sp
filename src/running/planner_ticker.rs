use crate::*;
use std::sync::Arc;

// DONE: PERF: two things cost more than they needed to:
//   - `keys` was built by concatenating `get_all_var_keys()` over every
//     operation plus the operation names, with no `sort()/dedup()`. Operations
//     typically share most of their variables, so the per-tick `MGET` sent the
//     same key many times over.
//   - the tick ran at 500 ms and did a full `MGET` + `build_state` + diff even
//     though the only thing it reacts to is `{sp_id}_replan_trigger`. Planning
//     is rare and bursty, so polling the whole operation model twice a second
//     to discover that nothing is requested was most of this task's cost. It
//     now reads the two flags that decide whether there is anything to do and
//     fetches the planning key set only when there is.
// PERF (still open): a notification on `{sp_id}_replan_trigger` would remove
// the poll entirely.
pub async fn planner_ticker(
    sp_id: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = runner_interval();
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
            .map(|op| op.name.clone())
            .collect::<Vec<String>>(),
    );

    // Operations share most of their variables, so without this the per-tick
    // `MGET` sends the same key many times over.
    keys.sort_unstable();
    keys.dedup();

    // The two flags that decide whether this tick has anything to do at all.
    let trigger_key = format!("{}_replan_trigger", sp_id);
    let replanned_key = format!("{}_replanned", sp_id);
    let trigger_keys = vec![trigger_key.clone(), replanned_key.clone()];

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

        // Fast path. With no replan requested, a full tick reads the planning
        // key set, rebuilds a `State` from it, runs the tick (which only sets
        // `_replanned` to the false it already holds) and diffs it back to
        // nothing. Two booleans are enough to know that in advance.
        let triggers =
            match StateManager::get_state_for_keys(&mut con, &trigger_keys, &log_target).await {
                Some(s) => s,
                None => continue,
            };
        let replan_trigger = triggers.get_bool_or_default_to_false(&trigger_key, &log_target);
        let replanned = triggers.get_bool_or_default_to_false(&replanned_key, &log_target);
        if !replan_trigger && !replanned {
            continue;
        }

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

/// The planner tick, without Redis.
///
/// `process_planner_tick` is where the whole replanning protocol lives: the
/// handshake with `goal_runner` over `_replan_trigger` / `_replanned`, the
/// `_planner_state` gate, and the plan-with-unique-step-ids that `plan_runner`
/// then executes. It takes and returns a `State`, so all of it is reachable
/// without a running Redis - only the surrounding loop needs one.
///
/// The handshake is worth pinning carefully, because it is a three-runner
/// protocol implemented through shared keys: goal_runner sets the trigger,
/// planner_ticker plans and sets `_replanned`, and on the *following* tick it
/// clears both. A change that collapses that into one tick would break the
/// window in which goal_runner observes that a plan was produced.
#[cfg(test)]
mod planner_tick_tests {
    use super::*;

    const SP: &str = "sp";
    const TARGET: &str = "test";

    /// A two-step world: `pos` goes a -> b -> c, one operation per step.
    fn model() -> (State, Arc<Vec<Operation>>) {
        let pos = v!("pos");
        let mut state = State::new();
        state.add_mut(SPAssignment::new(pos.clone(), "a".to_spvalue()), TARGET);

        let step = |name: &str, from: &str, to: &str, state: &State| {
            Operation::new(
                name,
                None,
                None,
                None,
                None,
                false,
                vec![Transition::parse(
                    &format!("start_{name}"),
                    &format!("var:pos == {from}"),
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    state,
                )],
                vec![Transition::parse(
                    &format!("complete_{name}"),
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

        let operations = vec![
            step("op_a_to_b", "a", "b", &state),
            step("op_b_to_c", "b", "c", &state),
        ];

        for op in &operations {
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&op.name, SPValueType::String),
                    "initial".to_spvalue(),
                ),
                TARGET,
            );
        }

        (state, Arc::new(operations))
    }

    /// The planner variables the tick reads, with sensible starting values.
    fn with_planner_vars(state: &State, goal: &str, planner_state: &str) -> State {
        let mut state = state.clone();
        let strings = [
            ("planner_information", "".to_string()),
            ("planner_state", planner_state.to_string()),
            ("plan_id", "".to_string()),
            ("current_goal_predicate", goal.to_string()),
        ];
        for (suffix, value) in strings {
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&format!("{SP}_{suffix}"), SPValueType::String),
                    value.to_spvalue(),
                ),
                TARGET,
            );
        }
        for (suffix, value) in [("replan_trigger", false), ("replanned", false)] {
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&format!("{SP}_{suffix}"), SPValueType::Bool),
                    value.to_spvalue(),
                ),
                TARGET,
            );
        }
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{SP}_plan_counter"), SPValueType::Int64),
                0.to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{SP}_plan"), SPValueType::Array),
                Vec::<SPValue>::new().to_spvalue(),
            ),
            TARGET,
        );
        state
    }

    fn text(state: &State, suffix: &str) -> String {
        state.get_string_or_default_to_unknown(&format!("{SP}_{suffix}"), TARGET)
    }

    fn flag(state: &State, suffix: &str) -> bool {
        state.get_bool_or_default_to_false(&format!("{SP}_{suffix}"), TARGET)
    }

    fn plan(state: &State) -> Vec<String> {
        state
            .get_array_or_default_to_empty(&format!("{SP}_plan"), TARGET)
            .iter()
            .map(|v| v.to_string())
            .collect()
    }

    async fn tick(state: &State, operations: &Arc<Vec<Operation>>) -> State {
        process_planner_tick(SP, operations, state, TARGET).await
    }

    /// Nothing requested: the tick is a no-op, and specifically it must produce
    /// no diff at all. This runs at the tick rate on every deployment, so a
    /// single spurious write here is a write per tick forever.
    #[tokio::test]
    async fn an_untriggered_tick_changes_nothing() {
        let (state, operations) = model();
        let state = with_planner_vars(&state, "var:pos == c", "ready");

        let next = tick(&state, &operations).await;

        assert!(
            state.get_diff_partial_state(&next).state.is_empty(),
            "an untriggered planner tick must not write"
        );
    }

    /// The handshake, both halves of it.
    #[tokio::test]
    async fn a_replan_request_produces_a_plan_and_then_clears_itself() {
        let (state, operations) = model();
        let state = with_planner_vars(&state, "var:pos == c", "ready");
        let state = state.update(&format!("{SP}_replan_trigger"), true.to_spvalue());

        // Tick one: plan.
        let planned = tick(&state, &operations).await;

        assert_eq!(text(&planned, "planner_state"), "found");
        assert!(flag(&planned, "replanned"), "goal_runner learns of the plan here");
        assert!(
            flag(&planned, "replan_trigger"),
            "the trigger is still set on the planning tick"
        );
        assert_eq!(planned.get_value(&format!("{SP}_plan_counter"), TARGET), Some(1.to_spvalue()));

        let steps = plan(&planned);
        assert_eq!(steps.len(), 2, "a -> b -> c is two operations: {steps:?}");
        assert!(steps[0].starts_with("op_a_to_b_"), "{:?}", steps[0]);
        assert!(steps[1].starts_with("op_b_to_c_"), "{:?}", steps[1]);
        assert!(!text(&planned, "plan_id").is_empty());
        assert!(text(&planned, "planner_information").starts_with("Got a new plan"));

        // Tick two: the handshake closes.
        let settled = tick(&planned, &operations).await;
        assert!(!flag(&settled, "replan_trigger"));
        assert!(!flag(&settled, "replanned"));
        assert_eq!(
            plan(&settled),
            steps,
            "closing the handshake must not disturb the plan"
        );
    }

    /// Every step carries a fresh nanoid suffix, so two operations of the same
    /// template in one plan - or the same plan produced twice - never collide
    /// on their bookkeeping variables. This is what `find_step_operation` in
    /// the plan runner resolves back to a template.
    #[tokio::test]
    async fn every_step_gets_a_unique_id_and_its_tracking_variables() {
        let (state, operations) = model();
        let state = with_planner_vars(&state, "var:pos == c", "ready");
        let state = state.update(&format!("{SP}_replan_trigger"), true.to_spvalue());

        let first = tick(&state, &operations).await;
        let second = tick(&state, &operations).await;

        let a = plan(&first);
        let b = plan(&second);
        assert_ne!(a, b, "two replans must not reuse the same step ids");
        assert_ne!(text(&first, "plan_id"), text(&second, "plan_id"));

        // And each step's bookkeeping variables were created alongside it.
        for step in &a {
            assert!(first.contains(step), "{step} has no state variable");
            for suffix in [
                "_information",
                "_elapsed_executing_ms",
                "_elapsed_disabled_ms",
                "_failure_retry_counter",
                "_timeout_retry_counter",
            ] {
                assert!(
                    first.contains(&format!("{step}{suffix}")),
                    "{step}{suffix} is missing"
                );
            }
        }
    }

    /// A goal that already holds is not an error and is not a plan - the
    /// planner says so and produces nothing to execute. `_replanned` staying
    /// false here is what stops the plan runner from being handed an empty plan.
    #[tokio::test]
    async fn a_goal_that_already_holds_produces_no_plan() {
        let (state, operations) = model();
        let state = with_planner_vars(&state, "var:pos == a", "ready");
        let state = state.update(&format!("{SP}_replan_trigger"), true.to_spvalue());

        let next = tick(&state, &operations).await;

        assert_eq!(text(&next, "planner_state"), "found");
        assert!(plan(&next).is_empty());
        assert!(!flag(&next, "replanned"));
        assert_eq!(
            text(&next, "planner_information"),
            "We are already in the goal. No action needed."
        );
    }

    #[tokio::test]
    async fn an_unreachable_goal_reports_not_found() {
        let (state, operations) = model();
        let state = with_planner_vars(&state, "var:pos == nowhere", "ready");
        let state = state.update(&format!("{SP}_replan_trigger"), true.to_spvalue());

        let next = tick(&state, &operations).await;

        assert_eq!(text(&next, "planner_state"), "not_found");
        assert_eq!(text(&next, "plan_id"), "");
        assert!(plan(&next).is_empty());
        assert_eq!(
            text(&next, "planner_information"),
            "Planner triggered but no plan was found."
        );
    }

    /// The `_planner_state` gate: only `ready` admits a replan. This is how the
    /// goal runner keeps a second replan from starting while the last one is
    /// still being consumed.
    #[tokio::test]
    async fn a_replan_is_ignored_unless_the_planner_is_ready() {
        for planner_state in ["found", "not_found", "UNKNOWN", "nonsense"] {
            let (state, operations) = model();
            let state = with_planner_vars(&state, "var:pos == c", planner_state);
            let state = state.update(&format!("{SP}_replan_trigger"), true.to_spvalue());

            let next = tick(&state, &operations).await;

            assert!(
                plan(&next).is_empty(),
                "planner_state '{planner_state}' should not have produced a plan"
            );
            assert!(!flag(&next, "replanned"), "planner_state '{planner_state}'");
            assert_eq!(
                text(&next, "planner_state"),
                planner_state,
                "the gate must not rewrite the state it refused on"
            );
        }
    }

    /// A stale plan from a previous goal must not survive a refused replan -
    /// otherwise the plan runner keeps executing the old one.
    #[tokio::test]
    async fn a_refused_replan_still_clears_the_previous_plan() {
        let (state, operations) = model();
        let state = with_planner_vars(&state, "var:pos == c", "found");
        let state = state.update(
            &format!("{SP}_plan"),
            vec!["op_stale_AAAAAAAAAA".to_spvalue()].to_spvalue(),
        );
        let state = state.update(&format!("{SP}_replan_trigger"), true.to_spvalue());

        let next = tick(&state, &operations).await;

        assert!(plan(&next).is_empty(), "the stale plan must be dropped");
    }

    /// `_replanned` set without `_replan_trigger` is the trailing half of a
    /// handshake nobody completed; the tick has to clear it rather than leave
    /// the goal runner believing a plan just arrived.
    #[tokio::test]
    async fn a_stray_replanned_flag_is_cleared() {
        let (state, operations) = model();
        let state = with_planner_vars(&state, "var:pos == c", "ready");
        let state = state.update(&format!("{SP}_replanned"), true.to_spvalue());

        let next = tick(&state, &operations).await;

        assert!(!flag(&next, "replanned"));
        assert!(!flag(&next, "replan_trigger"));
    }

    /// The plan counter counts plans, not ticks - it is what a dashboard reads
    /// to see how often the system is replanning.
    #[tokio::test]
    async fn the_plan_counter_counts_plans() {
        let (state, operations) = model();
        let base = with_planner_vars(&state, "var:pos == c", "ready");

        let mut state = base.clone();
        for expected in 1..=3 {
            state = state.update(&format!("{SP}_replan_trigger"), true.to_spvalue());
            state = state.update(&format!("{SP}_planner_state"), "ready".to_spvalue());
            state = tick(&state, &operations).await;
            assert_eq!(
                state.get_value(&format!("{SP}_plan_counter"), TARGET),
                Some(expected.to_spvalue())
            );
            // Close the handshake before the next one.
            state = tick(&state, &operations).await;
        }
    }
}

/// The planner ticker's loop, against a real Redis.
///
/// The tick logic is covered above without Redis; what only the loop does is
/// the fast path - reading two booleans and stopping there when no replan is
/// requested. That is the optimisation the module note describes, and the only
/// way to check it is to watch what the runner actually writes.
#[cfg(test)]
mod runner_tests {
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

    /// `pos` walks a -> b -> c, one operation per hop.
    async fn deploy(manager: &Arc<ConnectionManager>) -> Model {
        let mut domain = State::new();
        domain.add_mut(
            SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
            TARGET,
        );

        let hop = |name: &str, from: &str, to: &str| {
            Operation::new(
                name,
                None,
                None,
                None,
                None,
                false,
                vec![Transition::parse(
                    "start",
                    &format!("var:pos == {from}"),
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &domain,
                )],
                vec![Transition::parse(
                    "complete",
                    "true",
                    "true",
                    vec![format!("var:pos <- {to}").as_str()],
                    Vec::<&str>::new(),
                    &domain,
                )],
                vec![],
                vec![],
                vec![],
                vec![],
            )
        };

        let model = Model::new(
            SP,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![hop("a_to_b", "a", "b"), hop("b_to_c", "b", "c")],
        );

        let mut state = generate_runner_state_variables(SP, 0, TARGET);
        state.extend_mut(generate_operation_state_variables(&model, false, TARGET), true);
        state.extend_mut(domain, true);
        state = state.update(&key("planner_state"), "ready".to_spvalue());
        state = state.update(&key("current_goal_predicate"), "var:pos == c".to_spvalue());

        let mut con = manager.get_connection().await;
        StateManager::set_state(&mut con, &state).await;
        model
    }

    fn spawn_runner(manager: &Arc<ConnectionManager>, model: Model) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = planner_ticker(SP, &model, &manager).await;
        })
    }

    async fn plan(con: &mut SPConnection) -> Vec<String> {
        match StateManager::get_sp_value(con, &key("plan")).await {
            Some(SPValue::Array(ArrayOrUnknown::Array(items))) => {
                items.iter().map(|v| v.to_string()).collect()
            }
            _ => vec![],
        }
    }

    /// The whole loop: a trigger goes in, a plan comes out, and the handshake
    /// closes by itself.
    #[tokio::test]
    #[serial]
    async fn a_trigger_produces_a_plan_and_the_handshake_closes() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager).await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &key("replan_trigger"), &true.to_spvalue()).await;

        let deadline = std::time::Instant::now() + Duration::from_millis(5000);
        while std::time::Instant::now() < deadline && plan(&mut con).await.is_empty() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let steps = plan(&mut con).await;
        assert_eq!(steps.len(), 2, "expected a two-step plan, got {steps:?}");

        // Both handshake flags settle back to false on the following tick.
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        while std::time::Instant::now() < deadline {
            let trigger = StateManager::get_sp_value(&mut con, &key("replan_trigger")).await;
            let replanned = StateManager::get_sp_value(&mut con, &key("replanned")).await;
            if trigger == Some(false.to_spvalue()) && replanned == Some(false.to_spvalue()) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(!runner.is_finished());
        runner.abort();

        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("replan_trigger")).await,
            Some(false.to_spvalue())
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("replanned")).await,
            Some(false.to_spvalue())
        );
        assert_eq!(plan(&mut con).await, steps, "the plan survives the handshake");
    }

    /// The fast path. With no replan requested the tick reads two booleans and
    /// stops, so nothing at all changes - this is the loop that runs at the tick
    /// rate on every deployment for as long as no goal changes.
    #[tokio::test]
    #[serial]
    async fn an_untriggered_runner_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager).await;

        let runner = spawn_runner(&manager, model);
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(400)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(!runner.is_finished());
        runner.abort();
        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "an untriggered planner must not write: {:?}",
            before.get_diff_partial_state(&after)
        );
    }

    /// A goal the planner cannot reach comes back as `not_found` rather than
    /// leaving the trigger set forever.
    #[tokio::test]
    #[serial]
    async fn an_unreachable_goal_reports_not_found_through_the_loop() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        let model = deploy(&manager).await;

        StateManager::set_sp_value(
            &mut con,
            &key("current_goal_predicate"),
            &"var:pos == nowhere".to_spvalue(),
        )
        .await;

        let runner = spawn_runner(&manager, model);
        StateManager::set_sp_value(&mut con, &key("replan_trigger"), &true.to_spvalue()).await;

        let deadline = std::time::Instant::now() + Duration::from_millis(5000);
        let mut planner_state = String::new();
        while std::time::Instant::now() < deadline {
            planner_state = match StateManager::get_sp_value(&mut con, &key("planner_state")).await {
                Some(SPValue::String(StringOrUnknown::String(s))) => s,
                other => format!("{other:?}"),
            };
            if planner_state == "not_found" {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        runner.abort();

        assert_eq!(planner_state, "not_found");
        assert!(plan(&mut con).await.is_empty());
    }
}
