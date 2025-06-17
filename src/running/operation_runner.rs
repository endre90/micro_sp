use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{Duration, interval},
};

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

        // let replan_trigger = state.get_bool_or_default_to_false(
        //     &format!("{}_planner", sp_id),
        //     &format!("{}_replan_trigger", sp_id),
        // );
        // let replanned = state.get_bool_or_default_to_false(
        //     &format!("{}_planner", sp_id),
        //     &format!("{}_replanned", sp_id),
        // );

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

        // if replan_trigger && !replanned {
        //     plan_current_step = 0;
        //     plan = vec!();
        //     // plan_state = PlanState::Initial.to_string();
        //     new_state = reset_all_operations(&new_state);
        // }

        match PlanState::from_str(&plan_state) {
            PlanState::Initial => {

                // plan_current_step = 0;
                // plan_state = PlanState::Executing.to_string();
                if planner_state == PlannerState::Found.to_string() {
                    plan_state = PlanState::Executing.to_string();
                    plan_current_step = 0;
                }
                // planner_state = PlannerState::Ready.to_string();
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

                    // let operation_start_time = state.get_int_or_default_to_zero(
                    //     &format!("{}_operation_runner", sp_id),
                    //     &format!("{}_start_time", operation.name),
                    // );

                    match OperationState::from_str(&operation_state) {
                        OperationState::Initial => {
                            if operation.eval_running(&new_state) {
                                new_state = operation.start_running(&new_state);
                                operation_information =
                                    format!("Operation '{}' started execution", operation.name);
                            }
                        //     let (eval, idx) =
                        //         operation.eval_running_with_transition_index(&new_state);
                        //     if eval {
                        //         log::error!(target: &format!("{}_operation_runner", sp_id), "INITIAL, EVALS TO TRUE.");
                        //         new_state = new_state.update(
                        //             &format!("{}_start_time", operation.name),
                        //             now_as_millis_i64().to_spvalue(),
                        //         );
                        //         tokio::time::sleep(Duration::from_millis(
                        //             operation.preconditions[idx].delay_ms,
                        //         ))
                        //         .await;
                        //         new_state = operation.start_running(&new_state);
                        //         operation_information =
                        //             format!("Operation '{}' started execution", operation.name);
                        //     } else {
                        //         log::error!(target: &format!("{}_operation_runner", sp_id), "INITIAL, EVALS TO FALSE.");
                        //         new_state = operation.block_running(&new_state);
                        //     }
                        // }
                        // OperationState::Blocked => {
                        //     if operation.eval_running(&new_state) {
                        //         new_state = operation.start_running(&new_state);
                        //         operation_information =
                        //             format!("Operation '{}' started execution", operation.name);
                        //     }
                            // let (eval, idx) =
                            //     operation.eval_running_with_transition_index(&new_state);
                            // if eval {
                            //     log::error!(target: &format!("{}_operation_runner", sp_id), "BLOCKED, EVALS TO TRUE.");
                            //     new_state = operation.start_running(&new_state);
                            //     operation_information =
                            //         format!("Operation '{}' started execution", operation.name);
                            // } else {
                            //     log::error!(target: &format!("{}_operation_runner", sp_id), "BLOCKED, EVALS TO FALSE.");
                            //     log::error!(target: &format!("{}_operation_runner", sp_id), "GUARD: {}", operation.preconditions[idx].runner_guard);
                            //     operation_information = format!(
                            //         "Operation '{}' can't start yet, blocked by guard: {}",
                            //         operation.name, operation.preconditions[idx].runner_guard
                            //     );
                            // }
                        }
                        // probably causing problems
                        // OperationState::Executing => {
                        //     match operation.timeout_ms {
                        //         Some(timeout) => {
                        //             if operation_start_time > 0 {
                        //             let elapsed_ms =
                        //                 now_as_millis_i64().saturating_sub(operation_start_time);
                        //             if elapsed_ms >= timeout {
                        //                 // log::error!(target: &format!("{}_operation_runner", sp_id), "HAS TO TIMEOUT HERE!");
                        //                 new_state = operation.timeout_running(&new_state);
                        //                 operation_information =
                        //                     format!("Operation '{}' timed out", operation.name);
                        //             } else {
                        //                 if operation.can_be_failed(&new_state) {
                        //                     // log::error!(target: &format!("{}_operation_runner", sp_id), "HAS TO FAIL HERE!");
                        //                     new_state = operation.clone().fail_running(&new_state);
                        //                     operation_information =
                        //                         format!("Failing {}", operation.name);
                        //                 } else {
                        //                     let (eval, idx) = operation
                        //                         .can_be_completed_with_transition_index(&new_state);
                        //                     tokio::time::sleep(Duration::from_millis(
                        //                         operation.postconditions[idx].delay_ms,
                        //                     ))
                        //                     .await;
                        //                     if eval {
                        //                         // log::error!(target: &format!("{}_operation_runner", sp_id), "HAS TO COMPLETE HERE!");
                        //                         new_state =
                        //                             operation.clone().complete_running(&new_state);
                        //                         operation_information =
                        //                             format!("Completing {}", operation.name);
                        //                     } else {
                        //                         operation_information = format!(
                        //                             "Waiting for {} to be completed",
                        //                             operation.name
                        //                         );
                        //                     }
                        //                 }
                        //                 }
                        //             }
                        //         }
                        //         None => {
                        //             if operation.can_be_failed(&new_state) {
                        //                 new_state = operation.clone().fail_running(&new_state);
                        //                 operation_information =
                        //                     format!("Failing {}", operation.name);
                        //             } else {
                        //                 let (eval, idx) = operation
                        //                     .can_be_completed_with_transition_index(&new_state);
                        //                 tokio::time::sleep(Duration::from_millis(
                        //                     operation.postconditions[idx].delay_ms,
                        //                 ))
                        //                 .await;
                        //                 if eval {
                        //                     new_state =
                        //                         operation.clone().complete_running(&new_state);
                        //                     operation_information =
                        //                         format!("Completing {}", operation.name);
                        //                 } else {
                        //                     operation_information = format!(
                        //                         "Waiting for {} to be completed",
                        //                         operation.name
                        //                     );
                        //                 }
                        //             }
                        //         }
                        //     }
                        // }
                        //
                        OperationState::Executing => {
                            if operation.can_be_completed(&state) {
                                new_state = operation.clone().complete_running(&new_state);
                                operation_information = "Completing operation".to_string();
                            } else if operation.can_be_failed(&state) {
                                new_state = operation.clone().fail_running(&new_state);
                                operation_information = "Failing operation".to_string();
                            } else {
                                operation_information = "Waiting to be completed".to_string();
                            }
                        }
                        OperationState::Completed => {
                            // new_state = operation.reinitialize_running(&new_state);
                            operation_information =
                                format!("Operation {} completed, reinitializing", operation.name);
                            new_state = new_state.update(
                                &format!("{}_retry_counter", operation.name),
                                0.to_spvalue(),
                            );
                            new_state = new_state
                                .update(&format!("{}_start_time", operation.name), 0.to_spvalue());
                            plan_current_step = plan_current_step + 1;
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
                            plan_state = PlanState::Failed.to_string();
                            // new_state = operation.reinitialize_running(&new_state); // reinitialize globally when new plan is found
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
                plan_current_step = 0;
                // plan = vec!();
                // plan_state = PlanState::Initial.to_string();
                new_state = reset_all_operations(&new_state);
                plan_state = PlanState::Initial.to_string();
                planner_state = PlannerState::Ready.to_string();
            }
            // PlanState::NotFound => {}
            PlanState::Completed => {
                plan_current_step = 0;
                // plan = vec!();
                // plan_state = PlanState::Initial.to_string();
                new_state = reset_all_operations(&new_state);
                plan_state = PlanState::Initial.to_string();
                planner_state = PlannerState::Ready.to_string();
            }
            // PlanState::Cancelled => {}
            PlanState::UNKNOWN => {
                plan_current_step = 0;
                // plan = vec!();
                // plan_state = PlanState::Initial.to_string();
                new_state = reset_all_operations(&new_state);
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
            // .update(&format!("{}_plan", sp_id), plan.to_spvalue())
            ;

        let modified_state = state.get_diff_partial_state(&new_state);
        command_sender
            .send(StateManagement::SetPartialState(modified_state))
            .await?;

        interval.tick().await;
    }
}
