// use std::time::{Instant, SystemTime};

use std::time::SystemTime;

use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

pub async fn cycle_operation(
    sp_id: &str,
    operation: Operation,
    state: State,
    // operation_state_old: String,
    // operation_information_old: String,
) -> State {
    // ) -> Result<OperationState, Box<dyn std::error::Error>> {

    let mut new_state = state.clone();

    let operation_state = state.get_string_or_default_to_unknown(
        &format!("{}_cycle_operation", sp_id),
        &format!("{}", operation.name),
    );

    let mut operation_information = state.get_string_or_default_to_unknown(
        &format!("{}_cycle_operation", sp_id),
        &format!("{}_information", operation.name),
    );

    let mut operation_retry_counter = state.get_int_or_default_to_zero(
        &format!("{}_cycle_operation", sp_id),
        &format!("{}_retry_counter", operation.name),
    );

    let mut operation_start_time = state.get_time_or_unknown(
        &format!("{}_cycle_operation", sp_id),
        &format!("{}_start_time", operation.name),
    );

    // // Log only when something changes and not every tick
    // if operation_state_old != operation_state {
    //     log::info!(target: &format!("{}_single_operation_runner", sp_id), "Current state of operation {}: {}.", operation.name, operation_state);
    //     operation_state_old = operation_state.clone()
    // }

    // if operation_information_old != operation_information {
    //     log::info!(target: &format!("{}_single_operation_runner", sp_id), "Current operation '{}' info: {}.", operation.name, operation_information);
    //     operation_information_old = operation_information.clone()
    // }

    match OperationState::from_str(&operation_state) {
        OperationState::Initial => {
            if operation.eval_running(&state) {
                new_state = operation.start_running(&new_state);
                let now = SystemTime::now();
                operation_start_time = TimeOrUnknown::Time(now);
                new_state = new_state.update(
                    &format!("{}_information", operation.name),
                    operation_information.to_spvalue(),
                );

                // new_state = state.get_diff_partial_state(&new_state);
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
                match operation_start_time {
                    TimeOrUnknown::Time(start_time_result) => match start_time_result.elapsed() {
                        Ok(start_time) => match operation.timeout_ms {
                            Some(timeout) => {
                                if start_time.as_millis() > timeout {
                                    new_state = operation.timeout_running(&new_state);
                                }
                            }
                            None => (),
                        },
                        Err(_) => {}
                    },
                    _ => {}
                }
            }
            new_state = new_state.update(
                &format!("{}_information", operation.name),
                operation_information.to_spvalue(),
            );

            // new_state = state.get_diff_partial_state(&new_state);
        }
        OperationState::Completed => {
            operation_retry_counter = 0;
            new_state = new_state.update(
                &format!("{}_retry_counter", operation.name),
                operation_retry_counter.to_spvalue(),
            );

            new_state = new_state.update(
                &format!("{}_information", operation.name),
                operation_information.to_spvalue(),
            );

            // new_state = state.get_diff_partial_state(&new_state);
        }
        OperationState::Timedout => {
            operation_information = format!("Operation '{}' timedout.", operation.name);
            new_state = new_state.update(
                &format!("{}_information", operation.name),
                operation_information.to_spvalue(),
            );
        }
        OperationState::Failed => {
            if operation_retry_counter < operation.retries {
                operation_retry_counter = operation_retry_counter + 1;
                operation_information = format!(
                    "Operation failed. Retrying. Retry nr. {} out of {}.",
                    operation_retry_counter, operation.retries
                );
                new_state = operation.clone().retry_running(&new_state);
                new_state = new_state.update(
                    &format!("{}_retry_counter", operation.name),
                    operation_retry_counter.to_spvalue(),
                );
                new_state = new_state.update(
                    &format!("{}_information", operation.name),
                    operation_information.to_spvalue(),
                );

                // new_state = state.get_diff_partial_state(&new_state);
            } else {
                operation_retry_counter = 0;
                new_state = new_state.update(
                    &format!("{}_retry_counter", operation.name),
                    operation_retry_counter.to_spvalue(),
                );
                operation_information = format!("No more retries left. Abandoning the operation");
                new_state = new_state.update(
                    &format!("{}_information", operation.name),
                    operation_information.to_spvalue(),
                );

                new_state = operation.abandon_running(&new_state);
                // new_state = state.get_diff_partial_state(&new_state);
            }
        }
        OperationState::Abandoned => {
            operation_information = format!("Operation abandoned.");
            new_state = new_state.update(
                &format!("{}_information", operation.name),
                operation_information.to_spvalue(),
            );
        }
        OperationState::UNKNOWN => (),
    }

    new_state
}

pub async fn plan_runner(
    sp_id: &str,
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // For nicer logging
    let mut plan_state_old = "".to_string();
    // let mut operation_state_old = "".to_string();
    // let mut operation_information_old = "".to_string();

    log::info!(target: &&format!("{}_plan_runner", sp_id), "Online.");

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;
        let mut new_state = state.clone();

        let mut plan_state = state.get_string_or_default_to_unknown(
            &format!("{}_plan_runner", sp_id),
            &format!("{}_plan_state", sp_id),
        );
        let mut plan_current_step = state.get_int_or_default_to_zero(
            &format!("{}_plan_runner", sp_id),
            &format!("{}_plan_current_step", sp_id),
        );
        let plan_of_sp_values = state.get_array_or_default_to_empty(
            &format!("{}_plan_runner", sp_id),
            &format!("{}_plan", sp_id),
        );

        let plan: Vec<String> = plan_of_sp_values
            .iter()
            .filter(|val| val.is_string())
            .map(|y| y.to_string())
            .collect();

        // Log only when something changes and not every tick
        if plan_state_old != plan_state {
            log::info!(target: &format!("{}_plan_runner", sp_id), "Plan current state: {plan_state}.");
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

                    new_state = cycle_operation(sp_id, operation.clone(), state.clone()).await;
                    let operation_state = new_state.get_string_or_default_to_unknown(
                        &format!("{}_plan_runner", sp_id),
                        &format!("{}", operation.name),
                    );
                    match OperationState::from_str(&operation_state) {
                        OperationState::Completed => {
                            plan_current_step = plan_current_step + 1;
                        }
                        // If retries have need exhausted, fail the plan
                        OperationState::Abandoned => {
                            plan_state = PlanState::Failed.to_string();
                        }
                        _ => (),
                    }
                } else {
                    plan_state = PlanState::Completed.to_string();
                }
            }
            PlanState::Paused => {}
            PlanState::Failed => {}
            PlanState::NotFound => {}
            PlanState::Completed => {}
            PlanState::UNKNOWN => {}
        }

        new_state = new_state
            .update(&format!("{}_plan_state", sp_id), plan_state.to_spvalue())
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

// Super simple for now only sequences, later extend with alternative, paralell, loops, etc.
pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // For nicer logging
    let mut sop_state_old = "".to_string();
    // let mut operation_state_old = "".to_string();
    // let mut operation_information_old = "".to_string();

    log::info!(target: &&format!("{}_sop_runner", sp_id), "Online.");

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

        let mut sop_request_trigger = state.get_bool_or_default_to_false(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_request_trigger", sp_id),
        );

        // Log only when something changes and not every tick
        if sop_state_old != sop_state {
            log::info!(target: &format!("{}_sop_runner", sp_id), "SOP current state: {sop_state}.");
            sop_state_old = sop_state.clone()
        }

        if sop_request_trigger {
            sop_state = ActionRequestState::Executing.to_string();
            sop_current_step = 0;
            let sop = model
                .sops
                .iter()
                .find(|sop| sop.id == sop_id.to_string())
                .unwrap()
                .to_owned();
            match ActionRequestState::from_str(&sop_state) {
                ActionRequestState::Initial => {}
                ActionRequestState::Executing => {
                    if sop.sop.len() > sop_current_step as usize {
                        let operation = model
                            .operations
                            .iter()
                            .find(|op| op.name == sop.sop[sop_current_step as usize].to_string())
                            .unwrap()
                            .to_owned();


                        // Might be a problem here if we are cycling the same operation from 2 different places
                        new_state = cycle_operation(
                            &format!("{sp_id}_sop"),
                            operation.clone(),
                            state.clone(),
                        )
                        .await;
                        let operation_state = new_state.get_string_or_default_to_unknown(
                            &format!("{}_plan_runner", sp_id),
                            &format!("{}", operation.name),
                        );
                        match OperationState::from_str(&operation_state) {
                            OperationState::Completed => {
                                sop_current_step = sop_current_step + 1;
                            }
                            // If retries have need exhausted, fail the sop
                            OperationState::Abandoned => {
                                sop_state = ActionRequestState::Failed.to_string();
                            }
                            _ => (),
                        }
                    } else {
                        sop_state = ActionRequestState::Succeeded.to_string();
                        
                    }
                }
                ActionRequestState::Succeeded => {
                    sop_request_trigger = false;
                    log::info!(target: &&format!("{}_sop_runner", sp_id), "SOP suceeded.");
                }
                ActionRequestState::Failed => {
                    sop_request_trigger = false;
                    log::info!(target: &&format!("{}_sop_runner", sp_id), "SOP failed.");
                }
                ActionRequestState::UNKNOWN => {}
            }
        }

        new_state = new_state
            .update(&format!("{}_sop_state", sp_id), sop_state.to_spvalue())
            .update(
                &format!("{}_sop_current_step", sp_id),
                sop_current_step.to_spvalue(),
            )
            .update(
                &format!("{}_sop_request_trigger", sp_id),
                sop_request_trigger.to_spvalue(),
            );

        let modified_state = state.get_diff_partial_state(&new_state);
        command_sender
            .send(StateManagement::SetPartialState(modified_state))
            .await?;

        interval.tick().await;
    }
}

// No planner, just runner. In this case the model has to be different
// pub fn simple_single_operation_runner() {}

// This is working below!!!
// /// A planned operation runner is an algorithm which executes the plan P based on the model
// /// M, the current state of the system S, and a goal predicate G. While
// /// running, both the planning and running components of guards and actions
// /// of operation pre- and postconditions are evaluated and taken.
// pub async fn planned_operation_runner(
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let sp_id = &model.name;
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();

//     // For nicer logging
//     let mut plan_state_old = "".to_string();
//     let mut operation_state_old = "".to_string();
//     let mut operation_information_old = "".to_string();

//     log::info!(target: &&format!("{}_single_operation_runner", sp_id), "Online.");
//     // command_sender
//     //     .send(StateManagement::Set((
//     //         format!("{}_single_operation_runner_online", sp_id),
//     //         SPValue::Bool(BoolOrUnknown::Bool(true)),
//     //     )))
//     //     .await?;

//     loop {
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender
//             .send(StateManagement::GetState(response_tx))
//             .await?;
//         let state = response_rx.await?;
//         let mut new_state = state.clone();

//         let mut plan_state = state.get_string_or_default_to_unknown(
//             &format!("{}_single_operation_runner", sp_id),
//             &format!("{}_plan_state", sp_id),
//         );
//         let mut plan_current_step = state.get_int_or_default_to_zero(
//             &format!("{}_single_operation_runner", sp_id),
//             &format!("{}_plan_current_step", sp_id),
//         );
//         let plan_of_sp_values = state.get_array_or_default_to_empty(
//             &format!("{}_single_operation_runner", sp_id),
//             &format!("{}_plan", sp_id),
//         );

//         let plan: Vec<String> = plan_of_sp_values
//             .iter()
//             .filter(|val| val.is_string())
//             .map(|y| y.to_string())
//             .collect();

//         // Log only when something changes and not every tick
//         if plan_state_old != plan_state {
//             log::info!(target: &format!("{}_single_operation_runner", sp_id), "Plan current state: {plan_state}.");
//             plan_state_old = plan_state.clone()
//         }

//         match PlanState::from_str(&plan_state) {
//             PlanState::Initial => {
//                 plan_state = PlanState::Executing.to_string();
//                 plan_current_step = 0;
//             }
//             PlanState::Executing => {
//                 if plan.len() > plan_current_step as usize {
//                     let operation = model
//                         .operations
//                         .iter()
//                         .find(|op| op.name == plan[plan_current_step as usize].to_string())
//                         .unwrap()
//                         .to_owned();

//                     let operation_state = state.get_string_or_default_to_unknown(
//                         &format!("{}_single_operation_runner", sp_id),
//                         &format!("{}", operation.name),
//                     );

//                     let mut operation_information = state.get_string_or_default_to_unknown(
//                         &format!("{}_single_operation_runner", sp_id),
//                         &format!("{}_information", operation.name),
//                     );

//                     let mut operation_retry_counter = state.get_int_or_default_to_zero(
//                         &format!("{}_single_operation_runner", sp_id),
//                         &format!("{}_retry_counter", operation.name),
//                     );

//                     // let mut _operation_start_time = state.get_or_default_f64(
//                     //     &format!("{}_single_operation_runner", sp_id),
//                     //     &format!("{}_start_time", operation.name),
//                     // );

//                     // Log only when something changes and not every tick
//                     if operation_state_old != operation_state {
//                         log::info!(target: &format!("{}_single_operation_runner", sp_id), "Current state of operation {}: {}.", operation.name, operation_state);
//                         operation_state_old = operation_state.clone()
//                     }

//                     if operation_information_old != operation_information {
//                         log::info!(target: &format!("{}_single_operation_runner", sp_id), "Current operation '{}' info: {}.", operation.name, operation_information);
//                         operation_information_old = operation_information.clone()
//                     }

//                     match OperationState::from_str(&operation_state) {
//                         OperationState::Initial => {
//                             // let now = Instant::now();
//                             if operation.eval_running(&state) {
//                                 new_state = operation.start_running(&new_state);
//                                 // _operation_start_time = Instant::now().duration_since(now).as_micros() as f64;
//                             }
//                         }
//                         OperationState::Disabled => todo!(),
//                         OperationState::Executing => {
//                             if operation.can_be_completed(&state) {
//                                 new_state = operation.clone().complete_running(&new_state);
//                                 operation_information = "Completing operation.".to_string();
//                             } else if operation.can_be_failed(&state) {
//                                 new_state = operation.clone().fail_running(&new_state);
//                                 operation_information = "Failing operation.".to_string();
//                             } else {
//                                 operation_information = "Waiting to be completed.".to_string();
//                             }
//                         }
//                         OperationState::Completed => {
//                             operation_retry_counter = 0;
//                             new_state = new_state.update(
//                                 &format!("{}_retry_counter", operation.name),
//                                 operation_retry_counter.to_spvalue(),
//                             );
//                             plan_current_step = plan_current_step + 1;
//                         }
//                         OperationState::Timedout => todo!(),
//                         OperationState::Failed => {
//                             if operation_retry_counter < operation.retries {
//                                 operation_retry_counter = operation_retry_counter + 1;
//                                 operation_information = format!(
//                                     "Retrying. Retry nr. {} out of {}.",
//                                     operation_retry_counter, operation.retries
//                                 );
//                                 new_state = operation.clone().retry_running(&new_state);
//                                 new_state = new_state.update(
//                                     &format!("{}_retry_counter", operation.name),
//                                     operation_retry_counter.to_spvalue(),
//                                 );
//                             } else {
//                                 operation_retry_counter = 0;
//                                 new_state = new_state.update(
//                                     &format!("{}_retry_counter", operation.name),
//                                     operation_retry_counter.to_spvalue(),
//                                 );
//                                 operation_information =
//                                     format!("No more retries left. Failing the plan: {:?}", plan);
//                                 plan_state = PlanState::Failed.to_string();
//                             }
//                         }
//                         OperationState::UNKNOWN => (),
//                     }

//                     new_state = new_state.update(
//                         &format!("{}_information", operation.name),
//                         operation_information.to_spvalue(),
//                     );
//                 } else {
//                     plan_state = PlanState::Completed.to_string();
//                 }
//             }
//             PlanState::Paused => {}
//             PlanState::Failed => {}
//             PlanState::NotFound => {}
//             PlanState::Completed => {}
//             PlanState::Cancelled => {}
//             PlanState::UNKNOWN => {}
//         }

//         new_state = new_state
//             .update(&format!("{}_plan_state", sp_id), plan_state.to_spvalue())
//             .update(
//                 &format!("{}_plan_current_step", sp_id),
//                 plan_current_step.to_spvalue(),
//             )
//             .update(&format!("{}_plan", sp_id), plan.to_spvalue());

//         let modified_state = state.get_diff_partial_state(&new_state);
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;

//         interval.tick().await;
//     }
// }

// // No planner, just runner. In this case the model has to be different
// // pub fn simple_single_operation_runner() {}
