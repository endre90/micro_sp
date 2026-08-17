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
    // Wall-clock milliseconds the caller's tick actually took. The elapsed
    // counters advance by this, so they track real time no matter which runner
    // is driving the operation or how badly a tick slipped.
    tick_elapsed_ms: i64,
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
            elapased_disabled_ms += tick_elapsed_ms;
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
            elapased_executing_ms += tick_elapsed_ms;
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

        // The file log's `OP` line. `logging_log` is the short tag each arm
        // above already computes ("Starting", "Completing", "Retrying 2/3",
        // ...); it was written and then thrown away for as long as the Redis
        // op-logger was disconnected, which is what the "assigned but never
        // read" warnings were about.
        //
        // Both guards matter. `new_op_info != old_operation_information` is the
        // crate's own "this is news" check, and reusing it is what keeps a
        // terminated operation - which re-enters the same arm on every tick
        // until it is cleaned up - from emitting an identical line several
        // times a second. The empty-tag check drops the arms that changed the
        // message without taking a decision worth recording.
        if !logging_log.is_empty() {
            // Read the resulting state back rather than predicting it: the arms
            // reach it through `start`/`complete`/`fail`/`terminate`/..., and a
            // hand-maintained mapping here would be one more thing to keep in
            // step with them.
            let resulting_state = new_state
                .get_string_or_default_to_unknown(&format!("{}", operation.name), &log_target);
            activity_log::log_operation(
                &log_target,
                &operation.name,
                &operation_state,
                &resulting_state,
                &logging_log,
            );
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
            // a 200 ms tick, matching the runners' cadence
            200,
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


#[cfg(test)]
mod elapsed_tests {
    use super::*;

    const SP_ID: &str = "sp";
    const TARGET: &str = "test";

    /// An operation that starts and then sits in Executing forever.
    fn executing_operation() -> (State, Operation) {
        let mut state = State::new();
        state.add_mut(
            SPAssignment::new(SPVariable::new("go", SPValueType::Bool), true.to_spvalue()),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{}_dashboard_command", SP_ID), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );

        let operation = Operation::new(
            "slow",
            Some(1000),
            Some(1000),
            None,
            None,
            false,
            vec![Transition::parse(
                "start",
                "var:go == true",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &state,
            )],
            // Never satisfiable, so it stays Executing.
            vec![Transition::parse(
                "complete",
                "var:go == false",
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

        let state =
            add_operation_state_tracking_variable(&vec![operation.name.clone()], &state, TARGET);
        let state = add_operation_meta_tracking_variables(
            &vec![operation.name.clone()],
            &state,
            false,
            TARGET,
        );
        (state, operation)
    }

    async fn tick(state: State, operation: &Operation, tick_elapsed_ms: i64) -> State {
        process_operation(
            SP_ID,
            state,
            operation,
            OperationProcessingType::Automatic,
            None,
            None,
            tick_elapsed_ms,
            TARGET,
        )
        .await
    }

    /// The bug: the increment was a compile-time constant of 200 ms, so an
    /// operation driven by `sop_runner` at 100 ms aged twice as fast as real
    /// time and hit its deadline at half the configured value.
    #[tokio::test]
    async fn elapsed_tracks_the_callers_real_tick_period() {
        let (state, operation) = executing_operation();
        let key = format!("{}_elapsed_executing_ms", operation.name);

        // Tick 1 starts it; from then on it accumulates.
        let mut state = tick(state, &operation, 100).await;
        assert_eq!(
            state.get_string_or_default_to_unknown(&operation.name, TARGET),
            "executing"
        );

        for _ in 0..10 {
            state = tick(state, &operation, 100).await;
        }
        assert_eq!(
            state.get_value(&key, TARGET),
            Some(1000.to_spvalue()),
            "ten 100 ms ticks must count as 1000 ms, not 2000"
        );
    }

    /// A slipped tick counts for the time it actually took.
    #[tokio::test]
    async fn a_slow_tick_counts_the_time_it_really_took() {
        let (state, operation) = executing_operation();
        let key = format!("{}_elapsed_executing_ms", operation.name);

        let state = tick(state, &operation, 200).await;
        let state = tick(state, &operation, 200).await;
        let state = tick(state, &operation, 950).await;

        assert_eq!(
            state.get_value(&key, TARGET),
            Some(1150.to_spvalue()),
            "a 950 ms tick must count as 950 ms"
        );
    }

    /// And the timeout that reads the counter fires after the configured
    /// deadline in *real* time. Before the fix, a 1000 ms deadline driven at
    /// 100 ms per tick fired after ~600 ms of real time, because each tick
    /// charged the 200 ms constant.
    ///
    /// Note the counter is read at the start of a tick and incremented at the
    /// end, so the timeout is observed a tick or two after the deadline is
    /// actually crossed. That lag is pre-existing and not what this pins down -
    /// what matters is that it can no longer fire *early*.
    #[tokio::test]
    async fn the_deadline_is_reached_in_real_time_not_early() {
        let (state, operation) = executing_operation();

        // 1000 ms deadline, driven at 100 ms per tick.
        let mut state = tick(state, &operation, 100).await;

        let mut real_ms_elapsed = 0;
        let mut timed_out_after = None;
        for _ in 0..40 {
            state = tick(state, &operation, 100).await;
            real_ms_elapsed += 100;
            if state.get_string_or_default_to_unknown(&operation.name, TARGET) == "timedout" {
                timed_out_after = Some(real_ms_elapsed);
                break;
            }
        }

        let timed_out_after = timed_out_after.expect("the operation should have timed out");
        assert!(
            timed_out_after >= 1000,
            "timed out after only {timed_out_after} ms of real time, but the deadline is 1000 ms"
        );
        assert!(
            timed_out_after <= 1300,
            "timed out after {timed_out_after} ms, which is more lag than the read-then-increment \
             ordering explains"
        );
    }
}

/// The operation state machine, arm by arm.
///
/// `process_operation` is the single place where an operation's lifecycle is
/// decided, and all three operation runners funnel through it: the plan runner
/// with `Planned`, the SOP runner with `SOP`, the auto runner with `Automatic`.
/// Every arm is reachable from a real deployment, several of them only on a bad
/// day (a timeout, a failure with retries exhausted, an operator pressing stop),
/// and those are exactly the ones nobody exercises by hand.
///
/// The tests below walk each arm of the `match` and pin: which state the
/// operation lands in, which bookkeeping variable is written, and - for the
/// `Planned` type - what happens to the plan cursor and the plan state, since
/// those are the values the plan runner writes back to Redis for the goal
/// runner to read.
#[cfg(test)]
mod state_machine_tests {
    use super::*;

    const SP_ID: &str = "sp";
    const TARGET: &str = "test";
    const OP: &str = "op_test";

    /// A world with three switches in it: `go` gates the precondition, `done`
    /// the postcondition, `broken` the failure transition. Plus the dashboard
    /// command the cancel guard reads.
    fn world() -> State {
        let mut state = State::new();
        for name in ["go", "done", "broken"] {
            state.add_mut(
                SPAssignment::new(SPVariable::new(name, SPValueType::Bool), false.to_spvalue()),
                TARGET,
            );
        }
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{}_dashboard_command", SP_ID), SPValueType::String),
                "none".to_spvalue(),
            ),
            TARGET,
        );
        state
    }

    fn transition(name: &str, guard: &str, actions: Vec<&str>, state: &State) -> Transition {
        Transition::parse(name, guard, "true", actions, Vec::<&str>::new(), state)
    }

    /// The operation under test: starts on `go`, completes on `done`, fails on
    /// `broken`. Timeouts and retries are configurable per test.
    fn operation(
        timeout_executing_ms: Option<i64>,
        timeout_disabled_ms: Option<i64>,
        failure_retries: Option<i64>,
        timeout_retries: Option<i64>,
        can_be_bypassed: bool,
    ) -> (State, Operation) {
        let state = world();
        let operation = Operation::new(
            OP,
            timeout_executing_ms,
            timeout_disabled_ms,
            failure_retries,
            timeout_retries,
            can_be_bypassed,
            vec![transition(
                "start",
                "var:go == true",
                vec!["var:go <- false"],
                &state,
            )],
            vec![transition("complete", "var:done == true", vec![], &state)],
            vec![transition("fail", "var:broken == true", vec![], &state)],
            vec![],
            vec![],
            vec![],
        );

        let state = add_operation_state_tracking_variable(&vec![OP.to_string()], &state, TARGET);
        let state =
            add_operation_meta_tracking_variables(&vec![OP.to_string()], &state, false, TARGET);
        (state, operation)
    }

    /// A plain operation with no timeouts worth reaching and no retries.
    fn plain() -> (State, Operation) {
        operation(Some(10_000), Some(10_000), None, None, false)
    }

    async fn tick(state: State, operation: &Operation, elapsed_ms: i64) -> State {
        process_operation(
            SP_ID,
            state,
            operation,
            OperationProcessingType::Automatic,
            None,
            None,
            elapsed_ms,
            TARGET,
        )
        .await
    }

    async fn tick_planned(
        state: State,
        operation: &Operation,
        step: &mut i64,
        plan_state: &mut String,
    ) -> State {
        process_operation(
            SP_ID,
            state,
            operation,
            OperationProcessingType::Planned,
            Some(step),
            Some(plan_state),
            10,
            TARGET,
        )
        .await
    }

    fn op_state(state: &State) -> String {
        state.get_string_or_default_to_unknown(OP, TARGET)
    }

    fn info(state: &State) -> String {
        state.get_string_or_default_to_unknown(&format!("{OP}_information"), TARGET)
    }

    fn counter(state: &State, suffix: &str) -> i64 {
        state.get_int_or_default_to_zero(&format!("{OP}_{suffix}"), TARGET)
    }

    fn set(state: &State, key: &str, value: SPValue) -> State {
        state.update(key, value)
    }

    fn stop_pressed(state: &State) -> State {
        set(
            state,
            &format!("{}_dashboard_command", SP_ID),
            "stop".to_spvalue(),
        )
    }

    /// Drive the operation into a given state, so each arm's test can start
    /// from it directly.
    fn in_state(state: &State, operation_state: &str) -> State {
        set(state, OP, operation_state.to_spvalue())
    }

    // ---------------------------------------------------------------- UNKNOWN

    /// An operation whose tracking variable holds something nobody recognises -
    /// a key that was never initialised, or one left over from an older build -
    /// is put back to `initial` rather than being acted on.
    #[tokio::test]
    async fn an_unknown_operation_is_initialised() {
        let (state, operation) = plain();
        let state = in_state(&state, "nonsense");

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "initial");
    }

    // ---------------------------------------------------------------- Initial

    #[tokio::test]
    async fn an_initial_operation_whose_guard_holds_starts() {
        let (state, operation) = plain();
        let state = set(&state, "go", true.to_spvalue());

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "executing");
        assert_eq!(info(&state), format!("Starting initialized operation '{OP}'."));
        assert_eq!(
            state.get_value("go", TARGET),
            Some(false.to_spvalue()),
            "starting must also take the precondition's actions"
        );
    }

    #[tokio::test]
    async fn an_initial_operation_whose_guard_does_not_hold_is_disabled() {
        let (state, operation) = plain();

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "disabled");
        assert_eq!(info(&state), format!("Disabling operation '{OP}'."));
    }

    #[tokio::test]
    async fn stop_cancels_an_initial_operation_before_it_starts() {
        let (state, operation) = plain();
        let state = set(&state, "go", true.to_spvalue());
        let state = stop_pressed(&state);

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "cancelled");
        assert_eq!(
            state.get_value("go", TARGET),
            Some(true.to_spvalue()),
            "cancelling must not run the precondition's actions"
        );
    }

    // --------------------------------------------------------------- Disabled

    /// The steady state: a disabled operation builds its "please satisfy the
    /// runner guard" message once and then leaves it alone. This is the
    /// allocation-free skip that the `info_already_reads_as` check exists for,
    /// and the property it has to preserve is that the message stops changing.
    #[tokio::test]
    async fn a_disabled_operation_settles_on_one_message() {
        let (state, operation) = plain();

        let state = tick(state, &operation, 10).await; // -> disabled
        let state = tick(state, &operation, 10).await; // builds the long message
        let settled = info(&state);
        assert!(
            settled.starts_with(&format!("Operation '{OP}' disabled.")),
            "unexpected message: {settled}"
        );

        for _ in 0..5 {
            let next = tick(state.clone(), &operation, 10).await;
            assert_eq!(info(&next), settled, "the message must stop changing");
        }
    }

    /// Time spent disabled accumulates in its own counter, separately from
    /// executing time.
    #[tokio::test]
    async fn disabled_time_accumulates_in_its_own_counter() {
        let (state, operation) = plain();

        let mut state = tick(state, &operation, 10).await; // -> disabled
        assert_eq!(counter(&state, "elapsed_disabled_ms"), 0);

        for _ in 0..4 {
            state = tick(state, &operation, 25).await;
        }

        assert_eq!(counter(&state, "elapsed_disabled_ms"), 100);
        assert_eq!(
            counter(&state, "elapsed_executing_ms"),
            0,
            "a disabled operation must not age its executing counter"
        );
    }

    #[tokio::test]
    async fn a_disabled_operation_starts_once_its_guard_holds() {
        let (state, operation) = plain();

        let state = tick(state, &operation, 10).await; // -> disabled
        assert_eq!(op_state(&state), "disabled");

        let state = set(&state, "go", true.to_spvalue());
        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "executing");
        assert_eq!(info(&state), format!("Starting disabled operation '{OP}'."));
    }

    /// An operation nobody ever enables must not sit disabled forever - the
    /// disabled timeout is what turns "the guard never became true" into a
    /// visible failure.
    #[tokio::test]
    async fn a_disabled_operation_times_out_on_its_own_deadline() {
        let (state, operation) = operation(Some(10_000), Some(50), None, None, false);

        let mut state = tick(state, &operation, 10).await; // -> disabled
        for _ in 0..10 {
            state = tick(state, &operation, 20).await;
            if op_state(&state) == "timedout" {
                break;
            }
        }

        assert_eq!(op_state(&state), "timedout");
        assert_eq!(
            info(&state),
            format!("Timeout for disabled operation '{OP}'.")
        );
    }

    #[tokio::test]
    async fn stop_cancels_a_disabled_operation() {
        let (state, operation) = plain();
        let state = tick(state, &operation, 10).await; // -> disabled
        let state = stop_pressed(&state);

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "cancelled");
    }

    // -------------------------------------------------------------- Executing

    #[tokio::test]
    async fn an_executing_operation_completes_when_its_postcondition_holds() {
        let (state, operation) = plain();
        let state = in_state(&state, "executing");
        let state = set(&state, "done", true.to_spvalue());

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "completed");
        assert_eq!(info(&state), format!("Completing operation '{OP}'."));
    }

    /// The waiting message is the other steady state built once and then
    /// skipped.
    #[tokio::test]
    async fn an_executing_operation_settles_on_one_waiting_message() {
        let (state, operation) = plain();
        let state = in_state(&state, "executing");

        let state = tick(state, &operation, 10).await;
        assert_eq!(
            info(&state),
            format!("Waiting for operation '{OP}' to be completed.")
        );

        let settled = info(&state);
        for _ in 0..5 {
            let next = tick(state.clone(), &operation, 10).await;
            assert_eq!(info(&next), settled);
        }
    }

    #[tokio::test]
    async fn an_executing_operation_fails_when_its_failure_transition_fires() {
        let (state, operation) = plain();
        let state = in_state(&state, "executing");
        let state = set(&state, "broken", true.to_spvalue());

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "failed");
        assert_eq!(info(&state), format!("Failing operation '{OP}'."));
    }

    /// Failing beats completing when both guards hold at once - worth pinning
    /// because it is the arm ordering inside the `Executing` match, not
    /// anything the model expresses.
    #[tokio::test]
    async fn failing_takes_precedence_over_completing() {
        let (state, operation) = plain();
        let state = in_state(&state, "executing");
        let state = set(&state, "done", true.to_spvalue());
        let state = set(&state, "broken", true.to_spvalue());

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "failed");
    }

    /// And cancelling beats everything.
    #[tokio::test]
    async fn stop_takes_precedence_over_every_other_executing_outcome() {
        let (state, operation) = plain();
        let state = in_state(&state, "executing");
        let state = set(&state, "done", true.to_spvalue());
        let state = set(&state, "broken", true.to_spvalue());
        let state = stop_pressed(&state);

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "cancelled");
    }

    // -------------------------------------------------------------- Completed

    /// Completing resets both retry counters, so an operation that failed twice
    /// and then succeeded starts its next run with a full budget rather than
    /// with two retries already spent.
    #[tokio::test]
    async fn completing_resets_the_retry_counters_and_terminates() {
        let (state, operation) = plain();
        let state = in_state(&state, "completed");
        let state = set(
            &state,
            &format!("{OP}_failure_retry_counter"),
            2.to_spvalue(),
        );
        let state = set(
            &state,
            &format!("{OP}_timeout_retry_counter"),
            1.to_spvalue(),
        );

        let state = tick(state, &operation, 10).await;

        assert_eq!(counter(&state, "failure_retry_counter"), 0);
        assert_eq!(counter(&state, "timeout_retry_counter"), 0);
        assert_eq!(
            op_state(&state),
            "terminated_completed",
            "a completed operation is terminated so the runner stops driving it"
        );
    }

    /// The plan cursor only advances for a `Planned` operation - a SOP or auto
    /// operation completing must not move somebody else's plan along.
    #[tokio::test]
    async fn completing_advances_the_plan_cursor_only_when_planned() {
        let (state, operation) = plain();
        let completed = in_state(&state, "completed");

        let mut step = 3;
        let mut plan_state = PlanState::Executing.to_string();
        let _ = tick_planned(completed.clone(), &operation, &mut step, &mut plan_state).await;
        assert_eq!(step, 4, "a planned operation completing advances the plan");

        let mut step = 3;
        let _ = process_operation(
            SP_ID,
            completed,
            &operation,
            OperationProcessingType::SOP,
            Some(&mut step),
            None,
            10,
            TARGET,
        )
        .await;
        assert_eq!(step, 3, "a SOP operation must not touch the plan cursor");
    }

    // --------------------------------------------------------------- Bypassed

    /// BUG: `Operation::terminate` only implements
    /// `TerminationReason::Completed` - its `_ => state.clone()` arm makes
    /// `Bypassed`, `Fatal` and `Cancelled` silent no-ops. So the
    /// `terminate(.., Bypassed)` call at the end of this arm does nothing and
    /// the operation stays in `bypassed` rather than reaching
    /// `terminated_bypassed`.
    ///
    /// Consequence: `SOP::get_state` maps `Bypassed` to `SOPState::Executing`
    /// and only `Terminated(Bypassed)` to `SOPState::Completed`, so a bypassed
    /// operation inside a SOP leaves that branch reporting `Executing` forever
    /// and the SOP never completes. See
    /// `sop_runner`'s `a_bypassed_operation_never_lets_its_sop_finish`.
    ///
    /// The plan runner is not affected the same way: it advances the cursor and
    /// moves on to the next step, so the stuck operation is simply never looked
    /// at again.
    #[tokio::test]
    async fn a_bypassed_operation_advances_the_plan_but_never_terminates() {
        let (state, operation) = plain();
        let state = in_state(&state, "bypassed");

        let mut step = 0;
        let mut plan_state = PlanState::Executing.to_string();
        let state = tick_planned(state, &operation, &mut step, &mut plan_state).await;

        assert_eq!(step, 1, "the plan moves on");
        assert!(info(&state).contains("bypassed"));
        assert_eq!(
            op_state(&state),
            "bypassed",
            "if this now reads terminated_bypassed the terminate() bug is fixed"
        );
    }

    /// Dead code, worth pinning so it is not mistaken for working: the
    /// `Bypassed` arm opens with a `can_be_cancelled` check, but
    /// `can_be_cancelled` only lists Initial / Executing / Disabled / Failed /
    /// Timedout - never Bypassed - so that branch can never be taken and
    /// pressing stop on a bypassed operation still advances the plan.
    #[tokio::test]
    async fn stop_cannot_cancel_a_bypassed_operation() {
        let (state, operation) = plain();
        let state = in_state(&state, "bypassed");
        let state = stop_pressed(&state);

        assert!(
            !operation.can_be_cancelled(SP_ID, &state, TARGET),
            "bypassed is not a cancellable state"
        );

        let mut step = 0;
        let mut plan_state = PlanState::Executing.to_string();
        let state = tick_planned(state, &operation, &mut step, &mut plan_state).await;

        assert_eq!(step, 1, "the cancel branch is unreachable, so the plan advances");
        assert!(info(&state).contains("bypassed"));
    }

    // --------------------------------------------------------------- Timedout

    /// With retries left, a timeout is retried and the counter goes up.
    #[tokio::test]
    async fn a_timedout_operation_retries_while_it_has_retries_left() {
        let (state, operation) = operation(Some(10_000), Some(10_000), None, Some(2), false);
        let state = in_state(&state, "timedout");

        let state = tick(state, &operation, 10).await;
        assert_eq!(counter(&state, "timeout_retry_counter"), 1);
        assert_eq!(
            info(&state),
            format!("Retrying operation (timeout) '{OP}'. Retry 1 out of 2.")
        );
        assert_eq!(op_state(&state), "initial", "a retry puts it back to initial");

        let state = in_state(&state, "timedout");
        let state = tick(state, &operation, 10).await;
        assert_eq!(counter(&state, "timeout_retry_counter"), 2);

        // Third time: the budget is spent, and with no bypass it is fatal.
        let state = in_state(&state, "timedout");
        let state = tick(state, &operation, 10).await;
        assert_eq!(op_state(&state), "fatal");
    }

    /// An operation marked `can_be_bypassed` skips instead of killing the plan.
    #[tokio::test]
    async fn a_timedout_operation_with_no_retries_is_bypassed_when_allowed() {
        let (state, operation) = operation(Some(10_000), Some(10_000), None, None, true);
        let state = in_state(&state, "timedout");

        let state = tick(state, &operation, 10).await;

        assert_eq!(info(&state), format!("Operation '{OP}' timedout. Bypassing."));
        assert_eq!(op_state(&state), "bypassed");
    }

    #[tokio::test]
    async fn a_timedout_operation_with_no_retries_and_no_bypass_is_fatal() {
        let (state, operation) = plain();
        let state = in_state(&state, "timedout");

        let state = tick(state, &operation, 10).await;

        assert_eq!(info(&state), format!("Operation '{OP}' timedout."));
        assert_eq!(op_state(&state), "fatal");
    }

    #[tokio::test]
    async fn stop_cancels_a_timedout_operation() {
        let (state, operation) = operation(Some(10_000), Some(10_000), None, Some(2), false);
        let state = in_state(&state, "timedout");
        let state = stop_pressed(&state);

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "cancelled");
        assert_eq!(
            counter(&state, "timeout_retry_counter"),
            0,
            "cancelling must not spend a retry"
        );
    }

    // ----------------------------------------------------------------- Failed

    #[tokio::test]
    async fn a_failed_operation_retries_while_it_has_retries_left() {
        let (state, operation) = operation(Some(10_000), Some(10_000), Some(2), None, false);
        let state = in_state(&state, "failed");

        let state = tick(state, &operation, 10).await;
        assert_eq!(counter(&state, "failure_retry_counter"), 1);
        assert_eq!(
            info(&state),
            format!("Retrying operation (failure) '{OP}'. Retry 1 out of 2.")
        );

        let state = in_state(&state, "failed");
        let state = tick(state, &operation, 10).await;
        assert_eq!(counter(&state, "failure_retry_counter"), 2);
        assert_eq!(op_state(&state), "initial");
    }

    /// Running out of failure retries clears *both* counters on the way out, so
    /// the operation's next run is not started with a spent budget.
    #[tokio::test]
    async fn a_failed_operation_out_of_retries_is_fatal_and_clears_the_counters() {
        let (state, operation) = operation(Some(10_000), Some(10_000), Some(1), None, false);
        let state = in_state(&state, "failed");
        let state = set(
            &state,
            &format!("{OP}_failure_retry_counter"),
            1.to_spvalue(),
        );
        let state = set(
            &state,
            &format!("{OP}_timeout_retry_counter"),
            1.to_spvalue(),
        );

        let state = tick(state, &operation, 10).await;

        assert_eq!(
            info(&state),
            format!("Operation '{OP}' has no more retries left.")
        );
        assert_eq!(op_state(&state), "fatal");
        assert_eq!(counter(&state, "failure_retry_counter"), 0);
        assert_eq!(counter(&state, "timeout_retry_counter"), 0);
    }

    #[tokio::test]
    async fn a_failed_operation_out_of_retries_is_bypassed_when_allowed() {
        let (state, operation) = operation(Some(10_000), Some(10_000), None, None, true);
        let state = in_state(&state, "failed");

        let state = tick(state, &operation, 10).await;

        assert_eq!(
            info(&state),
            format!("Operation '{OP}' has no more retries left. Bypassing.")
        );
        assert_eq!(op_state(&state), "bypassed");
    }

    #[tokio::test]
    async fn stop_cancels_a_failed_operation() {
        let (state, operation) = operation(Some(10_000), Some(10_000), Some(2), None, false);
        let state = in_state(&state, "failed");
        let state = stop_pressed(&state);

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "cancelled");
        assert_eq!(counter(&state, "failure_retry_counter"), 0);
    }

    // ------------------------------------------------------------------ Fatal

    /// A fatal operation fails the *plan*, which is how a dead operation
    /// reaches the goal runner. Only for `Planned` - a fatal SOP operation is
    /// the SOP's business, not the plan's.
    #[tokio::test]
    async fn a_fatal_planned_operation_fails_the_plan() {
        let (state, operation) = plain();
        let state = in_state(&state, "fatal");

        let mut step = 0;
        let mut plan_state = PlanState::Executing.to_string();
        let state = tick_planned(state, &operation, &mut step, &mut plan_state).await;

        assert_eq!(plan_state, PlanState::Failed.to_string());
        assert_eq!(
            info(&state),
            format!("Operation '{OP}' unrecoverable. Stopping execution.")
        );
        // Same `terminate` no-op as the bypass arm: it stays `fatal` rather
        // than reaching `terminated_fatal`.
        assert_eq!(op_state(&state), "fatal");
    }

    /// The `Fatal` and `Cancelled` arms are re-entered on every tick for as
    /// long as the runner keeps the operation in its active set, because
    /// `terminate` never moves it out of those states. That is idempotent here
    /// (the message and the plan state are the same every time, so the diff is
    /// empty and nothing is written), which is the reason it has gone
    /// unnoticed - but it does mean an operation that died stays in the runner's
    /// key set and its guards keep being evaluated.
    #[tokio::test]
    async fn a_fatal_operation_stays_fatal_and_settles() {
        let (state, operation) = plain();
        let state = in_state(&state, "fatal");

        let once = tick(state, &operation, 10).await;
        assert_eq!(op_state(&once), "fatal");

        let twice = tick(once.clone(), &operation, 10).await;
        assert_eq!(
            once.get_diff_partial_state(&twice).state.len(),
            0,
            "re-entering the Fatal arm must at least not write anything new"
        );
    }

    // -------------------------------------------------------------- Cancelled

    #[tokio::test]
    async fn a_cancelled_planned_operation_cancels_the_plan() {
        let (state, operation) = plain();
        let state = in_state(&state, "cancelled");

        let mut step = 0;
        let mut plan_state = PlanState::Executing.to_string();
        let state = tick_planned(state, &operation, &mut step, &mut plan_state).await;

        assert_eq!(plan_state, PlanState::Cancelled.to_string());
        assert_eq!(
            info(&state),
            format!("Operation '{OP}' cancelled. Stopping execution.")
        );
        assert_eq!(op_state(&state), "cancelled", "same terminate() no-op");

        // And note where that `plan_state` ends up: the plan runner writes it
        // to `{sp_id}_plan_state` as the string "cancelled", which
        // `PlanState::from_str` cannot parse back - see
        // `running::runner_states::tests::plan_state_cancelled_does_not_survive_the_round_trip`.
        assert_eq!(PlanState::from_str(&plan_state), PlanState::UNKNOWN);
    }

    // ------------------------------------------------------------- Terminated

    /// A terminated operation is inert: whatever the reason, further ticks
    /// leave it exactly where it is. The runners keep calling this for as long
    /// as the operation is in their active set, so "does nothing" has to be
    /// literally true, or a finished operation would keep writing on every tick.
    #[tokio::test]
    async fn a_terminated_operation_is_inert() {
        for reason in [
            "terminated_completed",
            "terminated_bypassed",
            "terminated_fatal",
            "terminated_cancelled",
        ] {
            let (state, operation) = plain();
            let state = in_state(&state, reason);

            let once = tick(state, &operation, 10).await;
            assert_eq!(op_state(&once), reason, "{reason} must stay put");
            assert_eq!(info(&once), format!("Operation '{OP}' terminated."));

            let twice = tick(once.clone(), &operation, 10).await;
            assert_eq!(
                once.get_diff_partial_state(&twice).state.len(),
                0,
                "a second tick on a {reason} operation must write nothing"
            );
        }
    }

    /// Even stop does not move a terminated operation - the cancel guard does
    /// not list the terminated states, so pressing stop after everything has
    /// finished is a no-op rather than a mass re-cancel. This is the bug the
    /// `can_be_cancelled` fix was about, from the other side.
    #[tokio::test]
    async fn stop_does_not_disturb_a_terminated_operation() {
        let (state, operation) = plain();
        let state = in_state(&state, "terminated_completed");
        let state = stop_pressed(&state);

        let state = tick(state, &operation, 10).await;

        assert_eq!(op_state(&state), "terminated_completed");
    }

    // ------------------------------------------------------------ whole cycle

    /// The happy path end to end, as the auto runner drives it: initial ->
    /// executing -> completed -> terminated, one tick per step.
    #[tokio::test]
    async fn the_happy_path_runs_start_to_terminated() {
        let (state, operation) = plain();

        let state = set(&state, "go", true.to_spvalue());
        let state = tick(state, &operation, 10).await;
        assert_eq!(op_state(&state), "executing");

        let state = set(&state, "done", true.to_spvalue());
        let state = tick(state, &operation, 10).await;
        assert_eq!(op_state(&state), "completed");

        let state = tick(state, &operation, 10).await;
        assert_eq!(op_state(&state), "terminated_completed");

        // And it stays there.
        let state = tick(state, &operation, 10).await;
        assert_eq!(op_state(&state), "terminated_completed");
    }

    /// The unhappy path, end to end: it starts, never completes, times out,
    /// spends its one retry, times out again, and dies. This is the sequence a
    /// runner actually walks through - the individual arms above are each one
    /// step of it.
    #[tokio::test]
    async fn the_timeout_path_retries_once_and_then_dies() {
        let (state, operation) = operation(Some(50), Some(10_000), None, Some(1), false);

        let mut state = set(&state, "go", true.to_spvalue());
        let mut seen: Vec<String> = vec![];
        for _ in 0..40 {
            state = tick(state, &operation, 20).await;
            let current = op_state(&state);
            if seen.last().map(|s| s.as_str()) != Some(current.as_str()) {
                seen.push(current.clone());
            }
            if current == "fatal" {
                break;
            }
            // Re-arm the precondition so a retry can start again.
            if current == "initial" {
                state = set(&state, "go", true.to_spvalue());
            }
        }

        assert_eq!(
            seen,
            vec!["executing", "timedout", "initial", "executing", "timedout", "fatal"],
            "the whole timeout-retry-timeout-die sequence"
        );
        assert_eq!(counter(&state, "timeout_retry_counter"), 1);
    }
}
