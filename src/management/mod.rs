//! Redis persistence for the runtime.
//!
//! [`connection`] owns the process-wide Redis handle, [`state`] reads and
//! writes state variables through it, and [`transforms`] does the same for the
//! 3D frames of the transform tree. Everything a runner persists goes through
//! one of these three.

pub mod connection;
pub mod state;
pub mod transforms;
