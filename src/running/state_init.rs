use std::time::{SystemTime, UNIX_EPOCH};

use crate::*;

// If coverability_tracking is true, generate variables to track how many
// times an operation has entered its different running states
pub fn generate_runner_state_variables(
    name: &str,
    number_of_timers: u64,
    log_target: &str,
) -> State {
    let mut state = State::new();

    // Define variables
    let runner_state = v!(&&format!("{}_runner_state", name)); // does nothing for now
    let current_goal_predicate = v!(&&format!("{}_current_goal_predicate", name)); // goal as a string predicate
    let current_goal_id = v!(&&format!("{}_current_goal_id", name)); // goal as a string predicate
    let current_goal_state = v!(&&format!("{}_current_goal_state", name)); // goal as a string predicate
    let plan = av!(&&format!("{}_plan", name)); // plan as array of string
    let plan_id = v!(&&format!("{}_plan_id", name)); // unique plan id
    let plan_counter = iv!(&&format!("{}_plan_counter", name)); // How many times has a plan been found
    let plan_exists = bv!(&&format!("{}_plan_exists", name)); // does nothing for now
    let plan_name = v!(&&format!("{}_plan_name", name)); // same as model name, should add nanoid!
    let plan_state = v!(&&format!("{}_plan_state", name)); // Initial, Executing, Failed, Completed, Unknown
    let planner_state = v!(&&format!("{}_planner_state", name)); // Initial, Executing, Failed, Completed, Unknown
    let plan_duration = fv!(&&format!("{}_plan_duration", name)); // does nothing for now
    let plan_current_step = iv!(&&format!("{}_plan_current_step", name)); // Index of the currently exec. operation in the plan
    let planner_information = v!(&&format!("{}_planner_information", name)); // current information about the plan
    let plan_runner_information = v!(&&format!("{}_plan_runner_information", name)); // current information about the plan
    let goal_runner_information = v!(&&format!("{}_goal_runner_information", name)); // current information about the plan
    let sop_runner_information = v!(&&format!("{}_sop_runner_information", name)); // current information about the plan
    let main_runner_information = v!(&&format!("{}_main_runner_information", name)); // current information about the plan
    let goal_scheduler_information = v!(&&format!("{}_goal_scheduler_information", name)); // current information about the plan
    let replanned = bv!(&&format!("{}_replanned", name)); // boolean for tracking the planner triggering
    let replan_for_same_goal = bv!(&&format!("{}_replan_for_same_goal", name));
    let replan_counter_total = iv!(&&format!("{}_replan_counter_total", name)); // How many times has the planner been called
    let replan_counter = iv!(&&format!("{}_replan_counter", name)); // How many times has the planner tried to replan for the same problem
    let replan_fail_counter = iv!(&&format!("{}_replan_fail_counter", name)); // How many times has the planner failed in
    let replan_trigger = bv!(&&format!("{}_replan_trigger", name)); // boolean for tracking the planner triggering
    let incoming_goals = av!(&&format!("{}_incoming_goals", name));
    let scheduled_goals = av!(&&format!("{}_scheduled_goals", name));
    let sop_enabled = bv!(&&format!("{}_sop_enabled", name));
    // let sop_request_state = v!(&&format!("{}_sop_request_state", name));
    let sop_current_step = iv!(&&format!("{}_sop_current_step", name));
    let sop_id = v!(&&format!("{}_sop_id", name));
    let sop_state = v!(&&format!("{}_sop_state", name));
    let sop_stack = v!(&&format!("{}_sop_stack", name));
    let start_time = iv!(&&format!("{}_start_time", name));
    let tf_request_trigger = bv!(&&format!("{}_tf_request_trigger", name));
    let tf_request_state = v!(&&format!("{}_tf_request_state", name));
    let tf_command = v!(&&format!("{}_tf_command", name));
    let tf_parent = v!(&&format!("{}_tf_parent", name));
    let tf_child = v!(&&format!("{}_tf_child", name));
    let tf_lookup_result = tfv!(&&format!("{}_tf_lookup_result", name));
    let tf_insert_transforms = av!(&&format!("{}_tf_insert_transforms", name));
    // let time_request_trigger = bv!(&&format!("{}_time_request_trigger", name));
    // let time_request_state = v!(&&format!("{}_time_request_state", name));
    // let time_command = v!(&&format!("{}_time_command", name));
    // let time_duration_ms = iv!(&&format!("{}_time_duration_ms", name));
    // let time_elapsed_ms = iv!(&&format!("{}_time_elapsed_ms", name));

    let empty = bv!(&&format!("empty"));
    state.add_mut(
        assign!(
            empty,
            SPValue::Bool(BoolOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    // add the timer variables to the state
    for timer_id in 1..=number_of_timers {
        let timer_request_trigger = bv!(&&format!("{}_timer_{}_request_trigger", name, timer_id));
        let timer_request_state = v!(&&format!("{}_timer_{}_request_state", name, timer_id));
        let timer_command = v!(&&format!("{}_timer_{}_command", name, timer_id));
        let timer_duration_ms = iv!(&&format!("{}_timer_{}_duration_ms", name, timer_id));
        let timer_elapsed_ms = iv!(&&format!("{}_timer_{}_elapsed_ms", name, timer_id));

        state.add_mut(
            assign!(timer_request_trigger, SPValue::Bool(BoolOrUnknown::Bool(false))),
            &log_target,
        );
        state.add_mut(
            assign!(
                timer_request_state,
                SPValue::String(StringOrUnknown::String("initial".to_string()))
            ),
            &log_target,
        );
        state.add_mut(
            assign!(timer_command, SPValue::String(StringOrUnknown::UNKNOWN)),
            &log_target,
        );
        state.add_mut(
            assign!(timer_duration_ms, SPValue::Int64(IntOrUnknown::UNKNOWN)),
            &log_target,
        );
        state.add_mut(
            assign!(timer_elapsed_ms, SPValue::Int64(IntOrUnknown::UNKNOWN)),
            &log_target,
        );
    }

    let terminated_operations = av!(&&format!("{}_terminated_operations", name));
    state.add_mut(
        assign!(
            terminated_operations,
            SPValue::Array(ArrayOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    let active_auto_operations = av!(&&format!("{}_active_auto_operations", name));
    state.add_mut(
        assign!(
            active_auto_operations,
            SPValue::Array(ArrayOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    // Pause, Stop, Run/Play/Continue
    let sp_dashboard_command = v!(&&format!("{}_dashboard_command", name));
    state.add_mut(
        assign!(
            sp_dashboard_command,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    // Initialize values
    state.add_mut(
        assign!(runner_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(
            current_goal_predicate,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(current_goal_id, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(
            current_goal_state,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(plan, SPValue::Array(ArrayOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(plan_exists, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(plan_name, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(plan_id, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(plan_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(planner_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(plan_duration, SPValue::Float64(FloatOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(plan_current_step, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(
            planner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            plan_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            goal_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            sop_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            main_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            goal_scheduler_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(replanned, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(replan_for_same_goal, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(replan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(replan_counter_total, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(plan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(replan_fail_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(replan_trigger, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(incoming_goals, SPValue::Array(ArrayOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(scheduled_goals, SPValue::Array(ArrayOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(sop_id, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(sop_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(sop_stack, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(sop_enabled, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(start_time, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(sop_current_step, SPValue::Int64(IntOrUnknown::Int64(0))),
        &log_target,
    );
    state.add_mut(
        assign!(
            tf_request_trigger,
            SPValue::Bool(BoolOrUnknown::Bool(false))
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            tf_request_state,
            SPValue::String(StringOrUnknown::String("initial".to_string()))
        ),
        &log_target,
    );
    state.add_mut(
        assign!(tf_command, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(tf_parent, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(tf_child, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(
            tf_lookup_result,
            SPValue::Transform(TransformOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            tf_insert_transforms,
            SPValue::Array(ArrayOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    // Define variables to keep track of the processes
    let state_manager_online = bv!(&&format!("state_manager_online"));
    let auto_transition_runner_online = bv!(&&format!("{}_auto_transition_runner_online", name));
    let planner_ticker_online = bv!(&&format!("{}_planner_ticker_online", name));
    let operation_planner_online = bv!(&&format!("{}_operation_planner_online", name));
    let operation_runner_online = bv!(&&format!("{}_operation_runner_online", name));
    state.add_mut(
        assign!(state_manager_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(
            auto_transition_runner_online,
            SPValue::Bool(BoolOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(planner_ticker_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state.add_mut(
        assign!(
            operation_planner_online,
            SPValue::Bool(BoolOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state.add_mut(
        assign!(
            operation_runner_online,
            SPValue::Bool(BoolOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    state
}

pub fn all_values_to_unknown(state: &State) -> State {
    let mut new_state = state.clone();
    for (key, value) in &state.state {
        let unknown_value = match value.val {
            SPValue::Bool(_) => SPValue::Bool(BoolOrUnknown::UNKNOWN),
            SPValue::Float64(_) => SPValue::Float64(FloatOrUnknown::UNKNOWN),
            SPValue::Int64(_) => SPValue::Int64(IntOrUnknown::UNKNOWN),
            SPValue::String(_) => SPValue::String(StringOrUnknown::UNKNOWN),
            SPValue::Time(_) => SPValue::Time(TimeOrUnknown::UNKNOWN),
            SPValue::Array(_) => SPValue::Array(ArrayOrUnknown::UNKNOWN),
            SPValue::Map(_) => SPValue::Map(MapOrUnknown::UNKNOWN),
            SPValue::Transform(_) => SPValue::Transform(TransformOrUnknown::UNKNOWN),
        };
        new_state = new_state.update(&key, unknown_value);
    }
    new_state
}

pub fn get_all_operations_from_sop(sop: &SOP) -> Vec<Operation> {
    let mut operations = Vec::new();
    get_all_operations_recursive(sop, &mut operations);
    operations
}

fn get_all_operations_recursive(sop: &SOP, operations: &mut Vec<Operation>) {
    match sop {
        // Base case: We found an operation. Clone it and add it to our list.
        SOP::Operation(op) => {
            operations.push(*op.clone());
        }
        // Recursive step: This is a container. Iterate through its children
        // and call this function on each of them.
        SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
            for child_sop in sops {
                get_all_operations_recursive(child_sop, operations);
            }
        }
    }
}

pub fn generate_operation_state_variables(
    model: &Model,
    coverability_tracking: bool,
    log_target: &str,
) -> State {
    let mut state = State::new();
    let mut operation_trackers: Vec<String> = vec!();
    // operations should be put in the initial state once they are part of the plan

    for sop in &model.sops {
        let ops_in_sop = get_all_operations_from_sop(&sop.sop);
        let sop_information = v!(&&format!("{}_sop_information", sop.id));
        state.add_mut(
            assign!(sop_information, SPValue::String(StringOrUnknown::UNKNOWN)),
            &log_target,
        );
        ops_in_sop.iter().for_each(|x| operation_trackers.push(x.name.clone()));
    }

    model.operations.iter().for_each(|x| operation_trackers.push(x.name.clone()));
    model.auto_operations.iter().for_each(|x| operation_trackers.push(x.name.clone()));
    model.mutexed_auto_operations.iter().for_each(|x| operation_trackers.push(x.name.clone()));
    operation_trackers.sort(); 
    operation_trackers.dedup();
    state = add_operation_state_tracking_variable(
        &operation_trackers,
        &state,
        &log_target,
    ); // remove later for unique on the fly



    for transition in &model.auto_transitions {
        if coverability_tracking {
            let taken = iv!(&&format!("transition_{}_taken", transition.name));
            state.add_mut(assign!(taken, 0.to_spvalue()), &log_target)
        }
    }

    // for operation in &model.auto_operations {
    //     let operation_state = v!(&&format!("{}", operation.name));
    //     state.add_mut(assign!(operation_state, "initial".to_spvalue()));
    //     if coverability_tracking {
    //         let taken = iv!(&&format!("{}_taken", operation.name));
    //         state.add_mut(assign!(taken, 0.to_spvalue()))
    //     }
    // }

    state
}

// pub fn reset_all_operations(state: &State) -> State {
//     let state = state.clone();
//     let mut mut_state = state.clone();
//     state.state.iter().for_each(|(k, _)| {
//         if k.starts_with("operation_") && k.ends_with("_state") {
//             mut_state = mut_state.update(&k, "initial".to_spvalue());
//         }
//     });
//     mut_state
// }

pub fn reset_all_operations(state: &State, model: &Model) -> State {
    let state = state.clone();
    let mut mut_state = state.clone();
    for op in &model.operations {
        mut_state = mut_state.update(&op.name, "initial".to_spvalue());
        // for all op instances (for now, we will have to remove these from the state when exec finishes)
        state.state.iter().for_each(|(k, _)| {
            if k.starts_with(&op.name)
                && !k.ends_with("_information")
                && !k.ends_with("_retry_counter")
                && !k.ends_with("_elapsed_executing_ms")
                && !k.ends_with("_elapsed_disabled_ms")
            {
                mut_state = mut_state.update(&k, "initial".to_spvalue());
            }
        });
    }

    for sop_struct in &model.sops {
        let operations = sop_struct.sop.get_all_operation_names();
        for op in operations {
            mut_state = mut_state.update(&op, "initial".to_spvalue());
        }
        // for all op instances (for now, we will have to remove these from the state when exec finishes)
        state.state.iter().for_each(|(k, _)| {
            if k.starts_with(&sop_struct.id)
                && !k.ends_with("_information")
                && !k.ends_with("_retry_counter")
                && !k.ends_with("_elapsed_executing_ms")
                && !k.ends_with("_elapsed_disabled_ms")
            {
                mut_state = mut_state.update(&k, "initial".to_spvalue());
            }
        });
    }

    for (key, _) in state.state {
        if key.ends_with("_request_state") {
            mut_state = mut_state.update(&key, "initial".to_spvalue());
        }
        if key.ends_with("_request_trigger") {
            mut_state = mut_state.update(&key, false.to_spvalue());
        }
    }

    mut_state
}

pub fn now_as_millis_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// If an operation has to be generated per item or per order
// fn fill_operation_parameters(op: Operation, parameter: &str, replacement: &str) -> Operation {
//     let mut mut_op = op.clone();
//     mut_op.name = op.name.replace(parameter, replacement);
//     mut_op.precondition.actions = op
//         .precondition
//         .actions
//         .iter()
//         .map(|x| {
//             if x.var_or_val == parameter.wrap() {
//                 Action::new(x.var.clone(), replacement.wrap())
//             } else {
//                 x.to_owned()
//             }
//         })
//         .collect();
//     mut_op
// }

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_model() {
        // let model = Model::new("ASDF", vec![], vec![]);
        let _ = generate_runner_state_variables("asdf", 1, "test");
    }
}

/// State initialisation.
///
/// This module decides what exists in Redis before any runner starts, and every
/// runner then reads a key set derived from the same model. The two have to
/// agree exactly: a variable a runner reads but that initialisation does not
/// create is not a default, it is a panic in that runner's task (see
/// `State::get_value`). So the property worth testing here is not "does it
/// create some variables" but "does it create *every* variable the runners ask
/// for" - which is what the first test below checks against
/// `running::runner_keys`.
#[cfg(test)]
mod state_init_tests {
    use crate::*;

    const SP: &str = "sp";
    const TARGET: &str = "test";

    fn domain() -> State {
        State::from_vec(&vec![
            (SPVariable::new("a", SPValueType::Bool), false.to_spvalue()),
            (SPVariable::new("b", SPValueType::Bool), false.to_spvalue()),
        ])
    }

    fn operation(name: &str, flag: &str, state: &State) -> Operation {
        Operation::new(
            name,
            None,
            None,
            None,
            None,
            false,
            vec![Transition::parse(
                "start",
                &format!("var:{flag} == false"),
                "true",
                vec![format!("var:{flag} <- true").as_str()],
                Vec::<&str>::new(),
                state,
            )],
            vec![Transition::parse(
                "complete",
                &format!("var:{flag} == true"),
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        )
    }

    fn model(state: &State) -> Model {
        Model::new(
            SP,
            vec![Transition::parse(
                "beat",
                "var:a == false",
                "true",
                vec!["var:a <- true"],
                Vec::<&str>::new(),
                state,
            )],
            vec![operation("auto", "a", state)],
            vec![operation("mutexed", "b", state)],
            vec![SOPStruct {
                id: "the_sop".to_string(),
                sop: SOP::Sequence(vec![SOP::Operation(Box::new(operation(
                    "in_sop", "a", state,
                )))]),
            }],
            vec![operation("planned", "b", state)],
        )
    }

    /// The contract between initialisation and every runner's read set. A key
    /// in `*_static_keys` that initialisation does not create takes that runner
    /// down on its first tick.
    #[test]
    fn initialisation_covers_every_key_the_runners_read() {
        let domain = domain();
        let model = model(&domain);

        let mut state = generate_runner_state_variables(SP, 2, TARGET);
        state.extend_mut(generate_operation_state_variables(&model, false, TARGET), true);
        state.extend_mut(domain, true);
        // Not model-derived, so a consumer has to declare it - which is exactly
        // the kind of thing this test is here to make visible.
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{SP}_dashboard_command"), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );

        for (label, keys) in [
            ("sop_runner", sop_runner_static_keys(SP, &model)),
            (
                "auto_operation_runner",
                auto_operation_runner_static_keys(SP, &model),
            ),
            ("plan_runner", plan_runner_static_keys(SP, &model)),
        ] {
            let missing: Vec<&String> = keys.iter().filter(|k| !state.contains(k)).collect();
            assert!(
                missing.is_empty(),
                "{label} reads keys that initialisation does not create: {missing:?}"
            );
        }
    }

    /// Every operation in the model - planned, auto, mutexed, and the ones
    /// inside a SOP - gets a state variable, and every SOP gets an information
    /// variable keyed by its template id.
    #[test]
    fn every_operation_in_the_model_gets_a_state_variable() {
        let domain = domain();
        let model = model(&domain);
        let state = generate_operation_state_variables(&model, false, TARGET);

        for name in ["op_planned", "op_auto", "op_mutexed", "in_sop"] {
            assert!(state.contains(name), "{name} has no state variable");
            assert_eq!(
                state.get_value(name, TARGET),
                Some("initial".to_spvalue()),
                "{name} should start in 'initial'"
            );
        }
        assert!(state.contains("the_sop_sop_information"));
    }

    /// Coverability tracking adds a counter per auto transition. It is off by
    /// default, and the difference is exactly those counters.
    #[test]
    fn coverability_tracking_adds_transition_counters() {
        let domain = domain();
        let model = model(&domain);

        let without = generate_operation_state_variables(&model, false, TARGET);
        let with = generate_operation_state_variables(&model, true, TARGET);

        assert!(!without.contains("transition_beat_taken"));
        assert!(with.contains("transition_beat_taken"));
        assert_eq!(with.get_value("transition_beat_taken", TARGET), Some(0.to_spvalue()));
    }

    /// The timer variables scale with the number of timers asked for, and are
    /// exactly the ones `time_interface_runner` reads.
    #[test]
    fn the_timer_variables_match_the_number_of_timers() {
        let two = generate_runner_state_variables(SP, 2, TARGET);

        for timer in 1..=2 {
            for suffix in [
                "request_trigger",
                "request_state",
                "command",
                "duration_ms",
                "elapsed_ms",
            ] {
                let key = format!("{SP}_timer_{timer}_{suffix}");
                assert!(two.contains(&key), "{key} missing");
            }
        }
        assert!(!two.contains(&format!("{SP}_timer_3_request_trigger")));

        let none = generate_runner_state_variables(SP, 0, TARGET);
        assert!(!none.contains(&format!("{SP}_timer_1_request_trigger")));
    }

    /// The bookkeeping variables `process_operation` reads for every operation
    /// it drives. All five have to exist or the first tick panics.
    #[test]
    fn the_operation_meta_variables_are_all_created() {
        let ops = vec!["op_one".to_string(), "op_two".to_string()];
        let state = add_operation_meta_tracking_variables(&ops, &State::new(), false, TARGET);

        for op in &ops {
            for suffix in [
                "_information",
                "_elapsed_executing_ms",
                "_elapsed_disabled_ms",
                "_failure_retry_counter",
                "_timeout_retry_counter",
            ] {
                assert!(state.contains(&format!("{op}{suffix}")), "{op}{suffix} missing");
            }
        }

        // They start UNKNOWN rather than zero, and the accessors turn that into
        // the zero the runners then count up from.
        assert_eq!(state.get_int_or_default_to_zero("op_one_elapsed_executing_ms", TARGET), 0);

        let covered = add_operation_meta_tracking_variables(&ops, &State::new(), true, TARGET);
        assert!(covered.contains("op_one_visited_executing"));
        assert!(!state.contains("op_one_visited_executing"));
    }

    /// `all_values_to_unknown` keeps every variable and its *type* while
    /// clearing the value - it is the "we lost track of the world" reset, not a
    /// delete.
    #[test]
    fn all_values_to_unknown_clears_values_but_keeps_the_variables() {
        let state = State::from_vec(&vec![
            (SPVariable::new("b", SPValueType::Bool), true.to_spvalue()),
            (SPVariable::new("i", SPValueType::Int64), 7.to_spvalue()),
            (SPVariable::new("f", SPValueType::Float64), 1.5.to_spvalue()),
            (SPVariable::new("s", SPValueType::String), "x".to_spvalue()),
            (
                SPVariable::new("arr", SPValueType::Array),
                vec![1.to_spvalue()].to_spvalue(),
            ),
            (
                SPVariable::new("map", SPValueType::Map),
                vec![("k".to_spvalue(), "v".to_spvalue())].to_spvalue(),
            ),
            (
                SPVariable::new("t", SPValueType::Time),
                std::time::SystemTime::now().to_spvalue(),
            ),
            (
                SPVariable::new("tf", SPValueType::Transform),
                SPTransformStamped {
                    active_transform: true,
                    enable_transform: true,
                    time_stamp: std::time::SystemTime::now(),
                    parent_frame_id: "world".to_string(),
                    child_frame_id: "robot".to_string(),
                    transform: SPTransform::default(),
                    metadata: MapOrUnknown::UNKNOWN,
                }
                .to_spvalue(),
            ),
        ]);

        let cleared = all_values_to_unknown(&state);

        assert_eq!(cleared.state.len(), state.state.len(), "nothing is removed");
        let expected = [
            ("b", SPValue::Bool(BoolOrUnknown::UNKNOWN)),
            ("i", SPValue::Int64(IntOrUnknown::UNKNOWN)),
            ("f", SPValue::Float64(FloatOrUnknown::UNKNOWN)),
            ("s", SPValue::String(StringOrUnknown::UNKNOWN)),
            ("arr", SPValue::Array(ArrayOrUnknown::UNKNOWN)),
            ("map", SPValue::Map(MapOrUnknown::UNKNOWN)),
            ("t", SPValue::Time(TimeOrUnknown::UNKNOWN)),
            ("tf", SPValue::Transform(TransformOrUnknown::UNKNOWN)),
        ];
        for (name, unknown) in expected {
            assert_eq!(
                cleared.get_value(name, TARGET),
                Some(unknown),
                "{name} should have been cleared to its type's UNKNOWN"
            );
        }
    }

    /// `get_all_operations_from_sop` is the traversal every caller actually
    /// uses, and unlike `SOP::get_all_operation_names` it reaches through
    /// branches - see the note on that one in `modelling::sops`.
    #[test]
    fn get_all_operations_from_sop_reaches_every_leaf() {
        let domain = domain();
        let tree = SOP::Sequence(vec![
            SOP::Operation(Box::new(operation("one", "a", &domain))),
            SOP::Parallel(vec![
                SOP::Operation(Box::new(operation("two", "b", &domain))),
                SOP::Alternative(vec![SOP::Operation(Box::new(operation("three", "a", &domain)))]),
            ]),
        ]);

        let names: Vec<String> = get_all_operations_from_sop(&tree)
            .iter()
            .map(|o| o.name.clone())
            .collect();
        assert_eq!(names, vec!["one", "two", "three"]);

        assert!(get_all_operations_from_sop(&SOP::Sequence(vec![])).is_empty());
    }

    /// `reset_all_operations` puts the planned operations and every request
    /// interface back to their starting values - it is what a "reset the
    /// system" button calls.
    #[test]
    fn reset_all_operations_resets_operations_and_request_interfaces() {
        let domain = domain();
        let model = model(&domain);

        let mut state = generate_operation_state_variables(&model, false, TARGET);
        state = state.update("op_planned", "executing".to_spvalue());
        // A running instance of that operation, as the plan runner creates it.
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("op_planned_A1b2C3d4E5", SPValueType::String),
                "failed".to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("op_planned_A1b2C3d4E5_information", SPValueType::String),
                "keep me".to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("gripper_request_state", SPValueType::String),
                "succeeded".to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("gripper_request_trigger", SPValueType::Bool),
                true.to_spvalue(),
            ),
            TARGET,
        );

        let reset = reset_all_operations(&state, &model);

        assert_eq!(reset.get_value("op_planned", TARGET), Some("initial".to_spvalue()));
        assert_eq!(
            reset.get_value("op_planned_A1b2C3d4E5", TARGET),
            Some("initial".to_spvalue()),
            "running instances of the operation are reset too"
        );
        assert_eq!(
            reset.get_value("op_planned_A1b2C3d4E5_information", TARGET),
            Some("keep me".to_spvalue()),
            "the bookkeeping variables are excluded by suffix"
        );
        assert_eq!(
            reset.get_value("gripper_request_state", TARGET),
            Some("initial".to_spvalue())
        );
        assert_eq!(
            reset.get_value("gripper_request_trigger", TARGET),
            Some(false.to_spvalue())
        );
    }

    /// BUG (consequence of `SOP::get_all_operation_names` losing everything
    /// below a branch - see `modelling::sops`): `reset_all_operations` iterates
    /// that traversal to reset a SOP's operations, so it resets none of them.
    ///
    /// A SOP operation left in a terminal state across a reset means the SOP
    /// reports as already finished the next time it is enabled.
    #[test]
    fn reset_all_operations_does_not_reset_a_sops_operations() {
        let domain = domain();
        let model = model(&domain);

        let mut state = generate_operation_state_variables(&model, false, TARGET);
        state = state.update("in_sop", "terminated_completed".to_spvalue());

        let reset = reset_all_operations(&state, &model);

        assert_eq!(
            reset.get_value("in_sop", TARGET),
            Some("terminated_completed".to_spvalue()),
            "if this now reads 'initial' the get_all_operation_names bug is fixed"
        );
    }

    /// `now_as_millis_i64` is used for the runner start time; the only thing
    /// worth pinning is that it is a real, monotonically-sane wall clock rather
    /// than the `0` its error path falls back to.
    #[test]
    fn now_as_millis_returns_a_plausible_wall_clock() {
        let first = now_as_millis_i64();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let second = now_as_millis_i64();

        // Some time after 2020-01-01, which is when the error fallback of 0
        // would be obvious.
        assert!(first > 1_577_836_800_000, "got {first}");
        assert!(second >= first, "time went backwards: {first} -> {second}");
    }
}

// PERF: calls `State::add` five times per operation, and `add` currently clones
// the entire state map *twice* per call (see the note there). Registering a
// 20-operation SOP therefore performs ~200 full-state copies inside a single
// tick - a latency spike at exactly the moment the SOP starts, which is when it
// is most visible. Called again from `handle_replan_request` for every step of
// a new plan. Suggested: build the new assignments into a small `State` (or a
// `Vec<SPAssignment>`) and merge once, or use an in-place `add_mut`.
pub fn add_operation_meta_tracking_variables(
    ops: &Vec<String>,
    state: &State,
    coverability_tracking: bool,
    log_target: &str,
) -> State {
    let mut state = state.clone();
    for operation in ops {
        let operation_information = v!(&&format!("{}_information", operation));
        let operation_elapsed_executing_ms = iv!(&&format!("{}_elapsed_executing_ms", operation)); // to timeout if it takes too long
        let operation_elapsed_disabled_ms = iv!(&&format!("{}_elapsed_disabled_ms", operation));
        let operation_failure_retry_counter = iv!(&&format!("{}_failure_retry_counter", operation)); // without scrapping the current plan, how many times has an operation retried
        let operation_timeout_retry_counter = iv!(&&format!("{}_timeout_retry_counter", operation));
        state.add_mut(
            assign!(
                operation_information,
                SPValue::String(StringOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state.add_mut(
            assign!(
                operation_elapsed_executing_ms,
                SPValue::Int64(IntOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state.add_mut(
            assign!(
                operation_elapsed_disabled_ms,
                SPValue::Int64(IntOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state.add_mut(
            assign!(
                operation_failure_retry_counter,
                SPValue::Int64(IntOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state.add_mut(
            assign!(
                operation_timeout_retry_counter,
                SPValue::Int64(IntOrUnknown::UNKNOWN)
            ),
            &log_target,
        );

        if coverability_tracking {
            // coverability tracking does nothing for now
            let initial = iv!(&&format!("{}_visited_initial", operation));
            let executing = iv!(&&format!("{}_visited_executing", operation));
            let timedout = iv!(&&format!("{}_visited_timedout", operation)); // Operation should have optional deadline field
            let disabled = iv!(&&format!("{}_visited_disabled", operation));
            let failed = iv!(&&format!("{}_visited_failed", operation));
            let completed = iv!(&&format!("{}_visited_completed", operation));

            for cov in vec![initial, executing, timedout, disabled, failed, completed] {
                state.add_mut(assign!(cov, 0.to_spvalue()), &log_target);
            }
        }
    }
    state
}

pub fn add_operation_state_tracking_variable(
    ops: &Vec<String>,
    state: &State,
    log_target: &str,
) -> State {
    let mut state = state.clone();
    for operation in ops {
        let operation_state = v!(&&format!("{}", operation)); // Initial, Executing, Failed, Completed, Unknown
        state.add_mut(
            assign!(operation_state, "initial".to_spvalue()),
            &log_target,
        );
    }
    state
}
