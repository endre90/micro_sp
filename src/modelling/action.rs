use serde::{Deserialize, Serialize};

// use crate::{SPVariable, SPWrapped, State};
use crate::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum ActionType {
    Assign,
    Increment,
    Decrement,
    // Addition,
    // Subtraction
}

/// Actions update the assignments of the state variables.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct Action {
    pub var: SPVariable,
    pub var_or_val: SPWrapped,
    pub action_type: ActionType,
}

impl Action {
    pub fn empty() -> Action {
        Action {
            var: SPVariable::new("empty", SPValueType::Bool),
            var_or_val: SPWrapped::SPValue(SPValue::Bool(BoolOrUnknown::Bool(false))),
            action_type: ActionType::Assign,
        }
    }

    pub fn new(var: SPVariable, var_or_val: SPWrapped) -> Action {
        Action {
            var,
            var_or_val,
            action_type: ActionType::Assign,
        }
    }

    pub fn inc(var: SPVariable, var_or_val: SPWrapped) -> Action {
        Action {
            var,
            var_or_val,
            action_type: ActionType::Increment,
        }
    }

    pub fn dec(var: SPVariable, var_or_val: SPWrapped) -> Action {
        Action {
            var,
            var_or_val,
            action_type: ActionType::Decrement,
        }
    }

    // could provide a vector and then add all in the vectro...
    // pub fn addition(var: SPVariable, var_or_val: SPWrapped, var_or_val_2: SPWrapped) -> Action {
    //     Action {
    //         var,
    //         var_or_val,
    //         action_type: ActionType::Addition,
    //     }
    // }

    pub fn assign(self, state: &State, log_target: &str) -> State {
        match self.action_type {
            ActionType::Assign => {
                let value_to_assign = match self.var_or_val {
                    SPWrapped::SPVariable(x) => state
                        .get_value(&x.name, log_target)
                        .unwrap_or_else(|| panic!("Source variable '{}' not in state.", x.name)),
                    SPWrapped::SPValue(x) => x,
                };
                state.update(&self.var.name, value_to_assign)
            }

            ActionType::Increment => {
                let current_val = state
                    .get_value(&self.var.name, log_target)
                    .unwrap_or_else(|| panic!("Variable '{}' not in state.", self.var.name));

                let increment_val = match self.var_or_val {
                    SPWrapped::SPVariable(x) => state
                        .get_value(&x.name, log_target)
                        .unwrap_or_else(|| panic!("Source variable '{}' not in state.", x.name)),
                    SPWrapped::SPValue(x) => x,
                };

                let new_val = match (current_val, increment_val) {
                    (
                        SPValue::Int64(IntOrUnknown::Int64(x)),
                        SPValue::Int64(IntOrUnknown::Int64(y)),
                    ) => SPValue::Int64(IntOrUnknown::Int64(x + y)),
                    (
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(x))),
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(y))),
                    ) => SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(
                        x + y,
                    ))),
                    (
                        SPValue::Int64(IntOrUnknown::Int64(_)),
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(y))),
                    ) => {
                        panic!(
                            "Cannot increment integer variable {} with a float value {}.",
                            self.var.name, y
                        );
                    }
                    (
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(_))),
                        SPValue::Int64(IntOrUnknown::Int64(y)),
                    ) => {
                        panic!(
                            "Cannot increment float variable {} with an integer value {}.",
                            self.var.name, y
                        );
                    }
                    other => {
                        panic!(
                            "Variable '{}' holds non-numeric value '{:?}' and cannot be incremented.",
                            self.var.name, other
                        );
                    }
                };

                state.update(&self.var.name, new_val)
            }

            ActionType::Decrement => {
                let current_val = state
                    .get_value(&self.var.name, log_target)
                    .unwrap_or_else(|| panic!("Variable '{}' not in state.", self.var.name));

                let increment_val = match self.var_or_val {
                    SPWrapped::SPVariable(x) => state
                        .get_value(&x.name, log_target)
                        .unwrap_or_else(|| panic!("Source variable '{}' not in state.", x.name)),
                    SPWrapped::SPValue(x) => x,
                };

                let new_val = match (current_val, increment_val) {
                    (
                        SPValue::Int64(IntOrUnknown::Int64(x)),
                        SPValue::Int64(IntOrUnknown::Int64(y)),
                    ) => SPValue::Int64(IntOrUnknown::Int64(x - y)),
                    (
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(x))),
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(y))),
                    ) => SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(
                        x - y,
                    ))),
                    (
                        SPValue::Int64(IntOrUnknown::Int64(_)),
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(y))),
                    ) => {
                        panic!(
                            "Cannot increment integer variable {} with a float value {}.",
                            self.var.name, y
                        );
                    }
                    (
                        SPValue::Float64(FloatOrUnknown::Float64(ordered_float::OrderedFloat(_))),
                        SPValue::Int64(IntOrUnknown::Int64(y)),
                    ) => {
                        panic!(
                            "Cannot increment float variable {} with an integer value {}.",
                            self.var.name, y
                        );
                    }
                    other => {
                        panic!(
                            "Variable '{}' holds non-numeric value '{:?}' and cannot be incremented.",
                            self.var.name, other
                        );
                    }
                };

                state.update(&self.var.name, new_val)
            }
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.action_type {
            ActionType::Assign => {
                write!(fmtr, "{} <= {}", self.var, self.var_or_val)
            }
            ActionType::Increment => {
                write!(fmtr, "{} += {}", self.var, self.var_or_val)
            }
            ActionType::Decrement => {
                write!(fmtr, "{} -= {}", self.var, self.var_or_val)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v!("name");
        let surname = v!("surname");
        let height = iv!("height");
        let weight = fv!("weight");
        let smart = bv!("smart");

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
        let weight = fv!("weight");
        let a1 = Action::new(weight.clone(), 82.5.wrap());
        let a2 = Action::new(weight.clone(), 85.0.wrap());
        let s_next_1 = a1.assign(&s, "t");
        let s_next_2 = a2.assign(&s_next_1, "t");
        assert_eq!(s_next_1.get_value("weight", "t"), Some(82.5.to_spvalue()));
        assert_eq!(s_next_2.get_value("weight", "t"), Some(85.0.to_spvalue()));
    }

    #[test]
    fn test_action_increment() {
        let s = State::from_vec(&john_doe());
        let height = iv!("height");
        let inc1 = Action::inc(height.clone(), 5.wrap());
        let inc2 = Action::inc(height, 7.wrap());
        let s_next_1 = inc1.assign(&s, "t");
        let s_next_2 = inc2.assign(&s_next_1, "t");
        assert_eq!(s_next_1.get_value("height", "t"), Some(190.to_spvalue()));
        assert_eq!(s_next_2.get_value("height", "t"), Some(197.to_spvalue()));
    }

    #[test]
    #[should_panic]
    fn test_action_assign_panic() {
        let s = State::from_vec(&john_doe());
        let bitrhyear = iv!("bitrhyear");
        let a1 = Action::new(bitrhyear.clone(), 1967.wrap());
        a1.assign(&s, "t");
    }

    #[test]
    fn test_action_assign_macro() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let s_next_1 = a1.assign(&s, "t");
        let s_next_2 = a2.assign(&s_next_1, "t");
        assert_eq!(s_next_1.get_value("weight", "t"), Some(82.5.to_spvalue()));
        assert_eq!(s_next_2.get_value("weight", "t"), Some(85.0.to_spvalue()));
    }

    #[test]
    #[should_panic]
    fn test_action_assign_panic_macro() {
        let s = State::from_vec(&john_doe());
        let bitrhyear = iv!("bitrhyear");
        let a1 = a!(bitrhyear.clone(), 1967.wrap());
        a1.assign(&s, "t");
    }
}
