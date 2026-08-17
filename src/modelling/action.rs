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

/// Arithmetic actions and rendering.
///
/// `Assign` is exercised everywhere; `Increment` and `Decrement` are not, and
/// they are the two that can *panic* - deliberately, on a type mismatch,
/// because incrementing a string or mixing an integer variable with a float
/// value is a model error rather than something to paper over at runtime. Since
/// these run inside a runner tick, a panic here takes that runner's task down,
/// so exactly which combinations panic is worth being explicit about.
#[cfg(test)]
mod arithmetic_tests {
    use crate::*;

    const TARGET: &str = "test";

    fn state() -> State {
        State::from_vec(&vec![
            (SPVariable::new("counter", SPValueType::Int64), 10.to_spvalue()),
            (SPVariable::new("ratio", SPValueType::Float64), 1.5.to_spvalue()),
            (SPVariable::new("label", SPValueType::String), "x".to_spvalue()),
            (SPVariable::new("step", SPValueType::Int64), 3.to_spvalue()),
        ])
    }

    fn action(var: &str, kind: ActionType, value: SPWrapped, state: &State) -> Action {
        Action {
            var: state.get_assignment(var, TARGET).var,
            var_or_val: value,
            action_type: kind,
        }
    }

    #[test]
    fn incrementing_an_integer_adds_to_it() {
        let state = state();
        let plus = action("counter", ActionType::Increment, 5.wrap(), &state);
        assert_eq!(
            plus.assign(&state, TARGET).get_value("counter", TARGET),
            Some(15.to_spvalue())
        );
    }

    #[test]
    fn decrementing_an_integer_subtracts_from_it() {
        let state = state();
        let minus = action("counter", ActionType::Decrement, 4.wrap(), &state);
        assert_eq!(
            minus.assign(&state, TARGET).get_value("counter", TARGET),
            Some(6.to_spvalue())
        );
    }

    #[test]
    fn floats_increment_and_decrement_too() {
        let state = state();
        let plus = action("ratio", ActionType::Increment, 0.5.wrap(), &state);
        assert_eq!(
            plus.assign(&state, TARGET).get_value("ratio", TARGET),
            Some(2.0.to_spvalue())
        );

        let minus = action("ratio", ActionType::Decrement, 0.5.wrap(), &state);
        assert_eq!(
            minus.assign(&state, TARGET).get_value("ratio", TARGET),
            Some(1.0.to_spvalue())
        );
    }

    /// The value can come from another variable, not just a literal - which is
    /// how a step size lives in the state.
    #[test]
    fn the_amount_can_come_from_another_variable() {
        let state = state();
        let step = state.get_assignment("step", TARGET).var;
        let plus = action("counter", ActionType::Increment, step.wrap(), &state);
        assert_eq!(
            plus.assign(&state, TARGET).get_value("counter", TARGET),
            Some(13.to_spvalue())
        );
    }

    /// Negative amounts are allowed, so increment and decrement are genuine
    /// inverses rather than "add a positive number" helpers.
    #[test]
    fn a_negative_amount_reverses_the_direction() {
        let state = state();
        let plus_negative = action("counter", ActionType::Increment, (-3).wrap(), &state);
        assert_eq!(
            plus_negative.assign(&state, TARGET).get_value("counter", TARGET),
            Some(7.to_spvalue())
        );
    }

    /// The four ways to get it wrong, each of which panics rather than
    /// silently coercing. Mixing integer and float is deliberately rejected in
    /// both directions - the alternative would be a variable quietly losing
    /// precision or changing type.
    #[test]
    fn mismatched_types_panic_rather_than_coercing() {
        let state = state();
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        let cases: Vec<(&str, Action)> = vec![
            (
                "int variable, float amount",
                action("counter", ActionType::Increment, 0.5.wrap(), &state),
            ),
            (
                "float variable, int amount",
                action("ratio", ActionType::Increment, 1.wrap(), &state),
            ),
            (
                "non-numeric variable",
                action("label", ActionType::Increment, 1.wrap(), &state),
            ),
            (
                "non-numeric variable, decrement",
                action("label", ActionType::Decrement, 1.wrap(), &state),
            ),
            (
                "int variable, float amount, decrement",
                action("counter", ActionType::Decrement, 0.5.wrap(), &state),
            ),
            (
                "float variable, int amount, decrement",
                action("ratio", ActionType::Decrement, 1.wrap(), &state),
            ),
        ];

        for (label, action) in cases {
            let state = state.clone();
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action.assign(&state, TARGET)));
            assert!(result.is_err(), "{label} should have panicked");
        }

        std::panic::set_hook(previous);
    }

    /// The empty action is what a model with an unparseable action ends up
    /// holding (see `Transition::parse`), and it is an assignment to a variable
    /// called "empty" - so applying it to a real state panics, which is how the
    /// model error eventually surfaces.
    #[test]
    fn the_empty_action_assigns_to_a_variable_nobody_declares() {
        let empty = Action::empty();
        assert_eq!(empty.var.name, "empty");
        assert_eq!(empty.action_type, ActionType::Assign);

        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let state = state();
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| empty.assign(&state, TARGET)));
        std::panic::set_hook(previous);

        assert!(
            result.is_err(),
            "applying the empty action to a state that has no 'empty' variable must panic"
        );
    }

    /// `Display` distinguishes the three action types - this is what shows up
    /// in the "please satisfy the runner guard" message.
    #[test]
    fn display_shows_which_kind_of_action_it_is() {
        let state = state();
        assert_eq!(
            action("counter", ActionType::Assign, 1.wrap(), &state).to_string(),
            "counter <= 1"
        );
        assert_eq!(
            action("counter", ActionType::Increment, 1.wrap(), &state).to_string(),
            "counter += 1"
        );
        assert_eq!(
            action("counter", ActionType::Decrement, 1.wrap(), &state).to_string(),
            "counter -= 1"
        );
    }

    /// The owned `assign` must leave its input alone.
    #[test]
    fn the_owned_assign_does_not_mutate_its_input() {
        let state = state();
        let plus = action("counter", ActionType::Increment, 1.wrap(), &state);
        let after = plus.assign(&state, TARGET);

        assert_eq!(state.get_value("counter", TARGET), Some(10.to_spvalue()));
        assert_eq!(after.get_value("counter", TARGET), Some(11.to_spvalue()));
    }
}
