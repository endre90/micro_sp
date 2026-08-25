//! `micro_sp` is a Sequence Planner runtime for controlling automation systems.
//!
//! You describe *what the system can do* as a [`Model`] of guarded operations,
//! and the runtime figures out and executes *what it should do now*. The whole
//! system state lives in Redis, so several processes - and any external tool -
//! can observe and drive the same system.
//!
//! # The pieces
//!
//! - **State.** [`State`] maps variable names to [`SPValue`]s. Every value is
//!   also allowed to be `UNKNOWN`, which is what a freshly started system reads
//!   before anything has measured it.
//! - **Behaviour.** A [`Transition`] is a guard plus a set of assignments. An
//!   [`Operation`] wraps transitions into a lifecycle (initial → executing →
//!   completed, with timeouts, retries and cancellation). A [`SOPStruct`]
//!   sequences operations into a procedure.
//! - **A [`Model`]** collects the operations, automatic transitions and SOPs a
//!   system has.
//! - **Runners.** [`main_runner`] spawns the tasks that actually execute a
//!   model: a planner, a plan runner, a SOP runner, automatic operation and
//!   transition runners, timers and a transform interface.
//! - **Persistence.** [`StateManager`] reads and writes state through a
//!   [`ConnectionManager`], and [`TransformsManager`] does the same for 3D
//!   frames.
//!
//! # Modelling, without a runtime
//!
//! The modelling layer is pure and needs no Redis, which makes it easy to test:
//!
//! ```
//! use micro_sp::*;
//!
//! // A domain with one variable: where the robot is.
//! let mut state = State::new();
//! state.add_mut(
//!     SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
//!     "docs",
//! );
//!
//! // "When pos is a, set it to b."
//! let move_to_b = Transition::parse(
//!     "move_to_b",
//!     "var:pos == a",        // guard: when this holds ...
//!     "true",                // runner guard: ... and the operator allows it
//!     vec!["var:pos <- b"],  // effects
//!     Vec::<&str>::new(),
//!     &state,
//! );
//!
//! assert!(move_to_b.eval(&state, "docs"));
//! let state = move_to_b.take(&state, "docs");
//! assert_eq!(state.get_value("pos", "docs"), Some("b".to_spvalue()));
//! ```
//!
//! # Running a model
//!
//! [`main_runner`] needs a Redis instance (`docker run -p 6379:6379 -d redis`):
//!
//! ```no_run
//! use micro_sp::*;
//! use std::sync::Arc;
//!
//! # async fn example(model: Model) {
//! let connection_manager = Arc::new(ConnectionManager::new().await);
//!
//! // Seed Redis with the model's variables, then hand it to the runners.
//! main_runner(&"sp".to_string(), model, 3, &connection_manager).await;
//! # }
//! ```
//!
//! Ask the system for something by writing a goal predicate into the state; the
//! planner finds a sequence of operations that reaches it and the plan runner
//! executes them. See [`running`] for the individual runners.
//!
//! # Environment variables
//!
//! | Variable | Effect |
//! |---|---|
//! | `REDIS_HOST` / `REDIS_PORT` | Where to reach Redis (default `127.0.0.1:6379`). |
//! | `MICRO_SP_TICK_INTERVAL_MS` | Overrides the runners' tick period. |
//! | `MICRO_SP_READ_FULL_STATE` | Makes runners read the whole keyspace each tick. Debugging escape hatch; slow. |
//! | `MICRO_SP_ACTIVITY_LOG_DIR` | Turns on the on-disk [`activity_log`] and says where to write it. |
//! | `RUST_LOG` / `LOG_SHOW_TIME` | Console logging, via [`initialize_env_logger`]. |

/// Default execution deadline for an operation, in milliseconds.
///
/// Applies only to operations created without an explicit
/// `timeout_executing_ms`; see [`Operation::new`].
pub static MAX_ALLOWED_OPERATION_DURATION_MS: i64 = 600000; // milliseconds

/// How many times the planner may replan for one goal before giving up.
pub static MAX_REPLAN_RETRIES: i64 = 3;

/// Recursion depth ceiling for the SOP tree walk, guarding against a
/// pathological or cyclic model.
pub static MAX_RECURSION_DEPTH: u64 = 1000;

/// The alphabet behind every generated id in the crate.
///
/// Used as `nanoid::nanoid!(10, &NANOID_ALPHABET)` to give each activated
/// operation, SOP and plan a unique instance name. [`running::plan_runner`]
/// additionally relies on `NANOID_ALPHABET.contains(&c)` to tell which trailing
/// characters of a step name are a generated suffix rather than part of the
/// operation's name - so every character in it must stay distinct and
/// alphanumeric, or both uses break silently.
pub const NANOID_ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

pub mod core;
pub use crate::core::sp_assignment::*;
pub use crate::core::sp_state::*;
pub use crate::core::sp_value::*;
pub use crate::core::sp_variable::*;
pub use crate::core::sp_wrapped::*;

pub mod modelling;
pub use crate::modelling::action::*;
pub use crate::modelling::model::*;
pub use crate::modelling::operation::*;
pub use crate::modelling::parser::*;
pub use crate::modelling::predicate::*;
pub use crate::modelling::sops::*;
pub use crate::modelling::transition::*;

pub mod planning;
pub use crate::planning::operation::*;
pub use crate::planning::transition::*;

pub mod running;
pub use crate::running::auto_runner::*;
pub use crate::running::main_runner::*;
pub use crate::running::plan_runner::*;
pub use crate::running::planner_ticker::*;
pub use crate::running::runner_keys::*;
pub use crate::running::tick::*;
pub use crate::running::runner_states::*;
pub use crate::running::sop_runner::*;
pub use crate::running::state_init::*;
pub use crate::running::time_runner::*;

pub mod management;
pub use crate::management::connection::*;
pub use crate::management::state::*;
pub use crate::management::transforms::*;

pub mod transforms;
pub use crate::transforms::cycles::*;
pub use crate::transforms::loading::*;
pub use crate::transforms::lookup::*;
pub use crate::transforms::treeviz::*;

pub mod utils;
pub use crate::utils::info_logger::*;
pub use crate::utils::metadata::*;

/// The on-disk activity log. Deliberately re-exported as a *module* rather than
/// glob-imported: its API is `init`, `flush`, `is_enabled`, `log_operation`,
/// ... - names far too generic to put in the crate root, where they would
/// collide with anything a consuming package defines. Call it as
/// `micro_sp::activity_log::init_from_env()`.
pub use crate::utils::activity_log;
pub use crate::utils::activity_log::{
    ActivityKind, ActivityLogConfig, ActivityRecord, ActivityWriter,
};

pub mod macros;
#[allow(unused_imports)]
pub use crate::macros::action::*;
#[allow(unused_imports)]
pub use crate::macros::predicate::*;
#[allow(unused_imports)]
pub use crate::macros::sp_assignment::*;
#[allow(unused_imports)]
pub use crate::macros::sp_variable::*;
#[allow(unused_imports)]
pub use crate::macros::transition::*;

#[cfg(test)]
mod lib_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn nanoid_alphabet_is_62_distinct_alphanumeric_chars() {
        assert_eq!(NANOID_ALPHABET.len(), 62);

        let unique: HashSet<char> = NANOID_ALPHABET.iter().copied().collect();
        assert_eq!(
            unique.len(),
            NANOID_ALPHABET.len(),
            "every character in the alphabet must be distinct, or nanoid's \
             collision-resistance and plan_runner's suffix detection both weaken"
        );

        assert!(
            NANOID_ALPHABET.iter().all(|c| c.is_ascii_alphanumeric()),
            "plan_runner strips id suffixes assuming this alphabet is plain alphanumeric"
        );
    }

    /// These are read by `Operation::start`/friends as *defaults* that only
    /// apply when an operation/config does not override them - pin the
    /// documented values so a change here is a deliberate policy change, not
    /// an accidental one.
    #[test]
    fn tunable_limits_hold_their_documented_defaults() {
        assert_eq!(MAX_ALLOWED_OPERATION_DURATION_MS, 600_000);
        assert_eq!(MAX_REPLAN_RETRIES, 3);
        assert_eq!(MAX_RECURSION_DEPTH, 1000);
    }
}
