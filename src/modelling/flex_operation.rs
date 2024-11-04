use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct FlexOperation {
    pub name: String,
    pub state: OperationState,
    pub timeout: Option<OrderedFloat<f64>>,
    pub timeout_transitions: Vec<Transition>,
    pub retries: i64,
    pub preconditions: Vec<Transition>,
    pub postconditions: Vec<Transition>,
    pub fail_transitions: Vec<Transition>,
    pub reset_transitions: Vec<Transition>,
}

impl FlexOperation {
    pub fn new(
        name: &str,
        timeout: Option<f64>,
        timeout_transitions: Vec<Transition>,
        retries: Option<i64>,
        preconditions: Vec<Transition>,
        postconditions: Vec<Transition>,
        fail_transitions: Vec<Transition>,
        reset_transitions: Vec<Transition>,
    ) -> FlexOperation {
        FlexOperation {
            name: name.to_string(),
            state: OperationState::UNKNOWN,
            timeout: match timeout {
                None => Some(OrderedFloat::from(MAX_ALLOWED_OPERATION_DURATION)),
                Some(x) => Some(OrderedFloat::from(x)),
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
        // (bool, Option<String>) {
        if state.get_value(&self.name) == OperationState::Initial.to_spvalue() {
            for precondition in &self.preconditions {
                if precondition.clone().eval_planning(state) {
                    return true;
                    //  return (true, Some(precondition.name))
                }
            }
        }
        false
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

    /// Check the running timeout_transition guard.
    pub fn can_be_timeouted(&self, state: &State) -> bool {
        if state.get_value(&self.name) == OperationState::Completed.to_spvalue() {
            for timeout_transition in &self.timeout_transitions {
                if timeout_transition.clone().eval_running(&state) {
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
                    return reset_transition.clone().take_running(&action.assign(&state));
                }
            }
        }
        state.clone()
    }
}