use crate::{State, VarOrVal};

#[derive(Debug, PartialEq, Clone)]
pub struct Action {
    pub var: String,
    pub var_or_val: VarOrVal,
}

impl Action {
    pub fn new(var: &str, var_or_val: VarOrVal) -> Action {
        Action {
            var: var.to_string(),
            var_or_val
        }
    }
    pub fn assign(self, state: &State) -> State {
        match state.clone().contains(&self.var) {
            true => match self.var_or_val {
                VarOrVal::String(x) => match state.clone().contains(&x) {
                    true => state.clone().update(&self.var, state.clone().get(&x)),
                    false => panic!("Variable {x} not in the state."),
                },
                VarOrVal::SPValue(x) => state.clone().update(&self.var, x),
            },
            false => panic!("Variable {} not in the state.", self.var),
        }
    }
}
