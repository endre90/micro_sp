//! Shorthand constructors for [`Predicate`](crate::Predicate)s.
//!
//! These build the guard expressions a [`Transition`](crate::Transition) is
//! evaluated against. Operands are [`SPWrapped`](crate::SPWrapped), so either
//! side can be a variable (`var.wrap()`) or a literal (`5.wrap()`).

/// Builds `Predicate::EQ(a, b)` - "these two are equal".
///
/// Both operands are [`SPWrapped`](crate::SPWrapped).
///
/// ```
/// use micro_sp::*;
///
/// let pos = v!("pos");
/// let state = State::from_vec(&vec![(pos.clone(), "a".to_spvalue())]);
///
/// let at_a = eq!(pos.wrap(), "a".wrap());
/// assert!(at_a.eval(&state, "docs"));
///
/// let elsewhere = not!(at_a);
/// assert!(!elsewhere.eval(&state, "docs"));
/// ```
#[macro_export]
macro_rules! eq {
    ($a:expr, $b:expr) => {
        Predicate::EQ($a.clone(), $b.clone())
    };
}

/// Builds `Predicate::NEQ(a, b)` - "these two differ".
#[macro_export]
macro_rules! neq {
    ($a:expr, $b:expr) => {
        Predicate::NEQ($a.clone(), $b.clone())
    };
}

/// Builds `Predicate::NOT(p)` - the negation of one predicate.
#[macro_export]
macro_rules! not {
    ($a:expr) => {
        Predicate::NOT(Box::new($a.clone()))
    };
}

/// Builds `Predicate::AND(..)` from a `Vec<Predicate>` or from a
/// comma-separated list of predicates.
///
/// `and!(p, q)` and `and!(vec![p, q])` produce the same conjunction.
#[macro_export]
macro_rules! and {
    ($a:expr) => {
        Predicate::AND($a.to_owned())
    };
    ($( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.clone());
            )*
            Predicate::AND(temp_vec)
        }
    };
}

/// Builds `Predicate::OR(..)` from a `Vec<Predicate>` or from a
/// comma-separated list of predicates.
///
/// `or!(p, q)` and `or!(vec![p, q])` produce the same disjunction.
#[macro_export]
macro_rules! or {
    ($a:expr) => {
        Predicate::OR($a.to_owned())
    };
    ($( $x:expr ),* ) => {
        {
            let mut temp_vec: Vec<Predicate> = Vec::new();
            $(
                temp_vec.push($x.clone());
            )*
            Predicate::OR(temp_vec)
        }
    };
}
