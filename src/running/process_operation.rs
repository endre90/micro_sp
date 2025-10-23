use crate::*;

pub enum OperationProcessingType {
    Planned,
    SOP,
    Automatic,
}

pub enum OperationInfoLevel {
    Info,
    Warn,
    Error,
}

pub(super) async fn process_operation(
    mut new_state: State,
    operation: &Operation,
    operation_processing_type: OperationProcessingType,
    plan_current_step: Option<&mut i64>,
    plan_state: Option<&mut String>,
    mut con: redis::aio::MultiplexedConnection,
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

    let mut elapased_ms = new_state
        .get_int_or_default_to_zero(&format!("{}_elapsed_ms", operation.name), &log_target);

    let mut op_info_level = OperationInfoLevel::Info;
    match OperationState::from_str(&operation_state) {
        OperationState::Initial => {
            if operation.eval(&new_state, &log_target) {
                new_state = operation.start(&new_state, &log_target);
                new_op_info = format!("Starting operation '{}'.", operation.name);
                op_info_level = OperationInfoLevel::Info;
            } else {
                new_op_info = format!("Disabling operation '{}'.", operation.name).to_string();
                op_info_level = OperationInfoLevel::Warn;
                new_state = operation.disable(&new_state, &log_target);
            }
        }
        // TODO: Later, we can also add a timeout on how long the operation can be disabled
        OperationState::Disabled => {
            if operation.eval(&new_state, &log_target) {
                new_state = operation.start(&new_state, &log_target);
                new_op_info = format!("Starting operation '{}'.", operation.name);
                op_info_level = OperationInfoLevel::Info;
            } else {
                new_op_info = format!(
                    "Operation '{}' disabled. Please satisfy the guard: \n       {}",
                    operation.name, operation.preconditions[0].runner_guard
                );
                op_info_level = OperationInfoLevel::Warn;
            }
        }
        OperationState::Executing => {
            elapased_ms += OPERAION_RUNNER_TICK_INTERVAL_MS as i64;
            if operation.can_be_completed(&new_state, &log_target) {
                new_state = operation.clone().complete(&new_state, &log_target);
                new_op_info = format!("Completing operation '{}'.", operation.name).to_string();
                op_info_level = OperationInfoLevel::Info;
            } else if operation.can_be_failed(&new_state, &log_target) {
                new_state = operation.clone().fail(&new_state, &log_target);
                new_op_info = format!("Failing operation '{}'.", operation.name).to_string();
                op_info_level = OperationInfoLevel::Warn;
            } else if operation.can_be_timedout(&new_state, &log_target) {
                new_state = operation.clone().timeout(&new_state, &log_target);
                new_op_info = format!("Timeout for operation '{}'.", operation.name).to_string();
                op_info_level = OperationInfoLevel::Warn;
            } else {
                new_op_info = format!(
                    "Waiting for operation '{}' to be completed.",
                    operation.name
                )
                .to_string();
                op_info_level = OperationInfoLevel::Info;
            }
        }

        OperationState::Completed => {
            new_state =
                new_state.update(&format!("{}_failure_retry_counter", operation.name), 0.to_spvalue());
            new_state =
                new_state.update(&format!("{}_timeout_retry_counter", operation.name), 0.to_spvalue());
            if let OperationProcessingType::Planned = operation_processing_type {
                if let Some(plan_current_step) = plan_current_step {
                    *plan_current_step += 1;
                }
            }
            new_op_info = format!("Operation '{}' completed.", operation.name);
            op_info_level = OperationInfoLevel::Info;
            // new_state = new_state.remove(&operation.name, log_target);
            // StateManager::remove_sp_value(&mut con, &operation.name).await;
            // new_state = operation.terminate(&new_state, &log_target);
        }

        OperationState::Bypassed => {
            new_op_info = format!(
                "Operation '{}' bypassed. Continuing with the next operation.",
                operation.name
            );
            if let OperationProcessingType::Planned = operation_processing_type {
                if let Some(plan_current_step) = plan_current_step {
                    *plan_current_step += 1;
                }
            }
            op_info_level = OperationInfoLevel::Warn;
            // new_state = new_state.remove(&operation.name, log_target);
            // StateManager::remove_sp_value(&mut con, &operation.name).await;
            // new_state = operation.terminate(&new_state, &log_target);
        }

        OperationState::Timedout => {
            if operation_timeout_retry_counter < operation.timeout_retries {
                operation_timeout_retry_counter += 1;
                new_op_info = format!(
                    "Retrying operation (timeout) '{}'. Retry {} out of {}.",
                    operation.name, operation_timeout_retry_counter, operation.timeout_retries
                );
                op_info_level = OperationInfoLevel::Warn;
                new_state = operation.clone().retry(&new_state, &log_target);
                new_state = new_state.update(
                    &format!("{}_timeout_retry_counter", operation.name),
                    operation_timeout_retry_counter.to_spvalue(),
                );
            } else if operation.can_be_bypassed {
                new_state = operation.bypass(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout. Bypassing.", operation.name);
                op_info_level = OperationInfoLevel::Warn;
            } else {
                new_state = operation.unrecover(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout.", operation.name);
                op_info_level = OperationInfoLevel::Warn;
            }
        }
        OperationState::Failed => {
            if operation_failure_retry_counter < operation.failure_retries {
                operation_failure_retry_counter += 1;
                new_op_info = format!(
                    "Retrying operation (failure) '{}'. Retry {} out of {}.",
                    operation.name, operation_failure_retry_counter, operation.failure_retries
                );
                op_info_level = OperationInfoLevel::Warn;
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
                    op_info_level = OperationInfoLevel::Warn;
                } else {
                    new_state = operation.unrecover(&new_state, &log_target);
                    new_op_info =
                        format!("Operation '{}' has no more retries left.", operation.name);
                    op_info_level = OperationInfoLevel::Error;
                }
                new_state =
                    new_state.update(&format!("{}_failure_retry_counter", operation.name), 0.to_spvalue());
                new_state =
                    new_state.update(&format!("{}_timeout_retry_counter", operation.name), 0.to_spvalue());
            }
        }
        OperationState::Unrecoverable => {
            new_op_info = format!(
                "Operation '{}' is unrecoverable. Stopping execution.",
                operation.name
            );
            op_info_level = OperationInfoLevel::Error;
            match operation_processing_type {
                OperationProcessingType::Planned => {
                    if let Some(plan_state) = plan_state {
                        *plan_state = PlanState::Failed.to_string();
                    }
                }
                _ => (),
            }
            // new_state = new_state.remove(&operation.name, log_target);
            // StateManager::remove_sp_value(&mut con, &operation.name).await;
            // new_state = operation.terminate(&new_state, &log_target);
        }
        OperationState::Terminated => {
            new_op_info = format!("Operation '{}' terminated.", operation.name);
            op_info_level = OperationInfoLevel::Info;
            new_state = new_state.remove(&operation.name, log_target);
            StateManager::remove_sp_value(&mut con, &operation.name).await;
        }
        OperationState::UNKNOWN => {
            new_state = operation.initialize(&new_state, &log_target);
        }
    }

    if new_op_info != old_operation_information {
        match op_info_level {
            OperationInfoLevel::Info => log::info!(target: &log_target, "{}", new_op_info),
            OperationInfoLevel::Warn => log::warn!(target: &log_target, "{}", new_op_info),
            OperationInfoLevel::Error => log::error!(target: &log_target, "{}", new_op_info),
        }
    }

    new_state
        .update(
            &format!("{}_information", operation.name),
            new_op_info.to_spvalue(),
        )
        .update(
            &format!("{}_elapsed_ms", operation.name),
            elapased_ms.to_spvalue(),
        )
}
