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

#[cfg(test)]
mod tests {

    use crate::*;

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v_estimated!("name", vec!("John", "Jack"));
        let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
        let height = iv_estimated!("height", vec!(180, 185, 190));
        let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
        let smart = bv_estimated!("smart");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (smart, true.to_spvalue()),
        ]
    }

    #[test]
    fn test_action_assign() {
        let s = State::from_vec(&john_doe());
        let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
        let a1 = Action::new(weight.clone(), 82.5.wrap());
        let a2 = Action::new(weight.clone(), 85.0.wrap());
        let s_next_1 = a1.assign(&s);
        let s_next_2 = a2.assign(&s_next_1);
        assert_eq!(s_next_1.get_value("weight"), 82.5.to_spvalue());
        assert_eq!(s_next_2.get_value("weight"), 85.0.to_spvalue());
    }

    #[test]
    #[should_panic]
    fn test_action_assign_panic() {
        let s = State::from_vec(&john_doe());
        let bitrhyear = iv_estimated!("bitrhyear", vec!(1967, 1966));
        let a1 = Action::new(bitrhyear.clone(), 1967.wrap());
        a1.assign(&s);
    }

    #[test]
    fn test_action_assign_macro() {
        let s = State::from_vec(&john_doe());
        let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let s_next_1 = a1.assign(&s);
        let s_next_2 = a2.assign(&s_next_1);
        assert_eq!(s_next_1.get_value("weight"), 82.5.to_spvalue());
        assert_eq!(s_next_2.get_value("weight"), 85.0.to_spvalue());
    }

    #[test]
    #[should_panic]
    fn test_action_assign_panic_macro() {
        let s = State::from_vec(&john_doe());
        let bitrhyear = iv_estimated!("bitrhyear", vec!(1967, 1966));
        let a1 = a!(bitrhyear.clone(), 1967.wrap());
        a1.assign(&s);
    }
}
