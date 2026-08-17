//! The value and state types everything else is built on.
//!
//! An [`SPVariable`](crate::SPVariable) names a typed slot, an
//! [`SPValue`](crate::SPValue) is what fits in it, an
//! [`SPAssignment`](crate::SPAssignment) pairs the two, and a
//! [`State`](crate::State) is a map of assignments. [`SPWrapped`](crate::SPWrapped)
//! is the operand form used inside predicates and actions.

pub mod sp_assignment;
pub mod sp_state;
pub mod sp_value;
pub mod sp_variable;
pub mod sp_wrapped;
