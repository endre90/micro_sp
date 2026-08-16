use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationProcessingType {
    Planned,
    SOP,
    Automatic,
}

// PERF: called once per active operation per tick by three different runners,
// so everything in here is multiplied by the number of running operations.
//
// 1. Key building: the function opens with six `format!("{}_...", operation.name)`
//    calls and closes with three more, plus more inside the match arms - about
//    a dozen heap allocations per operation per tick purely to name variables
//    that never change. Suggested: build these once when the operation becomes
//    active and carry them in a small struct alongside the `Operation`.
// 2. Each of those getters goes through `State::get_value`, which currently
//    clones the entire state map (see the note there). Six clones of the whole
//    state before any real work happens, per operation, per tick.
// 3. The `format!` for `new_op_info` runs on every arm every tick, but the
//    result is only used if it differs from `old_operation_information`. The
//    `Disabled` arm is the expensive one: it clones every precondition's guard
//    and runner guard into two `Predicate::OR` trees and renders both via
//    `Display` - a full predicate-tree walk with string building - *every
//    200 ms for every disabled operation*, and a disabled operation is exactly
//    the one that stays disabled for a long time. This is a very plausible
//    cause of the CPU spikes during SOP execution. Suggested: compute a cheap
//    discriminant (state + a small enum for the reason) and only build the
//    string when that discriminant changed.
// 4. DONE: `operation.clone().cancel(..)` / `.timeout(..)` / `.fail(..)` /
//    `.complete(..)` / `.retry(..)` deep-copied the whole `Operation` (all six
//    transition vectors) just to call a `&self` method. All twelve `.clone()`
//    calls are gone.
// 5. `elapased_executing_ms += OPERAION_RUNNER_TICK_INTERVAL_MS` assumes this
//    is called exactly on a 200 ms cadence, but `sop_runner` calls it at 100 ms
//    and `auto_operation_runner` skips ticks whenever Redis is slow. Timeouts
//    are therefore wrong in both directions. Suggested: store the start
//    `SystemTime` in the state when the operation enters Executing/Disabled and
//    compute elapsed time from the wall clock - more accurate, and it also
//    removes two state writes per operation per tick.
// 6. The three chained `.update(..)` calls at the end each clone the whole
//    state map (see `State::update`), and `_elapsed_executing_ms` /
//    `_elapsed_disabled_ms` are written every single tick for every operation,
//    which means the delta is never empty and every tick produces Redis
//    traffic even when the system is completely idle. Suggested: only write the
//    elapsed counters when they are actually consulted (i.e. derive them from a
//    stored start time, per point 5), which lets an idle system produce an
//    empty diff and skip the MSET entirely.
pub(super) async fn process_operation(
    sp_id: &str,
    mut new_state: State,
    operation: &Operation,
    operation_processing_type: OperationProcessingType,
    plan_current_step: Option<&mut i64>,
    plan_state: Option<&mut String>,
    // sop_state: Option<&mut String>,
    // logging_tx: mpsc::Sender<LogMsg>,
    // mut con: crate::SPConnection,
    log_target: &str,
    // terminated_operations: &mut Vec<String>
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


    // Test if this can be done, removing ops once they are terminated
    // let mut terminated_operations = new_state
    //     .get_array_or_default_to_empty(&format!("{}_terminated_operations", sp_id), &log_target);

    let mut logging_log = "".to_string();
    let mut op_info_level = log::Level::Info;
    match OperationState::from_str(&operation_state) {
        OperationState::Initial => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling");
                op_info_level = log::Level::Warn;
            } else if operation.eval(&new_state, &log_target) {
                new_state = operation.start(&new_state, &log_target);
                new_op_info = format!("Starting initialized operation '{}'.", operation.name);
                logging_log = format!("Starting");
                op_info_level = log::Level::Info;
            } else {
                new_op_info = format!("Disabling operation '{}'.", operation.name).to_string();
                logging_log = format!("Disabling");
                op_info_level = log::Level::Warn;
                new_state = operation.disable(&new_state, &log_target);
            }
        }
        OperationState::Disabled => {
            elapased_disabled_ms += OPERAION_RUNNER_TICK_INTERVAL_MS as i64;
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_timedout(&new_state, &log_target) {
                new_state = operation.timeout(&new_state, &log_target);
                new_op_info =
                    format!("Timeout for disabled operation '{}'.", operation.name).to_string();
                logging_log = format!("Timeout");
                op_info_level = log::Level::Warn;
            } else if operation.eval(&new_state, &log_target) {
                new_state = operation.start(&new_state, &log_target);
                new_op_info = format!("Starting disabled operation '{}'.", operation.name);
                logging_log = format!("Starting");
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
                logging_log = format!("Disabled");

                op_info_level = log::Level::Warn;
            }
        }
        OperationState::Executing => {
            elapased_executing_ms += OPERAION_RUNNER_TICK_INTERVAL_MS as i64;
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_failed(&new_state, &log_target) {
                new_state = operation.fail(&new_state, &log_target);
                new_op_info = format!("Failing operation '{}'.", operation.name).to_string();
                logging_log = format!("Failing");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_timedout(&new_state, &log_target) {
                new_state = operation.timeout(&new_state, &log_target);
                new_op_info =
                    format!("Timeout for executing operation '{}'.", operation.name).to_string();
                logging_log = format!("Timeout");
                op_info_level = log::Level::Warn;
            } else if operation.can_be_completed(&new_state, &log_target) {
                new_state = operation.complete(&new_state, &log_target);
                new_op_info = format!("Completing operation '{}'.", operation.name).to_string();
                logging_log = format!("Completing");
                op_info_level = log::Level::Info;
            } else {
                new_op_info = format!(
                    "Waiting for operation '{}' to be completed.",
                    operation.name
                )
                .to_string();
                logging_log = format!("Executing");
                op_info_level = log::Level::Info;
            }
        }
        OperationState::Completed => {
            new_state.update_mut(
                &format!("{}_failure_retry_counter", operation.name),
                0.to_spvalue(),
            );
            new_state.update_mut(
                &format!("{}_timeout_retry_counter", operation.name),
                0.to_spvalue(),
            );
            if let OperationProcessingType::Planned = operation_processing_type {
                if let Some(plan_current_step) = plan_current_step {
                    *plan_current_step += 1;
                }
            }
            // if let OperationProcessingType::Automatic = operation_processing_type {
            //     new_state = operation.initialize(&new_state, &log_target);
            // }
            
            new_op_info = format!("Operation '{}' completed.", operation.name);
            logging_log = format!("Completed");
            op_info_level = log::Level::Info;

            // commentout
            // match operation_processing_type {
            // OperationProcessingType::SOP | OperationProcessingType::Automatic => {
            new_state = operation.terminate(&new_state, TerminationReason::Completed, &log_target);
            // }
            // _ => (),
            // }
        }
        OperationState::Bypassed => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling");
            } else {
                new_op_info = format!(
                    "Operation '{}' bypassed. Continuing with the next operation.",
                    operation.name
                );
                logging_log = format!("Bypassed");
                if let OperationProcessingType::Planned = operation_processing_type {
                    if let Some(plan_current_step) = plan_current_step {
                        *plan_current_step += 1;
                    }
                }
            }
            op_info_level = log::Level::Warn;
            // match operation_processing_type {
            // OperationProcessingType::SOP => {
            new_state = operation.terminate(&new_state, TerminationReason::Bypassed, &log_target);
            // }
            // _ => (),
            // }
        }
        OperationState::Timedout => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling");
                op_info_level = log::Level::Warn;
            } else if operation_timeout_retry_counter < operation.timeout_retries {
                operation_timeout_retry_counter += 1;
                new_op_info = format!(
                    "Retrying operation (timeout) '{}'. Retry {} out of {}.",
                    operation.name, operation_timeout_retry_counter, operation.timeout_retries
                );
                logging_log = format!(
                    "Retrying {}/{}",
                    operation_timeout_retry_counter, operation.timeout_retries
                );
                op_info_level = log::Level::Warn;
                new_state = operation.retry(&new_state, &log_target);
                new_state.update_mut(
                    &format!("{}_timeout_retry_counter", operation.name),
                    operation_timeout_retry_counter.to_spvalue(),
                );
            } else if operation.can_be_bypassed {
                new_state = operation.bypass(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout. Bypassing.", operation.name);
                logging_log = format!("Bypassing");
                op_info_level = log::Level::Warn;
            } else {
                new_state = operation.fatal(&new_state, &log_target);
                new_op_info = format!("Operation '{}' timedout.", operation.name);
                logging_log = format!("Fatal timeout");
                op_info_level = log::Level::Warn;
            }
        }
        OperationState::Failed => {
            if operation.can_be_cancelled(&sp_id, &new_state, &log_target) {
                new_state = operation.cancel(&new_state, &log_target);
                new_op_info = format!("Cancelling operation '{}'.", operation.name).to_string();
                logging_log = format!("Cancelling");
                op_info_level = log::Level::Warn;
            } else if operation_failure_retry_counter < operation.failure_retries {
                operation_failure_retry_counter += 1;
                new_op_info = format!(
                    "Retrying operation (failure) '{}'. Retry {} out of {}.",
                    operation.name, operation_failure_retry_counter, operation.failure_retries
                );
                logging_log = format!(
                    "Retrying {}/{}",
                    operation_failure_retry_counter, operation.failure_retries
                );
                op_info_level = log::Level::Warn;
                new_state = operation.retry(&new_state, &log_target);
                new_state.update_mut(
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
                    logging_log = format!("Bypassing");
                    op_info_level = log::Level::Warn;
                } else {
                    new_state = operation.fatal(&new_state, &log_target);
                    new_op_info =
                        format!("Operation '{}' has no more retries left.", operation.name);
                    logging_log = format!("Fatal failure");
                    op_info_level = log::Level::Warn;
                }
                new_state.update_mut(
                    &format!("{}_failure_retry_counter", operation.name),
                    0.to_spvalue(),
                );
                new_state.update_mut(
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
            logging_log = format!("Unrecoverable");
            op_info_level = log::Level::Error;
            match operation_processing_type {
                OperationProcessingType::Planned => {
                    if let Some(plan_state) = plan_state {
                        *plan_state = PlanState::Failed.to_string();
                    }
                }

                // OperationProcessingType::SOP => {
                //     new_state =
                //         operation.terminate(&new_state, TerminationReason::Fatal, &log_target);
                // }
                _ => (),
            }
            // newly added
            new_state = operation.terminate(&new_state, TerminationReason::Fatal, &log_target);
        }
        OperationState::Cancelled => {
            new_op_info = format!(
                "Operation '{}' cancelled. Stopping execution.",
                operation.name
            );
            logging_log = format!("Cancelled");
            op_info_level = log::Level::Warn;
            match operation_processing_type {
                OperationProcessingType::Planned => {
                    if let Some(plan_state) = plan_state {
                        *plan_state = PlanState::Cancelled.to_string();
                    }
                }
                // OperationProcessingType::SOP => {
                //     new_state =
                //         operation.terminate(&new_state, TerminationReason::Cancelled, &log_target);
                // }
                _ => (),
            }
            new_state = operation.terminate(&new_state, TerminationReason::Cancelled, &log_target);
        }
        OperationState::UNKNOWN => {
            new_state = operation.initialize(&new_state, &log_target);
        }
        
        OperationState::Terminated(termination_reason) => {
            // terminated_operations.push(operation.name.to_spvalue());
            match termination_reason {
                TerminationReason::Bypassed => {
                    logging_log = format!("Bypassed");
                    new_op_info = format!(
                        "Operation '{}' terminated.",
                        operation.name
                    )
                }
                TerminationReason::Completed => {
                    logging_log = format!("Completed");
                    new_op_info = format!(
                        "Operation '{}' terminated.",
                        operation.name
                    )
                }
                TerminationReason::Fatal => {
                    logging_log = format!("Fatal");
                    new_op_info =
                        format!("Operation '{}' terminated.", operation.name)
                }
                TerminationReason::Cancelled => {
                    logging_log = format!("Cancelled");
                    new_op_info = format!(
                        "Operation '{}' terminated.",
                        operation.name
                    )
                }
            }
        }
    }

    if new_op_info != old_operation_information {
        match op_info_level {
            log::Level::Info => log::info!(target: &log_target, "{}", new_op_info),
            log::Level::Warn => log::warn!(target: &log_target, "{}", new_op_info),
            log::Level::Error => log::error!(target: &log_target, "{}", new_op_info),
            _ => (),
        }
        // No need to log terminated
        // if OperationState::from_str(&operation_state)
        //     != OperationState::Terminated(TerminationReason::Completed)
        // {
        // let operation_msg = OperationMsg {
        //     operation_name: operation.name.clone(),
        //     operation_processing_type: operation_processing_type,
        //     timestamp: Utc::now(),
        //     severity: op_info_level,
        //     state: OperationState::from_str(&operation_state),
        //     log: logging_log.to_string(),
        // };
        // let log_msg = LogMsg::OperationMsg(operation_msg);
        // match logging_tx.send(log_msg).await {
        //     Ok(()) => (),
        //     Err(e) => {
        //         log::error!(target: &log_target, "Failed to send logging with: {e}.")
        //     }
        // }
        // }
    }

    // DONE: this used to be three chained `.update(..)` calls, each cloning the
    // whole state map, for every active operation on every tick. Writing in
    // place costs nothing beyond the three map lookups.
    new_state.update_mut(
        &format!("{}_information", operation.name),
        new_op_info.to_spvalue(),
    );
    new_state.update_mut(
        &format!("{}_elapsed_executing_ms", operation.name),
        elapased_executing_ms.to_spvalue(),
    );
    new_state.update_mut(
        &format!("{}_elapsed_disabled_ms", operation.name),
        elapased_disabled_ms.to_spvalue(),
    );

    new_state
        // .update(
        //     &format!("{}_terminated_operations", sp_id),
        //     terminated_operations.to_spvalue(),
        // )
}
