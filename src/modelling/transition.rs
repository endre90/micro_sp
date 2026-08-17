//! [`Transition`]s: a guard plus the assignments to make when it is taken.
//!
//! A transition has two halves. The *planning* half (`guard`, `actions`) is all
//! the planner sees; the *runner* half (`runner_guard`, `runner_actions`) is
//! added on top when the plan is actually executed. [`Transition::parse`] builds
//! one from strings, which is how models are normally written.

use crate::*;
use serde::{Deserialize, Serialize};
use std::{fmt, hash::Hash};

/// A planning transition T contains a guard predicate G : S → {false, true},
/// and a set of action functions A, where ∀a ∈ A, a : S → S models
/// the updates of the state variables. If the guard predicate evaluates to
/// true, the transition can occur, after which the actions of the transition
/// describe how the variables are updated. The notation we use to represent
/// a planning transition is T : G/A.
///
/// A running transition Tr extends the planning transition with an additional
/// running guard Gr and additional running action Ar. We write
/// running transitions as Tr : G / Gr / A / Ar , where g and gr are both guard
/// predicates and G ∧ Gr : S → {false, true}, and A and Ar are both action
/// functions, where ∀a ∈ A ∪ Ar , a : S → S model the updates of the values
/// of the state variables. While planning, only G and A are considered, i.e.
/// the running transition is evaluated and taken as a planning transition.
/// When the execution engine is running the plan, it is considering all
/// components of Tr, i.e. the running transition guard becomes G ∧ Gr and the
/// set of transition actions becomes A ∪ Ar.
///
/// Two transitions compare equal when their guards and actions match, *ignoring
/// the name*.
#[derive(Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Transition {
    /// Name of the transition, used for logging and plan steps.
    pub name: String,
    /// The planning guard: the condition the planner reasons about.
    pub guard: Predicate,
    /// The extra guard that must also hold while actually executing. Invisible
    /// to the planner - typically feedback from the real system, or an operator
    /// interlock.
    pub runner_guard: Predicate,
    /// The planning effects: what the planner assumes this transition changes.
    pub actions: Vec<Action>,
    /// Extra assignments applied only when executing, not when planning.
    pub runner_actions: Vec<Action>,
}

impl Transition {
    /// Define a new transition from already-built parts.
    ///
    /// Prefer [`Transition::parse`], which takes the same thing as strings.
    pub fn new(
        name: &str,
        guard: Predicate,
        runner_guard: Predicate,
        actions: Vec<Action>,
        runner_actions: Vec<Action>,
    ) -> Transition {
        Transition {
            name: name.to_string(),
            guard,
            runner_guard,
            actions,
            runner_actions,
        }
    }

    /// Define a new transition using the string DSL. This is how models are
    /// normally written.
    ///
    /// # Syntax
    ///
    /// Guards (`guard`, `runner_guard`) are predicates:
    ///
    /// | Form | Meaning |
    /// |---|---|
    /// | `true` / `false` | the constant predicate |
    /// | `var:name == value` | equality; also `!=`, `<`, `<=`, `>`, `>=` |
    /// | `var:a == var:b` | compare two variables |
    /// | `a && b`, `a \|\| b` | conjunction, disjunction |
    /// | `!a`, `(a)` | negation, grouping |
    /// | `a -> b` | implication, sugar for `!a \|\| b` |
    ///
    /// Actions (`actions`, `runner_actions`) are assignments:
    ///
    /// | Form | Meaning |
    /// |---|---|
    /// | `var:name <- value` | assign a literal |
    /// | `var:a <- var:b` | assign another variable's value |
    /// | `var:n += 1`, `var:n -= 1` | increment / decrement a numeric variable |
    ///
    /// Values are bare words (`a`, `moving`), numbers (`5`, `-1.5`), `true` /
    /// `false`, `"quoted strings"`, arrays (`[1, 2, 3]`), or the typed unknowns
    /// (`UNKNOWN_bool`, `UNKNOWN_int`, ...).
    ///
    /// `state` is only used to look the variables up, so every `var:name` must
    /// already exist in it.
    ///
    /// # Errors
    ///
    /// Parse failures are non-fatal and only logged: an unparseable guard
    /// becomes [`Predicate::FALSE`] (the transition can never fire) and an
    /// unparseable action becomes [`Action::empty`] (which panics if applied).
    /// A typo in a model disables one transition rather than stopping the
    /// process.
    ///
    /// # Example
    ///
    /// ```
    /// use micro_sp::*;
    ///
    /// let mut state = State::new();
    /// state.add_mut(
    ///     SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
    ///     "docs",
    /// );
    /// state.add_mut(
    ///     SPAssignment::new(SPVariable::new("enabled", SPValueType::Bool), true.to_spvalue()),
    ///     "docs",
    /// );
    ///
    /// let move_to_b = Transition::parse(
    ///     "move_to_b",
    ///     "var:pos == a || var:pos == c",  // planning guard
    ///     "var:enabled == true",           // runner guard
    ///     vec!["var:pos <- b"],            // planning actions
    ///     Vec::<&str>::new(),              // runner actions
    ///     &state,
    /// );
    ///
    /// assert!(move_to_b.eval(&state, "docs"));
    /// let state = move_to_b.take(&state, "docs");
    /// assert_eq!(state.get_value("pos", "docs"), Some("b".to_spvalue()));
    /// ```
    pub fn parse(
        name: &str,
        guard: &str,
        runner_guard: &str,
        actions: Vec<&str>,
        runner_actions: Vec<&str>,
        state: &State,
    ) -> Transition {
        Transition::new(
            name,
            match pred_parser::pred(guard, state) {
                Ok(guard_predicate) => guard_predicate,
                Err(e) => {
                    log::error!(target: &&format!("transition_parser"), 
                        "Failed to parse guard {guard} with: {e}");
                    log::error!(target: &&format!("transition_parser"), 
                        "Guard set to FALSE, fix the model.");
                    Predicate::FALSE
                }
            },
            match pred_parser::pred(runner_guard, state) {
                Ok(guard_predicate) => guard_predicate,
                Err(e) => {
                    log::error!(target: &&format!("transition_parser"), 
                        "Failed to parse guard {runner_guard} with: {e}");
                    log::error!(target: &&format!("transition_parser"), 
                        "Runner guard set to FALSE, fix the model.");
                    Predicate::FALSE
                }
            },
            actions
                .iter()
                .map(|action| match pred_parser::action(action, state) {
                    Ok(action_def) => action_def,
                    Err(e) => {
                        log::error!(target: &&format!("transition_parser"), 
                            "Failed to parse action {action} with: {e}");
                        log::error!(target: &&format!("transition_parser"), 
                            "Action set to EMPTY, fix the model.");
                        Action::empty()
                    }
                })
                .collect::<Vec<Action>>(),
            runner_actions
                .iter()
                .map(|action| match pred_parser::action(action, state) {
                    Ok(action_def) => action_def,
                    Err(e) => {
                        log::error!(target: &&format!("transition_parser"), 
                            "Failed to parse runner_actions {action} with: {e}");
                        log::error!(target: &&format!("transition_parser"), 
                            "Runner action set to EMPTY, fix the model.");
                        Action::empty()
                    }
                })
                .collect::<Vec<Action>>(),
        )
    }

    /// A placeholder transition named `"empty"` that can never be taken.
    ///
    /// Both guards are [`Predicate::FALSE`] and it has no actions.
    pub fn empty() -> Transition {
        Transition::new(
            "empty",
            Predicate::FALSE,
            Predicate::FALSE,
            vec![],
            vec![],
        )
    }

    /// Evaluate only the planning guard, as the planner does.
    ///
    /// `log_target` is the `log` crate target that any warning is reported
    /// under; it does not affect the result.
    pub fn eval_planning(&self, state: &State, log_target: &str) -> bool {
        self.guard.eval(state, &log_target)
    }

    /// Evaluate the full running guard: `guard && runner_guard`.
    pub fn eval(&self, state: &State, log_target: &str) -> bool {
        self.guard.eval(state, &log_target) && self.runner_guard.eval(state, &log_target)
    }

    /// Apply this transition's planning actions to `state` in place.
    pub fn take_planning_mut(&self, state: &mut State, log_target: &str) {
        for a in &self.actions {
            a.assign_mut(state, &log_target);
        }
    }

    /// Owned form of [`Transition::take_planning_mut`] - clones the state once.
    pub fn take_planning(self, state: &State, log_target: &str) -> State {
        let mut new_state = state.clone();
        self.take_planning_mut(&mut new_state, &log_target);
        new_state
    }

    /// Apply this transition's planning *and* runner actions to `state` in
    /// place.
    pub fn take_mut(&self, state: &mut State, log_target: &str) {
        for a in &self.actions {
            a.assign_mut(state, &log_target);
        }
        for a in &self.runner_actions {
            a.assign_mut(state, &log_target);
        }
    }

    /// Owned form of [`Transition::take_mut`] - clones the state once.
    pub fn take(self, state: &State, log_target: &str) -> State {
        let mut new_state = state.clone();
        self.take_mut(&mut new_state, &log_target);
        new_state
    }
    /// Every state variable this transition reads or writes.
    ///
    /// This feeds the runners' `get_state_for_keys` key sets, so anything
    /// missing here is a variable missing from the state the runner reads - and
    /// reading a variable that is not in the state panics.
    ///
    /// Note both halves of an action count: `var:a <- var:b` writes `a` *and*
    /// reads `b`, and an array/map right-hand side can reference any number of
    /// further variables (`Action::assign_mut` evaluates it through
    /// `SPWrapped::evaluate`). Collecting only `a.var` - which is what this did
    /// originally - silently dropped every right-hand side variable.
    pub fn get_all_var_keys(&self) -> Vec<String> {
        fn action_var_keys(action: &Action) -> Vec<String> {
            let mut keys = vec![action.var.name.clone()];
            keys.extend(
                action
                    .var_or_val
                    .get_variables()
                    .into_iter()
                    .map(|var| var.name),
            );
            keys
        }

        let mut all_keys: Vec<String> = self.guard.get_predicate_var_keys()
            .into_iter()
            .chain(self.runner_guard.get_predicate_var_keys())
            .chain(self.actions.iter().flat_map(action_var_keys))
            .chain(self.runner_actions.iter().flat_map(action_var_keys))
            .collect();

        all_keys.sort();
        all_keys.dedup();
        all_keys
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Transition) -> bool {
        self.guard == other.guard
            && self.runner_guard == other.runner_guard
            && self.actions == other.actions
            && self.runner_actions == other.runner_actions
    }
}

impl Default for Transition {
    fn default() -> Self {
        Transition {
            name: "unknown".to_string(),
            guard: Predicate::TRUE,
            runner_guard: Predicate::TRUE,
            actions: vec![],
            runner_actions: vec![],
        }
    }
}
impl fmt::Display for Transition {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut action_string = "".to_string();
        let mut actions = self.actions.clone();
        match actions.pop() {
            Some(last_action) => {
                action_string = actions
                    .iter()
                    .map(|x| format!("{}, ", x.to_string()))
                    .collect::<String>();
                let last_action_string = &format!("{}", last_action.to_string());
                action_string.extend(last_action_string.chars());
            }
            None => (),
        }
        write!(fmtr, "{}: {} / [{}]", self.name, self.guard, action_string)
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
        let alive = bv!("alive");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (smart, true.to_spvalue()),
            (alive, true.to_spvalue()),
        ]
    }

    #[test]
    fn test_transition_new() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 85.0.wrap());
        let t1 = Transition::new(
            "gains_weight",
            Predicate::TRUE,
            Predicate::TRUE,
            vec![a1.clone()],
            vec![],
        );
        let t2 = Transition::new(
            "gains_weight",
            Predicate::TRUE,
            Predicate::TRUE,
            vec![a1],
            vec![],
        );
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_transition_new_macro() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
        let t2 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1));
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_transition_eval_planning() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!("gains_weight", Predicate::TRUE, vec!(a1.clone()));
        let t2 = t_plan!("gains_weight", Predicate::FALSE, vec!(a1));
        assert!(t1.eval_planning(&s, "t"));
        assert!(!t2.eval_planning(&s, "t"));
    }

    #[test]
    fn test_transition_eval_running() {
        let s = State::from_vec(&john_doe());
        let t1 = t!(
            "gains_weight",
            "true",
            "true",
            vec!("var:weight <- 85.0", "var:height <- 190"),
            Vec::<&str>::new(),
            &s
        );
        let t2 = t!(
            "gains_weight",
            "true",
            "false",
            vec!("var:weight <- 85.0"),
            Vec::<&str>::new(),
            &s
        );
        assert!(t1.eval(&s, "t"));
        assert!(!t2.eval(&s, "t"));
    }

    #[test]
    #[should_panic]
    fn test_transition_planner_var_in_runner_guard_panic() {
        let s = State::from_vec(&john_doe());
        let t1 = t!(
            "gains_weight",
            "true",
            "var:weight == 85.0",
            vec!("var:weight <- 85.0", "var:height <- 190"),
            Vec::<&str>::new(),
            &s
        );
        assert!(t1.eval(&s, "t"));
    }

    #[test]
    fn test_transition_take_planning() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!("gains_weight", eq!(weight.wrap(), 80.0.wrap()), vec!(a1));
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(weight.wrap(), 82.5.wrap()),
            vec!(a2)
        );
        let s_next_1 = t1.take_planning(&s, "t");
        let s_next_2 = t2.take_planning(&s_next_1, "t");
        let new_state = s.clone()
            .update("weight", 82.5.to_spvalue()) // Pushes 80.0 to history
            .update("weight", 85.0.to_spvalue()); // Pushes 82.5 to history
        assert_eq!(s_next_2, new_state);
    }

    #[test]
    fn test_transition_action_ordering() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!(
            "gains_weight",
            eq!(weight.wrap(), 80.0.wrap()),
            vec!(a1, a2)
        );
        let s_next_1 = t1.take_planning(&s, "t");
        assert_eq!(s_next_1.get_value("weight", "t"), Some(85.0.to_spvalue()));
    }

    #[test]
    #[should_panic]
    fn test_transition_action_ordering_panic() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());
        let t1 = t_plan!(
            "gains_weight",
            eq!(weight.wrap(), 80.0.wrap()),
            vec!(a1, a3, a2)
        );
        let s_next_1 = t1.take_planning(&s, "t");
        assert_eq!(s_next_1.get_value("weight", "t"), Some(87.5.to_spvalue()));
    }

    #[test]
    fn test_transition_action_ordering_fail() {
        let s = State::from_vec(&john_doe());
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let t1 = t_plan!(
            "gains_weight",
            eq!(weight.wrap(), 80.0.wrap()),
            vec!(a2, a1)
        );
        let s_next_1 = t1.take_planning(&s, "t");
        assert_ne!(s_next_1.get_value("weight", "t"), Some(85.0.to_spvalue()));
    }

    #[test]
    fn test_transition_equality() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());

        // Transitions should be equal even if they have a different name
        let t1 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t3 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t4 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let t5 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 85.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        assert_eq!(t1, t2);
        assert_eq!(t1, t3);
        assert_ne!(t3, t4);
        assert_ne!(t4, t5);
    }

    #[test]
    fn test_transition_contained_in_vec() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());

        // Transitions should be equal even if they have a different name
        let t1 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t3 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t4 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let t5 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 85.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let trans2 = vec![t2];
        let trans3 = vec![t3];
        let trans4 = vec![t4.clone()];
        let trans5 = vec![t4, t5];
        assert!(trans2.contains(&t1));
        assert!(trans3.contains(&t1));
        assert!(!trans4.contains(&t1));
        assert!(!trans5.contains(&t1));
    }

    #[test]
    fn test_transition_vec_equality() {
        let weight = fv!("weight");
        let a1 = a!(weight.clone(), 82.5.wrap());
        let a2 = a!(weight.clone(), 85.0.wrap());
        let a3 = a!(weight.clone(), 87.5.wrap());

        // Transitions should be equal even if they have a different name
        let t1 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t2 = t_plan!(
            "gains_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t3 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a1.clone(), a2.clone(), a3.clone())
        );
        let t4 = t_plan!(
            "loses_weight_again",
            eq!(&weight.wrap(), 80.0.wrap()),
            vec!(a3.clone(), a2.clone())
        );
        let trans1 = vec![t1.clone(), t3.clone()];
        let trans2 = vec![t2.clone(), t3.clone()];
        let trans3 = vec![t2.clone(), t4.clone()];
        assert_eq!(trans1, trans2);
        assert_ne!(trans2, trans3);
    }
}

/// Model parsing and rendering.
///
/// `Transition::parse` is how every model in every consuming package is
/// written, and its error handling is deliberately non-fatal: a guard that does
/// not parse becomes `FALSE` and an action that does not parse becomes
/// `Action::empty()`, both with a log line. That means a typo in a model does
/// not stop the process - it silently disables one transition instead, which is
/// worth pinning precisely because it is so quiet.
#[cfg(test)]
mod parse_tests {
    use crate::*;

    const TARGET: &str = "test";

    fn state() -> State {
        State::from_vec(&vec![
            (SPVariable::new("a", SPValueType::Bool), false.to_spvalue()),
            (SPVariable::new("b", SPValueType::Bool), false.to_spvalue()),
            (SPVariable::new("n", SPValueType::Int64), 1.to_spvalue()),
        ])
    }

    #[test]
    fn a_well_formed_transition_parses_into_its_parts() {
        let state = state();
        let transition = Transition::parse(
            "go",
            "var:a == false",
            "var:b == false",
            vec!["var:a <- true"],
            vec!["var:b <- true"],
            &state,
        );

        assert_eq!(transition.name, "go");
        assert_eq!(transition.actions.len(), 1);
        assert_eq!(transition.runner_actions.len(), 1);
        assert!(transition.eval(&state, TARGET));

        let taken = transition.clone().take(&state, TARGET);
        assert_eq!(taken.get_value("a", TARGET), Some(true.to_spvalue()));
        assert_eq!(
            taken.get_value("b", TARGET),
            Some(true.to_spvalue()),
            "take applies the runner actions too"
        );
    }

    /// A guard that does not parse becomes `FALSE`, so the transition can never
    /// fire. The model still loads - the failure is a log line and a dead
    /// transition, not a startup error.
    #[test]
    fn an_unparseable_guard_becomes_false_rather_than_failing_to_load() {
        let state = state();
        let broken = Transition::parse(
            "broken",
            "var:a === maybe",
            "true",
            Vec::<&str>::new(),
            Vec::<&str>::new(),
            &state,
        );

        assert_eq!(broken.guard, Predicate::FALSE);
        assert!(!broken.eval(&state, TARGET), "a dead transition never fires");
    }

    /// Same for the runner guard, which is the half a model author is most
    /// likely to leave malformed since it is often just "true".
    #[test]
    fn an_unparseable_runner_guard_becomes_false() {
        let state = state();
        let broken = Transition::parse(
            "broken",
            "var:a == false",
            "not a predicate at all",
            Vec::<&str>::new(),
            Vec::<&str>::new(),
            &state,
        );

        assert_eq!(broken.runner_guard, Predicate::FALSE);
        assert!(
            !broken.eval(&state, TARGET),
            "eval requires both guards, so a broken runner guard disables it"
        );
    }

    /// An action that does not parse becomes `Action::empty()` - which assigns
    /// `false` to a variable called "empty". It does not remove the action, so
    /// the transition still fires and still does *something*, just not what the
    /// model said.
    #[test]
    fn an_unparseable_action_becomes_the_empty_action() {
        let state = state();
        let broken = Transition::parse(
            "broken",
            "true",
            "true",
            vec!["var:a <<== nonsense"],
            vec!["also nonsense"],
            &state,
        );

        assert_eq!(broken.actions.len(), 1, "the action is replaced, not dropped");
        assert_eq!(broken.actions[0], Action::empty());
        assert_eq!(broken.runner_actions[0], Action::empty());

        // And the transition still evaluates as enabled, which is the part that
        // makes this quiet: a model with a typo'd action runs.
        assert!(broken.eval(&state, TARGET));
    }

    /// A well-formed action next to a broken one is unaffected.
    #[test]
    fn a_broken_action_does_not_take_its_neighbours_with_it() {
        let state = state();
        let transition = Transition::parse(
            "mixed",
            "true",
            "true",
            vec!["var:a <- true", "garbage", "var:b <- true"],
            Vec::<&str>::new(),
            &state,
        );

        assert_eq!(transition.actions.len(), 3);
        assert_eq!(transition.actions[1], Action::empty());

        let mut taken = state.clone();
        // `empty` is not in the state, so applying it would panic - check the
        // two good ones by evaluating them directly instead.
        transition.actions[0].assign_mut(&mut taken, TARGET);
        transition.actions[2].assign_mut(&mut taken, TARGET);
        assert_eq!(taken.get_value("a", TARGET), Some(true.to_spvalue()));
        assert_eq!(taken.get_value("b", TARGET), Some(true.to_spvalue()));
    }

    /// The empty transition is the "never fires, does nothing" element.
    #[test]
    fn the_empty_transition_never_fires() {
        let state = state();
        let empty = Transition::empty();

        assert_eq!(empty.name, "empty");
        assert_eq!(empty.guard, Predicate::FALSE);
        assert!(!empty.eval(&state, TARGET));
        assert!(empty.actions.is_empty());
        assert!(empty.get_all_var_keys().is_empty());
    }

    /// The default is the opposite: it always fires and does nothing, which is
    /// what makes `..Default::default()` usable when building an `Operation` in
    /// a test or a fixture.
    #[test]
    fn the_default_transition_always_fires_and_does_nothing() {
        let state = state();
        let default = Transition::default();

        assert_eq!(default.name, "unknown");
        assert_eq!(default.guard, Predicate::TRUE);
        assert_eq!(default.runner_guard, Predicate::TRUE);
        assert!(default.eval(&state, TARGET));
        assert_eq!(default.clone().take(&state, TARGET), state);
    }

    /// `Display` is what every "operation disabled, please satisfy" message is
    /// built from, so it has to render the guard and all the actions.
    #[test]
    fn display_renders_the_guard_and_every_action() {
        let state = state();

        let none = Transition::parse(
            "none",
            "var:a == false",
            "true",
            Vec::<&str>::new(),
            Vec::<&str>::new(),
            &state,
        );
        assert_eq!(none.to_string(), "none: a = false / []");

        let one = Transition::parse(
            "one",
            "var:a == false",
            "true",
            vec!["var:a <- true"],
            Vec::<&str>::new(),
            &state,
        );
        assert_eq!(one.to_string(), "one: a = false / [a <= true]");

        let many = Transition::parse(
            "many",
            "var:a == false",
            "true",
            vec!["var:a <- true", "var:b <- true"],
            Vec::<&str>::new(),
            &state,
        );
        assert_eq!(many.to_string(), "many: a = false / [a <= true, b <= true]");
    }

    /// The owned `take` must not disturb the state it was given - the runners
    /// rely on being able to diff before against after.
    #[test]
    fn the_owned_take_leaves_the_original_state_alone() {
        let state = state();
        let transition = Transition::parse(
            "go",
            "true",
            "true",
            vec!["var:n <- 5"],
            Vec::<&str>::new(),
            &state,
        );

        let after = transition.take(&state, TARGET);
        assert_eq!(state.get_value("n", TARGET), Some(1.to_spvalue()));
        assert_eq!(after.get_value("n", TARGET), Some(5.to_spvalue()));
    }
}
