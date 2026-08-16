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
// 3. DONE: the `format!` for `new_op_info` ran on every arm every tick, but the
//    result is only used if it differs from `old_operation_information`. The
//    `Disabled` arm was the expensive one: it clones every precondition's guard
//    and runner guard into two `Predicate::OR` trees and renders both via
//    `Display` - a full predicate-tree walk with string building - *every
//    200 ms for every disabled operation*, and a disabled operation is exactly
//    the one that stays disabled for a long time.
//    Both steady-state messages (`Disabled` and the waiting branch of
//    `Executing`) are pure functions of the operation, so they were rebuilt
//    byte-identically every tick and then discarded by the `!=` check. They are
//    now built once, when the operation first reports that state, and skipped
//    afterwards via `info_already_reads_as` - an allocation-free prefix test
//    against the message already in the state.
//    The other arms fire on state transitions rather than every tick, so their
//    `format!` calls are not on the steady-state path and are left alone.
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
// 6. DONE (partly): the three chained `.update(..)` calls at the end each
//    cloned the whole state map; they are `update_mut` now.
//    The rest of the original note was wrong and the correction matters:
//    `_elapsed_executing_ms` / `_elapsed_disabled_ms` are only *incremented* in
//    the Executing and Disabled arms, so in every other state they are written
//    back unchanged and produce no diff - and an idle system has no active
//    operations to call this with at all. Measured: eight runners idling for
//    five seconds issue zero MSETs. The remaining per-tick write is for an
//    operation that is genuinely running, where the elapsed counter really did
//    change. Deriving it from a stored start time (point 5) would remove that
//    write too, and would fix the tick-constant bug, but it is a timeout
//    semantics change rather than an idle-load one.
/// True when `info` already reads as `{before}{op_name}{after}...`.
///
/// The steady-state arms of `process_operation` build a message that is a pure
/// function of the operation, so it is the *same string* on every tick for as
/// long as the operation stays in that state. `new_op_info` starts out as the
/// value already in the state, so when this returns true there is nothing to
/// rebuild - the message, the logging decision and the write are all unchanged.
///
/// The check itself is allocation-free, which is the point: it has to be much
/// cheaper than the message it avoids building.
fn info_already_reads_as(info: &str, before: &str, op_name: &str, after: &str) -> bool {
    info.strip_prefix(before)
        .and_then(|rest| rest.strip_prefix(op_name))
        .map_or(false, |rest| rest.starts_with(after))
}

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
                op_info_level = log::Level::Warn;

                // DONE: PERF: this is the expensive arm, and it is the one an
                // operation sits in for minutes at a time. Building the message
                // clones every precondition's guard *and* runner guard, wraps
                // them in two `Predicate::OR` trees and renders both through
                // `Display` - a full recursive tree walk with string building,
                // for every disabled operation on every tick. The result is a
                // pure function of the operation, so it was byte-identical
                // every time and thrown away again by the `!=` below.
                // Now it is built once, when the operation first reports as
                // disabled, and skipped for as long as that message stands.
                if !info_already_reads_as(
                    &old_operation_information,
                    "Operation '",
                    &operation.name,
                    "' disabled.",
                ) {
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
                }
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
                op_info_level = log::Level::Info;

                // Same idea as the `Disabled` arm, and the same steady state:
                // an executing operation waits here tick after tick with a
                // message that never changes.
                if !info_already_reads_as(
                    &old_operation_information,
                    "Waiting for operation '",
                    &operation.name,
                    "' to be completed.",
                ) {
                    new_op_info =
                        format!("Waiting for operation '{}' to be completed.", operation.name);
                    logging_log = format!("Executing");
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    const SP_ID: &str = "sp";
    const TARGET: &str = "test";

    #[test]
    fn info_already_reads_as_matches_only_the_right_operation() {
        let msg = "Operation 'op_move' disabled. Please satisfy the runner guard: ...";

        assert!(info_already_reads_as(
            msg,
            "Operation '",
            "op_move",
            "' disabled."
        ));

        // The hazard worth guarding: one operation name being a prefix of
        // another must not make them look like the same message.
        assert!(!info_already_reads_as(
            msg,
            "Operation '",
            "op_mo",
            "' disabled."
        ));
        assert!(!info_already_reads_as(
            "Operation 'op_move_to_b' disabled. ...",
            "Operation '",
            "op_move",
            "' disabled."
        ));

        // A different message for the same operation must not match either.
        assert!(!info_already_reads_as(
            "Disabling operation 'op_move'.",
            "Operation '",
            "op_move",
            "' disabled."
        ));
        assert!(!info_already_reads_as("", "Operation '", "op_move", "' disabled."));
    }

    fn disabled_operation_state() -> (State, Operation) {
        let mut state = State::new();
        state.add_mut(
            SPAssignment::new(SPVariable::new("ready", SPValueType::Bool), false.to_spvalue()),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(SPVariable::new("armed", SPValueType::Bool), false.to_spvalue()),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{}_dashboard_command", SP_ID), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );

        // Two preconditions, so the disabled message renders a non-trivial
        // `Predicate::OR` tree - the thing that used to be rebuilt every tick.
        let operation = Operation::new(
            "stuck",
            None,
            None,
            None,
            None,
            false,
            vec![
                Transition::parse(
                    "start_a",
                    "var:ready == true",
                    "var:armed == true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &state,
                ),
                Transition::parse(
                    "start_b",
                    "var:armed == true",
                    "var:ready == true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &state,
                ),
            ],
            vec![Transition::parse(
                "complete",
                "true",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        let state = add_operation_state_tracking_variable(
            &vec![operation.name.clone()],
            &state,
            TARGET,
        );
        let state = add_operation_meta_tracking_variables(
            &vec![operation.name.clone()],
            &state,
            false,
            TARGET,
        );

        (state, operation)
    }

    async fn tick(state: State, operation: &Operation) -> State {
        process_operation(
            SP_ID,
            state,
            operation,
            OperationProcessingType::Automatic,
            None,
            None,
            TARGET,
        )
        .await
    }

    /// The message must be produced in full the first time, and must be exactly
    /// the same on every tick afterwards - the skip path has to be
    /// indistinguishable from rebuilding it.
    #[tokio::test]
    async fn the_disabled_message_is_built_once_and_then_stays_identical() {
        let (state, operation) = disabled_operation_state();
        let info_key = format!("{}_information", operation.name);

        // Tick 1: Initial -> disabled.
        let state = tick(state, &operation).await;
        assert_eq!(
            state.get_string_or_default_to_unknown(&operation.name, TARGET),
            "disabled"
        );

        // Tick 2: the Disabled arm builds the full message.
        let state = tick(state, &operation).await;
        let first = state.get_string_or_default_to_unknown(&info_key, TARGET);
        assert!(
            first.starts_with(&format!("Operation '{}' disabled.", operation.name)),
            "unexpected disabled message: {first}"
        );
        assert!(
            first.contains("Debug full guard:"),
            "the message should still render both guard trees: {first}"
        );
        assert!(
            first.contains("armed = true") && first.contains("ready = true"),
            "the message should still name the runner guard variables: {first}"
        );

        // Ticks 3..: the skip path must reproduce it byte for byte.
        let mut state = state;
        for _ in 0..5 {
            state = tick(state, &operation).await;
            let again = state.get_string_or_default_to_unknown(&info_key, TARGET);
            assert_eq!(again, first, "the disabled message must not change");
        }
    }

    /// A disabled operation whose guard becomes satisfiable must still leave the
    /// Disabled arm - the skip must not pin the operation to its old message.
    #[tokio::test]
    async fn a_disabled_operation_still_starts_when_its_guard_is_satisfied() {
        let (state, operation) = disabled_operation_state();

        let state = tick(state, &operation).await;
        let mut state = tick(state, &operation).await;
        assert_eq!(
            state.get_string_or_default_to_unknown(&operation.name, TARGET),
            "disabled"
        );

        state.update_mut("ready", true.to_spvalue());
        state.update_mut("armed", true.to_spvalue());

        let state = tick(state, &operation).await;
        assert_eq!(
            state.get_string_or_default_to_unknown(&operation.name, TARGET),
            "executing",
            "the operation should have started once its guards were satisfiable"
        );
        assert!(
            state
                .get_string_or_default_to_unknown(&format!("{}_information", operation.name), TARGET)
                .starts_with("Starting disabled operation"),
            "the information should have been replaced, not skipped"
        );
    }
}

