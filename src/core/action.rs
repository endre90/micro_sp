use crate::{State, SPCommon, SPVariable, SPValue};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Hash)]
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
        match state.clone().contains_name(&self.var.name) {
            true => match self.common {
                SPCommon::SPVariable(x) => match state.clone().contains(&x) {
                    true => state.clone().update(&self.var.name, &state.clone().get(&x)),
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