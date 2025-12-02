use serde::{Deserialize, Serialize};
use std::fmt;

use crate::*;

/// Initial:   The operation planned and ready to be executed.
/// Blocked:   Can't move to executing stet because the precondition guard is false.
/// Executing: The precondition guard is enabled and the actions of the precondition are taken.
/// Completed: The postcondition guard is enabled and the actions of the postcondition are taken.
///            The operation is successfully completed.
/// Timedout:  The operation was in the executing state for more time than its deadline allows.
/// Failed:    The operations has failed due to an error.
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum OperationState {
    Initial,
    Disabled,
    Executing,
    Completed,
    Bypassed,
    Timedout,
    Failed,
    Fatal,
    Cancelled,
    // Paused,
    UNKNOWN,
}

impl Default for OperationState {
    fn default() -> Self {
        OperationState::UNKNOWN
    }
}

impl OperationState {
    pub fn from_str(x: &str) -> OperationState {
        match x {
            "initial" => OperationState::Initial,
            "disabled" => OperationState::Disabled,
            "executing" => OperationState::Executing,
            "timedout" => OperationState::Timedout,
            "failed" => OperationState::Failed,
            "fatal" => OperationState::Fatal,
            "completed" => OperationState::Completed,
            "bypassed" => OperationState::Bypassed,
            "cancelled" => OperationState::Cancelled,
            _ => OperationState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        self.to_string().to_spvalue()
    }
}

impl fmt::Display for OperationState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OperationState::Initial => write!(f, "initial"),
            OperationState::Disabled => write!(f, "disabled"),
            OperationState::Executing => write!(f, "executing"),
            OperationState::Timedout => write!(f, "timedout"),
            OperationState::Failed => write!(f, "failed"),
            OperationState::Fatal => write!(f, "fatal"),
            OperationState::Completed => write!(f, "completed"),
            OperationState::Bypassed => write!(f, "bypassed"),
            OperationState::Cancelled => write!(f, "cancelled"),
            OperationState::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub state: OperationState,
    pub timeout_executing_ms: Option<i64>,
    pub timeout_disabled_ms: Option<i64>,
    pub failure_retries: i64,
    pub timeout_retries: i64,
    pub can_be_bypassed: bool,
    pub preconditions: Vec<Transition>,
    pub postconditions: Vec<Transition>,
    pub failure_transitions: Vec<Transition>,
    pub bypass_transitions: Vec<Transition>,
    pub timeout_transitions: Vec<Transition>,
    pub cancel_transitions: Vec<Transition>,
}

impl Default for Operation {
    fn default() -> Self {
        Operation {
            name: "unknown".to_string(),
            state: OperationState::UNKNOWN,
            timeout_executing_ms: None,
            timeout_disabled_ms: None,
            failure_retries: 0,
            timeout_retries: 0,
            can_be_bypassed: false,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            failure_transitions: Vec::new(),
            timeout_transitions: Vec::new(),
            bypass_transitions: Vec::new(),
            cancel_transitions: Vec::new(),
        }
    }
}

impl Operation {
    pub fn new(
        name: &str,
        timeout_executing_ms: Option<i64>,
        timeout_disabled_ms: Option<i64>,
        fail_retries: Option<i64>,
        timeout_retries: Option<i64>,
        can_be_bypassed: bool,
        preconditions: Vec<Transition>,
        postconditions: Vec<Transition>,
        failure_transitions: Vec<Transition>,
        timeout_transitions: Vec<Transition>,
        bypass_transitions: Vec<Transition>,
        cancel_transitions: Vec<Transition>,
    ) -> Operation {
        Operation {
            name: name.to_string(),
            state: OperationState::UNKNOWN,
            timeout_executing_ms: match timeout_executing_ms {
                None => Some(MAX_ALLOWED_OPERATION_DURATION_MS),
                Some(x) => Some(x),
            },
            timeout_disabled_ms: match timeout_disabled_ms {
                None => Some(MAX_ALLOWED_OPERATION_DURATION_MS),
                Some(x) => Some(x),
            },
            timeout_transitions,
            failure_retries: match fail_retries {
                Some(x) => x,
                None => 0,
            },
            timeout_retries: match timeout_retries {
                Some(x) => x,
                None => 0,
            },
            can_be_bypassed,
            preconditions,
            postconditions,
            failure_transitions,
            bypass_transitions,
            cancel_transitions,
        }
    }

    /// Check the guard of the planning precondidion transition.
    pub fn eval_planning(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Initial.to_spvalue() {
                for precondition in &self.preconditions {
                    if precondition.clone().eval_planning(state, &log_target) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Execute the planing actions of both the pre and post conditions.
    /// Inex 0 taken as to indicate that the firstly defined transition should be taken when planning.
    pub fn take_planning(&self, state: &State, log_target: &str) -> State {
        self.postconditions[0].clone().take_planning(
            &self.preconditions[0]
                .clone()
                .take_planning(state, &log_target),
            &log_target,
        )
    }

    pub fn eval(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Initial.to_spvalue()
                || value == OperationState::Disabled.to_spvalue()
            {
                for precondition in &self.preconditions {
                    if precondition.clone().eval(state, &log_target) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check the guard and return a tuple: (is_enabled, index_of_enabled_transition)
    pub fn evaluate_with_transition_index(&self, state: &State, log_target: &str) -> (bool, usize) {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Initial.to_spvalue() {
                for (index, precondition) in self.preconditions.iter().enumerate() {
                    if precondition.clone().eval(state, &log_target) {
                        return (true, index);
                    }
                }
            }
        }
        (false, 0)
    }

    /// Check the running postondition guard.
    pub fn can_be_completed_with_transition_index(
        &self,
        state: &State,
        log_target: &str,
    ) -> (bool, usize) {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Executing.to_spvalue() {
                for (index, postcondition) in self.postconditions.iter().enumerate() {
                    if postcondition.clone().eval(state, &log_target) {
                        return (true, index);
                    }
                }
            }
        }
        (false, 0)
    }

    /// Check the running postondition guard.
    pub fn can_be_completed(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Executing.to_spvalue() {
                for postcondition in &self.postconditions {
                    if postcondition.clone().eval(&state, &log_target) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check the running fail_transition guard.
    pub fn can_be_failed(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Executing.to_spvalue() {
                for fail_transition in &self.failure_transitions {
                    if fail_transition.clone().eval(&state, &log_target) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn can_be_timedout(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Executing.to_spvalue() {
                if let Some(timeout_executing_ms) = self.timeout_executing_ms {
                    let elapased_ms = state.get_int_or_default_to_zero(
                        &format!("{}_elapsed_executing_ms", &self.name),
                        &log_target,
                    );
                    if elapased_ms > timeout_executing_ms {
                        return true;
                    }
                }
            }
            if value == OperationState::Disabled.to_spvalue() {
                if let Some(timeout_disabled_ms) = self.timeout_disabled_ms {
                    let elapased_ms = state.get_int_or_default_to_zero(
                        &format!("{}_elapsed_disabled_ms", &self.name),
                        &log_target,
                    );
                    if elapased_ms > timeout_disabled_ms {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check the running reset_transition guard.
    // pub fn can_be_reset(&self, state: &State, log_target: &str) -> bool {
    //     if let Some(value) = state.get_value(&self.name, &log_target) {
    //         if value == OperationState::Completed.to_spvalue() {
    //             for reset_transition in &self.reset_transitions {
    //                 if reset_transition.clone().eval(&state, &log_target) {
    //                     return true;
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }

    /// Check if we can stop the execution and cancel the operations
    pub fn can_be_cancelled(&self, sp_id: &str, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value == OperationState::Initial.to_spvalue()
                || value == OperationState::Executing.to_spvalue()
                || value != OperationState::Disabled.to_spvalue()
                || value != OperationState::Failed.to_spvalue()
                || value != OperationState::Timedout.to_spvalue()
            {
                if let Some(dashboard_command) =
                    state.get_value(&format!("{}_dashboard_command", sp_id), &log_target)
                {
                    if let SPValue::String(StringOrUnknown::String(db)) = dashboard_command {
                        match db.as_str() {
                            "stop" => return true,
                            _ => (),
                        }
                    }
                }
            }
        }
        false
    }

    /// Start executing the operation. Check for eval_running() first.
    pub fn disable(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Initial.to_spvalue() {
            let action = Action::new(assignment.var, OperationState::Disabled.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            log::error!(target: &log_target, "Can't block an operation which is not in its initial state.");
            state.clone()
        }
    }

    pub fn cancel(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        let action = Action::new(
            assignment.var,
            OperationState::Cancelled.to_spvalue().wrap(),
        );
        action.assign(&state, &log_target)
    }

    /// Start executing the operation. Check for eval_running() first.
    pub fn start(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Initial.to_spvalue()
            || assignment.val == OperationState::Disabled.to_spvalue()
        {
            for precondition in &self.preconditions {
                if precondition.clone().eval(state, &log_target) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Executing.to_spvalue().wrap(),
                    );
                    return action
                        .assign(&precondition.clone().take(state, &log_target), &log_target);
                }
            }
        }
        state.clone()
    }

    /// Complete executing the operation. Check for can_be_completed() first.
    pub fn complete(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Executing.to_spvalue() {
            for postcondition in &self.postconditions {
                if postcondition.clone().eval(&state, &log_target) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Completed.to_spvalue().wrap(),
                    );
                    return postcondition
                        .clone()
                        .take(&action.assign(&state, &log_target), &log_target);
                }
            }
        }
        state.clone()
    }

    /// Fail the executing operation. Check for can_be_failed() first.
    pub fn fail(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Executing.to_spvalue() {
            for fail_transition in &self.failure_transitions {
                if fail_transition.clone().eval(&state, &log_target) {
                    let action =
                        Action::new(assignment.var, OperationState::Failed.to_spvalue().wrap());
                    return fail_transition
                        .clone()
                        .take(&action.assign(&state, &log_target), &log_target);
                }
            }
        }
        state.clone()
    }

    pub fn unrecover(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Failed.to_spvalue()
            || assignment.val == OperationState::Timedout.to_spvalue()
        {
            let action = Action::new(assignment.var, OperationState::Fatal.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            log::error!(target: &log_target, "Can't unrecover an operation which hasn't failed or timedout.");
            state.clone()
        }
    }

    pub fn bypass(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Failed.to_spvalue()
            || assignment.val == OperationState::Timedout.to_spvalue()
        {
            if self.bypass_transitions.len() > 0 {
                for bypass_transition in &self.bypass_transitions {
                    if bypass_transition.clone().eval(&state, &log_target) {
                        // Carefull: this can forbid the operation to bypass!
                        // Useful when you want to have different options to bypass and add some alternative conditions here
                        let action = Action::new(
                            assignment.var,
                            OperationState::Bypassed.to_spvalue().wrap(),
                        );
                        return bypass_transition
                            .clone()
                            .take(&action.assign(&state, &log_target), &log_target);
                    }
                }
            } else {
                let action =
                    Action::new(assignment.var, OperationState::Bypassed.to_spvalue().wrap());
                return action.assign(&state, &log_target);
            }
        }
        state.clone()
    }

    /// Timeout an executing the operation.
    pub fn timeout(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Executing.to_spvalue()
            || assignment.val == OperationState::Disabled.to_spvalue()
        {
            if self.timeout_transitions.len() > 0 {
                for timeout_transition in &self.timeout_transitions {
                    if timeout_transition.clone().eval(&state, &log_target) {
                        // Carefull: this can forbid the operation to timeout!
                        // Useful when you want to have different options to timeout and add some alternative conditions here
                        let action = Action::new(
                            assignment.var,
                            OperationState::Timedout.to_spvalue().wrap(),
                        );
                        return timeout_transition
                            .clone()
                            .take(&action.assign(&state, &log_target), &log_target);
                    }
                }
            } else {
                let action =
                    Action::new(assignment.var, OperationState::Timedout.to_spvalue().wrap());
                return action.assign(&state, &log_target);
            }
        }
        state.clone()
    }

    /// Retry the execution of the operation, allows for retries without immediate replanning.
    /// However, do we have to reset the variables before we can go back the initial state?
    /// Otherwise we might end up in disabled? Let's try withthe emulation.
    pub fn retry(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Failed.to_spvalue()
            || assignment.val == OperationState::Timedout.to_spvalue()
        {
            let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            state.clone()
        }
    }

    pub fn initialize(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
        action.assign(&state, &log_target)
    }

    pub fn reinitialize(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if assignment.val == OperationState::Completed.to_spvalue()
            || assignment.val == OperationState::Fatal.to_spvalue()
        {
            let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            state.clone()
        }
    }

    /// Continue executing the next operation if this one has failed
    // pub fn continue_running_next(&self, state: &State, log_target: &str) -> State {
    //     let assignment = state.get_assignment(&self.name, &log_target);
    //     if assignment.val == OperationState::Bypassed.to_spvalue()
    //     {
    //         for postcondition in &self.bypass_transitions {
    //             if postcondition.clone().eval(&state, &log_target) {
    //                 let action = Action::new(
    //                     assignment.var,
    //                     OperationState::Completed.to_spvalue().wrap(),
    //                 );
    //                 return postcondition
    //                     .clone()
    //                     .take(&action.assign(&state, &log_target), &log_target);
    //             }
    //         }
    //     }
    //     state.clone()
    // }

    // pub fn terminate(&self, state: &State, log_target: &str) -> State {
    //     let assignment = state.get_assignment(&self.name, &log_target);
    //     if assignment.val == OperationState::Unrecoverable.to_spvalue()
    //         || assignment.val == OperationState::Bypassed.to_spvalue()
    //         || assignment.val == OperationState::Completed.to_spvalue()
    //     {
    //         let action = Action::new(
    //             assignment.var,
    //             OperationState::Terminated.to_spvalue().wrap(),
    //         );
    //         action.assign(&state, &log_target)
    //     } else {
    //         log::error!(target: &log_target, "Can't terminate an operation which is not unrecoverable, bypassed, or completed.");
    //         state.clone()
    //     }
    // }

    pub fn get_all_var_keys(&self) -> Vec<String> {
        let mut all_keys: Vec<String> = self
            .preconditions
            .iter()
            .flat_map(|t| t.get_all_var_keys())
            .chain(
                self.postconditions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .chain(
                self.failure_transitions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .chain(
                self.timeout_transitions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .chain(
                self.cancel_transitions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .collect();

        all_keys.sort_unstable();
        all_keys.dedup();

        all_keys
    }

    // Tricky, wait with this, maybe we want to resrt when it failed.
    // Reset the completed operation. Check for can_be_reset() first.
    // pub fn reset_running(&self, state: &State) -> State {
    //     let assignment = state.get_assignment(&self.name);
    //     if assignment.val == OperationState::Completed.to_spvalue() {
    //         for reset_transition in &self.reset_transitions {
    //             if reset_transition.clone().eval_running(&state) {
    //                 let action =
    //                     Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
    //                 return reset_transition
    //                     .clone()
    //                     .take_running(&action.assign(&state));
    //             }
    //         }
    //     }
    //     state.clone()
    // }
}
