//! Key sets for the runners that used to read the whole database.
//!
//! `sop_runner`, `auto_operation_runner` and `planned_operation_runner` each
//! called `StateManager::get_full_state`, which is a `KEYS *` + `MGET`. `KEYS`
//! is O(total keyspace) and blocks the single-threaded Redis server for its
//! whole duration, so three runners ticking at 100-200 ms meant roughly 20
//! blocking full-keyspace scans a second over a keyspace that also holds every
//! transform and every log blob - delaying every other command in the process.
//!
//! They now use `get_state_for_keys` (one `MGET` of a known key list, no
//! keyspace scan) like the planner, goal, timer and tf runners already did.
//!
//! The catch is that reading a variable that is *not* in the state panics (see
//! `State::get_value`), so these key sets have to be complete. Each runner's
//! set is built from two parts:
//!
//!   * a **static** part, derived from the model once before the loop: the
//!     `{sp_id}_*` variables the runner itself reads and writes, plus
//!     `Operation::get_all_var_keys()` over the relevant operations (their
//!     guard and action variables, which are the same for a template operation
//!     and for a uniquified copy of it);
//!   * a **dynamic** part: the six bookkeeping variables that
//!     `process_operation` reads and writes for each *currently active*
//!     operation. Active operations are created at runtime with a `nanoid`
//!     suffix, so this part is rebuilt when the active set changes - not on
//!     every tick.

use crate::*;

/// Suffixes of the five per-operation bookkeeping variables created by
/// [`add_operation_meta_tracking_variables`]. Together with the bare operation
/// name (created by [`add_operation_state_tracking_variable`]) these are what
/// `process_operation` reads and writes for every operation on every tick, and
/// what the runners delete when an operation terminates.
pub const OPERATION_META_SUFFIXES: [&str; 5] = [
    "_information",
    "_elapsed_executing_ms",
    "_elapsed_disabled_ms",
    "_failure_retry_counter",
    "_timeout_retry_counter",
];

/// Append an operation's state variable and its five bookkeeping variables.
pub fn push_operation_keys(keys: &mut Vec<String>, op_name: &str) {
    keys.push(op_name.to_string());
    for suffix in OPERATION_META_SUFFIXES {
        keys.push(format!("{}{}", op_name, suffix));
    }
}

/// Sort and deduplicate a key list. Worth doing once per rebuild: operations
/// typically share most of their variables, and without this the per-tick
/// `MGET` sends the same key many times over.
pub fn normalize_keys(mut keys: Vec<String>) -> Vec<String> {
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// `static_keys` plus the bookkeeping variables of the currently active
/// operations.
pub fn keys_with_active_operations(static_keys: &[String], active_ops: &[String]) -> Vec<String> {
    let mut keys = static_keys.to_vec();
    for op_name in active_ops {
        push_operation_keys(&mut keys, op_name);
    }
    normalize_keys(keys)
}

/// Environment escape hatch: with `MICRO_SP_READ_FULL_STATE` set to `1`/`true`,
/// the three operation runners go back to `StateManager::get_full_state` (a
/// `KEYS *` scan) instead of their key sets.
///
/// This exists because reading a variable that is not in the state panics, so a
/// key set with a hole takes a runner down. The sets below are derived from the
/// model and are meant to be complete, but a downstream package that builds its
/// model in a way this derivation does not see can flip this on and keep
/// running - at the cost of the blocking keyspace scan - instead of being stuck.
/// If you ever need it, that is a bug in the derivation worth reporting.
pub fn read_full_state_enabled() -> bool {
    match std::env::var("MICRO_SP_READ_FULL_STATE") {
        Ok(value) => matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true"),
        Err(_) => false,
    }
}

/// Every state variable referenced anywhere in the model.
///
/// All three runners use this whole union rather than just the variables of the
/// operations they happen to drive. Guards routinely reference variables that
/// another operation group writes (an auto operation reacting to something a
/// planned operation set, a SOP guard reading an interface variable), and
/// splitting the sets per group makes that a runtime panic. The union is still
/// only the model's variables - it does not touch transforms, log blobs or
/// anything else in the keyspace - so it costs a slightly longer `MGET` and
/// saves a whole class of missing-variable failures.
///
/// The *template* operation names are included too: `Operation::eval` reads
/// `{op.name}` to decide whether an operation may start, so those variables are
/// consulted on every tick even when nothing is active.
pub fn model_variable_keys(model: &Model) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();

    for op in model
        .operations
        .iter()
        .chain(model.auto_operations.iter())
        .chain(model.mutexed_auto_operations.iter())
    {
        keys.push(op.name.clone());
        keys.extend(op.get_all_var_keys());
    }

    for transition in &model.auto_transitions {
        keys.extend(transition.get_all_var_keys());
    }

    for sop in &model.sops {
        // `{sop_id}_sop_information` is keyed by the *template* id.
        keys.push(format!("{}_sop_information", sop.id));
        // Guard and action variables are unaffected by uniquifying, so the
        // template's key set covers the running copy too.
        keys.extend(sop.sop.get_all_var_keys());
        for op in get_all_operations_from_sop(&sop.sop) {
            keys.push(op.name.clone());
        }
    }

    normalize_keys(keys)
}

/// Keys `sop_runner` reads on every tick regardless of what is running.
///
/// The per-operation bookkeeping variables of a running SOP are added on
/// activation via [`keys_with_active_operations`], once the operations have
/// been uniquified and their names are known.
pub fn sop_runner_static_keys(sp_id: &str, model: &Model) -> Vec<String> {
    let mut keys = vec![
        format!("{}_sop_state", sp_id),
        format!("{}_sop_enabled", sp_id),
        format!("{}_sop_id", sp_id),
        // read by `Operation::can_be_cancelled` for every operation processed
        format!("{}_dashboard_command", sp_id),
    ];
    keys.extend(model_variable_keys(model));
    normalize_keys(keys)
}

/// Keys `auto_operation_runner` reads on every tick regardless of what is
/// running.
pub fn auto_operation_runner_static_keys(sp_id: &str, model: &Model) -> Vec<String> {
    let mut keys = vec![format!("{}_dashboard_command", sp_id)];
    keys.extend(model_variable_keys(model));
    normalize_keys(keys)
}

/// Keys `planned_operation_runner` reads on every tick regardless of what is
/// running.
///
/// The bookkeeping variables of the plan's steps are added per plan via
/// [`keys_with_active_operations`]; unlike the other two runners this one does
/// not create its plan itself (`planner_ticker` does), so it has to notice the
/// plan changing in the state and rebuild from there.
pub fn plan_runner_static_keys(sp_id: &str, model: &Model) -> Vec<String> {
    let mut keys = vec![
        format!("{}_planner_state", sp_id),
        format!("{}_current_goal_state", sp_id),
        format!("{}_plan_state", sp_id),
        format!("{}_plan_current_step", sp_id),
        format!("{}_plan", sp_id),
        format!("{}_terminated_operations", sp_id),
        // read by `Operation::can_be_cancelled` for every operation processed
        format!("{}_dashboard_command", sp_id),
    ];
    keys.extend(model_variable_keys(model));
    normalize_keys(keys)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashSet;

    const SP_ID: &str = "sp";
    const TARGET: &str = "test";

    /// The variables the model's guards and actions refer to.
    fn model_state() -> State {
        let mut state = State::new();
        for (var, val) in [
            (bv!("trigger"), false.to_spvalue()),
            (v!("action_state"), "initial".to_spvalue()),
            (v!("pose"), "a".to_spvalue()),
            // touched only by a bypass transition - the regression case for
            // `Operation::get_all_var_keys`, which used to skip
            // `bypass_transitions` entirely
            (bv!("bypassed_marker"), false.to_spvalue()),
            // touched only by a cancel transition
            (bv!("cancelled_marker"), false.to_spvalue()),
            // touched only by a failure transition
            (bv!("failed_marker"), false.to_spvalue()),
            // referenced only on the *right-hand side* of an action
            // (`var:pose <- var:rhs_source`), the way a downstream package's
            // own variables usually enter a model
            (v!("rhs_source"), "b".to_spvalue()),
            (v!("rhs_in_array"), "x".to_spvalue()),
            (v!("rhs_in_map_key"), "k".to_spvalue()),
            (v!("rhs_in_map_value"), "v".to_spvalue()),
            (av!("collected"), Vec::<SPValue>::new().to_spvalue()),
            (mv!("mapped"), Vec::<(SPValue, SPValue)>::new().to_spvalue()),
        ] {
            state.add_mut(assign!(var, val), TARGET);
        }
        state
    }

    fn test_operation(name: &str, state: &State) -> Operation {
        Operation::new(
            name,
            None,
            None,
            None,
            None,
            true,
            vec![t!(
                "start",
                "var:trigger == false && var:action_state == initial",
                "true",
                vec!("var:trigger <- true"),
                Vec::<&str>::new(),
                state
            )],
            vec![t!(
                "complete",
                "var:action_state == done",
                "true",
                // `var:pose <- var:rhs_source` reads `rhs_source`; it is not
                // named in any guard, so the only way it reaches a key set is
                // through the action's right-hand side.
                vec!("var:trigger <- false", "var:pose <- var:rhs_source"),
                Vec::<&str>::new(),
                state
            )],
            vec![t!(
                "fail",
                "var:action_state == failed",
                "true",
                vec!("var:failed_marker <- true"),
                Vec::<&str>::new(),
                state
            )],
            vec![],
            vec![t!(
                "bypass",
                "true",
                "true",
                vec!("var:bypassed_marker <- true"),
                Vec::<&str>::new(),
                state
            )],
            vec![t!(
                "cancel",
                "true",
                "true",
                vec!("var:cancelled_marker <- true"),
                Vec::<&str>::new(),
                state
            )],
        )
    }

    fn test_model(state: &State) -> Model {
        Model::new(
            SP_ID,
            vec![],
            vec![test_operation("auto_op", state)],
            vec![test_operation("mutexed_op", state)],
            vec![SOPStruct {
                id: "sop_one".to_string(),
                sop: SOP::Sequence(vec![SOP::Operation(Box::new(test_operation(
                    "sop_op", state,
                )))]),
            }],
            vec![test_operation("planned_op", state)],
        )
    }

    /// Everything a runner could read: the runner variables, the operation
    /// trackers built from the model, and the model's own variables.
    fn full_state(model: &Model, state: &State) -> State {
        let mut full = generate_runner_state_variables(SP_ID, 0, TARGET);
        full.extend_mut(
            generate_operation_state_variables(model, false, TARGET),
            true,
        );
        full.extend_mut(state.clone(), true);
        full
    }

    /// What a runner actually receives from `get_state_for_keys`: only the
    /// requested keys, and only those that exist.
    fn restrict(full: &State, keys: &[String]) -> State {
        let mut restricted = State::new();
        for key in keys {
            if let Some(assignment) = full.state.get(key) {
                restricted.add_mut(assignment.clone(), TARGET);
            }
        }
        restricted
    }

    /// The regression the runners actually tripped over: an action's
    /// right-hand side is *read* when the action is applied
    /// (`Action::assign_mut` -> `SPWrapped::evaluate` -> `State::get_value`),
    /// but `Transition::get_all_var_keys` used to collect only the action's
    /// target variable. A downstream package that assigns from its own
    /// variables therefore had them missing from every runner key set.
    #[test]
    fn action_right_hand_side_variables_are_in_the_key_set() {
        let state = model_state();

        let plain = t!(
            "assign_from_variable",
            "true",
            "true",
            vec!("var:pose <- var:rhs_source"),
            Vec::<&str>::new(),
            &state
        );
        let keys: HashSet<String> = plain.get_all_var_keys().into_iter().collect();
        assert!(keys.contains("pose"), "the action target must be included");
        assert!(
            keys.contains("rhs_source"),
            "the variable the action assigns *from* must be included"
        );

        // The same applies to a runner action, and to variables nested inside
        // an array or map right-hand side.
        let nested = Transition::new(
            "nested",
            Predicate::TRUE,
            Predicate::TRUE,
            vec![Action::new(
                av!("collected"),
                SPWrapped::Array(vec![
                    SPWrapped::SPVariable(v!("rhs_in_array")),
                    SPWrapped::SPValue("literal".to_spvalue()),
                ]),
            )],
            vec![Action::new(
                mv!("mapped"),
                SPWrapped::Map(vec![(
                    SPWrapped::SPVariable(v!("rhs_in_map_key")),
                    SPWrapped::SPVariable(v!("rhs_in_map_value")),
                )]),
            )],
        );
        let keys: HashSet<String> = nested.get_all_var_keys().into_iter().collect();
        for key in [
            "collected",
            "rhs_in_array",
            "mapped",
            "rhs_in_map_key",
            "rhs_in_map_value",
        ] {
            assert!(keys.contains(key), "missing '{key}' from a nested right-hand side");
        }
    }

    /// A variable that only appears on the right-hand side of an action has to
    /// survive all the way into the runners' key sets, not just into
    /// `Transition::get_all_var_keys`.
    #[test]
    fn runner_key_sets_include_action_right_hand_side_variables() {
        let state = model_state();
        let model = test_model(&state);

        for (name, keys) in [
            ("sop_runner", sop_runner_static_keys(SP_ID, &model)),
            (
                "auto_operation_runner",
                auto_operation_runner_static_keys(SP_ID, &model),
            ),
            ("plan_runner", plan_runner_static_keys(SP_ID, &model)),
        ] {
            let keys: HashSet<String> = keys.into_iter().collect();
            assert!(
                keys.contains("rhs_source"),
                "{name} is missing an action's right-hand side variable"
            );
        }
    }

    /// Every runner reads the whole model's variables, not just those of the
    /// operations it drives - guards routinely reference variables another
    /// operation group writes.
    #[test]
    fn every_runner_sees_every_model_variable() {
        let state = model_state();
        let model = test_model(&state);
        let model_keys: HashSet<String> = model_variable_keys(&model).into_iter().collect();

        for (name, keys) in [
            ("sop_runner", sop_runner_static_keys(SP_ID, &model)),
            (
                "auto_operation_runner",
                auto_operation_runner_static_keys(SP_ID, &model),
            ),
            ("plan_runner", plan_runner_static_keys(SP_ID, &model)),
        ] {
            let keys: HashSet<String> = keys.into_iter().collect();
            for key in &model_keys {
                assert!(
                    keys.contains(key),
                    "{name} is missing model variable '{key}'"
                );
            }
        }
    }

    #[test]
    fn bypass_and_cancel_variables_are_in_the_key_set() {
        let state = model_state();
        let op = test_operation("op", &state);
        let keys: HashSet<String> = op.get_all_var_keys().into_iter().collect();

        assert!(
            keys.contains("bypassed_marker"),
            "variables only touched by a bypass transition must be included"
        );
        assert!(
            keys.contains("cancelled_marker"),
            "variables only touched by a cancel transition must be included"
        );
        assert!(keys.contains("trigger"));
        assert!(keys.contains("action_state"));
        assert!(keys.contains("pose"));
    }

    #[test]
    fn key_sets_are_sorted_and_deduplicated() {
        let state = model_state();
        let model = test_model(&state);

        for keys in [
            sop_runner_static_keys(SP_ID, &model),
            auto_operation_runner_static_keys(SP_ID, &model),
            plan_runner_static_keys(SP_ID, &model),
        ] {
            let mut sorted = keys.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(keys, sorted, "key sets must be sorted and deduplicated");
        }
    }

    #[test]
    fn static_key_sets_cover_the_runner_variables_each_runner_reads() {
        let state = model_state();
        let model = test_model(&state);

        let sop_keys: HashSet<String> = sop_runner_static_keys(SP_ID, &model).into_iter().collect();
        for key in [
            "sp_sop_state",
            "sp_sop_enabled",
            "sp_sop_id",
            "sp_dashboard_command",
            "sop_one_sop_information",
            "trigger",
            "bypassed_marker",
        ] {
            assert!(sop_keys.contains(key), "sop_runner is missing '{key}'");
        }

        let auto_keys: HashSet<String> = auto_operation_runner_static_keys(SP_ID, &model)
            .into_iter()
            .collect();
        for key in [
            "sp_dashboard_command",
            // `Operation::eval` reads the template's own tracker every tick
            "op_auto_op",
            "op_mutexed_op",
            "trigger",
            "bypassed_marker",
        ] {
            assert!(auto_keys.contains(key), "auto runner is missing '{key}'");
        }

        let plan_keys: HashSet<String> =
            plan_runner_static_keys(SP_ID, &model).into_iter().collect();
        for key in [
            "sp_planner_state",
            "sp_current_goal_state",
            "sp_plan_state",
            "sp_plan_current_step",
            "sp_plan",
            "sp_terminated_operations",
            "sp_dashboard_command",
            "trigger",
            "bypassed_marker",
        ] {
            assert!(plan_keys.contains(key), "plan runner is missing '{key}'");
        }
    }

    #[test]
    fn active_operation_keys_cover_the_bookkeeping_variables() {
        let keys: HashSet<String> =
            keys_with_active_operations(&[], &["op_x_abc".to_string()]).into_iter().collect();

        for key in [
            "op_x_abc",
            "op_x_abc_information",
            "op_x_abc_elapsed_executing_ms",
            "op_x_abc_elapsed_disabled_ms",
            "op_x_abc_failure_retry_counter",
            "op_x_abc_timeout_retry_counter",
        ] {
            assert!(keys.contains(key), "missing bookkeeping key '{key}'");
        }
    }

    /// The real guarantee we need: a state restricted to a runner's key set is
    /// enough to drive an operation through `process_operation` without a
    /// missing-variable panic. `State::get_value` panics on an absent key, so a
    /// key set with a hole is a crash at runtime, not a degraded read.
    #[tokio::test]
    async fn a_restricted_state_can_drive_process_operation() {
        let state = model_state();
        let model = test_model(&state);
        let full = full_state(&model, &state);

        // An operation activated at runtime, as the runners create them.
        let active = "op_auto_op_abc123".to_string();
        let full = add_operation_meta_tracking_variables(
            &vec![active.clone()],
            &full,
            false,
            TARGET,
        );
        let full = add_operation_state_tracking_variable(&vec![active.clone()], &full, TARGET);

        let keys = keys_with_active_operations(
            &auto_operation_runner_static_keys(SP_ID, &model),
            &[active.clone()],
        );
        let restricted = restrict(&full, &keys);

        let mut operation = model.auto_operations[0].clone();
        operation.name = active.clone();

        // Initial -> Executing, then an executing tick, then failure and
        // bypass - the path that reads the bookkeeping variables, the
        // dashboard command and the failure/bypass transitions' variables.
        let mut tick_state = restricted;
        for _ in 0..2 {
            tick_state = running::process_operation::process_operation(
                SP_ID,
                tick_state,
                &operation,
                running::process_operation::OperationProcessingType::Automatic,
                None,
                None,
                // a 200 ms tick, matching the runners' cadence
                200,
                TARGET,
            )
            .await;
        }
        assert_eq!(
            tick_state.get_value(&active, TARGET),
            Some(OperationState::Executing.to_spvalue()),
            "the operation should have started"
        );

        tick_state.update_mut("action_state", "failed".to_spvalue());
        tick_state = running::process_operation::process_operation(
            SP_ID,
            tick_state,
            &operation,
            running::process_operation::OperationProcessingType::Automatic,
            None,
            None,
            200,
            TARGET,
        )
        .await;
        assert_eq!(
            tick_state.get_value("failed_marker", TARGET),
            Some(true.to_spvalue()),
            "the failure transition's action must have been applied"
        );

        tick_state = operation.bypass(&tick_state, TARGET);
        assert_eq!(
            tick_state.get_value("bypassed_marker", TARGET),
            Some(true.to_spvalue()),
            "the bypass transition's action must have been applied"
        );
    }
}
