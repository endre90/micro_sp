use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{Duration, interval},
};

pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // For nicer logging
    let mut sop_state_old = "".to_string();
    let mut sop_old: Vec<String> = vec![];
    let mut operation_state_old = "".to_string();
    let mut operation_information_old = "".to_string();

    log::info!(target: &&format!("{}_sop_runner", sp_id), "Online.");

    // let sops = model.sops;

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;
        let mut new_state = state.clone();

        let mut sop_state = state.get_string_or_default_to_unknown(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_state", sp_id),
        );

        let mut sop_current_step = state.get_int_or_default_to_zero(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_current_step", sp_id),
        );
        let sop_id = state.get_string_or_default_to_unknown(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_id", sp_id),
        );

        let mut sop_enabled = state.get_bool_or_default_to_false(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_enabled", sp_id),
        );

        // Log only when something changes and not every tick
        if sop_state_old != sop_state {
            log::info!(target: &format!("{}_sop_runner", sp_id), "SOP current state: {sop_state}.");
            sop_state_old = sop_state.clone()
        }

        match SOPState::from_str(&sop_state) {
            SOPState::Initial => {
                if sop_enabled {
                    sop_state = SOPState::Executing.to_string();
                    sop_enabled = false;
                }
            }
            SOPState::Executing => {
                let sop_struct = &model
                    .sops
                    .iter()
                    .find(|sop| sop.id == sop_id.to_string())
                    .unwrap()
                    .to_owned();

                if sop_old != sop_struct.sop {
                    log::info!(
                        target: &format!("{}_sop_runner", sp_id),
                        "Got a sop:\n{}",
                        sop_struct.sop.iter()
                            .enumerate()
                            .map(|(index, step)| format!("       {} -> {}", index + 1, step))
                            .collect::<Vec<String>>()
                            .join("\n")
                    );
                    sop_old = sop_struct.sop.clone()
                }

                if sop_struct.sop.len() > sop_current_step as usize {
                    let operation = &model
                        .operations
                        .iter()
                        .find(|op| op.name == sop_struct.sop[sop_current_step as usize].to_string())
                        .unwrap()
                        .to_owned();

                    let operation_state = state.get_string_or_default_to_unknown(
                        &format!("{}_sop_runner", sp_id),
                        &format!("{}", operation.name),
                    );

                    let mut operation_information = state.get_string_or_default_to_unknown(
                        &format!("{}_sop_runner", sp_id),
                        &format!("{}_information", operation.name),
                    );

                    let mut operation_retry_counter = state.get_int_or_default_to_zero(
                        &format!("{}_sop_runner", sp_id),
                        &format!("{}_retry_counter", operation.name),
                    );

                    // Log only when something changes and not every tick
                    if operation_state_old != operation_state {
                        log::info!(target: &format!("{}_sop_runner", sp_id), "Current state of operation {}: {}.", operation.name, operation_state);
                        operation_state_old = operation_state.clone()
                    }

                    if operation_information_old != operation_information {
                        log::info!(target: &format!("{}_sop_runner", sp_id), "{}.", operation_information);
                        operation_information_old = operation_information.clone()
                    }

                    let operation_start_time = state.get_int_or_default_to_zero(
                        &format!("{}_sop_runner", sp_id),
                        &format!("{}_start_time", operation.name),
                    );

                    match OperationState::from_str(&operation_state) {
                        OperationState::Initial => {
                            if operation.eval_running(&new_state) {
                                new_state = operation.start_running(&new_state);
                                operation_information =
                                    format!("Operation '{}' started execution", operation.name);
                            }
                            let (eval, idx) =
                                operation.eval_running_with_transition_index(&new_state);
                            if eval {
                                new_state = new_state.update(
                                    &format!("{}_start_time", operation.name),
                                    now_as_millis_i64().to_spvalue(),
                                );
                                tokio::time::sleep(Duration::from_millis(
                                    operation.preconditions[idx].delay_ms,
                                ))
                                .await;
                                new_state = operation.start_running(&new_state);
                                operation_information =
                                    format!("Operation '{}' started execution", operation.name);
                            }
                            else {
                                new_state = operation.block_running(&new_state);
                            }
                        }
                        OperationState::Blocked => {
                            // if operation.eval_running(&new_state) {
                            //     new_state = operation.start_running(&new_state);
                            //     operation_information =
                            //         format!("Operation '{}' started execution", operation.name);
                            // }
                            let (eval, idx) =
                                operation.eval_running_with_transition_index(&new_state);
                            if eval {
                                new_state = operation.start_running(&new_state);
                                operation_information =
                                    format!("Operation '{}' started execution", operation.name);
                            } else {
                                operation_information = format!(
                                    "Operation '{}' can't start yet, blocked by guard: {}",
                                    operation.name, operation.preconditions[idx].runner_guard
                                );
                            }
                        }

                        // probbaly causeing problems
                        OperationState::Executing => {
                            match operation.timeout_ms {
                                Some(timeout) => {
                                    if operation_start_time > 0 {
                                    let elapsed_ms =
                                        now_as_millis_i64().saturating_sub(operation_start_time);
                                    if elapsed_ms >= timeout {
                                        // log::error!(target: &format!("{}_sop_runner", sp_id), "HAS TO TIMEOUT HERE!");
                                        new_state = operation.timeout_running(&new_state);
                                        operation_information =
                                            format!("Operation '{}' timed out", operation.name);
                                    } else {
                                        if operation.can_be_failed(&new_state) {
                                            // log::error!(target: &format!("{}_sop_runner", sp_id), "HAS TO FAIL HERE!");
                                            new_state = operation.clone().fail_running(&new_state);
                                            operation_information =
                                                format!("Failing {}", operation.name);
                                        } else {
                                            let (eval, idx) = operation
                                                .can_be_completed_with_transition_index(&new_state);
                                            tokio::time::sleep(Duration::from_millis(
                                                operation.postconditions[idx].delay_ms,
                                            ))
                                            .await;
                                            if eval {
                                                // log::error!(target: &format!("{}_sop_runner", sp_id), "HAS TO COMPLETE HERE!");
                                                new_state =
                                                    operation.clone().complete_running(&new_state);
                                                operation_information =
                                                    format!("Completing {}", operation.name);
                                            } else {
                                                operation_information = format!(
                                                    "Waiting for {} to be completed",
                                                    operation.name
                                                );
                                            }
                                        }
                                        }
                                    }
                                }
                                None => {
                                    if operation.can_be_failed(&new_state) {
                                        new_state = operation.clone().fail_running(&new_state);
                                        operation_information =
                                            format!("Failing {}", operation.name);
                                    } else {
                                        let (eval, idx) = operation
                                            .can_be_completed_with_transition_index(&new_state);
                                        tokio::time::sleep(Duration::from_millis(
                                            operation.postconditions[idx].delay_ms,
                                        ))
                                        .await;
                                        if eval {
                                            new_state =
                                                operation.clone().complete_running(&new_state);
                                            operation_information =
                                                format!("Completing {}", operation.name);
                                        } else {
                                            operation_information = format!(
                                                "Waiting for {} to be completed",
                                                operation.name
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        // OperationState::Executing => {
                        //     if operation.can_be_completed(&state) {
                        //         new_state = operation.clone().complete_running(&new_state);
                        //         operation_information = "Completing operation.".to_string();
                        //     } else if operation.can_be_failed(&state) {
                        //         new_state = operation.clone().fail_running(&new_state);
                        //         operation_information = "Failing operation.".to_string();
                        //     } else {
                        //         operation_information = "Waiting to be completed.".to_string();
                        //     }
                        // }
                        OperationState::Completed => {
                            new_state = operation.reinitialize_running(&new_state);
                            operation_information =
                                format!("Operation {} completed, reinitializeing", operation.name);
                            new_state = new_state.update(
                                &format!("{}_retry_counter", operation.name),
                                0.to_spvalue(),
                            );
                            new_state = new_state
                                .update(&format!("{}_start_time", operation.name), 0.to_spvalue());
                            sop_current_step = sop_current_step + 1;
                        }
                        OperationState::Timedout => {
                            new_state = operation.unrecover_running(&new_state);
                            operation_information =
                                format!("Timedout {}. Unrecoverable", operation.name);
                        }
                        OperationState::Failed => {
                            if operation_retry_counter < operation.retries {
                                operation_retry_counter = operation_retry_counter + 1;
                                operation_information = format!(
                                    "Retrying '{}'. Retry nr. {} out of {}",
                                    operation.name, operation_retry_counter, operation.retries
                                );
                                new_state = operation.clone().retry_running(&new_state);
                                new_state = new_state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                            } else {
                                new_state = operation.unrecover_running(&new_state);
                                new_state = new_state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    0.to_spvalue(),
                                );
                                operation_information = format!(
                                    "Operation failed, no more retries left. Unrecoverable"
                                );
                            }
                        }
                        OperationState::Unrecoverable => {
                            sop_state = SOPState::Failed.to_string();
                            new_state = operation.reinitialize_running(&new_state);
                            operation_information = format!("Failing the sop: {:?}", sop_struct);
                        }
                        OperationState::UNKNOWN => (),
                    }
                    new_state = new_state.update(
                        &format!("{}_information", operation.name),
                        operation_information.to_spvalue(),
                    );
                } else {
                    sop_state = SOPState::Completed.to_string();
                }
            }
            // PlanState::Paused => {}
            SOPState::Failed => {
                // sop_state = SOPState::Initial.to_string();
                // planner_state = PlannerState::Ready.to_string();
            }
            // PlanState::NotFound => {}
            SOPState::Completed => {
                // sop_state = SOPState::Initial.to_string();
                // planner_state = PlannerState::Ready.to_string();
            }
            // PlanState::Cancelled => {}
            SOPState::UNKNOWN => {
                // sop_state = SOPState::Initial.to_string();
                // planner_state = PlannerState::Ready.to_string();
            }
        }
        // }
        new_state = new_state
            .update(&format!("{}_sop_state", sp_id), sop_state.to_spvalue())
            .update(&format!("{}_sop_enabled", sp_id), sop_enabled.to_spvalue())
            .update(
                &format!("{}_sop_current_step", sp_id),
                sop_current_step.to_spvalue(),
            );

        let modified_state = state.get_diff_partial_state(&new_state);
        command_sender
            .send(StateManagement::SetPartialState(modified_state))
            .await?;

        interval.tick().await;
    }
}
