use std::time::{SystemTime, UNIX_EPOCH};

use crate::*;

// If coverability_tracking is true, generate variables to track how many
// times an operation has entered its different running states
pub fn generate_runner_state_variables(name: &str) -> State {
    let mut state = State::new();

    // Define variables
    let runner_state = v!(&&format!("{}_runner_state", name)); // does nothing for now
    let current_goal_predicate = v!(&&format!("{}_current_goal_predicate", name)); // goal as a string predicate
    let current_goal_id = v!(&&format!("{}_current_goal_id", name)); // goal as a string predicate
    let current_goal_state = v!(&&format!("{}_current_goal_state", name)); // goal as a string predicate
    let plan = av!(&&format!("{}_plan", name)); // plan as array of string
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
    let replan_counter_total = iv!(&&format!("{}_replan_counter_total", name)); // How many times has the planner been called
    let replan_counter = iv!(&&format!("{}_replan_counter", name)); // How many times has the planner tried to replan for the same problem
    let replan_fail_counter = iv!(&&format!("{}_replan_fail_counter", name)); // How many times has the planner failed in
    let replan_trigger = bv!(&&format!("{}_replan_trigger", name)); // boolean for tracking the planner triggering
    let incoming_goals = mv!(&&format!("{}_incoming_goals", name));
    let scheduled_goals = mv!(&&format!("{}_scheduled_goals", name));
    let sop_enabled = bv!(&&format!("{}_sop_enabled", name));
    // let sop_request_state = v!(&&format!("{}_sop_request_state", name));
    let sop_current_step = iv!(&&format!("{}_sop_current_step", name));
    let sop_id = v!(&&format!("{}_sop_id", name));
    let sop_state = v!(&&format!("{}_sop_state", name));
    let start_time = iv!(&&format!("{}_start_time", name));
    let tf_request_trigger = bv!(&&format!("{}_tf_request_trigger", name));
    let tf_request_state = v!(&&format!("{}_tf_request_state", name));
    let tf_command = v!(&&format!("{}_tf_command", name));
    let tf_parent = v!(&&format!("{}_tf_parent", name));
    let tf_child = v!(&&format!("{}_tf_child", name));
    let tf_lookup_result = tfv!(&&format!("{}_tf_lookup_result", name));
    
    // Initialize values
    state = state.add(assign!(runner_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(current_goal_predicate, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(current_goal_id, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(current_goal_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan, SPValue::Array(ArrayOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_exists, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_name, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(planner_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_duration, SPValue::Float64(FloatOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_current_step, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(planner_information, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_runner_information, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(goal_runner_information, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(sop_runner_information, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(main_runner_information, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(goal_scheduler_information, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(replanned, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_counter_total, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_fail_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_trigger, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(incoming_goals, SPValue::Map(MapOrUnknown::UNKNOWN)));
    state = state.add(assign!(scheduled_goals, SPValue::Map(MapOrUnknown::UNKNOWN)));
    // state = state.add(assign!(sop_request_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(sop_id, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(sop_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(sop_enabled, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(start_time, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(sop_current_step, SPValue::Int64(IntOrUnknown::Int64(0))));
    state = state.add(assign!(tf_request_trigger, SPValue::Bool(BoolOrUnknown::Bool(false))));
    state = state.add(assign!(tf_request_state, SPValue::String(StringOrUnknown::String("initial".to_string()))));
    state = state.add(assign!(tf_command, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(tf_parent, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(tf_child, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(tf_lookup_result, SPValue::Transform(TransformOrUnknown::UNKNOWN)));


    // Define variables to keep track of the processes
    let state_manager_online = bv!(&&format!("state_manager_online"));
    let auto_transition_runner_online = bv!(&&format!("{}_auto_transition_runner_online", name));
    let planner_ticker_online = bv!(&&format!("{}_planner_ticker_online", name));
    let operation_planner_online = bv!(&&format!("{}_operation_planner_online", name));
    let operation_runner_online = bv!(&&format!("{}_operation_runner_online", name));
    state = state.add(assign!(state_manager_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(auto_transition_runner_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(planner_ticker_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(operation_planner_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(operation_runner_online, SPValue::Bool(BoolOrUnknown::UNKNOWN)));

    state
}

pub fn generate_operation_state_variables(model: &Model, coverability_tracking: bool) -> State {
    let mut state = State::new();
    // operations should be put in the initial state once they are part of the plan

    for sop in &model.sops {
        for operation in &sop.sop {
            let operation_state = v!(&&format!("{}_sop", operation.name)); // Initial, Executing, Failed, Completed, Unknown
            let operation_information = v!(&&format!("{}_sop_information", operation.name));
            let operation_start_time = iv!(&&format!("{}_sop_start_time", operation.name)); // to timeout if it takes too long
            let operation_retry_counter = iv!(&&format!("{}_sop_retry_counter", operation.name)); // without scrapping the current plan, how many times has an operation retried
            state = state.add(assign!(operation_state, "initial".to_spvalue()));
            state = state.add(assign!(operation_information, SPValue::String(StringOrUnknown::UNKNOWN)));
            state = state.add(assign!(operation_start_time, SPValue::Int64(IntOrUnknown::UNKNOWN)));
            state = state.add(assign!(operation_retry_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));
        }
    }

    for operation in &model.operations {
        let operation_state = v!(&&format!("{}", operation.name)); // Initial, Executing, Failed, Completed, Unknown
        let operation_information = v!(&&format!("{}_information", operation.name));
        let operation_start_time = iv!(&&format!("{}_start_time", operation.name)); // to timeout if it takes too long
        let operation_retry_counter = iv!(&&format!("{}_retry_counter", operation.name)); // without scrapping the current plan, how many times has an operation retried
        state = state.add(assign!(operation_state, "initial".to_spvalue()));
        state = state.add(assign!(operation_information, SPValue::String(StringOrUnknown::UNKNOWN)));
        state = state.add(assign!(operation_start_time, SPValue::Int64(IntOrUnknown::UNKNOWN)));
        state = state.add(assign!(operation_retry_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));

        if coverability_tracking {
            // coverability tracking does nothing for now
            let initial = iv!(&&format!("{}_visited_initial", operation.name));
            let executing = iv!(&&format!("{}_visited_executing", operation.name));
            let timedout = iv!(&&format!("{}_visited_timedout", operation.name)); // Operation should have optional deadline field
            let disabled = iv!(&&format!("{}_visited_disabled", operation.name));
            let failed = iv!(&&format!("{}_visited_failed", operation.name));
            let completed = iv!(&&format!("{}_visited_completed", operation.name));

            for cov in vec![initial, executing, timedout, disabled, failed, completed] {
                state = state.add(assign!(cov, 0.to_spvalue()));
            }
        }
    }

    for transition in &model.auto_transitions {
        if coverability_tracking {
            let taken = iv!(&&format!("transition_{}_taken", transition.name));
            state = state.add(assign!(taken, 0.to_spvalue()))
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

pub fn reset_all_operations(state: &State) -> State {
    let state = state.clone();
    let mut mut_state = state.clone();
    state.state.iter().for_each(|(k, _)| {
        if k.starts_with("operation_") && k.ends_with("_state") {
            mut_state = mut_state.update(&k, "initial".to_spvalue());
        }
    });
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
        let _ = generate_runner_state_variables("asdf");
    }
}
