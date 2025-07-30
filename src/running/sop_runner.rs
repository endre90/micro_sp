use std::sync::Arc;

use crate::*;
use tokio::time::{Duration, interval};

pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(250));
    let log_target = &format!("{}_sop_runner", sp_id);

    log::info!(target: log_target, "Online and managing SOP.");

    let mut old_sop_id = String::new();

    let mut con = connection_manager.get_connection().await;
    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let state = match StateManager::get_full_state(&mut con).await {
            Some(s) => s,
            None => continue,
        };

        let current_sop_id =
            state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);

        if old_sop_id != current_sop_id && !current_sop_id.is_empty() {
            if let Some(root_sop) = model.sops.iter().find(|s| s.id == current_sop_id) {
                log::info!(target: log_target, "Now executing new SOP '{}':", current_sop_id);
                log::info!(target: log_target, "{:?}", visualize_sop(&root_sop.sop));
            }
            old_sop_id = current_sop_id;
        }

        let new_state = process_sop_tick(sp_id, model, &state, &log_target)?;
        let modified_state = state.get_diff_partial_state(&new_state);

        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

fn process_sop_tick(
    sp_id: &str,
    model: &Model,
    state: &State,
    log_target: &str,
) -> Result<State, Box<dyn std::error::Error>> {
    let mut new_state = state.clone();
    let mut sop_overall_state =
        state.get_string_or_default_to_unknown(&format!("{}_sop_state", sp_id), &log_target);

    match SOPState::from_str(&sop_overall_state) {
        SOPState::Initial => {
            handle_sop_initial(
                sp_id,
                model,
                state,
                &mut new_state,
                &mut sop_overall_state,
                &log_target,
            )?;
        }
        SOPState::Executing => {
            handle_sop_executing(
                sp_id,
                model,
                state,
                &mut new_state,
                &mut sop_overall_state,
                &log_target,
            );
        }
        SOPState::Completed | SOPState::Failed => {}
        SOPState::UNKNOWN => {
            log::warn!(target: &log_target, "SOP in UNKNOWN state. Resetting.");
            sop_overall_state = SOPState::Initial.to_string();
        }
    }

    new_state = new_state.update(
        &format!("{}_sop_state", sp_id),
        sop_overall_state.to_spvalue(),
    );
    Ok(new_state)
}

fn handle_sop_initial(
    sp_id: &str,
    model: &Model,
    state: &State,
    new_state: &mut State,
    sop_overall_state: &mut String,
    log_target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if state.get_bool_or_default_to_false(&format!("{}_sop_enabled", sp_id), &log_target) {
        log::info!(target: &log_target, "SOP enabled. Transitioning to Executing.");
        let sop_id =
            state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);

        let root_sop = model
            .sops
            .iter()
            .find(|sop| sop.id == sop_id)
            .ok_or_else(|| format!("SOP with id '{}' not found in model", sop_id))?;

        let initial_stack = vec![root_sop.sop.clone()];
        *new_state = new_state.update(
            &format!("{}_sop_stack", sp_id),
            serde_json::to_string(&initial_stack)?.to_spvalue(),
        );
        *new_state = new_state.update(&format!("{}_sop_enabled", sp_id), false.to_spvalue());
        *sop_overall_state = SOPState::Executing.to_string();
    }
    Ok(())
}

fn handle_sop_executing(
    sp_id: &str,
    model: &Model,
    state: &State,
    new_state: &mut State,
    sop_overall_state: &mut String,
    log_target: &str,
) {
    let sop_id = state.get_string_or_default_to_unknown(&format!("{}_sop_id", sp_id), &log_target);
    let stack_json = state.get_string_or_value(&format!("{}_sop_stack", sp_id), "[]".to_string(), &log_target);

    let Some(root_sop) = model.sops.iter().find(|s| s.id == sop_id) else {
        log::error!(target: &log_target, "SOP with id '{}' not found in model. Failing.", sop_id);
        *sop_overall_state = SOPState::Failed.to_string();
        return;
    };

    let (updated_state, new_stack_json) =
        run_sop_tick(sp_id, state, stack_json, &root_sop.sop, &log_target);
    *new_state = updated_state;

    *new_state = new_state.update(&format!("{}_sop_stack", sp_id), new_stack_json.to_spvalue());

    // Check for terminal conditions.
    if new_stack_json == "[]" {
        log::info!(target: &log_target, "Execution stack empty. SOP Completed.");
        *sop_overall_state = SOPState::Completed.to_string();
    } else if is_sop_failed(sp_id, &root_sop.sop, new_state, &log_target) {
        log::error!(target: &log_target, "Unrecoverable error detected. SOP Failed.");
        *sop_overall_state = SOPState::Failed.to_string();
    }
}

fn is_sop_failed(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let op_state_str = state.get_string_or_default_to_unknown(&operation.name, &log_target);
            OperationState::from_str(&op_state_str) == OperationState::Unrecoverable
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => sops
            .iter()
            .any(|child_sop| is_sop_failed(sp_id, child_sop, state, &log_target)),
    }
}

pub fn run_sop_tick(
    sp_id: &str,
    state: &State,
    stack_json: String,
    root_sop: &SOP,
    log_target: &str,
) -> (State, String) {
    let mut new_state = state.clone();

    let mut stack: Vec<SOP> = serde_json::from_str(&stack_json).unwrap_or_else(|_| {
        log::info!("SOP stack is empty or invalid, initializing with root SOP.");
        vec![root_sop.clone()]
    });

    // If the stack is empty after initialization, the SOP is done.
    if stack.is_empty() {
        log::info!("SOP execution is complete.");
        return (new_state, serde_json::to_string(&stack).unwrap());
    }
    let current_sop = stack.pop().unwrap();

    match &current_sop {
        SOP::Operation(operation) => {
            let operation_state =
                state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);

            let mut operation_information = state.get_string_or_default_to_unknown(
                &format!("{}_information", operation.name),
                &log_target,
            );

            let mut operation_retry_counter = state.get_int_or_default_to_zero(
                &format!("{}_retry_counter", operation.name),
                &log_target,
            );

            match OperationState::from_str(&operation_state) {
                OperationState::Initial => {
                    if operation.eval_running(&new_state, &log_target) {
                        new_state = operation.start_running(&new_state, &log_target);
                        log::info!("Operation '{}' started execution", operation.name);
                        operation_information =
                            format!("Operation '{}' started execution", operation.name);
                    }
                }
                OperationState::Executing => {
                    if operation.can_be_completed(&state, &log_target) {
                        new_state = operation.clone().complete_running(&new_state, &log_target);
                        operation_information = "Completing operation".to_string();
                        log::info!("Completing operation '{}'", operation.name);
                    } else if operation.can_be_failed(&state, &log_target) {
                        new_state = operation.clone().fail_running(&new_state, &log_target);
                        operation_information = "Failing operation".to_string();
                        log::info!("Failing operation '{}'", operation.name);
                    } else {
                        operation_information = "Waiting to be completed".to_string();
                        log::info!("Operation '{}' waiting to be completed", operation.name);
                    }
                }
                OperationState::Completed => {
                    // new_state = operation.reinitialize_running(&new_state);
                    operation_information =
                        format!("Operation {} completed, reinitializing", operation.name);
                    log::info!("Operation '{}' completed", operation.name);
                    new_state = new_state
                        .update(&format!("{}_retry_counter", operation.name), 0.to_spvalue());
                    new_state =
                        new_state.update(&format!("{}_start_time", operation.name), 0.to_spvalue());
                }
                OperationState::Timedout => {
                    new_state = operation.unrecover_running(&new_state, &log_target);
                    operation_information = format!("Timedout {}. Unrecoverable", operation.name);
                }
                OperationState::Failed => {
                    if operation_retry_counter < operation.retries {
                        operation_retry_counter = operation_retry_counter + 1;
                        log::info!(
                            "Retrying '{}'. Retry nr. {} out of {}",
                            operation.name,
                            operation_retry_counter,
                            operation.retries
                        );
                        operation_information = format!(
                            "Retrying '{}'. Retry nr. {} out of {}",
                            operation.name, operation_retry_counter, operation.retries
                        );
                        new_state = operation.clone().retry_running(&new_state, &log_target);
                        new_state = new_state.update(
                            &format!("{}_retry_counter", operation.name),
                            operation_retry_counter.to_spvalue(),
                        );
                    } else {
                        new_state = operation.unrecover_running(&new_state, &log_target);
                        new_state = new_state
                            .update(&format!("{}_retry_counter", operation.name), 0.to_spvalue());
                        log::info!("Operation failed, no more retries left. Unrecoverable");
                        operation_information =
                            format!("Operation failed, no more retries left. Unrecoverable");
                    }
                }
                OperationState::Unrecoverable => {
                    // new_state = operation.reinitialize_running(&new_state); // reinitialize globally when sop is done
                    operation_information = format!("Failing the sop: {:?}", root_sop);
                    log::info!("Failing the sop: {:?}", visualize_sop(root_sop));
                }
                OperationState::UNKNOWN => (),
            }

            new_state = new_state.update(
                &format!("{}_information", operation.name),
                operation_information.to_spvalue(),
            );
        }

        SOP::Sequence(sops) => {
            let next_sop_to_run = sops
                .iter()
                .find(|sub_sop| !is_sop_completed(sp_id, sub_sop, &new_state, &log_target));

            if let Some(sub_sop) = next_sop_to_run {
                // We found the next step that is not yet completed.
                // First, push the parent Sequence back onto the stack so we can re-evaluate it
                // after the child is processed. This prevents the stack from emptying prematurely.
                stack.push(current_sop.clone());
                // Then, push the specific step that needs to be processed onto the stack.
                // This will be the next thing to be executed.
                stack.push(sub_sop.clone());
            } else {
                // The sequence IS finished.
                // By doing nothing here, we allow the Sequence to be "consumed" from the stack.
                // If it was the last item, the SOP will correctly be flagged as completed on the next tick.
                log::info!("Sequence is complete.");
            }
        }

        SOP::Parallel(sops) => {
            log::debug!("Dispatching all unfinished children of a Parallel node.");
            for sub_sop in sops.iter().rev() {
                if !is_sop_completed(sp_id, sub_sop, &new_state, &log_target) {
                    stack.push(sub_sop.clone());
                }
            }
        }

        SOP::Alternative(sops) => {
            log::info!("Processing an Alternative node.");

            let chosen_path = sops
                .iter()
                .find(|sop| !is_sop_in_initial_state(sp_id, sop, &new_state, &log_target));

            if let Some(path) = chosen_path {
                log::info!(
                    "Alternative path {:?} is already active. Pushing for continued execution.",
                    path
                );
                stack.push(path.clone());
            } else {
                log::info!("No active path found. Evaluating new alternatives.");
                for sub_sop in sops {
                    if can_sop_start(sp_id, &sub_sop, &new_state, &log_target) {
                        log::info!(
                            "Found valid alternative {:?}. Pushing it to the stack.",
                            sub_sop
                        );
                        stack.push(sub_sop.clone());
                        break;
                    }
                }
            }
        }
    }

    // 4. Serialize the modified stack and return it with the new state.
    let new_stack_json = serde_json::to_string(&stack).unwrap();
    (new_state, new_stack_json)
}

/// Recursively checks if an SOP is in a 'Completed' state.
///
/// - For an `Operation`, it checks if its state is `Completed`.
/// - For a `Sequence` or `Parallel`, it checks if **all** children are completed.
/// - For an `Alternative`, it checks if **any** child is completed.
fn is_sop_completed(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state =
                state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
            OperationState::from_str(&operation_state) == OperationState::Completed
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) => {
            // A Sequence or Parallel SOP is completed only when all of its children are completed.
            sops.iter()
                .all(|child_sop| is_sop_completed(sp_id, child_sop, state, &log_target))
        }
        SOP::Alternative(sops) => {
            // An Alternative is considered completed as soon as one of its branches completes.
            sops.iter()
                .any(|child_sop| is_sop_completed(sp_id, child_sop, state, &log_target))
        }
    }
}

/// Recursively checks if an SOP and all its children are in their initial state.
///
/// This is used to determine if an `Alternative` path has been chosen yet.
fn is_sop_in_initial_state(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state =
                state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
            OperationState::from_str(&operation_state) == OperationState::Initial
                || OperationState::from_str(&operation_state) == OperationState::UNKNOWN
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
            // Any container SOP is in its initial state only if all children are also in their initial state.
            sops.iter()
                .all(|child_sop| is_sop_in_initial_state(sp_id, child_sop, state, &log_target))
        }
    }
}

/// Recursively checks if an SOP is ready to start execution based on its preconditions.
///
/// - For an `Operation`, it checks if it's `Initial` and its `eval_running` guard is true.
/// - For a `Sequence`, it checks if its **first** child can start.
/// - For a `Parallel` it checks if **all** children can start.
/// - For an `Alternative`, it checks if **any** child can start.
fn can_sop_start(sp_id: &str, sop: &SOP, state: &State, log_target: &str) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state =
                state.get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
            (OperationState::from_str(&operation_state) == OperationState::Initial)
                && operation.eval_running(state, &log_target)
        }
        SOP::Sequence(sops) => {
            // A sequence can start if its very first element can start.
            sops.first().map_or(false, |first_sop| {
                can_sop_start(sp_id, first_sop, state, &log_target)
            })
        }
        SOP::Parallel(sops) => {
            // A Parallel or Alternative block can start if any of its children can start.
            sops.iter()
                .all(|child_sop| can_sop_start(sp_id, child_sop, state, &log_target))
        }
        SOP::Alternative(sops) => {
            // A Parallel or Alternative block can start if any of its children can start.
            sops.iter()
                .any(|child_sop| can_sop_start(sp_id, child_sop, state, &log_target))
        }
    }
}

pub fn uniquify_sop_operations(sop: SOP) -> SOP {
    match sop {
        // Base case: We found an Operation.
        SOP::Operation(op) => {
            // Generate a short, unique ID.
            let unique_id = nanoid::nanoid!(6);

            // Create the new, unique name.
            let new_name = format!("{}_{}", op.name, unique_id);

            // Return a new Operation SOP with the updated name.
            SOP::Operation(Box::new(Operation {
                name: new_name,
                state: op.state,
                timeout_ms: op.timeout_ms,
                retries: op.retries,
                preconditions: op.preconditions,
                postconditions: op.postconditions,
                fail_transitions: op.fail_transitions,
                timeout_transitions: op.timeout_transitions,
                reset_transitions: op.reset_transitions,
            }))
        }

        // Recursive cases: We found a container.
        // We process the children and then rebuild the container.
        SOP::Sequence(sops) => {
            let unique_children = sops
                .into_iter() // Consumes the vector
                .map(uniquify_sop_operations) // Recursively call on each child
                .collect(); // Collect into a new Vec<SOP>
            SOP::Sequence(unique_children)
        }
        SOP::Parallel(sops) => {
            let unique_children = sops.into_iter().map(uniquify_sop_operations).collect();
            SOP::Parallel(unique_children)
        }
        SOP::Alternative(sops) => {
            let unique_children = sops.into_iter().map(uniquify_sop_operations).collect();
            SOP::Alternative(unique_children)
        }
    }
}
