use crate::*;
use redis::aio::MultiplexedConnection;
use tokio::time::{Duration, interval};

pub async fn sop_runner(
    sp_id: &str,
    model: &Model,
    mut con: MultiplexedConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let log_target = &format!("{}_sop_runner", sp_id);

    log::info!(target: log_target, "Online and managing SOP.");

    // Get only the relevant keys from the state
    let mut keys: Vec<String> = model
        .sops
        .iter()
        .flat_map(|sop_struct| sop_struct.sop.get_all_var_keys())
        .collect();

    // We also need some of the planner vars
    keys.extend(vec![
        format!("{}_sop_stack", sp_id),
        format!("{}_sop_id", sp_id),
        format!("{}_sop_state", sp_id),
        format!("{}_sop_enabled", sp_id),
    ]);

    // And the vars to keep trask of operation states
    keys.extend(
        model
            .operations
            .iter()
            .flat_map(|op| {
                vec![
                    format!("{}", op.name),
                    format!("{}_information", op.name),
                    format!("{}_retry_counter", op.name),
                ]
            })
            .collect::<Vec<String>>(),
    );

    loop {
        if let Some(state) = redis_get_full_state(&mut con).await {
            let old_stack_json = state.get_string_or_value(
                &format!("{}_sop_runner", sp_id),
                &format!("{}_sop_stack", sp_id),
                "[]".to_string(),
            );

            let new_state = process_sop_tick(sp_id, model, &state)?;

            let new_stack_json = new_state.get_string_or_value(
                &format!("{}_sop_runner", sp_id),
                &format!("{}_sop_stack", sp_id),
                "[]".to_string(),
            );
            if old_stack_json != new_stack_json
                && !new_stack_json.is_empty()
                && new_stack_json != "[]"
            {
                let sop_id = new_state.get_string_or_default_to_unknown(
                    &format!("{}_sop_runner", sp_id),
                    &format!("{}_sop_id", sp_id),
                );

                if let Some(root_sop) = model.sops.iter().find(|s| s.id == sop_id) {
                    log::info!(target: log_target, "{:?}", visualize_sop(&root_sop.sop));
                }
            }

            let modified_state = state.get_diff_partial_state(&new_state);
            if !modified_state.state.is_empty() {
                redis_set_state(&mut con, modified_state).await;
            }
        }

        interval.tick().await;
    }
}

fn process_sop_tick(
    sp_id: &str,
    model: &Model,
    state: &State,
) -> Result<State, Box<dyn std::error::Error>> {
    let mut new_state = state.clone();
    let mut sop_overall_state = state.get_string_or_default_to_unknown(
        &format!("{}_sop_runner", sp_id),
        &format!("{}_sop_state", sp_id),
    );

    match SOPState::from_str(&sop_overall_state) {
        SOPState::Initial => {
            handle_sop_initial(sp_id, model, state, &mut new_state, &mut sop_overall_state)?;
        }
        SOPState::Executing => {
            handle_sop_executing(sp_id, model, state, &mut new_state, &mut sop_overall_state);
        }
        SOPState::Completed | SOPState::Failed => {}
        SOPState::UNKNOWN => {
            log::warn!(target: &format!("{}_sop_runner", sp_id), "SOP in UNKNOWN state. Resetting.");
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
) -> Result<(), Box<dyn std::error::Error>> {
    if state.get_bool_or_default_to_false(
        &format!("{}_sop_runner", sp_id),
        &format!("{}_sop_enabled", sp_id),
    ) {
        log::info!(target: &format!("{}_sop_runner", sp_id), "SOP enabled. Transitioning to Executing.");
        let sop_id = state.get_string_or_default_to_unknown(
            &format!("{}_sop_runner", sp_id),
            &format!("{}_sop_id", sp_id),
        );

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

/// Handles the `Executing` state logic.
fn handle_sop_executing(
    sp_id: &str,
    model: &Model,
    state: &State,
    new_state: &mut State,
    sop_overall_state: &mut String,
) {
    let sop_id = state.get_string_or_default_to_unknown(
        &format!("{}_sop_runner", sp_id),
        &format!("{}_sop_id", sp_id),
    );
    let stack_json =
        state.get_string_or_value(sp_id, &format!("{}_sop_stack", sp_id), "[]".to_string());

    let Some(root_sop) = model.sops.iter().find(|s| s.id == sop_id) else {
        log::error!(target: &format!("{}_sop_runner", sp_id), "SOP with id '{}' not found in model. Failing.", sop_id);
        *sop_overall_state = SOPState::Failed.to_string();
        return;
    };

    let (updated_state, new_stack_json) = run_sop_tick(sp_id, state, stack_json, &root_sop.sop);
    *new_state = updated_state;

    // Persist the new stack state for the next tick.
    *new_state = new_state.update(&format!("{}_sop_stack", sp_id), new_stack_json.to_spvalue());

    // Check for terminal conditions.
    if new_stack_json == "[]" {
        log::info!(target: &format!("{}_sop_runner", sp_id), "Execution stack empty. SOP Completed.");
        *sop_overall_state = SOPState::Completed.to_string();
    } else if is_sop_failed(sp_id, &root_sop.sop, new_state) {
        log::error!(target: &format!("{}_sop_runner", sp_id), "Unrecoverable error detected. SOP Failed.");
        *sop_overall_state = SOPState::Failed.to_string();
    }
}

fn is_sop_failed(sp_id: &str, sop: &SOP, state: &State) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let op_state_str = state.get_string_or_default_to_unknown(
                &format!("{}_sop_runner", sp_id),
                &operation.name,
            );
            OperationState::from_str(&op_state_str) == OperationState::Unrecoverable
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => sops
            .iter()
            .any(|child_sop| is_sop_failed(sp_id, child_sop, state)),
    }
}

/// Represents the logic for a single tick of the SOP executor using the Re-dispatch pattern.
/// It is stateless and relies on the state and stack passed to it.
pub fn run_sop_tick(
    sp_id: &str,
    state: &State,
    stack_json: String, // The current evaluation stack
    root_sop: &SOP,     // The full SOP, in case the stack is empty
) -> (State, String) {
    // Returns the new state and the new stack_json

    let mut new_state = state.clone();

    // 1. Deserialize the stack. Initialize if it's empty or invalid.
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

            match OperationState::from_str(&operation_state) {
                OperationState::Initial => {
                    if operation.eval_running(&new_state) {
                        new_state = operation.start_running(&new_state);
                        log::info!("Operation '{}' started execution", operation.name);
                        operation_information =
                            format!("Operation '{}' started execution", operation.name);
                    }
                }
                OperationState::Executing => {
                    if operation.can_be_completed(&state) {
                        new_state = operation.clone().complete_running(&new_state);
                        operation_information = "Completing operation".to_string();
                        log::info!("Completing operation '{}'", operation.name);
                    } else if operation.can_be_failed(&state) {
                        new_state = operation.clone().fail_running(&new_state);
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
                    new_state = operation.unrecover_running(&new_state);
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
                        new_state = operation.clone().retry_running(&new_state);
                        new_state = new_state.update(
                            &format!("{}_retry_counter", operation.name),
                            operation_retry_counter.to_spvalue(),
                        );
                    } else {
                        new_state = operation.unrecover_running(&new_state);
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
                .find(|sub_sop| !is_sop_completed(sp_id, sub_sop, &new_state));

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
                if !is_sop_completed(sp_id, sub_sop, &new_state) {
                    stack.push(sub_sop.clone());
                }
            }
        }

        SOP::Alternative(sops) => {
            log::info!("Processing an Alternative node.");

            let chosen_path = sops
                .iter()
                .find(|sop| !is_sop_in_initial_state(sp_id, sop, &new_state));

            if let Some(path) = chosen_path {
                log::info!(
                    "Alternative path {:?} is already active. Pushing for continued execution.",
                    path
                );
                stack.push(path.clone());
            } else {
                log::info!("No active path found. Evaluating new alternatives.");
                for sub_sop in sops {
                    if can_sop_start(sp_id, &sub_sop, &new_state) {
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
fn is_sop_completed(sp_id: &str, sop: &SOP, state: &State) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state = state.get_string_or_default_to_unknown(
                &format!("{}_sop_runner", sp_id),
                &format!("{}", operation.name),
            );
            OperationState::from_str(&operation_state) == OperationState::Completed
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) => {
            // A Sequence or Parallel SOP is completed only when all of its children are completed.
            sops.iter()
                .all(|child_sop| is_sop_completed(sp_id, child_sop, state))
        }
        SOP::Alternative(sops) => {
            // An Alternative is considered completed as soon as one of its branches completes.
            sops.iter()
                .any(|child_sop| is_sop_completed(sp_id, child_sop, state))
        }
    }
}

/// Recursively checks if an SOP and all its children are in their initial state.
///
/// This is used to determine if an `Alternative` path has been chosen yet.
fn is_sop_in_initial_state(sp_id: &str, sop: &SOP, state: &State) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state = state.get_string_or_default_to_unknown(
                &format!("{}_sop_runner", sp_id),
                &format!("{}", operation.name),
            );
            OperationState::from_str(&operation_state) == OperationState::Initial
                || OperationState::from_str(&operation_state) == OperationState::UNKNOWN
        }
        SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
            // Any container SOP is in its initial state only if all children are also in their initial state.
            sops.iter()
                .all(|child_sop| is_sop_in_initial_state(sp_id, child_sop, state))
        }
    }
}

/// Recursively checks if an SOP is ready to start execution based on its preconditions.
///
/// - For an `Operation`, it checks if it's `Initial` and its `eval_running` guard is true.
/// - For a `Sequence`, it checks if its **first** child can start.
/// - For a `Parallel` it checks if **all** children can start.
/// - For an `Alternative`, it checks if **any** child can start.
fn can_sop_start(sp_id: &str, sop: &SOP, state: &State) -> bool {
    match sop {
        SOP::Operation(operation) => {
            let operation_state = state.get_string_or_default_to_unknown(
                &format!("{}_sop_runner", sp_id),
                &format!("{}", operation.name),
            );
            (OperationState::from_str(&operation_state) == OperationState::Initial)
                && operation.eval_running(state)
        }
        SOP::Sequence(sops) => {
            // A sequence can start if its very first element can start.
            sops.first()
                .map_or(false, |first_sop| can_sop_start(sp_id, first_sop, state))
        }
        SOP::Parallel(sops) => {
            // A Parallel or Alternative block can start if any of its children can start.
            sops.iter()
                .all(|child_sop| can_sop_start(sp_id, child_sop, state))
        }
        SOP::Alternative(sops) => {
            // A Parallel or Alternative block can start if any of its children can start.
            sops.iter()
                .any(|child_sop| can_sop_start(sp_id, child_sop, state))
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

// New, experimental (Now actually old working):
// The main async task that drives the SOP execution tick by tick.
// pub async fn sop_runner(
//     sp_id: &str,
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));

//     let mut sop_old = SOP::Operation(Box::new(Operation::default()));

//     log::info!(target: &format!("{}_sop_runner", sp_id), "Online and managing SOP.");

//     loop {
//         // 1. Fetch the latest state from the state manager.
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender
//             .send(StateManagement::GetState(response_tx))
//             .await?;
//         let state = response_rx.await?;
//         let mut new_state = state.clone();

//         // 2. Determine the overall state of the SOP (Executing, Completed, etc.).
//         // let sop_state_key = format!("{}_sop_state", sp_id);
//         let mut sop_overall_state = state.get_string_or_default_to_unknown(
//             &format!("{}_sop_runner", sp_id),
//             &format!("{}_sop_state", sp_id),
//         );

//         let sop_id = state.get_string_or_default_to_unknown(
//             &format!("{}_sop_runner", sp_id),
//             &format!("{}_sop_id", sp_id),
//         );

//         // Find the specific SOP definition this runner is responsible for, once at the start.
//         // This assumes your `Model` has a way to look up a SOP by its ID.
//         // let root_sop = model
//         //     .sops
//         //     .iter()
//         //     .find(|sop| sop.id == sop_id) // This assumes your SOP struct in the model has an `id` field.
//         //     .ok_or_else(|| format!("SOP with id '{}' not found in model", sop_id))?
//         //     // .sop
//         //     .clone();

//         // let mut sop_overall_state =
//         //     SOPState::from_str(&state.get_string_or_default(&sop_state_key, "Initial"));

//         // 3. Act based on the overall SOP state.
//         match SOPState::from_str(&sop_overall_state) {
//             SOPState::Initial => {
//                 let mut sop_enabled = state.get_bool_or_default_to_false(
//                     &format!("{}_sop_runner", sp_id),
//                     &format!("{}_sop_enabled", sp_id),
//                 );
//                 if sop_enabled {
//                     log::info!(target: &format!("{}_sop_runner", sp_id), "SOP enabled. Transitioning to Executing.");
//                     let root_sop = &model
//                         .sops
//                         .iter()
//                         .find(|sop| sop.id == sop_id.to_string())
//                         .unwrap()
//                         .to_owned();
//                     // Initialize the stack with the root SOP.
//                     let initial_stack = vec![root_sop.sop.clone()];
//                     let stack_key = format!("{}_sop_stack", sp_id);
//                     new_state = new_state.update(
//                         &stack_key,
//                         serde_json::to_string(&initial_stack)?.to_spvalue(),
//                     );

//                     // Set the overall state to Executing.
//                     sop_overall_state = SOPState::Executing.to_string();
//                     new_state =
//                         new_state.update(&format!("{}_sop_state", sp_id), false.to_spvalue()); // Consume the enable trigger
//                 }
//             }
//             SOPState::Executing => {
//                 let root_sop = &model
//                     .sops
//                     .iter()
//                     .find(|sop| sop.id == sop_id.to_string())
//                     .unwrap()
//                     .to_owned();
//                 // Fetch the current execution stack.
//                 let mut stack_json = state.get_string_or_value(
//                     &format!("{}_sop_runner", sp_id),
//                     &format!("{}_sop_stack", sp_id),
//                     "[]".to_string(),
//                 );

//                 // *** THIS IS THE CORE CALL TO YOUR NEW TICK FUNCTION ***
//                 let (updated_state, new_stack_json) =
//                     run_sop_tick(sp_id, &state, stack_json, &root_sop.sop);

//                 // Log only when something changes and not every tick
//                 if sop_old != root_sop.sop {
//                     log::info!("Got SOP:");
//                     log::info!("{:?}", visualize_sop(&root_sop.sop));
//                     sop_old = root_sop.sop.clone()
//                 }

//                 new_state = updated_state;

//                 // Persist the new stack state for the next tick.
//                 new_state =
//                     new_state.update(&format!("{}_sop_stack", sp_id), new_stack_json.to_spvalue());

//                 // Check for terminal conditions.
//                 if new_stack_json == "[]" {
//                     log::info!(target: &format!("{}_sop_runner", sp_id), "Execution stack is empty. SOP is Completed.");
//                     sop_overall_state = SOPState::Completed.to_string();
//                 } else if is_sop_failed(sp_id, &root_sop.sop, &new_state) {
//                     log::error!(target: &format!("{}_sop_runner", sp_id), "Unrecoverable error detected in an operation. SOP has Failed.");
//                     sop_overall_state = SOPState::Failed.to_string();
//                 }
//             }
//             // For terminal states, the runner will idle until the state is reset externally.
//             SOPState::Completed => { /* SOP is done. Do nothing. */ }
//             SOPState::Failed => { /* SOP has failed. Do nothing. */ }
//             SOPState::UNKNOWN => {
//                 log::warn!(target: &format!("{}_sop_runner", sp_id), "SOP is in an UNKNOWN state. Resetting to Initial.");
//                 sop_overall_state = SOPState::Initial.to_string();
//             }
//         }

//         // 4. Update the overall SOP state variable.
//         new_state = new_state.update(
//             &format!("{}_sop_state", sp_id),
//             sop_overall_state.to_string().to_spvalue(),
//         );

//         // 5. Commit all changes made during this tick to the central state manager.
//         let modified_state = state.get_diff_partial_state(&new_state);
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;

//         // 6. Wait for the next tick.
//         interval.tick().await;
//     }
// }

// Helper function to detect if any operation within the SOP has become unrecoverable.
// fn is_sop_failed(sp_id: &str, sop: &SOP, state: &State) -> bool {
//     match sop {
//         SOP::Operation(operation) => {
//             let op_state_str = state.get_string_or_default_to_unknown(
//                 &format!("{}_sop_runner", sp_id),
//                 &operation.name,
//             );
//             OperationState::from_str(&op_state_str) == OperationState::Unrecoverable
//         }
//         SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
//             // If any child in any branch has failed, the entire SOP is considered failed.
//             sops.iter()
//                 .any(|child_sop| is_sop_failed(sp_id, child_sop, state))
//         }
//     }
// }

// Super old, working
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
//     let mut operation_state_old = "".to_string();
//     let mut operation_information_old = "".to_string();

//     log::info!(target: &&format!("{}_sop_runner", sp_id), "Online.");

//     // let sops = model.sops;

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

//         let mut sop_enabled = state.get_bool_or_default_to_false(
//             &format!("{}_sop_runner", sp_id),
//             &format!("{}_sop_enabled", sp_id),
//         );

//         // Log only when something changes and not every tick
//         if sop_state_old != sop_state {
//             log::info!(target: &format!("{}_sop_runner", sp_id), "SOP current state: {sop_state}.");
//             sop_state_old = sop_state.clone()
//         }

//         match SOPState::from_str(&sop_state) {
//             SOPState::Initial => {
//                 if sop_enabled {
//                     sop_state = SOPState::Executing.to_string();
//                     sop_enabled = false;
//                 }
//             }
//             SOPState::Executing => {
//                 let sop_struct = &model
//                     .sops
//                     .iter()
//                     .find(|sop| sop.id == sop_id.to_string())
//                     .unwrap()
//                     .to_owned();

//                 let sop_names_list: Vec<String> = sop_struct.sop.iter().map(|op| op.name.clone()).collect();
//                 if sop_old != sop_names_list {
//                     sop_current_step = 0;
//                     log::info!(
//                         target: &format!("{}_sop_runner", sp_id),
//                         "Got a sop:\n{}",
//                         sop_names_list.iter()
//                             .enumerate()
//                             .map(|(index, step)| format!("       {} -> {}", index + 1, step))
//                             .collect::<Vec<String>>()
//                             .join("\n")
//                     );
//                     sop_old = sop_names_list
//                 }

//                 if sop_struct.sop.len() > sop_current_step as usize {
//                     let operation = sop_struct.sop[sop_current_step as usize].clone();

//                     let operation_state = state.get_string_or_default_to_unknown(
//                         &format!("{}_sop_runner", sp_id),
//                         &format!("{}", operation.name),
//                     );

//                     let mut operation_information = state.get_string_or_default_to_unknown(
//                         &format!("{}_sop_runner", sp_id),
//                         &format!("{}_information", operation.name),
//                     );

//                     let mut operation_retry_counter = state.get_int_or_default_to_zero(
//                         &format!("{}_sop_runner", sp_id),
//                         &format!("{}_retry_counter", operation.name),
//                     );

//                     // Log only when something changes and not every tick
//                     if operation_state_old != operation_state {
//                         log::info!(target: &format!("{}_sop_runner", sp_id), "Current state of operation {}: {}.", operation.name, operation_state);
//                         operation_state_old = operation_state.clone()
//                     }

//                     if operation_information_old != operation_information {
//                         log::info!(target: &format!("{}_sop_runner", sp_id), "{}.", operation_information);
//                         operation_information_old = operation_information.clone()
//                     }

//                     // let operation_start_time = state.get_int_or_default_to_zero(
//                     //     &format!("{}_sop_runner", sp_id),
//                     //     &format!("{}_start_time", operation.name),
//                     // );

//                     match OperationState::from_str(&operation_state) {
//                         OperationState::Initial => {
//                             if operation.eval_running(&new_state) {
//                                 new_state = operation.start_running(&new_state);
//                                 operation_information =
//                                     format!("Operation '{}' started execution", operation.name);
//                             }
//                             // let (eval, idx) =
//                             //     operation.eval_running_with_transition_index(&new_state);
//                             // if eval {
//                             //     new_state = new_state.update(
//                             //         &format!("{}_start_time", operation.name),
//                             //         now_as_millis_i64().to_spvalue(),
//                             //     );
//                             //     tokio::time::sleep(Duration::from_millis(
//                             //         operation.preconditions[idx].delay_ms,
//                             //     ))
//                             //     .await;
//                             //     new_state = operation.start_running(&new_state);
//                             //     operation_information =
//                             //         format!("Operation '{}' started execution", operation.name);
//                             // }
//                             // else {
//                             //     new_state = operation.block_running(&new_state);
//                             // }
//                         }
//                         // OperationState::Blocked => {
//                         //     if operation.eval_running(&new_state) {
//                         //         new_state = operation.start_running(&new_state);
//                         //         operation_information =
//                         //             format!("Operation '{}' started execution", operation.name);
//                         //     }
//                         //     // let (eval, idx) =
//                         //     //     operation.eval_running_with_transition_index(&new_state);
//                         //     // if eval {
//                         //     //     new_state = operation.start_running(&new_state);
//                         //     //     operation_information =
//                         //     //         format!("Operation '{}' started execution", operation.name);
//                         //     // } else {
//                         //     //     operation_information = format!(
//                         //     //         "Operation '{}' can't start yet, blocked by guard: {}",
//                         //     //         operation.name, operation.preconditions[idx].runner_guard
//                         //     //     );
//                         //     // }
//                         // }

//                         // probbaly causeing problems
//                         // OperationState::Executing => {
//                         //     match operation.timeout_ms {
//                         //         Some(timeout) => {
//                         //             if operation_start_time > 0 {
//                         //             let elapsed_ms =
//                         //                 now_as_millis_i64().saturating_sub(operation_start_time);
//                         //             if elapsed_ms >= timeout {
//                         //                 // log::error!(target: &format!("{}_sop_runner", sp_id), "HAS TO TIMEOUT HERE!");
//                         //                 new_state = operation.timeout_running(&new_state);
//                         //                 operation_information =
//                         //                     format!("Operation '{}' timed out", operation.name);
//                         //             } else {
//                         //                 if operation.can_be_failed(&new_state) {
//                         //                     // log::error!(target: &format!("{}_sop_runner", sp_id), "HAS TO FAIL HERE!");
//                         //                     new_state = operation.clone().fail_running(&new_state);
//                         //                     operation_information =
//                         //                         format!("Failing {}", operation.name);
//                         //                 } else {
//                         //                     let (eval, idx) = operation
//                         //                         .can_be_completed_with_transition_index(&new_state);
//                         //                     tokio::time::sleep(Duration::from_millis(
//                         //                         operation.postconditions[idx].delay_ms,
//                         //                     ))
//                         //                     .await;
//                         //                     if eval {
//                         //                         // log::error!(target: &format!("{}_sop_runner", sp_id), "HAS TO COMPLETE HERE!");
//                         //                         new_state =
//                         //                             operation.clone().complete_running(&new_state);
//                         //                         operation_information =
//                         //                             format!("Completing {}", operation.name);
//                         //                     } else {
//                         //                         operation_information = format!(
//                         //                             "Waiting for {} to be completed",
//                         //                             operation.name
//                         //                         );
//                         //                     }
//                         //                 }
//                         //                 }
//                         //             }
//                         //         }
//                         //         None => {
//                         //             if operation.can_be_failed(&new_state) {
//                         //                 new_state = operation.clone().fail_running(&new_state);
//                         //                 operation_information =
//                         //                     format!("Failing {}", operation.name);
//                         //             } else {
//                         //                 let (eval, idx) = operation
//                         //                     .can_be_completed_with_transition_index(&new_state);
//                         //                 tokio::time::sleep(Duration::from_millis(
//                         //                     operation.postconditions[idx].delay_ms,
//                         //                 ))
//                         //                 .await;
//                         //                 if eval {
//                         //                     new_state =
//                         //                         operation.clone().complete_running(&new_state);
//                         //                     operation_information =
//                         //                         format!("Completing {}", operation.name);
//                         //                 } else {
//                         //                     operation_information = format!(
//                         //                         "Waiting for {} to be completed",
//                         //                         operation.name
//                         //                     );
//                         //                 }
//                         //             }
//                         //         }
//                         //     }
//                         // }
//                         OperationState::Executing => {
//                             if operation.can_be_completed(&state) {
//                                 new_state = operation.clone().complete_running(&new_state);
//                                 operation_information = "Completing operation".to_string();
//                             } else if operation.can_be_failed(&state) {
//                                 new_state = operation.clone().fail_running(&new_state);
//                                 operation_information = "Failing operation".to_string();
//                             } else {
//                                 operation_information = "Waiting to be completed".to_string();
//                             }
//                         }
//                         OperationState::Completed => {
//                             new_state = operation.reinitialize_running(&new_state);
//                             operation_information =
//                                 format!("Operation {} completed, reinitializing", operation.name);
//                             new_state = new_state.update(
//                                 &format!("{}_retry_counter", operation.name),
//                                 0.to_spvalue(),
//                             );
//                             new_state = new_state
//                                 .update(&format!("{}_start_time", operation.name), 0.to_spvalue());
//                             sop_current_step = sop_current_step + 1;
//                         }
//                         OperationState::Timedout => {
//                             new_state = operation.unrecover_running(&new_state);
//                             operation_information =
//                                 format!("Timedout {}. Unrecoverable", operation.name);
//                         }
//                         OperationState::Failed => {
//                             if operation_retry_counter < operation.retries {
//                                 operation_retry_counter = operation_retry_counter + 1;
//                                 operation_information = format!(
//                                     "Retrying '{}'. Retry nr. {} out of {}",
//                                     operation.name, operation_retry_counter, operation.retries
//                                 );
//                                 new_state = operation.clone().retry_running(&new_state);
//                                 new_state = new_state.update(
//                                     &format!("{}_retry_counter", operation.name),
//                                     operation_retry_counter.to_spvalue(),
//                                 );
//                             } else {
//                                 new_state = operation.unrecover_running(&new_state);
//                                 new_state = new_state.update(
//                                     &format!("{}_retry_counter", operation.name),
//                                     0.to_spvalue(),
//                                 );
//                                 operation_information = format!(
//                                     "Operation failed, no more retries left. Unrecoverable"
//                                 );
//                             }
//                         }
//                         OperationState::Unrecoverable => {
//                             sop_state = SOPState::Failed.to_string();
//                             new_state = operation.reinitialize_running(&new_state);
//                             operation_information = format!("Failing the sop: {:?}", sop_struct);
//                         }
//                         OperationState::UNKNOWN => (),
//                     }
//                     new_state = new_state.update(
//                         &format!("{}_information", operation.name),
//                         operation_information.to_spvalue(),
//                     );
//                 } else {
//                     sop_state = SOPState::Completed.to_string();
//                 }
//             }
//             // PlanState::Paused => {}
//             SOPState::Failed => {
//                 // sop_state = SOPState::Initial.to_string();
//                 // planner_state = PlannerState::Ready.to_string();
//             }
//             // PlanState::NotFound => {}
//             SOPState::Completed => {
//                 // sop_state = SOPState::Initial.to_string();
//                 // planner_state = PlannerState::Ready.to_string();
//             }
//             // PlanState::Cancelled => {}
//             SOPState::UNKNOWN => {
//                 // sop_state = SOPState::Initial.to_string();
//                 // planner_state = PlannerState::Ready.to_string();
//             }
//         }
//         // }
//         new_state = new_state
//             .update(&format!("{}_sop_state", sp_id), sop_state.to_spvalue())
//             .update(&format!("{}_sop_enabled", sp_id), sop_enabled.to_spvalue())
//             .update(
//                 &format!("{}_sop_current_step", sp_id),
//                 sop_current_step.to_spvalue(),
//             );

//         let modified_state = state.get_diff_partial_state(&new_state);
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;

//         interval.tick().await;
//     }
// }
