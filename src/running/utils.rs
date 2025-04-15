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
    let plan_duration = fv!(&&format!("{}_plan_duration", name)); // does nothing for now
    let plan_current_step = iv!(&&format!("{}_plan_current_step", name)); // Index of the currently exec. operation in the plan
    let planner_information = v!(&&format!("{}_planner_information", name)); // current information about the plan
    let replanned = bv!(&&format!("{}_replanned", name)); // boolean for tracking the planner triggering
    let replan_counter_total = iv!(&&format!("{}_replan_counter_total", name)); // How many times has the planner been called
    let replan_counter = iv!(&&format!("{}_replan_counter", name)); // How many times has the planner tried to replan for the same problem
    let replan_fail_counter = iv!(&&format!("{}_replan_fail_counter", name)); // How many times has the planner failed in
    let replan_trigger = bv!(&&format!("{}_replan_trigger", name)); // boolean for tracking the planner triggering
    let incoming_goals = mv!(&&format!("{}_incoming_goals", name));
    let scheduled_goals = mv!(&&format!("{}_scheduled_goals", name));
    

    // Initialize values
    state = state.add(assign!(runner_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(current_goal_predicate, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(current_goal_id, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(current_goal_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan, SPValue::Array(ArrayOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_exists, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_name, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_state, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_duration, SPValue::Float64(FloatOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_current_step, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(planner_information, SPValue::String(StringOrUnknown::UNKNOWN)));
    state = state.add(assign!(replanned, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_counter_total, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(plan_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_fail_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));
    state = state.add(assign!(replan_trigger, SPValue::Bool(BoolOrUnknown::UNKNOWN)));
    state = state.add(assign!(incoming_goals, SPValue::Map(MapOrUnknown::UNKNOWN)));
    state = state.add(assign!(scheduled_goals, SPValue::Map(MapOrUnknown::UNKNOWN)));

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
    for operation in &model.operations {
        let operation_state = v!(&&format!("operation_{}", operation.name)); // Initial, Executing, Failed, Completed, Unknown
        let operation_information = v!(&&format!("operation_{}_information", operation.name));
        let operation_start_time = tv!(&&format!("operation_{}_start_time", operation.name)); // to timeout if it takes too long
        let operation_retry_counter = iv!(&&format!("operation_{}_retry_counter", operation.name)); // without scrapping the current plan, how many times has an operation retried
        state = state.add(assign!(operation_state, "initial".to_spvalue()));
        state = state.add(assign!(operation_information, SPValue::String(StringOrUnknown::UNKNOWN)));
        state = state.add(assign!(operation_start_time, SPValue::Time(TimeOrUnknown::UNKNOWN)));
        state = state.add(assign!(operation_retry_counter, SPValue::Int64(IntOrUnknown::UNKNOWN)));

        if coverability_tracking {
            // coverability tracking does nothing for now
            let initial = iv!(&&format!("operation_{}_visited_initial", operation.name));
            let executing = iv!(&&format!("operation_{}_visited_executing", operation.name));
            let timedout = iv!(&&format!("operation_{}_visited_timedout", operation.name)); // Operation should have optional deadline field
            let disabled = iv!(&&format!("operation_{}_visited_disabled", operation.name));
            let failed = iv!(&&format!("operation_{}_visited_failed", operation.name));
            let completed = iv!(&&format!("operation_{}_visited_completed", operation.name));

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

    for operation in &model.auto_operations {
        let operation_state = v!(&&format!("operation_{}", operation.name));
        state = state.add(assign!(operation_state, "initial".to_spvalue()));
        if coverability_tracking {
            let taken = iv!(&&format!("operation_{}_taken", operation.name));
            state = state.add(assign!(taken, 0.to_spvalue()))
        }
    }

    state
}

pub fn reset_all_operations(state: &State) -> State {
    let state = state.clone();
    let mut mut_state = state.clone();
    state.state.iter().for_each(|(k, _)| {
        if k.starts_with("operation_") {
            mut_state = mut_state.update(&k, "initial".to_spvalue());
        }
    });
    mut_state
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
