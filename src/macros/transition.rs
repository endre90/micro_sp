//! Shorthand constructors for [`Transition`](crate::Transition)s - a guard plus
//! the assignments taken when it fires.

/// Builds a planning-only [`Transition`](crate::Transition) from an
/// already-built [`Predicate`](crate::Predicate) and a slice of
/// [`Action`](crate::Action)s.
///
/// Shorthand for `Transition::new(name, guard, Predicate::TRUE, actions, vec![])`:
/// the runner guard is always `TRUE` and there are no runner actions, because
/// the planner never executes anything.
#[macro_export]
macro_rules! t_plan {
    ($a:expr, $b:expr, $c:expr) => {
        Transition::new(
            $a,
            $b.clone(),
            Predicate::TRUE,
            $c.iter().map(|x| x.to_owned()).collect::<Vec<Action>>(),
            Vec::<Action>::new(),
        )
    };
}

/// Builds a [`Transition`](crate::Transition) from the string DSL, i.e.
/// `t!(name, guard, runner_guard, actions, runner_actions, state)`.
///
/// Same syntax as [`Transition::parse`](crate::Transition::parse), but the
/// guards and actions are parsed with `unwrap`, so a malformed model **panics
/// here** instead of being logged and disabled. `state` only supplies the
/// variable declarations.
///
/// ```
/// use micro_sp::*;
///
/// let mut state = State::new();
/// state.add_mut(
///     SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
///     "docs",
/// );
///
/// let move_to_b = t!(
///     "move_to_b",
///     "var:pos == a",        // guard
///     "true",                // runner guard
///     vec!("var:pos <- b"),  // actions
///     Vec::<&str>::new(),    // runner actions
///     &state
/// );
///
/// assert!(move_to_b.eval(&state, "docs"));
/// let state = move_to_b.take(&state, "docs");
/// assert_eq!(state.get_value("pos", "docs"), Some("b".to_spvalue()));
/// ```
#[macro_export]
macro_rules! t {
    ($name:expr, $guard:expr, $runner_guard:expr, $actions:expr, $runner_actions:expr, $state:expr) => {
        Transition::new(
            $name,
            pred_parser::pred($guard.clone(), $state).unwrap(),
            pred_parser::pred($runner_guard.clone(), $state).unwrap(),
            $actions
                .iter()
                .map(|action| pred_parser::action(action.to_owned(), $state).unwrap())
                .collect::<Vec<Action>>(),
            $runner_actions
                .iter()
                .map(|action| pred_parser::action(action.to_owned(), $state).unwrap())
                .collect::<Vec<Action>>(),
        )
    };
}
