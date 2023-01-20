use serde::{Deserialize, Serialize};

use crate::{SPVariable, SPWrapped, State};
use std::fmt;

/// Actions update the assignments of the state variables.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct Action {
    pub var: SPVariable,
    pub var_or_val: SPWrapped,
}

impl Action {
    pub fn new(var: SPVariable, var_or_val: SPWrapped) -> Action {
        Action { var, var_or_val }
    }

    pub fn assign(self, state: &State) -> State {
        match state.contains(&self.var.name) {
            true => match self.var_or_val {
                SPWrapped::SPVariable(x) => match state.contains(&x.name) {
                    true => state
                        .clone()
                        .update(&self.var.name, state.get_value(&x.name)),
                    false => panic!("Variable {:?} not in the state.", x.name),
                },
                SPWrapped::SPValue(x) => state.clone().update(&self.var.name, x),
            },
            false => panic!("Variable {} not in the state.", self.var.name),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmtr, "{} <= {}", self.var, self.var_or_val)
    }
}
