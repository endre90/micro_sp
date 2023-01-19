use serde::{Serialize, Deserialize};

use crate::{
    eq, Action, Predicate, SPValue, SPValueType, SPVariable, State, ToSPValue,
    ToSPWrapped, ToSPWrappedVar, Transition,
};
use std::{collections::HashMap, fmt};

/// The idea is to save the operation states elsewhere to help the assist tool and the planner
// #[derive(Debug, PartialEq, Clone, Eq)]
// pub enum OperationState {
//     Initial,
//     Executing,
//     // Done,
//     // WaitingToRun,
//     // Reseting
// }

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub precondition: Transition,
    pub postcondition: Transition
}

impl Operation {
    pub fn new(
        name: &str,
        precondition: Transition,
        postcondition: Transition
    ) -> Operation {
        Operation {
            name: name.to_string(),
            precondition,
            postcondition
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
        // effects?
    }

    // pub fn start(self, state: &State) -> bool {
    //     if state.get_value(&self.name) == "initial".to_spvalue() {
    //         self.precondition.eval_running(state)
    //     } else {
    //         false
    //     }
    // }

}

// pub fn start(
//     self,
//     state: &State,
//     op_state: &HashMap<Operation, OperationState>,
// ) -> (State, HashMap<Operation, OperationState>) {
//     match op_state.get(&self) {
//         Some(_) => {
//             let mut mut_op_state = op_state.clone();
//             mut_op_state.insert(self.clone(), OperationState::Executing);
//             (self.precondition.take(state), mut_op_state.clone())
//         }
//         None => panic!("operation doesn't have a state!"),
//     }
// }

// pub fn complete(
//     self,
//     state: &State,
//     op_state: &HashMap<Operation, OperationState>,
// ) -> (State, HashMap<Operation, OperationState>) {
//     match op_state.get(&self) {
//         Some(_) => {
//             let mut mut_op_state = op_state.clone();
//             mut_op_state.insert(self.clone(), OperationState::Initial);
//             (self.postcondition.take(state), mut_op_state.clone())
//         }
//         None => panic!("operation doesn't have a state!"),
//     }
// }

// pub fn take_planning(self, state: &State) -> State {
//     self.postcondition
//         .take_planning(&self.precondition.take_planning(state))
// }
// }
