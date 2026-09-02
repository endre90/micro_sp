//! The runtime: the tasks that actually execute a [`crate::Model`].
//!
//! [`main_runner`] spawns one task per concern - a planner, a plan runner, a SOP
//! runner, automatic operation and transition runners, timers and a transform
//! interface - and they coordinate only through shared state in Redis, using the
//! keys in [`runner_keys`] and the lifecycle enums in [`runner_states`].

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
/// Executing several SOPs at the same time.
pub mod sop_multi;
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
