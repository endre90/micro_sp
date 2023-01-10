use crate::{
    eq, get_predicate_vars, Action, Predicate, SPValue, SPValueType, SPVariable, State, ToSPCommon,
    ToSPCommonVar, ToSPValue, Transition,
};
use std::{collections::HashMap, fmt};

/// The idea is to save the operation states elsewhere to help the assist tool and the planner
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum OperationState {
    Initial,
    Executing,
    // Done,
    WaitingToRun,
    // Reseting
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Operation {
    pub name: String,
    pub precondition: Transition,
    pub postcondition: Transition,
    // pub effect: Vec<Action>, // figure out in whixh scenarios do wen need this and in which is it enought to have only run
    pub run: Option<Transition>,
}

pub fn initialize_ops(ops: Vec<Operation>) -> HashMap<Operation, OperationState> {
    let mut operations = HashMap::new();
    ops.iter().for_each(
        |o| match operations.insert(o.clone(), OperationState::Initial) {
            _ => (),
        },
    );
    operations
}

impl Operation {
    pub fn new(
        name: &str,
        precondition: &Transition,
        postcondition: &Transition,
        run: &Option<Transition>,
    ) -> Operation {
        Operation {
            name: name.to_string(),
            precondition: precondition.to_owned(),
            postcondition: postcondition.to_owned(),
            run: run.to_owned(),
        }
    }

    pub fn eval(self, state: &State) -> bool { //, op_state: &HashMap<Operation>) -> bool { //, OperationState>) -> bool {
        self.precondition.eval(state)
        // match op_state.get(&self) {
        //     Some(op) => self.precondition.eval(state),  // && op.clone() == OperationState::Initial,
        //     None => panic!("operation doesn't have a state!"),
        // }
    }

    pub fn eval_run(self, state: &State, op_state: &HashMap<Operation, OperationState>) -> bool {
        match op_state.get(&self) {
            Some(op) => match self.run {
                Some(run_exists) => {
                    self.precondition.eval(state)
                        && run_exists.eval(state)
                        && op.clone() == OperationState::Initial
                }
                None => self.precondition.eval(state) && op.clone() == OperationState::Initial,
            },
            None => panic!("operation doesn't have a state!"),
        }
    }

    pub fn start(
        self,
        state: &State,
        op_state: &HashMap<Operation, OperationState>,
    ) -> (State, HashMap<Operation, OperationState>) {
        match op_state.get(&self) {
            Some(_) => {
                let mut mut_op_state = op_state.clone();
                mut_op_state.insert(self.clone(), OperationState::Executing);
                (self.precondition.take(state), mut_op_state.clone())
            }
            None => panic!("operation doesn't have a state!"),
        }
    }

    pub fn complete(
        self,
        state: &State,
        op_state: &HashMap<Operation, OperationState>,
    ) -> (State, HashMap<Operation, OperationState>) {
        match op_state.get(&self) {
            Some(_) => {
                let mut mut_op_state = op_state.clone();
                mut_op_state.insert(self.clone(), OperationState::Initial);
                (self.postcondition.take(state), mut_op_state.clone())
            }
            None => panic!("operation doesn't have a state!"),
        }
    }

    pub fn take_planning(self, state: &State) -> State {
        self.postcondition
            .take_planning(&self.precondition.take_planning(state))
    }
}
