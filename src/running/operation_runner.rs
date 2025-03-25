// use std::time::{Instant, SystemTime};

use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

/// A planned operation runner is an algorithm which executes the plan P based on the model
/// M, the current state of the system S, and a goal predicate G. While
/// running, both the planning and running components of guards and actions
/// of operation pre- and postconditions are evaluated and taken.
pub async fn planned_operation_runner(
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = &model.name;
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // For nicer logging
    let mut plan_state_old = "".to_string();
    let mut operation_state_old = "".to_string();
    let mut operation_information_old = "".to_string();

    'initialize: loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::Get((
                "state_manager_online".to_string(),
                response_tx,
            )))
            .await?;
        let state_manager_online = response_rx.await?;
        match state_manager_online {
            SPValue::Bool(BoolOrUnknown::Bool(true)) => break 'initialize,
            _ => {},
        }
        interval.tick().await;
    }

    log::info!(target: &&format!("{}_operation_runner", name), "Online.");

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;
        let mut new_state = state.clone();

        let mut plan_state = state.get_string_or_default_to_unknown(
            &format!("{}_operation_runner", name),
            &format!("{}_plan_state", name),
        );
        let mut plan_current_step = state.get_int_or_default_to_zero(
            &format!("{}_operation_runner", name),
            &format!("{}_plan_current_step", name),
        );
        let plan_of_sp_values = state.get_array_or_default_to_empty(
            &format!("{}_operation_runner", name),
            &format!("{}_plan", name),
        );

        let plan: Vec<String> = plan_of_sp_values
            .iter()
            .filter(|val| val.is_string())
            .map(|y| y.to_string())
            .collect();

        // Log only when something changes and not every tick
        if plan_state_old != plan_state {
            log::info!(target: &format!("{}_operation_runner", name), "Plan current state: {plan_state}.");
            plan_state_old = plan_state.clone()
        }

        match PlanState::from_str(&plan_state) {
            PlanState::Initial => {
                plan_state = PlanState::Executing.to_string();
                plan_current_step = 0;
            }
            PlanState::Executing => {
                if plan.len() > plan_current_step as usize {
                    let operation = model
                        .operations
                        .iter()
                        .find(|op| op.name == plan[plan_current_step as usize].to_string())
                        .unwrap()
                        .to_owned();

                    let operation_state = state.get_string_or_default_to_unknown(
                        &format!("{}_operation_runner", name),
                        &format!("{}", operation.name),
                    );

                    let mut operation_information = state.get_string_or_default_to_unknown(
                        &format!("{}_operation_runner", name),
                        &format!("{}_information", operation.name),
                    );

                    let mut operation_retry_counter = state.get_int_or_default_to_zero(
                        &format!("{}_operation_runner", name),
                        &format!("{}_retry_counter", operation.name),
                    );

                    // let mut _operation_start_time = state.get_or_default_f64(
                    //     &format!("{}_operation_runner", name),
                    //     &format!("{}_start_time", operation.name),
                    // );

                    // Log only when something changes and not every tick
                    if operation_state_old != operation_state {
                        log::info!(target: &format!("{}_operation_runner", name), "Current state of operation {}: {}.", operation.name, operation_state);
                        operation_state_old = operation_state.clone()
                    }

                    if operation_information_old != operation_information {
                        log::info!(target: &format!("{}_operation_runner", name), "Current operation '{}' info: {}.", operation.name, operation_information);
                        operation_information_old = operation_information.clone()
                    }

                    match OperationState::from_str(&operation_state) {
                        OperationState::Initial => {
                            // let now = Instant::now();
                            if operation.eval_running(&state) {
                                new_state = operation.start_running(&new_state);
                                // _operation_start_time = Instant::now().duration_since(now).as_micros() as f64;
                            }
                        }
                        OperationState::Disabled => todo!(),
                        OperationState::Executing => {
                            if operation.can_be_completed(&state) {
                                new_state = operation.clone().complete_running(&new_state);
                                operation_information = "Completing operation.".to_string();
                            } else if operation.can_be_failed(&state) {
                                new_state = operation.clone().fail_running(&new_state);
                                operation_information = "Failing operation.".to_string();
                            } else {
                                operation_information = "Waiting to be completed.".to_string();
                            }
                        }
                        OperationState::Completed => {
                            operation_retry_counter = 0;
                            new_state = new_state.update(
                                &format!("{}_retry_counter", operation.name),
                                operation_retry_counter.to_spvalue(),
                            );
                            plan_current_step = plan_current_step + 1;
                        }
                        OperationState::Timedout => todo!(),
                        OperationState::Failed => {
                            if operation_retry_counter < operation.retries {
                                operation_retry_counter = operation_retry_counter + 1;
                                operation_information = format!(
                                    "Retrying. Retry nr. {} out of {}.",
                                    operation_retry_counter, operation.retries
                                );
                                new_state = operation.clone().retry_running(&new_state);
                                new_state = new_state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                            } else {
                                operation_retry_counter = 0;
                                new_state = new_state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                                operation_information =
                                    format!("No more retries left. Failing the plan: {:?}", plan);
                                plan_state = PlanState::Failed.to_string();
                            }
                        }
                        OperationState::UNKNOWN => (),
                    }

                    new_state = new_state.update(
                        &format!("{}_information", operation.name),
                        operation_information.to_spvalue(),
                    );
                } else {
                    plan_state = PlanState::Completed.to_string();
                }
            }
            PlanState::Paused => {}
            PlanState::Failed => {}
            PlanState::NotFound => {}
            PlanState::Completed => {}
            PlanState::Cancelled => {}
            PlanState::UNKNOWN => {}
        }

        new_state = new_state
            .update(&format!("{}_plan_state", name), plan_state.to_spvalue())
            .update(
                &format!("{}_plan_current_step", name),
                plan_current_step.to_spvalue(),
            )
            .update(&format!("{}_plan", name), plan.to_spvalue());

        let modified_state = state.get_diff_partial_state(&new_state);
        command_sender
            .send(StateManagement::SetPartialState(modified_state))
            .await?;

        interval.tick().await;
    }
}
