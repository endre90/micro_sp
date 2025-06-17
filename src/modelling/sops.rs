use crate::Operation;
use crate::*;
use serde::{Deserialize, Serialize};

// I look at SOPS as function blocks with a rigid structure, sort of as a high level operation
// Maybe, just maybe, we can also have a "Planned" variant that should use a planner within a certain domain to get a sequence???
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum SOP {
    Operation(Box<Operation>),
    Sequence(Vec<SOP>),
    Parallel(Vec<SOP>),
    Alternative(Vec<SOP>),
    // Planned(Vec<SOP>), ?? Maybe
}

// Old, working
// #[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
// pub struct SOPStruct {
//     pub id: String,
//     pub sop: Vec<Operation>,
// }

//New, experimental
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct SOPStruct {
    pub id: String,
    pub sop: SOP,
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

    let mut operation_state_old = "".to_string();
    let mut operation_information_old = "".to_string();

    let mut new_state = state.clone();

    // 1. Deserialize the stack. Initialize if it's empty or invalid.
    let mut stack: Vec<SOP> = serde_json::from_str(&stack_json).unwrap_or_else(|_| {
        log::info!("Stack is empty or invalid, initializing with root SOP.");
        vec![root_sop.clone()]
    });

    // If the stack is empty after initialization, the SOP is done.
    if stack.is_empty() {
        log::info!("SOP execution is complete.");
        return (new_state, serde_json::to_string(&stack).unwrap());
    }

    // 2. Pop ONE item from the stack to process in this tick.
    let current_sop = stack.pop().unwrap();
    // log::info!("Popped SOP node for processing: {:?}", current_sop);

    

    // 3. Apply the execution logic for the popped node.
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

            // Log only when something changes and not every tick
            if operation_state_old != operation_state {
                log::info!(target: &format!("{}_operation_runner", sp_id), "Current state of operation {}: {}.", operation.name, operation_state);
                operation_state_old = operation_state.clone()
            }

            if operation_information_old != operation_information {
                log::info!(target: &format!("{}_operation_runner", sp_id), "{}.", operation_information);
                operation_information_old = operation_information.clone()
            }

            match OperationState::from_str(&operation_state) {
                OperationState::Initial => {
                    if operation.eval_running(&new_state) {
                        new_state = operation.start_running(&new_state);
                        operation_information =
                            format!("Operation '{}' started execution", operation.name);
                    }
                }
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
                        operation_information =
                            format!("Operation failed, no more retries left. Unrecoverable");
                    }
                }
                OperationState::Unrecoverable => {
                    // new_state = operation.reinitialize_running(&new_state); // reinitialize globally when sop is done
                    operation_information = format!("Failing the sop: {:?}", root_sop);
                }
                OperationState::UNKNOWN => (),
            }

            new_state = new_state.update(
                &format!("{}_information", operation.name),
                operation_information.to_spvalue(),
            );

            // Operation is consumed for this tick, not pushed back.
        }

        // In your `match current_sop { ... }`
        SOP::Sequence(sops) => {
            log::info!("Evaluating a Sequence node to find the next step.");

            // Your logic of finding the first unfinished step is correct.
            // We can express it concisely with `find()`.
            let next_sop_to_run = sops
                .iter()
                .find(|sub_sop| !is_sop_completed(sp_id, sub_sop, &new_state));

            // Now, we act based on whether we found an unfinished step.
            if let Some(sub_sop) = next_sop_to_run {
                // CASE 1: The sequence is NOT finished.
                // We found the next step that is not yet completed.

                // THE FIX:
                // First, push the parent Sequence back onto the stack so we can re-evaluate it
                // after the child is processed. This prevents the stack from emptying prematurely.
                log::info!("Re-queuing the parent Sequence to manage subsequent steps.");
                stack.push(current_sop.clone()); // `current_sop` is this Sequence instance

                // Then, push the specific step that needs to be processed onto the stack.
                // This will be the next thing to be executed.
                log::info!("Pushing next sequence step for execution: {:?}", sub_sop);
                stack.push(sub_sop.clone());
            } else {
                // CASE 2: The sequence IS finished.
                // `find()` returned `None`, meaning all sub-SOPs reported `is_sop_completed() == true`.
                // By doing nothing here, we allow the Sequence to be "consumed" from the stack.
                // If it was the last item, the SOP will correctly be flagged as completed on the next tick.
                log::info!("Sequence is complete. It will not be re-queued.");
            }
        }

        // SOP::Sequence(sops) => {
        //     log::info!("Finding next step in a Sequence node.");
        //     let mut found_active_child = false;
        //     for sub_sop in sops.iter() {
        //         if !is_sop_completed(sp_id, sub_sop, &new_state) {
        //             log::info!("Pushing next sequence step: {:?}", sub_sop);
        //             stack.push(sub_sop.clone());
        //             found_active_child = true;
        //             break; // Only push the first unfinished one.
        //         }
        //     }
        //     if !found_active_child {
        //         log::info!("Sequence is complete.");
        //     }
        // }
        SOP::Parallel(sops) => {
            log::info!("Dispatching all unfinished children of a Parallel node.");
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

// // To exec the pseudo async
// // Inside your main async loop
// loop {
//     let (response_tx, response_rx) = oneshot::channel();
//     command_sender
//         .send(StateManagement::GetState(response_tx))
//         .await?;
//     let state = response_rx.await?;
//     let mut new_state = state.clone();

//     // Get the SOP runner state (Initial, Executing, etc.)
//     let mut sop_overall_state = SOPState::from_str(&state.get_string_or_default(
//         // ... your key logic
//     ));

//     match sop_overall_state {
//         SOPState::Executing => {
//             // Get the root SOP definition
//             let root_sop = &model
//                 .sops
//                 .iter()
//                 .find(|sop| sop.id == sop_id.to_string()) // assuming sop_id is fetched
//                 .unwrap()
//                 .to_owned();

//             // Fetch the current stack from the state
//             let stack_key = format!("{}_sop_stack", sp_id);
//             let stack_json = state.get_string(&stack_key).unwrap_or_else(|| "[]".to_string());

//             // *** CALL THE NEW TICK FUNCTION ***
//             let (updated_state, new_stack_json) = run_sop_tick(&state, stack_json, root_sop).await;
//             new_state = updated_state;

//             // Update the stack in the new_state object
//             new_state = new_state.update(&stack_key, new_stack_json.to_spvalue());

//             // If the new stack is empty, the SOP is finished
//             if new_stack_json == "[]" {
//                 sop_overall_state = SOPState::Completed;
//             }
//         }
//         // ... other SOPState match arms (Initial, Completed, Failed)
//     }

//     new_state = new_state.update(
//         &format!("{}_sop_state", sp_id),
//         sop_overall_state.to_string().to_spvalue()
//     );

//     // Commit all changes (operation states AND stack state)
//     let modified_state = state.get_diff_partial_state(&new_state);
//     if !modified_state.is_empty() {
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;
//     }

//     interval.tick().await;
// }
