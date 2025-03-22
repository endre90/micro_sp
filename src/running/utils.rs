use crate::*;

// If coverability_tracking is true, generate variables to track how many
// times an operation has entered its different running states
pub fn generate_runner_state_variables(name: &str) -> State {
    let mut state = State::new();

    let runner_state = v!(&&format!("{}_runner_state", name)); // does nothing for now
    // let runner_ref_counter = iv!(&&format!("{}_runner_ref_counter", name)); // does nothing for now
    let goal = v!(&&format!("{}_goal", name)); // goal as a string predicate
    let goal_exists = bv!(&&format!("{}_goal_exists", name)); // does nothing for now
    let plan = av!(&&format!("{}_plan", name)); // plan as array of string
    let plan_counter = iv!(&&format!("{}_plan_counter", name)); // How many times has a plan been found
    let plan_exists = bv!(&&format!("{}_plan_exists", name)); // does nothing for now
    let plan_name = v!(&&format!("{}_plan_name", name)); // same as model name, should add nanoid!
    let plan_state = v!(&&format!("{}_plan_state", name)); // Initial, Executing, Failed, Completed, Unknown
    let plan_duration = fv!(&&format!("{}_plan_duration", name)); // does nothing for now
    let plan_current_step = iv!(&&format!("{}_plan_current_step", name)); // Index of the currently exec. operation in the plan
    // let planner_ref_counter = iv!(&&format!("{}_planner_ref_counter", name)); // does nothing
    let planner_information = v!(&&format!("{}_planner_information", name)); // current information about the plan
    let replanned = bv!(&&format!("{}_replanned", name)); // boolean for tracking the planner triggering
    let replan_counter_total = iv!(&&format!("{}_replan_counter_total", name)); // How many times has the planner been called
    let replan_counter = iv!(&&format!("{}_replan_counter", name)); // How many times has the planner tried to replan for the same problem
    let replan_fail_counter = iv!(&&format!("{}_replan_fail_counter", name)); // How many times has the planner failed in
    let replan_trigger = bv!(&&format!("{}_replan_trigger", name)); // boolean for tracking the planner triggering

    state = state.add(assign!(runner_state, SPValue::Unknown(SPValueType::String)));
    // state = state.add(assign!(runner_ref_counter, 1.to_spvalue()));
    state = state.add(assign!(goal, SPValue::Unknown(SPValueType::String)));
    state = state.add(assign!(goal_exists, SPValue::Unknown(SPValueType::Bool)));
    state = state.add(assign!(plan, SPValue::Unknown(SPValueType::Array)));
    state = state.add(assign!(plan_exists, SPValue::Unknown(SPValueType::Bool)));
    state = state.add(assign!(plan_name, SPValue::Unknown(SPValueType::String)));
    state = state.add(assign!(plan_state, SPValue::Unknown(SPValueType::String)));
    state = state.add(assign!(plan_duration, SPValue::Unknown(SPValueType::Float64)));
    state = state.add(assign!(plan_current_step, SPValue::Unknown(SPValueType::Int64)));
    // state = state.add(assign!(planner_ref_counter, 1.to_spvalue()));
    state = state.add(assign!(planner_information, SPValue::Unknown(SPValueType::String)));
    state = state.add(assign!(replanned, SPValue::Unknown(SPValueType::Bool)));
    state = state.add(assign!(replan_counter, SPValue::Unknown(SPValueType::Int64)));
    state = state.add(assign!(replan_counter_total, SPValue::Unknown(SPValueType::Int64)));
    state = state.add(assign!(plan_counter, SPValue::Unknown(SPValueType::Int64)));
    state = state.add(assign!(replan_fail_counter, SPValue::Unknown(SPValueType::Int64)));
    state = state.add(assign!(replan_trigger, SPValue::Unknown(SPValueType::Bool)));

    state
}

pub fn generate_operation_state_variables(model: &Model, coverability_tracking: bool) -> State {
    let mut state = State::new();
    // operations should be put in the initial state once they are part of the plan
    for operation in &model.operations {
        let operation_state = v!(&&format!("{}", operation.name)); // Initial, Executing, Failed, Completed, Unknown
        let operation_information = v!(&&format!("{}_information", operation.name));
        let operation_start_time = fv!(&&format!("{}_start_time", operation.name)); // does nothing for now
        let operation_retry_counter = iv!(&&format!("{}_retry_counter", operation.name)); // without scrapping the current plan, how many times has an operation retried
        state = state.add(assign!(operation_state, "initial".to_spvalue()));
        state = state.add(assign!(operation_information, SPValue::Unknown(SPValueType::String)));
        state = state.add(assign!(operation_start_time, SPValue::Unknown(SPValueType::Float64)));
        state = state.add(assign!(operation_retry_counter, SPValue::Unknown(SPValueType::Int64)));

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
            let taken = iv!(&&format!("{}_taken", transition.name));
            state = state.add(assign!(taken, 0.to_spvalue()))
        }
    }

    for operation in &model.auto_operations {
        let operation_state = v!(&&format!("{}", operation.name));
        state = state.add(assign!(operation_state, "initial".to_spvalue()));
        if coverability_tracking {
            let taken = iv!(&&format!("{}_taken", operation.name));
            state = state.add(assign!(taken, 0.to_spvalue()))
        }
    }

    state
}

pub fn reset_all_operations(state: &State) -> State {
    let state = state.clone();
    let mut mut_state = state.clone();
    state.state.iter().for_each(|(k, _)| {
        if k.starts_with("op_") {
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
