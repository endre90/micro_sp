//! Emulated hardware the examples drive.
//!
//! Each emulator is a tick loop over Redis with exactly the shape a real driver
//! has: read a `*_request_trigger`, act on the `*_command_*` variables, write
//! back a `*_request_state` of `succeeded` or `failed`. Nothing in `micro_sp`
//! knows these are fake - swapping in a real robot means replacing this loop,
//! not the model.
//!
//! How long they take and whether they fail is itself state, set through the
//! `*_emulate_*` / `*_emulated_*` variables; see the constants in
//! [`super`](crate::common).

pub mod gantry;
pub mod robot;
