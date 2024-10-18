use crate::*;

// If coverability_tracking is true, generate variables to track how many
// times an operation has entered its different running states
pub fn generate_runner_state_variables(
    model: &Model,
    name: &str,
    coverability_tracking: bool,
) -> State {
    let mut state = State::new();

    let runner_state = v!(&&format!("{}_runner_state", name));
    let runner_goal = v!(&&format!("{}_runner_goal", name));
    let runner_plan = av!(&&format!("{}_runner_plan", name));
    let runner_plan_counter = av!(&&format!("{}_runner_plan_counter", name)); // How many times in total has a planner been called
    let runner_plan_exists = bv!(&&format!("{}_runner_plan_exists", name));
    let runner_plan_name = v!(&&format!("{}_runner_plan_name", name));
    let runner_plan_state = v!(&&format!("{}_runner_plan_state", name));
    let runner_plan_duration = fv!(&&format!("{}_runner_plan_duration", name));
    let runner_plan_current_step = iv!(&&format!("{}_runner_plan_current_step", name));
    let runner_replanned = bv!(&&format!("{}_runner_replanned", name));
    let runner_replan_counter = iv!(&&format!("{}_runner_replan_counter", name)); // How many times has the planner tried to replan for the same problem
    let runner_replan_fail_counter = iv!(&&format!("{}_runner_replan_fail_counter", name));
    let runner_replan_trigger = bv!(&&format!("{}_runner_replan_trigger", name));

    state = state.add(assign!(runner_state, SPValue::UNKNOWN));
    state = state.add(assign!(runner_goal, SPValue::UNKNOWN));
    state = state.add(assign!(
        runner_plan,
        SPValue::Array(SPValueType::String, vec!())
    ));
    state = state.add(assign!(runner_plan_exists, false.to_spvalue()));
    state = state.add(assign!(runner_plan_name, SPValue::UNKNOWN));
    state = state.add(assign!(runner_plan_state, "initial".to_spvalue()));
    state = state.add(assign!(runner_plan_duration, 0.0.to_spvalue()));
    state = state.add(assign!(runner_plan_current_step, SPValue::Int64(0)));
    state = state.add(assign!(runner_replanned, SPValue::Bool(false)));
    state = state.add(assign!(runner_replan_counter, SPValue::Int64(0)));
    state = state.add(assign!(runner_plan_counter, SPValue::Int64(0)));
    state = state.add(assign!(runner_replan_fail_counter, SPValue::Int64(0)));
    state = state.add(assign!(runner_replan_trigger, SPValue::Bool(false)));

    // operations should be put in the initial state once they are part of the plan
    for operation in &model.operations {
        let operation_state = v!(&&format!("{}_state", operation.name));
        let operation_duration = fv!(&&format!("{}_duration", operation.name));
        state = state.add(assign!(operation_state, "initial".to_spvalue()));
        state = state.add(assign!(operation_duration, 0.0.to_spvalue()));

        if coverability_tracking {
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

    state
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