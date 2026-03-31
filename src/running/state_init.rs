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

    // add the timer variables to the state
    for timer_id in 1..=number_of_timers {
        let timer_request_trigger = bv!(&&format!("{}_timer_{}_request_trigger", name, timer_id));
        let timer_request_state = v!(&&format!("{}_timer_{}_request_state", name, timer_id));
        let timer_command = v!(&&format!("{}_timer_{}_command", name, timer_id));
        let timer_duration_ms = iv!(&&format!("{}_timer_{}_duration_ms", name, timer_id));
        let timer_elapsed_ms = iv!(&&format!("{}_timer_{}_elapsed_ms", name, timer_id));

        state = state.add(
            assign!(timer_request_trigger, SPValue::Bool(BoolOrUnknown::Bool(false))),
            &log_target,
        );
        state = state.add(
            assign!(
                timer_request_state,
                SPValue::String(StringOrUnknown::String("initial".to_string()))
            ),
            &log_target,
        );
        state = state.add(
            assign!(timer_command, SPValue::String(StringOrUnknown::UNKNOWN)),
            &log_target,
        );
        state = state.add(
            assign!(timer_duration_ms, SPValue::Int64(IntOrUnknown::UNKNOWN)),
            &log_target,
        );
        state = state.add(
            assign!(timer_elapsed_ms, SPValue::Int64(IntOrUnknown::UNKNOWN)),
            &log_target,
        );
    }

    let terminated_operations = av!(&&format!("{}_terminated_operations", name));
    state = state.add(
        assign!(
            terminated_operations,
            SPValue::Array(ArrayOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    let active_auto_operations = av!(&&format!("{}_active_auto_operations", name));
    state = state.add(
        assign!(
            active_auto_operations,
            SPValue::Array(ArrayOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    // Pause, Stop, Run/Play/Continue
    let sp_dashboard_command = v!(&&format!("{}_dashboard_command", name));
    state = state.add(
        assign!(
            sp_dashboard_command,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );

    // Logging:
    let empty_log: Vec<Vec<OperationLog>> = vec![vec![]];
    // let empty_log_agg: Vec<Vec<Vec<OperationLog>>> = vec![vec![vec![]]];
    let logger_planned_operations = v!(&&format!("{}_logger_planned_operations", name));
    state = state.add(
        assign!(
            logger_planned_operations,
            SPValue::String(StringOrUnknown::String(
                serde_json::to_string(&empty_log).unwrap()
            ))
        ),
        &log_target,
    );

    // let logger_planned_operations_agg = v!(&&format!("{}_logger_planned_operations_agg", name));
    // state = state.add(assign!(
    //     logger_planned_operations_agg,
    //     SPValue::String(StringOrUnknown::String(
    //         serde_json::to_string(&empty_log_agg).unwrap()
    //     ))
    // ));

    let logger_automatic_operations = v!(&&format!("{}_logger_automatic_operations", name));
    state = state.add(
        assign!(
            logger_automatic_operations,
            SPValue::String(StringOrUnknown::String(
                serde_json::to_string(&empty_log).unwrap()
            ))
        ),
        &log_target,
    );

    // let logger_automatic_operations_agg = v!(&&format!("{}_logger_automatic_operations_agg", name));
    // state = state.add(assign!(
    //     logger_automatic_operations_agg,
    //     SPValue::String(StringOrUnknown::String(
    //         serde_json::to_string(&empty_log_agg).unwrap()
    //     ))
    // ));

    let logger_sop_operations = v!(&&format!("{}_logger_sop_operations", name));
    state = state.add(
        assign!(
            logger_sop_operations,
            SPValue::String(StringOrUnknown::String(
                serde_json::to_string(&empty_log).unwrap()
            ))
        ),
        &log_target,
    );

    // let logger_sop_operations_agg = v!(&&format!("{}_logger_sop_operations_agg", name));
    // state = state.add(assign!(
    //     logger_sop_operations_agg,
    //     SPValue::String(StringOrUnknown::String(
    //         serde_json::to_string(&empty_log_agg).unwrap()
    //     ))
    // ));

    // Initialize values
    state = state.add(
        assign!(runner_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(
            current_goal_predicate,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(current_goal_id, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(
            current_goal_state,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(plan, SPValue::Array(ArrayOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(plan_exists, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(plan_name, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(plan_id, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(plan_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(planner_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(plan_duration, SPValue::Float64(FloatOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(plan_current_step, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(
            planner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(
            plan_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(
            goal_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(
            sop_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(
            main_runner_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(
            goal_scheduler_information,
            SPValue::String(StringOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(replanned, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(replan_for_same_goal, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(replan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(replan_counter_total, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(plan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(replan_fail_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(replan_trigger, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(incoming_goals, SPValue::Array(ArrayOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(scheduled_goals, SPValue::Array(ArrayOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(sop_id, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(sop_state, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(sop_stack, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(sop_enabled, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(start_time, SPValue::Int64(IntOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(sop_current_step, SPValue::Int64(IntOrUnknown::Int64(0))),
        &log_target,
    );
    state = state.add(
        assign!(
            tf_request_trigger,
            SPValue::Bool(BoolOrUnknown::Bool(false))
        ),
        &log_target,
    );
    state = state.add(
        assign!(
            tf_request_state,
            SPValue::String(StringOrUnknown::String("initial".to_string()))
        ),
        &log_target,
    );
    state = state.add(
        assign!(tf_command, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(tf_parent, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(tf_child, SPValue::String(StringOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(
            tf_lookup_result,
            SPValue::Transform(TransformOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(
            tf_insert_transforms,
            SPValue::Array(ArrayOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    // state = state.add(
    //     assign!(time_request_trigger, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
    //     &log_target,
    // );
    // state = state.add(
    //     assign!(
    //         time_request_state,
    //         SPValue::String(StringOrUnknown::String("initial".to_string()))
    //     ),
    //     &log_target,
    // );
    // state = state.add(
    //     assign!(time_command, SPValue::String(StringOrUnknown::UNKNOWN)),
    //     &log_target,
    // );
    // state = state.add(
    //     assign!(time_duration_ms, SPValue::Int64(IntOrUnknown::UNKNOWN)),
    //     &log_target,
    // );
    // state = state.add(
    //     assign!(time_elapsed_ms, SPValue::Int64(IntOrUnknown::UNKNOWN)),
    //     &log_target,
    // );

    // Define variables to keep track of the processes
    let state_manager_online = bv!(&&format!("state_manager_online"));
    let auto_transition_runner_online = bv!(&&format!("{}_auto_transition_runner_online", name));
    let planner_ticker_online = bv!(&&format!("{}_planner_ticker_online", name));
    let operation_planner_online = bv!(&&format!("{}_operation_planner_online", name));
    let operation_runner_online = bv!(&&format!("{}_operation_runner_online", name));
    state = state.add(
        assign!(state_manager_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(
            auto_transition_runner_online,
            SPValue::Bool(BoolOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
        assign!(planner_ticker_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)),
        &log_target,
    );
    state = state.add(
        assign!(
            operation_planner_online,
            SPValue::Bool(BoolOrUnknown::UNKNOWN)
        ),
        &log_target,
    );
    state = state.add(
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
    // operations should be put in the initial state once they are part of the plan

    for sop in &model.sops {
        // let ops_in_sop = get_all_operations_from_sop(&sop.sop);
        let sop_information = v!(&&format!("{}_sop_information", sop.id));
        state = state.add(
            assign!(sop_information, SPValue::String(StringOrUnknown::UNKNOWN)),
            &log_target,
        );
        // state = add_operation_meta_tracking_variables(
        //     &ops_in_sop.iter().map(|x| x.name.clone()).collect(),
        //     &state,
        //     false,
        //     &log_target,
        // ); // remove later for unique on the fly
        // state = add_operation_state_tracking_variable(
        //     &ops_in_sop.iter().map(|x| x.name.clone()).collect(),
        //     &state,
        //     &log_target,
        // ); // remove later for unique on the fly
    }

    // Not ideal, maybe there is a way to remove this dependancy
    // Still need this for the BFS planning level because the BFS needs the state, and the template has to exist in the state
    // state = add_operation_state_tracking_variable(
    //     &model.operations.iter().map(|x| x.name.clone()).collect(),
    //     &state,
    //     &log_target,
    // ); // remove later for unique on the fly
    // state = add_operation_meta_tracking_variables(
    //     &model.operations.iter().map(|x| x.name.clone()).collect(),
    //     &state,
    //     false,
    //     &log_target,
    // ); // remove later for unique on the fly

    state = add_operation_state_tracking_variable(
        &model
            .auto_operations
            .iter()
            .map(|x| x.name.clone())
            .collect(),
        &state,
        &log_target,
    ); // remove later for unique on the fly
    // state = add_operation_meta_tracking_variables(
    //     &model
    //         .auto_operations
    //         .iter()
    //         .map(|x| x.name.clone())
    //         .collect(),
    //     &state,
    //     false,
    //     &log_target,
    // ); // remove later for unique on the fly

    for transition in &model.auto_transitions {
        if coverability_tracking {
            let taken = iv!(&&format!("transition_{}_taken", transition.name));
            state = state.add(assign!(taken, 0.to_spvalue()), &log_target)
        }
    }

    // for operation in &model.auto_operations {
    //     let operation_state = v!(&&format!("{}", operation.name));
    //     state = state.add(assign!(operation_state, "initial".to_spvalue()));
    //     if coverability_tracking {
    //         let taken = iv!(&&format!("{}_taken", operation.name));
    //         state = state.add(assign!(taken, 0.to_spvalue()))
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
        state = state.add(
            assign!(
                operation_information,
                SPValue::String(StringOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state = state.add(
            assign!(
                operation_elapsed_executing_ms,
                SPValue::Int64(IntOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state = state.add(
            assign!(
                operation_elapsed_disabled_ms,
                SPValue::Int64(IntOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state = state.add(
            assign!(
                operation_failure_retry_counter,
                SPValue::Int64(IntOrUnknown::UNKNOWN)
            ),
            &log_target,
        );
        state = state.add(
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
                state = state.add(assign!(cov, 0.to_spvalue()), &log_target);
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
        state = state.add(
            assign!(operation_state, "initial".to_spvalue()),
            &log_target,
        );
    }
    state
}
