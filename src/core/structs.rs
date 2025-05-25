use std::fmt;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub name: String,
    pub goal: Predicate,
    pub plan: Vec<Operation>,
    pub time_step: u32,
    pub state: PlanState,
    pub time: std::time::Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanState {
    Initial,
    Executing,
    Paused,
    Failed,
    NotFound,
    Completed,
    // Cancelled,
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
            "not_found" => PlanState::NotFound,
            "completed" => PlanState::Completed,
            // "cancelled" => PlanState::Cancelled,
            _ => PlanState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match self {
            PlanState::Initial => "initial".to_spvalue(),
            PlanState::Executing => "executing".to_spvalue(),
            PlanState::Paused => "paused".to_spvalue(),
            PlanState::Failed => "failed".to_spvalue(),
            PlanState::NotFound => "not_found".to_spvalue(),
            PlanState::Completed => "completed".to_spvalue(),
            // PlanState::Cancelled => "cancelled".to_spvalue(),
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
            PlanState::NotFound => write!(f, "not_found"),
            PlanState::Completed => write!(f, "completed"),
            // PlanState::Cancelled => write!(f, "cancelled"),
        }
    }
}



pub enum ServiceRequestState {
    UNKNOWN,
    Initial,
    Succeeded,
    Failed,
}

impl Default for ServiceRequestState {
    fn default() -> Self {
        ServiceRequestState::UNKNOWN
    }
}

impl ServiceRequestState {
    pub fn from_str(x: &str) -> ServiceRequestState {
        match x {
            "initial" => ServiceRequestState::Initial,
            "succeeded" => ServiceRequestState::Succeeded,
            "failed" => ServiceRequestState::Failed,
            _ => ServiceRequestState::UNKNOWN,
        }
    }
}

impl fmt::Display for ServiceRequestState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServiceRequestState::Initial => write!(f, "initial"),
            ServiceRequestState::Succeeded => write!(f, "succeeded"),
            ServiceRequestState::Failed => write!(f, "failed"),
            ServiceRequestState::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}


pub enum ActionRequestState {
    UNKNOWN,
    Initial,
    Executing,
    Succeeded,
    Failed,
}

impl Default for ActionRequestState {
    fn default() -> Self {
        ActionRequestState::UNKNOWN
    }
}

impl ActionRequestState {
    pub fn from_str(x: &str) -> ActionRequestState {
        match x {
            "initial" => ActionRequestState::Initial,
            "executing" => ActionRequestState::Executing,
            "succeeded" => ActionRequestState::Succeeded,
            "failed" => ActionRequestState::Failed,
            _ => ActionRequestState::UNKNOWN,
        }
    }
}

impl fmt::Display for ActionRequestState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActionRequestState::Initial => write!(f, "initial"),
            ActionRequestState::Executing => write!(f, "executing"),
            ActionRequestState::Succeeded => write!(f, "succeeded"),
            ActionRequestState::Failed => write!(f, "failed"),
            ActionRequestState::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}
