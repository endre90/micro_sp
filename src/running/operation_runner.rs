use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

/// A runner is an algorithm which executes the plan P based on the model
/// M, the current state of the system S, and a goal predicate G. While
/// running, both the planning and running components of guards and actions
/// of operation pre- and postconditions are evaluated and taken.
pub async fn operation_runner(
    model: &Model,
    command_sender: mpsc::Sender<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = &model.name;
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // For nicer logging
    let mut plan_state_old = "".to_string();
    let mut operation_state_old = "".to_string();

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender.send(Command::GetState(response_tx)).await?;
        let state = response_rx.await?;
        let mut new_state = state.clone();

        let mut plan_state = state.get_or_default_string(
            &format!("{}_operation_runner", name),
            &format!("{}_plan_state", name),
        );
        let mut plan_current_step = state.get_or_default_i64(
            &format!("{}_operation_runner", name),
            &format!("{}_plan_current_step", name),
        );
        let plan = state.get_or_default_array_of_strings(
            &format!("{}_operation_runner", name),
            &format!("{}_plan", name),
        );

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

                    let operation_state = state.get_or_default_string(
                        &format!("{}_operation_runner", name),
                        &format!("{}", operation.name),
                    );

                    let mut operation_retry_counter = state.get_or_default_i64(
                        &format!("{}_operation_runner", name),
                        &format!("{}_retry_counter", operation.name),
                    );

                    // Log only when something changes and not every tick
                    if operation_state_old != operation_state {
                        log::info!(target: &format!("{}_operation_runner", name), "Current state of operation {}: {}.", operation.name, operation_state);
                        operation_state_old = operation_state.clone()
                    }

                    match OperationState::from_str(&operation_state) {
                        OperationState::Initial => {
                            // log::info!(target: &&format!("{}_operation_runner", name), 
                            //     "Current state of operation '{}': Initial.", operation.name);
                            if operation.eval_running(&state) {
                                // log::info!(target: &&format!("{}_operation_runner", name), 
                                //     "Starting operation: '{}'.", operation.name);
                                new_state = operation.start_running(&new_state);
                            }
                        }
                        OperationState::Disabled => todo!(),
                        OperationState::Executing => {
                            // log::info!(target: &&format!("{}_operation_runner", name), 
                            // "Current state of operation '{}': Executing.", operation.name);
                            if operation.can_be_completed(&state) {
                                new_state = operation.clone().complete_running(&new_state);
                                log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Completing operation: '{}'.", operation.name);
                            } else if operation.can_be_failed(&state) {
                                new_state = operation.clone().fail_running(&new_state);
                                log::error!(target: &&format!("{}_operation_runner", name), 
                                        "Failing operation: '{}'.", operation.name);
                            } else {
                                log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Waiting for operation: '{}' to be completed.", operation.name);
                            }
                        }
                        OperationState::Completed => {
                            // log::info!(target: &&format!("{}_runner", name), 
                            //     "Current state of operation '{}': Completed.", operation.name);
                            operation_retry_counter = 0;
                            new_state = new_state.update(
                                &format!("{}_retry_counter", operation.name),
                                operation_retry_counter.to_spvalue(),
                            );
                            plan_current_step = plan_current_step + 1;
                            // let current_model_operation = model
                            //     .operations
                            //     .iter()
                            //     .find(|op| op.name == current_model_operation.name)
                            //     .unwrap()
                            //     .to_owned();

                            //             if current_model_operation
                            //                 .clone()
                            //                 .can_be_reset(&shared_state_local)
                            //             {
                            //                 log::info!(target: &&format!("{}_runner", name),
                            // "Reseting operation: '{}'.", current_model_operation.name);

                            //                 let shared_state_local = shared_state.lock().unwrap().clone();
                            //                 let updated_state = current_model_operation
                            //                     .clone()
                            //                     .reset_running(&shared_state_local);
                            //                 *shared_state.lock().unwrap() = updated_state.clone();
                            //             }
                        }
                        OperationState::Timedout => todo!(),
                        OperationState::Failed => {
                            // log::error!(target: &&format!("{}_operation_runner", name), 
                            //             "Operation: '{}' has failed.", operation.name);

                            if operation_retry_counter < operation.retries {
                                operation_retry_counter = operation_retry_counter + 1;
                                log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Retrying operation: '{}'. Retry nr. {} out of {}.", operation.name, operation_retry_counter, operation.retries);
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
                                log::error!(target: &&format!("{}_operation_runner", name), 
                                        "Failing the plan '{:?}'.", plan);
                                plan_state = PlanState::Failed.to_string();
                            }
                        }
                        OperationState::UNKNOWN => (),
                    }
                } else {
                    log::info!(target: &&format!("{}_operation_runner", name), 
                "Completed plan: '{}'.", name);
                    plan_state = PlanState::Completed.to_string();
                }
            }
            PlanState::Paused => {
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Paused.", name)
            }
            PlanState::Failed => {
                log::error!(target: &&format!("{}_runner", name), "Current state of plan '{}': Failed.", name);
                // if operation has retried enough times it is time to fail and scrap the complete plan
            }
            PlanState::NotFound => {
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': NotFound.", name)
            }
            PlanState::Completed => {
                log::warn!(target: &&format!("{}_runner", name), "Current state of plan '{}': Completed.", name)
            }
            PlanState::Cancelled => {
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': Cancelled.", name)
            }
            PlanState::UNKNOWN => {
                log::info!(target: &&format!("{}_runner", name), "Current state of plan '{}': UNKNOWN.", name)
            }
        }

        let new_state = new_state
            .update(&format!("{}_plan_state", name), plan_state.to_spvalue())
            .update(
                &format!("{}_plan_current_step", name),
                plan_current_step.to_spvalue(),
            )
            .update(&format!("{}_plan", name), plan.to_spvalue());

        let modified_state = state.get_diff_partial_state(&new_state);
        command_sender
            .send(Command::SetPartialState(modified_state))
            .await?;

        interval.tick().await;
    }
}
