use serde::{Deserialize, Serialize};
use std::fmt;

use crate::*;

/// Initial:   The operation planned and ready to be executed.
/// Disabled:  The operation is ready for execution, but the precondition guard is not yet enabled.
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
    Timedout,
    Failed,
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
            "completed" => OperationState::Completed,
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
            OperationState::Completed => write!(f, "completed"),
            OperationState::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub state: OperationState,
    pub timeout_ms: Option<u128>,
    pub retries: i64,
    pub preconditions: Vec<Transition>,
    pub postconditions: Vec<Transition>,
    pub fail_transitions: Vec<Transition>,
    pub timeout_transitions: Vec<Transition>,
    pub reset_transitions: Vec<Transition>,
}

impl Operation {
    pub fn new(
        name: &str,
        timeout_ms: Option<u128>,
        retries: Option<i64>,
        preconditions: Vec<Transition>,
        postconditions: Vec<Transition>,
        fail_transitions: Vec<Transition>,
        timeout_transitions: Vec<Transition>,
        reset_transitions: Vec<Transition>,
    ) -> Operation {
        Operation {
            name: name.to_string(),
            state: OperationState::UNKNOWN,
            timeout_ms: match timeout_ms {
                None => Some(MAX_ALLOWED_OPERATION_DURATION_MS),
                Some(x) => Some(x),
            },
            timeout_transitions,
            retries: match retries {
                Some(x) => x,
                None => 0,
            },
            preconditions,
            postconditions,
            fail_transitions,
            reset_transitions,
        }
    }

    /// Check the guard of the planning precondidion transition.
    pub fn eval_planning(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Initial.to_spvalue() {
            for precondition in &self.preconditions {
                if precondition.clone().eval_planning(state) {
                    return true;
                }
            }
        }
        false
    }

    /// Execute the planing actions of both the pre and post conditions.
    /// Inex 0 taken as to indicate that the firstly defined transition should be taken when planning.
    pub fn take_planning(&self, state: &State) -> State {
        self.postconditions[0]
            .clone()
            .take_planning(&self.preconditions[0].clone().take_planning(state))
    }

    /// Check the guard of the running precondidion transition.
    pub fn eval_running(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Initial.to_spvalue() {
            for precondition in &self.preconditions {
                if precondition.clone().eval_running(state) {
                    return true;
                }
            }
        }
        false
    }

    /// Check the running postondition guard.
    pub fn can_be_completed(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Executing.to_spvalue() {
            for postcondition in &self.postconditions {
                if postcondition.clone().eval_running(&state) {
                    return true;
                }
            }
        }
        false
    }

    /// Check the running fail_transition guard.
    pub fn can_be_failed(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Executing.to_spvalue() {
            for fail_transition in &self.fail_transitions {
                if fail_transition.clone().eval_running(&state) {
                    return true;
                }
            }
        }
        false
    }

    /// Check the running reset_transition guard.
    pub fn can_be_reset(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Completed.to_spvalue() {
            for reset_transition in &self.reset_transitions {
                if reset_transition.clone().eval_running(&state) {
                    return true;
                }
            }
        }
        false
    }

    /// Start executing the operation. Check for eval_running() first.
    pub fn start_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Initial.to_spvalue() {
            for precondition in &self.preconditions {
                if precondition.clone().eval_running(state) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Executing.to_spvalue().wrap(),
                    );
                    return action.assign(&precondition.clone().take_running(state));
                }
            }
        }
        state.clone()
    }

    /// Complete executing the operation. Check for can_be_completed() first.
    pub fn complete_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Executing.to_spvalue() {
            for postcondition in &self.postconditions {
                if postcondition.clone().eval_running(&state) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Completed.to_spvalue().wrap(),
                    );
                    return postcondition.clone().take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }

    /// Fail the executing operation. Check for can_be_failed() first.
    pub fn fail_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Executing.to_spvalue() {
            for fail_transition in &self.fail_transitions {
                if fail_transition.clone().eval_running(&state) {
                    let action =
                        Action::new(assignment.var, OperationState::Failed.to_spvalue().wrap());
                    return fail_transition.clone().take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }

    /// Timeout an executing the operation.
    pub fn timeout_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Executing.to_spvalue() {
            for timeout_transition in &self.timeout_transitions {
                if timeout_transition.clone().eval_running(&state) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Timedout.to_spvalue().wrap(),
                    );
                    return timeout_transition.clone().take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }

    /// Retry the execution of the operation, allows for retries without immediate replanning.
    pub fn retry_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Failed.to_spvalue() {
            let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
            action.assign(&state)
        } else {
            state.clone()
        }
    }

    /// Reset the completed operation. Check for can_be_reset() first.
    pub fn reset_running(&self, state: &State) -> State {
        let assignment = state.get_assignment(&self.name);
        if assignment.val == OperationState::Completed.to_spvalue() {
            for reset_transition in &self.reset_transitions {
                if reset_transition.clone().eval_running(&state) {
                    let action =
                        Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
                    return reset_transition
                        .clone()
                        .take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }
}
