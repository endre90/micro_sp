use crate::*;

pub enum OperationProcessingType {
    Planned,
    SOP,
    Automatic,
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

    let mut operation_retry_counter = new_state
        .get_int_or_default_to_zero(&format!("{}_retry_counter", operation.name), &log_target);

    match OperationState::from_str(&operation_state) {
        OperationState::Initial => {
            if operation.eval_running(&new_state, &log_target) {
                new_state = operation.start_running(&new_state, &log_target);
                new_op_info = format!("Operation '{}' started execution.", operation.name);
            } else {
                new_op_info = format!(
                    "Operation '{}' disabled. Please satisfy the guard: {:#?}.",
                    operation.name, operation.preconditions[0].runner_guard
                );
            }
        }
        OperationState::Executing => {
            if operation.can_be_completed(&new_state, &log_target) {
                new_state = operation.clone().complete_running(&new_state, &log_target);
                new_op_info = format!("Completing operation '{}'.", operation.name).to_string();
            } else if operation.can_be_failed(&new_state, &log_target) {
                new_state = operation.clone().fail_running(&new_state, &log_target);
                new_op_info = format!("Failing operation '{}'.", operation.name).to_string();
            } else {
                new_op_info = format!(
                    "Waiting for operation '{}' to be completed.",
                    operation.name
                )
                .to_string();
            }
        }

        OperationState::Completed => {
            new_state =
                new_state.update(&format!("{}_retry_counter", operation.name), 0.to_spvalue());
            if let OperationProcessingType::Planned = operation_processing_type {
                if let Some(plan_current_step) = plan_current_step {
                    *plan_current_step += 1;
                }
            }
            new_op_info = format!("Operation '{}' completed.", operation.name);
            new_state = new_state.remove(&operation.name, log_target);
            StateManager::remove_sp_value(&mut con, &operation.name).await;
        }

        OperationState::Bypassed => {
            new_op_info = format!(
                "Operation '{}' bypassed. Continuing with the next operation.",
                operation.name
            );
            new_state = operation.continue_running_next(&new_state, &log_target);
        }

        OperationState::Timedout => {
            if operation.can_be_bypassed {
                new_state = operation.bypass_running(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout. Bypassing.", operation.name);
            } else {
                new_state = operation.unrecover_running(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout.", operation.name);
            }
        }
        OperationState::Failed => {
            if operation_retry_counter < operation.retries {
                operation_retry_counter += 1;
                new_op_info = format!(
                    "Retrying operation '{}'. Retry {} out of {}.",
                    operation.name, operation_retry_counter, operation.retries
                );
                new_state = operation.clone().retry_running(&new_state, &log_target);
                new_state = new_state.update(
                    &format!("{}_retry_counter", operation.name),
                    operation_retry_counter.to_spvalue(),
                );
            } else {
                if operation.can_be_bypassed {
                    new_state = operation.bypass_running(&new_state, &log_target);
                    new_op_info = format!(
                        "Operation '{}' has no more retries left. Bypassing.",
                        operation.name
                    );
                } else {
                    new_state = operation.unrecover_running(&new_state, &log_target);
                    new_op_info =
                        format!("Operation '{}' has no more retries left.", operation.name);
                }
                new_state =
                    new_state.update(&format!("{}_retry_counter", operation.name), 0.to_spvalue());
            }
        }
        OperationState::Unrecoverable => {
            new_op_info = format!(
                "Operation '{}' is unrecoverable. Stopping execution.",
                operation.name
            );
            match operation_processing_type {
                OperationProcessingType::Planned => {
                    if let Some(plan_state) = plan_state {
                        *plan_state = PlanState::Failed.to_string();
                    }
                }
                _ => (),
            }
        }
        OperationState::UNKNOWN => {
            new_state = operation.initialize_running(&new_state, &log_target);
        }
    }

    if new_op_info != old_operation_information {
        log::info!(target: &log_target, "{}", new_op_info);
    }

    new_state.update(
        &format!("{}_information", operation.name),
        new_op_info.to_spvalue(),
    )
}
