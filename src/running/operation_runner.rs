use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

pub async fn operation_runner(
    model: &Model,
    command_sender: mpsc::Sender<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = &model.name;
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    loop {
        // current try:
        // read the whole state, take the transition to produce a new state
        // then take the diff from the new state compared to the old state and send a request to change only those values

        let (response_tx, response_rx) = oneshot::channel();
        command_sender.send(Command::GetState(response_tx)).await?; // TODO: maybe we can just ask for values from the guard
        let mut state = response_rx.await?;

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

        match PlanState::from_str(&plan_state) {
            PlanState::Initial => {
                log::info!(target: &&format!("{}_operation_runner", name), "Current state of plan '{}': Initial.", name);
                log::info!(target: &&format!("{}_operation_runner", name), "Starting plan: '{:?}'.", plan);
                plan_state = PlanState::Executing.to_string();
            }
            PlanState::Executing => {
                log::info!(target: &&format!("{}_operation_runner", name), "Current state of plan '{}': Executing.", name);
                log::info!(target: &&format!("{}_operation_runner", name), "Executing plan: '{:?}'.", plan);

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

                    match OperationState::from_str(&operation_state) {
                        OperationState::Initial => {
                            log::info!(target: &&format!("{}_operation_runner", name), 
                                "Current state of operation '{}': Initial.", operation.name);
                            if operation.eval_running(&state) {
                                log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Starting operation: '{}'.", operation.name);
                                state = operation.start_running(&state);
                            }
                        }
                        OperationState::Disabled => todo!(),
                        OperationState::Executing => {
                            log::info!(target: &&format!("{}_operation_runner", name), 
                            "Current state of operation '{}': Executing.", operation.name);
                            if operation.can_be_completed(&state) {
                                state = operation.clone().complete_running(&state);
                                log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Completing operation: '{}'.", operation.name);
                            } else if operation.can_be_failed(&state) {
                                state = operation.clone().fail_running(&state);
                                log::error!(target: &&format!("{}_operation_runner", name), 
                                        "Failing operation: '{}'.", operation.name);
                            } else {
                                log::info!(target: &&format!("{}_operation_runner", name), 
                                    "Waiting for operation: '{}' to be completed.", operation.name);
                            }
                        }
                        OperationState::Completed => {
                            log::info!(target: &&format!("{}_runner", name), 
                                "Current state of operation '{}': Completed.", operation.name);
                            operation_retry_counter = 0;
                            state = state.update(
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
                            log::error!(target: &&format!("{}_operation_runner", name), 
                                        "Operation: '{}' has failed.", operation.name);

                            if operation_retry_counter < operation.retries {
                                operation_retry_counter = operation_retry_counter + 1;
                                log::error!(target: &&format!("{}_operation_runner", name), 
                                    "Retrying operation: '{}'. Retry nr. {} out of {}.", operation.name, operation_retry_counter, operation.retries);
                                state = operation.clone().retry_running(&state);
                                state = state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                            } else {
                                operation_retry_counter = 0;
                                state = state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                                log::error!(target: &&format!("{}_operation_runner", name), 
                                        "Failing the plan '{} : {:?}'.", name, plan);
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

        let new_state = state
            .update(&format!("{}_plan_state", name), plan_state.to_spvalue())
            .update(
                &format!("{}_plan_current_step", name),
                plan_current_step.to_spvalue(),
            )
            .update(&format!("{}_plan", name), plan.to_spvalue());

        let modified = state.get_diff(&new_state);
        for x in modified {
            command_sender.send(Command::Set((x.0, x.1 .1))).await?;
        }

        interval.tick().await;
    }
}
