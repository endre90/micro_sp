use std::fmt;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub name: String,
    pub goal: Predicate,
    pub plan: Vec<Operation>,
    pub time_step: u32,
    pub state: PlanState,
    pub time: std::time::Duration
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanState {
    Initial,
    Executing,
    Paused,
    Failed,
    Completed,
    Cancelled,
    UNKNOWN,
}

impl Default for PlanState {
    fn default() -> Self {
        PlanState::UNKNOWN
    }
}

impl PlanState {
    pub fn from_str(x: &str) -> PlanState {
        match x {
            "initial" => PlanState::Initial,
            "executing" => PlanState::Executing,
            "paused" => PlanState::Paused,
            "failed" => PlanState::Failed,
            "completed" => PlanState::Completed,
            "cancelled" => PlanState::Cancelled,
            _ => PlanState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match  self {
            PlanState::Initial => "initial".to_spvalue(),
            PlanState::Executing => "executing".to_spvalue(),
            PlanState::Paused => "paused".to_spvalue(),
            PlanState::Failed => "failed".to_spvalue(),
            PlanState::Completed => "completed".to_spvalue(),
            PlanState::Cancelled => "completed".to_spvalue(),
            PlanState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl fmt::Display for PlanState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlanState::UNKNOWN => write!(f, "UNKNOWN"),
            PlanState::Initial => write!(f, "initial"),
            PlanState::Executing => write!(f, "executing"),
            PlanState::Paused => write!(f, "paused"),
            PlanState::Failed => write!(f, "failed"),
            PlanState::Completed => write!(f, "completed"),
            PlanState::Cancelled => write!(f, "cancelled"),
        }
    }
}