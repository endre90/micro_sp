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
pub use crate::running::runner_states::*;
pub use crate::running::sop_runner::*;
pub use crate::running::state_init::*;
pub use crate::running::time_runner::*;

pub mod management;
pub use crate::management::connection::*;
pub use crate::management::snapshot::*;
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
pub use crate::utils::op_logger::*;

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
