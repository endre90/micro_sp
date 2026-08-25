//! The runtime: the tasks that actually execute a [`crate::Model`].
//!
//! There is one concern per module - a planner, a plan runner, a SOP runner,
//! automatic operation and transition runners, timers and a transform interface
//! - and they coordinate only through shared state in Redis, using the keys in
//! [`runner_keys`] and the lifecycle enums in [`runner_states`].
//!
//! [`main_runner`] drives them all from a single loop, [`sequential`], so that
//! each tick reads one snapshot and publishes one diff. Every runner is also a
//! standalone task that can be spawned on its own, which is what
//! `MICRO_SP_SEQUENTIAL=0` goes back to - at the cost of making
//! read-modify-write across runners non-atomic again.

/// Automatic transitions and automatic operations.
pub mod auto_runner;
/// Executing the operations of the current plan.
pub mod plan_runner;
/// Planning towards the current goal.
pub mod planner_ticker;
/// Accepting, queueing and tracking goals.
pub mod goal_runner;
/// Executing SOPs.
pub mod sop_runner;
/// Every runner in one loop, on one snapshot, with one writer.
pub mod sequential;
/// Spawning and supervising all of the above.
pub mod main_runner;
/// Generating the initial state a model needs.
pub mod state_init;
/// The state keys each runner reads and writes.
pub mod runner_keys;
/// Tick periods and the clock the runners age their timers with.
pub mod tick;
/// The lifecycle enums shared between runners.
pub mod runner_states;
/// Timers a model can start, stop and read.
pub mod time_runner;
/// The operation state machine every runner drives.
pub mod process_operation;
