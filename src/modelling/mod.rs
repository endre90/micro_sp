//! The modelling layer: how a system's behaviour is described.
//!
//! [`Predicate`](predicate::Predicate)s guard [`Transition`](transition::Transition)s,
//! transitions carry [`Action`](action::Action)s, [`Operation`](operation::Operation)s
//! wrap transitions into a lifecycle, [`SOP`](sops::SOP)s sequence operations, and a
//! [`Model`](model::Model) collects the lot. Everything here is pure - no Redis, no
//! runtime - so a model can be built and evaluated in a plain unit test.

pub mod action;
pub mod sops;
pub mod operation;
pub mod model;
pub mod parser;
pub mod predicate;
pub mod transition;
