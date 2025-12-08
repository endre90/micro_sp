use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum OperationProcessingType {
    Planned,
    SOP,
    Automatic,
}

pub(super) async fn process_operation(
    sp_id: &str,
    mut new_state: State,
    operation: &Operation,
    operation_processing_type: OperationProcessingType,
    plan_current_step: Option<&mut i64>,
    plan_state: Option<&mut String>,
    // sop_state: Option<&mut String>,
    logging_tx: mpsc::Sender<LogMsg>,
    // mut con: redis::aio::MultiplexedConnection,
    log_target: &str,
) -> State {
    let operation_state =
        new_state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);

    let old_operation_information = new_state
        .get_string_or_default_to_unknown(&format!("{}_information", operation.name), &log_target);

    let mut new_op_info = old_operation_information.clone();

    let mut operation_failure_retry_counter = new_state.get_int_or_default_to_zero(
        &format!("{}_failure_retry_counter", operation.name),
        &log_target,
    );

    let mut operation_timeout_retry_counter = new_state.get_int_or_default_to_zero(
        &format!("{}_timeout_retry_counter", operation.name),
        &log_target,
    );

    let mut elapased_executing_ms = new_state.get_int_or_default_to_zero(
        &format!("{}_elapsed_executing_ms", operation.name),
        &log_target,
    );

    let mut elapased_disabled_ms = new_state.get_int_or_default_to_zero(
        &format!("{}_elapsed_disabled_ms", operation.name),
        &log_target,
    );

    let mut logging_log = "".to_string();
    let mut op_info_level = log::Level::Info;
    match OperationState::from_str(&operation_state) {
        OperationState::Initial => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.clone().cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling operation.");
                op_info_level = log::Level::Warn;
            } else if operation.eval(&new_state, &log_target) {
                new_state = operation.start(&new_state, &log_target);
                new_op_info = format!("Starting initialized operation '{}'.", operation.name);
                logging_log = format!("Starting operation.");
                op_info_level = log::Level::Info;
            } else {
                new_op_info = format!("Disabling operation '{}'.", operation.name).to_string();
                logging_log = format!("Disabling operation.");
                op_info_level = log::Level::Warn;
                new_state = operation.disable(&new_state, &log_target);
            }
        }
        OperationState::Disabled => {
            elapased_disabled_ms += OPERAION_RUNNER_TICK_INTERVAL_MS as i64;
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.clone().cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling operation.");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_timedout(&new_state, &log_target) {
                new_state = operation.clone().timeout(&new_state, &log_target);
                new_op_info =
                    format!("Timeout for disabled operation '{}'.", operation.name).to_string();
                logging_log = format!("Timeout for operation.");
                op_info_level = log::Level::Warn;
            } else if operation.eval(&new_state, &log_target) {
                new_state = operation.start(&new_state, &log_target);
                new_op_info = format!("Starting disabled operation '{}'.", operation.name);
                logging_log = format!("Starting operation.");
                op_info_level = log::Level::Info;
            } else {
                let mut or_clause = vec![];
                let mut or_clause_full = vec![];
                for precondition in &operation.preconditions {
                    or_clause.push(precondition.runner_guard.clone());
                    or_clause_full.push(Predicate::AND(vec![
                        precondition.guard.clone(),
                        precondition.runner_guard.clone(),
                    ]));
                }
                new_op_info = format!(
                    "Operation '{}' disabled. Please satisfy the runner guard: \n       {}\n       Debug full guard: \n       {}",
                    operation.name,
                    Predicate::OR(or_clause),
                    Predicate::OR(or_clause_full)
                );
                logging_log = format!("Operation disabled.");

                op_info_level = log::Level::Warn;
            }
        }
        OperationState::Executing => {
            elapased_executing_ms += OPERAION_RUNNER_TICK_INTERVAL_MS as i64;
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.clone().cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling operation.");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_failed(&new_state, &log_target) {
                new_state = operation.clone().fail(&new_state, &log_target);
                new_op_info = format!("Failing operation '{}'.", operation.name).to_string();
                logging_log = format!("Failing operation.");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_timedout(&new_state, &log_target) {
                new_state = operation.clone().timeout(&new_state, &log_target);
                new_op_info =
                    format!("Timeout for executing operation '{}'.", operation.name).to_string();
                logging_log = format!("Timeout for operation.");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_completed(&new_state, &log_target) {
                new_state = operation.clone().complete(&new_state, &log_target);
                new_op_info = format!("Completing operation '{}'.", operation.name).to_string();
                logging_log = format!("Completing operation.");
                op_info_level = log::Level::Info;
            } else {
                new_op_info = format!(
                    "Waiting for operation '{}' to be completed.",
                    operation.name
                )
                .to_string();
                logging_log = format!("Waiting to be completed.");
                op_info_level = log::Level::Info;
            }
        }
        OperationState::Completed => {
            new_state = new_state.update(
                &format!("{}_failure_retry_counter", operation.name),
                0.to_spvalue(),
            );
            new_state = new_state.update(
                &format!("{}_timeout_retry_counter", operation.name),
                0.to_spvalue(),
            );
            if let OperationProcessingType::Planned = operation_processing_type {
                if let Some(plan_current_step) = plan_current_step {
                    *plan_current_step += 1;
                }
            }
            if let OperationProcessingType::Automatic = operation_processing_type {
                new_state = operation.initialize(&new_state, &log_target);
            }
            new_op_info = format!("Operation '{}' completed.", operation.name);
            logging_log = format!("Operation completed.");
            op_info_level = log::Level::Info;
            match operation_processing_type {
                OperationProcessingType::SOP => {
                    new_state =
                        operation.terminate(&new_state, TerminationReason::Completed, &log_target);
                }
                _ => (),
            }
        }
        OperationState::Bypassed => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.clone().cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling operation.");
            } else {
                new_op_info = format!(
                    "Operation '{}' bypassed. Continuing with the next operation.",
                    operation.name
                );
                logging_log = format!("Operation bypassed.");
                if let OperationProcessingType::Planned = operation_processing_type {
                    if let Some(plan_current_step) = plan_current_step {
                        *plan_current_step += 1;
                    }
                }
            }
            op_info_level = log::Level::Warn;
            match operation_processing_type {
                OperationProcessingType::SOP => {
                    new_state =
                        operation.terminate(&new_state, TerminationReason::Bypassed, &log_target);
                }
                _ => (),
            }
        }
        OperationState::Timedout => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.clone().cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling operation.");
                op_info_level = log::Level::Warn;
            } else if operation_timeout_retry_counter < operation.timeout_retries {
                operation_timeout_retry_counter += 1;
                new_op_info = format!(
                    "Retrying operation (timeout) '{}'. Retry {} out of {}.",
                    operation.name, operation_timeout_retry_counter, operation.timeout_retries
                );
                logging_log = format!(
                    "Retrying operation {} / {}.",
                    operation_timeout_retry_counter, operation.timeout_retries
                );
                op_info_level = log::Level::Warn;
                new_state = operation.clone().retry(&new_state, &log_target);
                new_state = new_state.update(
                    &format!("{}_timeout_retry_counter", operation.name),
                    operation_timeout_retry_counter.to_spvalue(),
                );
            } else if operation.can_be_bypassed {
                new_state = operation.bypass(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout. Bypassing.", operation.name);
                logging_log = format!("Operation timedout. Bypassing.");
                op_info_level = log::Level::Warn;
            } else {
                new_state = operation.fatal(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout.", operation.name);
                logging_log = format!("Operation timedout.");
                op_info_level = log::Level::Warn;
            }
        }
        OperationState::Failed => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.clone().cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling operation.");
                op_info_level = log::Level::Warn;
            } else if operation_failure_retry_counter < operation.failure_retries {
                operation_failure_retry_counter += 1;
                new_op_info = format!(
                    "Retrying operation (failure) '{}'. Retry {} out of {}.",
                    operation.name, operation_failure_retry_counter, operation.failure_retries
                );
                logging_log = format!(
                    "Retrying operation {} / {}.",
                    operation_failure_retry_counter, operation.failure_retries
                );
                op_info_level = log::Level::Warn;
                new_state = operation.clone().retry(&new_state, &log_target);
                new_state = new_state.update(
                    &format!("{}_failure_retry_counter", operation.name),
                    operation_failure_retry_counter.to_spvalue(),
                );
            } else {
                if operation.can_be_bypassed {
                    new_state = operation.bypass(&new_state, &log_target);
                    new_op_info = format!(
                        "Operation '{}' has no more retries left. Bypassing.",
                        operation.name
                    );
                    logging_log = format!("Operation failed. Bypassing.");
                    op_info_level = log::Level::Warn;
                } else {
                    new_state = operation.fatal(&new_state, &log_target);
                    new_op_info =
                        format!("Operation '{}' has no more retries left.", operation.name);
                    logging_log = format!("Operation has no more retries left.");
                    op_info_level = log::Level::Warn;
                }
                new_state = new_state.update(
                    &format!("{}_failure_retry_counter", operation.name),
                    0.to_spvalue(),
                );
                new_state = new_state.update(
                    &format!("{}_timeout_retry_counter", operation.name),
                    0.to_spvalue(),
                );
            }
        }
        OperationState::Fatal => {
            new_op_info = format!(
                "Operation '{}' unrecoverable. Stopping execution.",
                operation.name
            );
            logging_log = format!("Operation unrecoverable.");
            op_info_level = log::Level::Error;
            match operation_processing_type {
                OperationProcessingType::Planned => {
                    if let Some(plan_state) = plan_state {
                        *plan_state = PlanState::Failed.to_string();
                    }
                }

                OperationProcessingType::SOP => {
                    new_state =
                        operation.terminate(&new_state, TerminationReason::Fatal, &log_target);
                }
                _ => (),
            }
        }
        OperationState::Cancelled => {
            new_op_info = format!(
                "Operation '{}' cancelled. Stopping execution.",
                operation.name
            );
            logging_log = format!("Operation cancelled.");
            op_info_level = log::Level::Warn;
            match operation_processing_type {
                OperationProcessingType::Planned => {
                    if let Some(plan_state) = plan_state {
                        *plan_state = PlanState::Cancelled.to_string();
                    }
                }
                OperationProcessingType::SOP => {
                    new_state =
                        operation.terminate(&new_state, TerminationReason::Cancelled, &log_target);
                }
                _ => (),
            }
        }
        OperationState::UNKNOWN => {
            new_state = operation.initialize(&new_state, &log_target);
        }
        OperationState::Terminated(termination_reason) => match termination_reason {
            TerminationReason::Bypassed => {
                new_op_info = format!(
                    "Operation '{}' terminated. Reason: Bypassed.",
                    operation.name
                )
            }
            TerminationReason::Completed => {
                new_op_info = format!(
                    "Operation '{}' terminated. Reason: Completed.",
                    operation.name
                )
            }
            TerminationReason::Fatal => {
                new_op_info = format!("Operation '{}' terminated. Reason: Fatal.", operation.name)
            }
            TerminationReason::Cancelled => {
                new_op_info = format!(
                    "Operation '{}' terminated. Reason: Cancelled.",
                    operation.name
                )
            }
        },
        // OperationState::Void => (),
    }

    // For now, skip logging the SOP operations
    if new_op_info != old_operation_information {
        match op_info_level {
            log::Level::Info => log::info!(target: &log_target, "{}", new_op_info),
            log::Level::Warn => log::warn!(target: &log_target, "{}", new_op_info),
            log::Level::Error => log::error!(target: &log_target, "{}", new_op_info),
            _ => (),
        }
        let operation_msg = OperationMsg {
            operation_name: operation.name.clone(),
            operation_processing_type: operation_processing_type,
            timestamp: Utc::now(),
            severity: op_info_level,
            state: OperationState::from_str(&operation_state),
            log: logging_log.to_string(),
        };
        let log_msg = LogMsg::OperationMsg(operation_msg);
        match logging_tx.send(log_msg).await {
            Ok(()) => (),
            Err(e) => {
                log::error!(target: &log_target, "Failed to send logging with: {e}.")
            }
        }
    }

    new_state
        .update(
            &format!("{}_information", operation.name),
            new_op_info.to_spvalue(),
        )
        .update(
            &format!("{}_elapsed_executing_ms", operation.name),
            elapased_executing_ms.to_spvalue(),
        )
        .update(
            &format!("{}_elapsed_disabled_ms", operation.name),
            elapased_disabled_ms.to_spvalue(),
        )
}
