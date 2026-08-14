use serde::{Deserialize, Serialize};

// use crate::{SPVariable, SPWrapped, State};
use crate::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum ActionType {
    Assign,
    Increment,
    Decrement,
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

    // Apply this action to `state` in place.
    //
    // DONE: every arm used to finish with `state.update(..)`, which clones the
    // entire state map. Because `Transition::take` applies its actions in a
    // loop, a transition with k actions cost k full-state copies on top of the
    // one `take` already made. Writing through `State::update_mut` makes the
    // whole transition cost exactly one clone regardless of k - see
    // `Transition::take_mut`.
    //
    // PERF (still open): the Increment/Decrement arms call `state.get_value(..)`,
    // which clones the whole map to read one number - fix `State::get_value`
    // and this becomes free too.
    pub fn assign_mut(&self, state: &mut State, log_target: &str) {
        match self.action_type {
            ActionType::Assign => {
                let value_to_assign = self.var_or_val.evaluate(state, log_target);
                state.update_mut(&self.var.name, value_to_assign);
            }

            ActionType::Increment => {
                let current_val = state
                    .get_value(&self.var.name, log_target)
                    .unwrap_or_else(|| panic!("Variable '{}' not in state.", self.var.name));

                let increment_val = self.var_or_val.evaluate(state, log_target);

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

                state.update_mut(&self.var.name, new_val);
            }

            ActionType::Decrement => {
                let current_val = state
                    .get_value(&self.var.name, log_target)
                    .unwrap_or_else(|| panic!("Variable '{}' not in state.", self.var.name));

                let decrement_val = self.var_or_val.evaluate(state, log_target);

                let new_val = match (current_val, decrement_val) {
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

                state.update_mut(&self.var.name, new_val);
            }
        }
    }

    /// Owned form of [`Action::assign_mut`]: clones the state once, applies the
    /// action and returns the result. Kept so existing call sites are
    /// unchanged; prefer `assign_mut` when you already own a `State`.
    pub fn assign(self, state: &State, log_target: &str) -> State {
        let mut new_state = state.clone();
        self.assign_mut(&mut new_state, log_target);
        new_state
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
