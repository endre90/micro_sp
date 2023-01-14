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

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Operation {
    pub name: String,
    pub precondition: Transition,
    pub postcondition: Transition,
    // pub effect: Vec<Action>, // figure out in whixh scenarios do wen need this and in which is it enought to have only run
    // pub run: Option<Transition>,
}

// pub fn initialize_ops(ops: Vec<Operation>) -> HashMap<Operation, OperationState> {
//     let mut operations = HashMap::new();
//     ops.iter().for_each(
//         |o| match operations.insert(o.clone(), OperationState::Initial) {
//             _ => (),
//         },
//     );
//     operations
// }

impl Operation {
    pub fn new(
        name: &str,
        precondition: &Transition,
        postcondition: &Transition,
        // run: &Option<Transition>,
    ) -> Operation {
        Operation {
            name: name.to_string(),
            precondition: precondition.to_owned(),
            postcondition: postcondition.to_owned(),
            // run: run.to_owned(),
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
