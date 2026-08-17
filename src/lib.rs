pub static MAX_ALLOWED_OPERATION_DURATION_MS: i64 = 600000; // milliseconds
pub static MAX_REPLAN_RETRIES: i64 = 3;
pub static MAX_RECURSION_DEPTH: u64 = 1000;

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
// pub use crate::running::goal_runner::*;
// tests
// pub use crate::running::goal_scheduler::*;
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
// pub use crate::management::snapshot::*;
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
// pub use crate::utils::op_logger::*;

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

/// `NANOID_ALPHABET` backs every id generated with `nanoid::nanoid!(10,
/// &NANOID_ALPHABET)` across the runners (`auto_runner`, `goal_runner`,
/// `sop_runner`, `planner_ticker`, transform loading, ...), and
/// `running/plan_runner.rs` separately relies on
/// `NANOID_ALPHABET.contains(&c)` to recognise which suffix characters of a
/// step name are a generated id versus part of the operation name. Both uses
/// silently break the same way if the alphabet ever gained a duplicate
/// character or shrank: nanoid's collision-resistance and plan_runner's
/// suffix detection both assume every character in it is distinct.
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
