//! Shorthand constructor for [`SPAssignment`](crate::SPAssignment), the
//! variable-and-value pair a [`State`](crate::State) is built from.

/// Builds `SPAssignment::new(var, value)` - one entry of a state.
///
/// ```
/// use micro_sp::*;
///
/// let mut state = State::new();
/// state.add_mut(assign!(v!("pos"), "a".to_spvalue()), "docs");
/// assert_eq!(state.get_value("pos", "docs"), Some("a".to_spvalue()));
/// ```
#[macro_export]
macro_rules! assign {
    ($a:expr, $b:expr) => {
        SPAssignment::new($a, $b)
    };
}
