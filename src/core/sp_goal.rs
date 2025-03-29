use serde::{Deserialize, Serialize};
use std::{fmt, time::SystemTime};

use crate::*;

// #[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
// pub enum ScheduledGoalPriority {
//     Executing,
//     HighQueued,
//     HighIncoming, // Lower than above because we don't want to change the order
//     NormalQueued,
//     NormalIncoming,
//     LowQueued,
//     LowIncoming,
//     Concluded, // Completed, Aborted, Failed, Timeodout, etc.
// }

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
    High,
    Normal,
    Low,
}

// impl ScheduledGoalPriority {
//     pub fn from_int(x: &i64) -> ScheduledGoalPriority {
//         match x {
//             0 => ScheduledGoalPriority::Executing,
//             1 => ScheduledGoalPriority::HighQueued,
//             2 => ScheduledGoalPriority::HighIncoming,
//             3 => ScheduledGoalPriority::NormalQueued,
//             4 => ScheduledGoalPriority::NormalIncoming,
//             5 => ScheduledGoalPriority::LowQueued,
//             6 => ScheduledGoalPriority::LowIncoming,
//             7 => ScheduledGoalPriority::Concluded,
//             _ => {
//                 log::error!(target: &&format!("scheduled_goal_priority"), 
//                     "Priority out of range (0..7), defaulting to low_incoming.");
//                 ScheduledGoalPriority::LowIncoming
//             }
//         }
//     }

//     pub fn to_int(&self) -> i64 {
//         match self {
//             ScheduledGoalPriority::Executing => 0,
//             ScheduledGoalPriority::HighQueued => 1,
//             ScheduledGoalPriority::HighIncoming => 2,
//             ScheduledGoalPriority::NormalQueued => 3,
//             ScheduledGoalPriority::NormalIncoming => 4,
//             ScheduledGoalPriority::LowQueued => 5,
//             ScheduledGoalPriority::LowIncoming => 6,
//             ScheduledGoalPriority::Concluded => 7,
//         }
//     }

//     pub fn from_str(x: &str) -> ScheduledGoalPriority {
//         match x {
//             "executing" => ScheduledGoalPriority::Executing,
//             "high_queued" => ScheduledGoalPriority::HighQueued,
//             "high" => ScheduledGoalPriority::HighIncoming,
//             "normal_queued" => ScheduledGoalPriority::NormalQueued,
//             "normal" => ScheduledGoalPriority::NormalIncoming,
//             "low_queued" => ScheduledGoalPriority::LowQueued,
//             "low" => ScheduledGoalPriority::LowIncoming,
//             "concluded" => ScheduledGoalPriority::Concluded,
//             _ => {
//                 log::error!(target: &&format!("scheduled_goal_priority"), 
//                     "Unknown priority {}, defaulting to low.", x);
//                 ScheduledGoalPriority::LowIncoming
//             }
//         }
//     }
// }

impl GoalPriority {
    pub fn from_int(x: &i64) -> GoalPriority {
        match x {
            1 => GoalPriority::High,
            2 => GoalPriority::Normal,
            3 => GoalPriority::Low,
            _ => {
                log::error!(target: &&format!("goal_priority"), 
                    "Priority out of range [1, 2, 3], defaulting to low.");
                GoalPriority::Low
            }
        }
    }

    pub fn to_int(&self) -> i64 {
        match self {
            GoalPriority::High => 1,
            GoalPriority::Normal => 2,
            GoalPriority::Low => 3,
        }
    }

    pub fn from_str(x: &str) -> GoalPriority {
        match x {
            "high" => GoalPriority::High,
            "normal" => GoalPriority::Normal,
            "low" => GoalPriority::Low,
            _ => {
                log::error!(target: &&format!("goal_priority"), 
                    "Unknown priority {}, defaulting to low.", x);
                    GoalPriority::Low
            }
        }
    }
}

// impl fmt::Display for ScheduledGoalPriority {
//     fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             ScheduledGoalPriority::Executing => write!(fmtr, "executing"),
//             ScheduledGoalPriority::HighQueued => write!(fmtr, "high_queued"),
//             ScheduledGoalPriority::HighIncoming => write!(fmtr, "high"),
//             ScheduledGoalPriority::NormalQueued => write!(fmtr, "normal_queued"),
//             ScheduledGoalPriority::NormalIncoming => write!(fmtr, "normal"),
//             ScheduledGoalPriority::LowQueued => write!(fmtr, "low_queued"),
//             ScheduledGoalPriority::LowIncoming => write!(fmtr, "low"),
//             ScheduledGoalPriority::Concluded => write!(fmtr, "concluded"),
//         }
//     }
// }

impl fmt::Display for GoalPriority {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GoalPriority::High => write!(fmtr, "high"),
            GoalPriority::Normal => write!(fmtr, "normal"),
            GoalPriority::Low => write!(fmtr, "low"),
        }
    }
}

pub struct ErrorRecord {
    pub time: SystemTime,
    pub state: State,
    pub operation: Operation,
    pub operation_state: OperationState,
}

pub struct Goal {
    pub name: String,
    pub goal_string: String,
    pub priority: i64,
    pub time_arrived: SystemTime,
    // pub time_started: SystemTime,
    // pub time_finished: SystemTime,
    // pub planning_time: Duration,
    // pub execution_time: Duration,
    // pub plan: Option<ArrayOrUnknown>,
    // pub nr_failures: u32,
    // pub nr_timeouts: u32,
    // pub nr_replans: u32,
    // pub failures: Vec<ErrorRecord>,
    // pub timeouts: Vec<ErrorRecord>,
    // pub execution_path: Vec<String>, // both planned and autos
}
