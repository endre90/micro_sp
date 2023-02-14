use serde::{Deserialize, Serialize};

use crate::{Action, State, ToSPValue, ToSPWrapped, Transition};

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub precondition: Transition,
    pub postcondition: Transition,
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct OperationResult {
//     pub new_state: State,
//     pub success: bool,
//     pub info: String
// }

impl Operation {
    pub fn new(name: &str, precondition: Transition, postcondition: Transition) -> Operation {
        Operation {
            name: name.to_string(),
            precondition,
            postcondition,
        }
    }

    pub fn eval_planning(self, state: &State) -> bool {
        if state.get_value(&self.name) == "initial".to_spvalue() {
            self.precondition.eval_planning(state)
        } else {
            false
        }
    }

    pub fn eval_running(self, state: &State) -> bool {
        if state.get_value(&self.name) == "initial".to_spvalue() {
            self.precondition.eval_running(state)
        } else {
            false
        }
    }

    pub fn take_planning(self, state: &State) -> State {
        self.postcondition.take_planning(&self.precondition.take_planning(state))
    }

    pub fn start_running(self, state: &State) -> State {
        let assignment = state.get_all(&self.name);
        if assignment.val == "initial".to_spvalue() {
            let action = Action::new(assignment.var, "executing".wrap());
            action.assign(&self.precondition.take_running(state))
        } else {
            state.clone()
        }
    }

    pub fn complete_running(self, state: &State) -> State {
        let assignment = state.get_all(&self.name);
        if assignment.val == "executing".to_spvalue() {
            let action = Action::new(assignment.var, "initial".wrap());
            self.postcondition.take_running(&action.assign(&state))
        } else {
            state.clone()
        }
    }

    pub fn can_be_completed(self, state: &State) -> bool {
        state.get_value(&self.name) == "executing".to_spvalue() && self.postcondition.eval_running(&state)
    }
}
