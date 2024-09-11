use crate::*;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

pub fn generate_runner_state_variables(name: &str) -> State {
    let state = State::new();

    let runner_state = v!(&&format!("{}_runner_state", name));
    let runner_goal = v!(&&format!("{}_runner_goal", name));
    let runner_plan = av!(&&format!("{}_runner_plan", name));
    let runner_plan_name = v!(&&format!("{}_runner_plan_name", name));
    let runner_plan_state = v!(&&format!("{}_runner_plan_state", name));
    let runner_plan_current_step = iv!(&&format!("{}_runner_plan_current_step", name));
    let runner_replanned = bv!(&&format!("{}_runner_replanned", name));
    let runner_replan_counter = iv!(&&format!("{}_runner_replan_counter", name));
    let runner_replan_fail_counter = iv!(&&format!("{}_runner_replan_fail_counter", name));
    let runner_replan_trigger = bv!(&&format!("{}_runner_replan_trigger", name));

    let state = state.add(assign!(runner_state, SPValue::UNKNOWN));
    let state = state.add(assign!(runner_goal, SPValue::UNKNOWN));
    let state = state.add(assign!(runner_plan, SPValue::Array(SPValueType::String, vec!())));
    let state = state.add(assign!(runner_plan_name, SPValue::UNKNOWN));
    let state = state.add(assign!(runner_plan_state, "initial".to_spvalue()));
    let state = state.add(assign!(runner_plan_current_step, SPValue::Int64(0)));
    let state = state.add(assign!(runner_replanned, SPValue::Bool(false)));
    let state = state.add(assign!(runner_replan_counter, SPValue::Int64(0)));
    let state = state.add(assign!(runner_replan_fail_counter, SPValue::Int64(0)));
    let state = state.add(assign!(runner_replan_trigger, SPValue::Bool(false)));

    state
}

/// If an operation has to be generated per item or per order
fn fill_operation_parameters(op: Operation, parameter: &str, replacement: &str) -> Operation {
    let mut mut_op = op.clone();
    mut_op.name = op.name.replace(parameter, replacement);
    mut_op.precondition.actions = op
        .precondition
        .actions
        .iter()
        .map(|x| {
            if x.var_or_val == parameter.wrap() {
                Action::new(x.var.clone(), replacement.wrap())
            } else {
                x.to_owned()
            }
        })
        .collect();
    mut_op
}

pub async fn simple_operation_runner(
    name: &str,
    model: &Model,
    shared_state: &Arc<Mutex<State>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    loop {
        let shared_state_local = shared_state.lock().unwrap().clone();
        for t in &model.auto_transitions {
            if t.clone().eval_running(&shared_state_local) {
                log::info!(target: &&format!("{}_runner", name), "Executed free transition: '{}'.", t.name);
                let shared_state_local = shared_state.lock().unwrap().clone();
                let updated_state = t.clone().take_running(&shared_state_local);
                *shared_state.lock().unwrap() = updated_state;
            }
        }

        let runner_plan_name =
        match shared_state_local.get_value(&&format!("{}_runner_plan_name", name)) {
            SPValue::String(value) => value,
            _ => {
                log::error!(target: &&format!("{}_runner", name), 
            "Couldn't get '{}_runner_plan_name' from the shared state.", name);
                "unknown".to_string()
            }
        };

        let runner_plan_state =
            match shared_state_local.get_value(&&format!("{}_runner_plan_state", name)) {
                SPValue::String(value) => value,
                _ => {
                    log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_runner_plan_state' from the shared state.", name);
                    "unknown".to_string()
                }
            };

        match PlanState::from_str(&runner_plan_state) {
            PlanState::Initial => {
                let shared_state_local = shared_state.lock().unwrap().clone();
                let updated_state = shared_state_local.update(
                    &&format!("{}_runner_plan_state", name),
                    PlanState::Executing.to_spvalue(),
                );
                *shared_state.lock().unwrap() = updated_state;
            }
            PlanState::Executing => {
                log::info!(target: &&format!("{}_runner", name), 
                "Started executing plan: '{}'.", runner_plan_name);
                let runner_plan =
                    match shared_state_local.get_value(&&format!("{}_runner_plan", name)) {
                        SPValue::Array(_sp_value_type, value_array) => value_array,
                        _ => {
                            log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_runner_plan' from the shared state.", name);
                            vec![]
                        }
                    };
                let runner_plan_current_step = match shared_state_local
                    .get_value(&&format!("{}_runner_plan_current_step", name))
                {
                    SPValue::Int64(value) => value,
                    _ => {
                        log::error!(target: &&format!("{}_runner", name), 
                "Couldn't get '{}_runner_current_step' from the shared state.", name);
                        0
                    }
                };
                if runner_plan.len() > runner_plan_current_step as usize {
                    let current_model_operation = model
                        .operations
                        .iter()
                        .find(|op| {
                            op.name == runner_plan[runner_plan_current_step as usize].to_string()
                        })
                        .unwrap()
                        .to_owned();
                    let shared_state_local = shared_state.lock().unwrap().clone();
                    match OperationState::from_str(
                        &shared_state_local
                            .get_value(&current_model_operation.name)
                            .to_string(),
                    ) {
                        OperationState::Initial => {
                            if current_model_operation
                                .clone()
                                .eval_running(&shared_state_local)
                            {
                                log::info!(target: &&format!("{}_runner", name), 
                "Started running operation: '{}'.", current_model_operation.name);

                                let shared_state_local = shared_state.lock().unwrap().clone();
                                let updated_state =
                                    current_model_operation.start_running(&shared_state_local);
                                *shared_state.lock().unwrap() = updated_state.clone();
                            }
                        }
                        OperationState::Disabled => todo!(),
                        OperationState::Executing => {
                            if current_model_operation
                                .clone()
                                .can_be_completed(&shared_state_local)
                            {
                                log::info!(target: &&format!("{}_runner", name), 
                "Completed running operation: '{}'.", current_model_operation.name);

                                let shared_state_local = shared_state.lock().unwrap().clone();
                                let updated_state = current_model_operation
                                    .clone()
                                    .complete_running(&shared_state_local);
                                *shared_state.lock().unwrap() = updated_state.clone();
                            } else {
                                log::info!(target: &&format!("{}_runner", name), 
                "Waiting for operation: '{}' to be completed.", current_model_operation.name);
                            }
                        }
                        OperationState::Completed => {
                            let current_model_operation = model
                                .operations
                                .iter()
                                .find(|op| op.name == current_model_operation.name)
                                .unwrap()
                                .to_owned();

                            if current_model_operation
                                .clone()
                                .can_be_reset(&shared_state_local)
                            {
                                log::info!(target: &&format!("{}_runner", name), 
                "Reseting operation: '{}'.", current_model_operation.name);

                                let shared_state_local = shared_state.lock().unwrap().clone();
                                let updated_state = current_model_operation
                                    .clone()
                                    .reset_running(&shared_state_local);
                                *shared_state.lock().unwrap() = updated_state.clone();
                            }
                        }
                        OperationState::Timedout => todo!(),
                        OperationState::Failed => todo!(),
                        OperationState::UNKNOWN => todo!(),
                    }
                } else {
                    log::info!(target: &&format!("{}_runner", name), 
                "Completed plan: '{}'.", runner_plan_name);
                }
            }
            PlanState::Paused => todo!(),
            PlanState::Failed => todo!(),
            PlanState::Completed => todo!(),
            PlanState::Cancelled => todo!(),
            PlanState::UNKNOWN => todo!(),
        }

        interval.tick().await;
    }
}
