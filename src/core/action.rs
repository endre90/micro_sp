use crate::{VarOrVal, State};

#[derive(Debug, PartialEq, Clone)]
pub struct Action {
    pub var: String,
    pub var_or_val: VarOrVal
}

impl Action {
    pub fn assign(self, state: &State) -> State
 {
    match state.clone().contains(&self.var) {
        true => {
            match self.var_or_val {
                VarOrVal::String(x) => match state.clone().contains(&x) {
                    true => state.clone().update(&self.var, state.clone().get(&x)),
                    // {
                    //     match state.clone().get(&x) {
                    //         Some(val) => Some(state.clone().update(&self.var, val)),
                    //         None => None
                    //     }
                    // }
                    false => panic!("Variable {x} not in the state.")
                }
                VarOrVal::SPValue(x) => state.clone().update(&self.var, x)
            }
        },
        false => panic!("Variable {} not in the state.", self.var)
    }
 }}