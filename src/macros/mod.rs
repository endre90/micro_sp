//! Shorthand constructors for the modelling types.
//!
//! Every macro here is `#[macro_export]`ed, so it lives at the crate root:
//! `v!`/`bv!`/`iv!`/... for variables, `assign!` for assignments,
//! `eq!`/`and!`/... for predicates, `a!` for actions and `t!` for transitions.

pub mod action;
pub mod predicate;
pub mod sp_assignment;
pub mod sp_variable;
pub mod transition;
