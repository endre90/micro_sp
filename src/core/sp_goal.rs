use serde::{Deserialize, Serialize};
use std::{fmt, time::SystemTime};

use crate::*;

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
    High,
    Normal,
    Low,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CurrentGoalState {
    Empty,
    Initial,
    Executing,
    Paused,
    Failed,
    Cancelled,
    Completed
}

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

impl fmt::Display for GoalPriority {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GoalPriority::High => write!(fmtr, "high"),
            GoalPriority::Normal => write!(fmtr, "normal"),
            GoalPriority::Low => write!(fmtr, "low"),
        }
    }
}

impl CurrentGoalState {
    pub fn from_str(x: &str) -> CurrentGoalState {
        match x {
            "empty" => CurrentGoalState::Empty,
            "initial" => CurrentGoalState::Initial,
            "executing" => CurrentGoalState::Executing,
            "failed" => CurrentGoalState::Failed,
            "paused" => CurrentGoalState::Paused,
            "cancelled" => CurrentGoalState::Cancelled,
            "completed" => CurrentGoalState::Completed,
            _ => {
                // log::error!(target: &&format!("goal_priority"), 
                //     "Unknown goal state {}, defaulting to empty.", x);
                    CurrentGoalState::Empty
            }
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        self.to_string().to_spvalue()
    }
}

impl fmt::Display for CurrentGoalState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrentGoalState::Empty => write!(f, "empty"),
            CurrentGoalState::Initial => write!(f, "initial"),
            CurrentGoalState::Executing => write!(f, "executing"),
            CurrentGoalState::Cancelled => write!(f, "cancelled"),
            CurrentGoalState::Paused => write!(f, "paused"),
            CurrentGoalState::Failed => write!(f, "failed"),
            CurrentGoalState::Completed => write!(f, "completed")
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, Serialize, Deserialize)]
pub struct GoalLog {
    pub time: SystemTime,
    pub state: State,
    pub operation: Operation,
    pub operation_state: OperationState,
}
