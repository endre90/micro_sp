use crate::{State, SPCommon, SPVariable};
use std::fmt;

/// Actions update the assignments of the state variables.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Action {
    pub var: SPVariable,
    pub common: SPCommon,
}

impl Action {
    pub fn new(var: SPVariable, common: SPCommon) -> Action {
        Action {
            var,
            common
        }
    }
    
    pub fn assign(self, state: &State) -> State {
        match state.state.clone().contains_key(&self.var.name) {
            true => match self.common {
                SPCommon::SPVariable(x) => match state.state.clone().contains_key(&x.name) {
                    true => state.clone().update(&self.var.name, &state.state.clone().get(&x.name).unwrap().val),
                    false => panic!("Variable {:?} not in the state.", x.name),
                },
                SPCommon::SPValue(x) => state.clone().update(&self.var.name, &x),
            },
            false => panic!("Variable {} not in the state.", self.var.name),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmtr, "{} <= {}", self.var, self.common)
    }
}