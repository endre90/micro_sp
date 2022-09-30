use crate::{VarOrVal, State};

#[derive(Debug, PartialEq, Clone)]
pub struct Action {
    pub var: String,
    pub var_or_val: VarOrVal
}

impl Action {
    pub fn assign(self, state: &State) -> State
 {
    if !state.clone().contains(&self.var) {
        panic!("key {} is not in the state", self.var)
    }

    match self.var_or_val {
        VarOrVal::String(x) => match state.clone().contains(&x) {
            true => state.clone().update(&self.var, state.clone().get(&x).unwrap()),
            false => panic!("No such variable {} in the state", x)
        }
        VarOrVal::SPValue(x) => state.clone().update(&self.var, x)
    }
 }}