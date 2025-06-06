use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{Duration, interval},
};

// pub async fn cycle_operation(
//     sp_id: &str,
//     operation: Operation,
//     state: State,
//     // operation_state_old: String,
//     // operation_information_old: String,
// ) -> State {
//     // ) -> Result<OperationState, Box<dyn std::error::Error>> {

//     let mut new_state = state.clone();

//     let operation_state = state.get_string_or_default_to_unknown(
//         &format!("{}_cycle_operation", sp_id),
//         &format!("{}", operation.name),
//     );

//     let mut operation_information = state.get_string_or_default_to_unknown(
//         &format!("{}_cycle_operation", sp_id),
//         &format!("{}_information", operation.name),
//     );

//     let mut operation_retry_counter = state.get_int_or_default_to_zero(
//         &format!("{}_cycle_operation", sp_id),
//         &format!("{}_retry_counter", operation.name),
//     );

//     // // Log only when something changes and not every tick
//     // if operation_state_old != operation_state {
//     //     log::info!(target: &format!("{}_single_operation_runner", sp_id), "Current state of operation {}: {}.", operation.name, operation_state);
//     //     operation_state_old = operation_state.clone()
//     // }

//     // if operation_information_old != operation_information {
//     //     log::info!(target: &format!("{}_single_operation_runner", sp_id), "Current operation '{}' info: {}.", operation.name, operation_information);
//     //     operation_information_old = operation_information.clone()
//     // }

//     let operation_start_time = state.get_int_or_default_to_zero(
//         &format!("{}_operation_runner", sp_id),
//         &format!("{}_start_time", operation.name),
//     );

//     match OperationState::from_str(&operation_state) {
//         OperationState::Initial => {
//             let (eval, idx) = operation.eval_running_with_transition_index(&state);
//             if eval {
//                 new_state = new_state.update(
//                     &format!("{}_start_time", operation.name),
//                     now_as_millis_i64().to_spvalue(),
//                 );
//                 tokio::time::sleep(Duration::from_millis(
//                     operation.preconditions[idx].pre_action_delay_ms,
//                 ))
//                 .await;
//                 new_state = operation.start_running(&new_state);
//             }
//         }
//         OperationState::Executing => {
//             match operation.timeout_ms {
//                 Some(timeout) => {
//                     if operation_start_time > 0 {
//                         let elapsed_ms = now_as_millis_i64().saturating_sub(operation_start_time);
//                         if elapsed_ms >= timeout {
//                             log::warn!(target: &format!("{}_operation_runner", sp_id),
//                                 "Operation '{}' timed out after {}ms.", operation.name, elapsed_ms);
//                             new_state = operation.timeout_running(&state);
//                         }
//                     }
//                 }
//                 None => (),
//             }

//             if operation.can_be_completed(&state) {
//                 new_state = operation.clone().complete_running(&new_state);
//                 operation_information = "Completing operation.".to_string();
//             } else if operation.can_be_failed(&state) {
//                 new_state = operation.clone().fail_running(&new_state);
//                 operation_information = "Failing operation.".to_string();
//             } else {
//                 operation_information = "Waiting to be completed.".to_string();
//             }
//         }
//         OperationState::Completed => {
//             operation_retry_counter = 0;
//             new_state = new_state.update(
//                 &format!("{}_retry_counter", operation.name),
//                 operation_retry_counter.to_spvalue(),
//             );

//             new_state = new_state.update(
//                 &format!("{}_information", operation.name),
//                 operation_information.to_spvalue(),
//             );

//             // new_state = state.get_diff_partial_state(&new_state);
//         }
//         OperationState::Timedout => {
//             operation_information = format!("Operation '{}' timedout.", operation.name);
//             new_state = new_state.update(
//                 &format!("{}_information", operation.name),
//                 operation_information.to_spvalue(),
//             );
//         }
//         OperationState::Failed => {
//             if operation_retry_counter < operation.retries {
//                 operation_retry_counter = operation_retry_counter + 1;
//                 operation_information = format!(
//                     "Operation failed. Retrying. Retry nr. {} out of {}.",
//                     operation_retry_counter, operation.retries
//                 );
//                 new_state = operation.clone().retry_running(&new_state);
//                 new_state = new_state.update(
//                     &format!("{}_retry_counter", operation.name),
//                     operation_retry_counter.to_spvalue(),
//                 );
//                 new_state = new_state.update(
//                     &format!("{}_information", operation.name),
//                     operation_information.to_spvalue(),
//                 );

//                 // new_state = state.get_diff_partial_state(&new_state);
//             } else {
//                 operation_retry_counter = 0;
//                 new_state = new_state.update(
//                     &format!("{}_retry_counter", operation.name),
//                     operation_retry_counter.to_spvalue(),
//                 );
//                 operation_information = format!("No more retries left. Operation failed unrecoverably.");
//                 new_state = new_state.update(
//                     &format!("{}_information", operation.name),
//                     operation_information.to_spvalue(),
//                 );

//                 new_state = operation.unrecover_running(&new_state);
//                 // new_state = state.get_diff_partial_state(&new_state);
//             }
//         }
//         OperationState::Unrecoverable => {
//             operation_information = format!("Operation unrecoverable.");
//             new_state = new_state.update(
//                 &format!("{}_information", operation.name),
//                 operation_information.to_spvalue(),
//             );
//         }
//         OperationState::UNKNOWN => (),
//     }

//     new_state
// }

// pub async fn plan_runner(
//     sp_id: &str,
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();

//     // For nicer logging
//     let mut plan_state_old = "".to_string();
//     // let mut operation_state_old = "".to_string();
//     // let mut operation_information_old = "".to_string();

//     log::info!(target: &&format!("{}_plan_runner", sp_id), "Online.");

//     loop {
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender
//             .send(StateManagement::GetState(response_tx))
//             .await?;
//         let state = response_rx.await?;
//         let mut new_state = state.clone();

//         let mut replan_trigger = state.get_bool_or_default_to_false(
//             &format!("{}_planner_ticker", sp_id),
//             &format!("{}_replan_trigger", sp_id),
//         );
//         let replanned = state.get_bool_or_default_to_false(
//             &format!("{}_planner_ticker", sp_id),
//             &format!("{}_replanned", sp_id),
//         );

//         let mut plan_state = state.get_string_or_default_to_unknown(
//             &format!("{}_plan_runner", sp_id),
//             &format!("{}_plan_state", sp_id),
//         );
//         let mut plan_current_step = state.get_int_or_default_to_zero(
//             &format!("{}_plan_runner", sp_id),
//             &format!("{}_plan_current_step", sp_id),
//         );
//         let plan_of_sp_values = state.get_array_or_default_to_empty(
//             &format!("{}_plan_runner", sp_id),
//             &format!("{}_plan", sp_id),
//         );

//         let plan: Vec<String> = plan_of_sp_values
//             .iter()
//             .filter(|val| val.is_string())
//             .map(|y| y.to_string())
//             .collect();

//         // Log only when something changes and not every tick
//         if plan_state_old != plan_state {
//             log::info!(target: &format!("{}_plan_runner", sp_id), "Plan current state: {plan_state}.");
//             plan_state_old = plan_state.clone()
//         }

//         match PlanState::from_str(&plan_state) {
//             PlanState::Initial => {
//                 plan_state = PlanState::Executing.to_string();
//                 replan_trigger = false;
//             }
//             PlanState::Executing => {
//                 if plan.len() > plan_current_step as usize {
//                     let operation = model
//                         .operations
//                         .iter()
//                         .find(|op| op.name == plan[plan_current_step as usize].to_string())
//                         .unwrap()
//                         .to_owned();

//                     new_state = cycle_operation(sp_id, operation.clone(), state.clone()).await;
//                     let operation_state = new_state.get_string_or_default_to_unknown(
//                         &format!("{}_plan_runner", sp_id),
//                         &format!("{}", operation.name),
//                     );
//                     match OperationState::from_str(&operation_state) {
//                         OperationState::Completed => {
//                             plan_current_step = plan_current_step + 1;
//                         }
//                         // If retries have need exhausted, fail the plan
//                         OperationState::Abandoned => {
//                             plan_state = PlanState::Failed.to_string();
//                         }
//                         _ => (),
//                     }
//                 } else {
//                     plan_state = PlanState::Completed.to_string();
//                 }
//             }
//             // PlanState::Paused => {}
//             PlanState::Failed => {}
//             // PlanState::NotFound => {}
//             PlanState::Completed => {}
//             PlanState::UNKNOWN => {}
//         }

//         new_state = new_state
//             .update(&format!("{}_plan_state", sp_id), plan_state.to_spvalue())
//             .update(
//                 &format!("{}_plan_current_step", sp_id),
//                 plan_current_step.to_spvalue(),
//             )
//             .update(&format!("{}_plan", sp_id), plan.to_spvalue())
//             .update(
//                 &format!("{}_replan_trigger", sp_id),
//                 replan_trigger.to_spvalue(),
//             );

//         let modified_state = state.get_diff_partial_state(&new_state);
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;

//         interval.tick().await;
//     }
// }

// // Super simple for now only sequences, later extend with alternative, paralell, loops, etc.
// pub async fn sop_runner(
//     sp_id: &str,
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();

//     // For nicer logging
//     let mut sop_state_old = "".to_string();
//     let mut sop_old: Vec<String> = vec![];
//     // let mut operation_state_old = "".to_string();
//     // let mut operation_information_old = "".to_string();

//     log::info!(target: &&format!("{}_sop_runner", sp_id), "Online.");

//     loop {
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender
//             .send(StateManagement::GetState(response_tx))
//             .await?;
//         let state = response_rx.await?;
//         let mut new_state = state.clone();

//         let mut sop_state = state.get_string_or_default_to_unknown(
//             &format!("{}_sop_runner", sp_id),
//             &format!("{}_sop_state", sp_id),
//         );
//         let mut sop_current_step = state.get_int_or_default_to_zero(
//             &format!("{}_sop_runner", sp_id),
//             &format!("{}_sop_current_step", sp_id),
//         );
//         let sop_id = state.get_string_or_default_to_unknown(
//             &format!("{}_sop_runner", sp_id),
//             &format!("{}_sop_id", sp_id),
//         );

//         let mut sop_request_trigger = state.get_bool_or_default_to_false(
//             &format!("{}_sop_runner", sp_id),
//             &format!("{}_sop_request_trigger", sp_id),
//         );

//         // Log only when something changes and not every tick
//         if sop_state_old != sop_state {
//             log::info!(target: &format!("{}_sop_runner", sp_id), "SOP current state: {sop_state}.");
//             sop_state_old = sop_state.clone()
//         }

//         if sop_request_trigger {
//             sop_state = ActionRequestState::Executing.to_string();
//             // sop_current_step = 0;
//             let sop = model
//                 .sops
//                 .iter()
//                 .find(|sop| sop.id == sop_id.to_string())
//                 .unwrap()
//                 .to_owned();

//             if sop_old != sop.sop {
//                 log::info!(
//                     target: &format!("{}_sop_runner", sp_id),
//                     "Got a sop:\n{}",
//                     sop.sop.iter()
//                         .enumerate()
//                         .map(|(index, step)| format!("       {} -> {}", index + 1, step))
//                         .collect::<Vec<String>>()
//                         .join("\n")
//                 );
//                 sop_old = sop.sop.clone()
//             }

//             match ActionRequestState::from_str(&sop_state) {
//                 ActionRequestState::Initial => {}
//                 ActionRequestState::Executing => {
//                     if sop.sop.len() > sop_current_step as usize {
//                         let operation = model
//                             .operations
//                             .iter()
//                             .find(|op| op.name == sop.sop[sop_current_step as usize].to_string())
//                             .unwrap()
//                             .to_owned();

//                         new_state = cycle_operation(
//                             &format!("{sp_id}_sop"),
//                             operation.clone(),
//                             state.clone(),
//                         )
//                         .await;
//                         let operation_state = new_state.get_string_or_default_to_unknown(
//                             &format!("{}_sop_runner", sp_id),
//                             &format!("{}", operation.name),
//                         );
//                         match OperationState::from_str(&operation_state) {
//                             OperationState::Completed => {
//                                 log::info!(target: &format!("{}_sop_runner", sp_id), "Completed: {}.", operation.name);
//                                 sop_current_step = sop_current_step + 1;
//                             }
//                             // If retries have need exhausted, fail the sop
//                             OperationState::Unrecoverable => {
//                                 log::info!(target: &format!("{}_sop_runner", sp_id), "Abandoned: {}.", operation.name);
//                                 sop_state = ActionRequestState::Failed.to_string();
//                             }
//                             _ => (),
//                         }
//                     } else {
//                         sop_state = ActionRequestState::Succeeded.to_string();
//                     }
//                 }
//                 ActionRequestState::Succeeded => {
//                     sop_request_trigger = false;
//                     log::info!(target: &&format!("{}_sop_runner", sp_id), "SOP suceeded.");
//                 }
//                 ActionRequestState::Failed => {
//                     sop_request_trigger = false;
//                     log::info!(target: &&format!("{}_sop_runner", sp_id), "SOP failed.");
//                 }
//                 ActionRequestState::UNKNOWN => {}
//             }
//         }

//         new_state = new_state
//             .update(&format!("{}_sop_state", sp_id), sop_state.to_spvalue())
//             .update(
//                 &format!("{}_sop_current_step", sp_id),
//                 sop_current_step.to_spvalue(),
//             )
//             .update(
//                 &format!("{}_sop_request_trigger", sp_id),
//                 sop_request_trigger.to_spvalue(),
//             );

//         let modified_state = state.get_diff_partial_state(&new_state);
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;

//         interval.tick().await;
//     }
// }

// No planner, just runner. In this case the model has to be different
// pub fn simple_single_operation_runner() {}

// This is working below!!!
// /// A planned operation runner is an algorithm which executes the plan P based on the model
// /// M, the current state of the system S, and a goal predicate G. While
// /// running, both the planning and running components of guards and actions
// /// of operation pre- and postconditions are evaluated and taken.
pub async fn planned_operation_runner(
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sp_id = &model.name;
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // For nicer logging
    let mut plan_state_old = "".to_string();
    let mut operation_state_old = "".to_string();
    let mut operation_information_old = "".to_string();
    let mut plan_current_step_old = 0;

    log::info!(target: &&format!("{}_operation_runner", sp_id), "Online.");

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;
        let mut new_state = state.clone();

        let mut planner_state = state.get_string_or_default_to_unknown(
            &format!("{}_operation_runner", sp_id),
            &format!("{}_planner_state", sp_id),
        );

        let mut plan_state = state.get_string_or_default_to_unknown(
            &format!("{}_operation_runner", sp_id),
            &format!("{}_plan_state", sp_id),
        );
        let mut plan_current_step = state.get_int_or_default_to_zero(
            &format!("{}_operation_runner", sp_id),
            &format!("{}_plan_current_step", sp_id),
        );
        let plan_of_sp_values = state.get_array_or_default_to_empty(
            &format!("{}_operation_runner", sp_id),
            &format!("{}_plan", sp_id),
        );

        let plan: Vec<String> = plan_of_sp_values
            .iter()
            .filter(|val| val.is_string())
            .map(|y| y.to_string())
            .collect();

        // Log only when something changes and not every tick
        if plan_state_old != plan_state {
            log::info!(target: &format!("{}_operation_runner", sp_id), "Plan current state: {plan_state}.");
            plan_state_old = plan_state.clone()
        }

        // Log only when something changes and not every tick
        if plan_current_step_old != plan_current_step {
            log::info!(target: &format!("{}_operation_runner", sp_id), "Plan current step: {plan_current_step}.");
            plan_current_step_old = plan_current_step
        }

        match PlanState::from_str(&plan_state) {
            PlanState::Initial => {
                if planner_state == PlannerState::Found.to_string() {
                    plan_state = PlanState::Executing.to_string();
                    plan_current_step = 0;
                }
                planner_state = PlannerState::Ready.to_string();
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
                        &format!("{}_operation_runner", sp_id),
                        &format!("{}", operation.name),
                    );

                    let mut operation_information = state.get_string_or_default_to_unknown(
                        &format!("{}_operation_runner", sp_id),
                        &format!("{}_information", operation.name),
                    );

                    let mut operation_retry_counter = state.get_int_or_default_to_zero(
                        &format!("{}_operation_runner", sp_id),
                        &format!("{}_retry_counter", operation.name),
                    );

                    // Log only when something changes and not every tick
                    if operation_state_old != operation_state {
                        log::info!(target: &format!("{}_operation_runner", sp_id), "Current state of operation {}: {}.", operation.name, operation_state);
                        operation_state_old = operation_state.clone()
                    }

                    if operation_information_old != operation_information {
                        log::info!(target: &format!("{}_operation_runner", sp_id), "{}.", operation_information);
                        operation_information_old = operation_information.clone()
                    }

                    let operation_start_time = state.get_int_or_default_to_zero(
                        &format!("{}_operation_runner", sp_id),
                        &format!("{}_start_time", operation.name),
                    );

                    match OperationState::from_str(&operation_state) {
                        OperationState::Initial => {
                            let (eval, idx) = operation.eval_running_with_transition_index(&state);
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
                            } else {
                                new_state = operation.block_running(&new_state);
                            }
                        }
                        OperationState::Blocked => {
                            let (eval, idx) = operation.eval_running_with_transition_index(&state);
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
                        OperationState::Executing => {
                            match operation.timeout_ms {
                                Some(timeout) => {
                                    if operation_start_time > 0 {
                                        let elapsed_ms = now_as_millis_i64()
                                            .saturating_sub(operation_start_time);
                                        if elapsed_ms >= timeout {
                                            operation_information = format!(
                                                "Operation '{}' timed out.",
                                                operation.name
                                            );
                                            new_state = operation.timeout_running(&state);
                                        } else {
                                            if operation.can_be_failed(&state) {
                                                new_state =
                                                    operation.clone().fail_running(&new_state);
                                                operation_information =
                                                    format!("Failing {}.", operation.name);
                                            } else {
                                                let (eval, idx) = operation
                                                    .can_be_completed_with_transition_index(&state);
                                                tokio::time::sleep(Duration::from_millis(
                                                    operation.postconditions[idx].delay_ms,
                                                ))
                                                .await;
                                                if eval {
                                                    new_state = operation
                                                        .clone()
                                                        .complete_running(&new_state);
                                                    operation_information =
                                                        format!("Completing {}.", operation.name);
                                                } else {
                                                    operation_information = format!(
                                                        "Waiting for {} to be completed.",
                                                        operation.name
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                None => {
                                    if operation.can_be_failed(&state) {
                                        new_state = operation.clone().fail_running(&new_state);
                                        operation_information =
                                            format!("Failing {}.", operation.name);
                                    } else {
                                        let (eval, idx) = operation
                                            .can_be_completed_with_transition_index(&state);
                                        tokio::time::sleep(Duration::from_millis(
                                            operation.postconditions[idx].delay_ms,
                                        ))
                                        .await;
                                        if eval {
                                            new_state =
                                                operation.clone().complete_running(&new_state);
                                            operation_information =
                                                format!("Completing {}.", operation.name);
                                        } else {
                                            operation_information = format!(
                                                "Waiting for {} to be completed.",
                                                operation.name
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        OperationState::Completed => {
                            new_state = operation.reinitialize_running(&state);
                            operation_information =
                                format!("Operation {} completed, reinitializeing.", operation.name);
                            new_state = new_state.update(
                                &format!("{}_retry_counter", operation.name),
                                0.to_spvalue(),
                            );
                            new_state = new_state
                                .update(&format!("{}_start_time", operation.name), 0.to_spvalue());
                            plan_current_step = plan_current_step + 1;
                        }
                        OperationState::Timedout => {
                            new_state = operation.unrecover_running(&state);
                            operation_information =
                                format!("Timedout {}. Unrecoverable.", operation.name);
                        }
                        OperationState::Failed => {
                            if operation_retry_counter < operation.retries {
                                operation_retry_counter = operation_retry_counter + 1;
                                operation_information = format!(
                                    "Retrying '{}'. Retry nr. {} out of {}.",
                                    operation.name, operation_retry_counter, operation.retries
                                );
                                new_state = operation.clone().retry_running(&new_state);
                                new_state = new_state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    operation_retry_counter.to_spvalue(),
                                );
                            } else {
                                new_state = operation.unrecover_running(&state);
                                new_state = new_state.update(
                                    &format!("{}_retry_counter", operation.name),
                                    0.to_spvalue(),
                                );
                                operation_information = format!(
                                    "Operation failed, no more retries left. Unrecoverable."
                                );
                            }
                        }
                        OperationState::Unrecoverable => {
                            plan_state = PlanState::Failed.to_string();
                            new_state = operation.reinitialize_running(&state);
                            operation_information = format!("Failing the plan: {:?}", plan);
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
            // PlanState::Paused => {}
            PlanState::Failed => {
                plan_state = PlanState::Initial.to_string();
                planner_state = PlannerState::Ready.to_string();
            }
            // PlanState::NotFound => {}
            PlanState::Completed => {
                plan_state = PlanState::Initial.to_string();
                planner_state = PlannerState::Ready.to_string();
            }
            // PlanState::Cancelled => {}
            PlanState::UNKNOWN => {
                plan_state = PlanState::Initial.to_string();
                planner_state = PlannerState::Ready.to_string();
            }
        }

        new_state = new_state
            .update(&format!("{}_plan_state", sp_id), plan_state.to_spvalue())
            .update(
                &format!("{}_planner_state", sp_id),
                planner_state.to_spvalue(),
            )
            .update(
                &format!("{}_plan_current_step", sp_id),
                plan_current_step.to_spvalue(),
            )
            .update(&format!("{}_plan", sp_id), plan.to_spvalue());

        let modified_state = state.get_diff_partial_state(&new_state);
        command_sender
            .send(StateManagement::SetPartialState(modified_state))
            .await?;

        interval.tick().await;
    }
}

// // No planner, just runner. In this case the model has to be different
// // pub fn simple_single_operation_runner() {}
