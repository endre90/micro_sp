//! Shorthand constructor for [`Action`](crate::Action)s, the assignments a
//! [`Transition`](crate::Transition) performs when it is taken.

/// Builds `Action::new(var, value)` - "assign this to that variable".
///
/// The right-hand side is [`SPWrapped`](crate::SPWrapped), so it may be a
/// literal (`"b".wrap()`) or another variable (`other.wrap()`).
///
/// ```
/// use micro_sp::*;
///
/// let pos = v!("pos");
/// let state = State::from_vec(&vec![(pos.clone(), "a".to_spvalue())]);
///
/// let go_to_b = a!(pos, "b".wrap());
/// let after = go_to_b.assign(&state, "docs");
/// assert_eq!(after.get_value("pos", "docs"), Some("b".to_spvalue()));
/// ```
#[macro_export]
macro_rules! a {
    ($a:expr, $b:expr) => {
        Action::new($a.clone(), $b.clone())
    };
}
