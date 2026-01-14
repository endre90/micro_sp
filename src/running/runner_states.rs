use std::fmt;

use serde::{Deserialize, Serialize};

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
    // Empty,
    Initial,
    Executing,
    // Paused,
    Failed,
    // NotFound,
    Completed,
    Cancelled,
    UNKNOWN,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SOPState {
    Initial,
    Executing,
    Fatal,
    Completed,
    Cancelled,
    UNKNOWN,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlannerState {
    Found,
    NotFound,
    Ready,
    UNKNOWN,
}

impl Default for PlanState {
    fn default() -> Self {
        PlanState::UNKNOWN
    }
}

impl Default for SOPState {
    fn default() -> Self {
        SOPState::UNKNOWN
    }
}

impl Default for PlannerState {
    fn default() -> Self {
        PlannerState::UNKNOWN
    }
}

impl PlanState {
    pub fn from_str(x: &str) -> PlanState {
        match x {
            // "empty" => PlanState::Empty,
            "initial" => PlanState::Initial,
            "executing" => PlanState::Executing,
            // "paused" => PlanState::Paused,
            "failed" => PlanState::Failed,
            // "not_found" => PlanState::NotFound,
            "completed" => PlanState::Completed,
            // "cancelled" => PlanState::Cancelled,
            _ => PlanState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match self {
            // PlanState::Empty => "empty".to_spvalue(),
            PlanState::Initial => "initial".to_spvalue(),
            PlanState::Executing => "executing".to_spvalue(),
            // PlanState::Paused => "paused".to_spvalue(),
            PlanState::Failed => "failed".to_spvalue(),
            // PlanState::NotFound => "not_found".to_spvalue(),
            PlanState::Completed => "completed".to_spvalue(),
            PlanState::Cancelled => "cancelled".to_spvalue(),
            PlanState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl SOPState {
    pub fn from_str(x: &str) -> SOPState {
        match x {
            // "empty" => SOPState::Empty,
            "initial" => SOPState::Initial,
            "executing" => SOPState::Executing,
            // "paused" => SOPState::Paused,
            "fatal" => SOPState::Fatal,
            // "not_found" => SOPState::NotFound,
            "completed" => SOPState::Completed,
            // "advanceable" => SOPState::Advanceable,
            // "cancelled" => SOPState::Cancelled,
            _ => SOPState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match self {
            // SOPState::Empty => "empty".to_spvalue(),
            SOPState::Initial => "initial".to_spvalue(),
            SOPState::Executing => "executing".to_spvalue(),
            // SOPState::Paused => "paused".to_spvalue(),
            SOPState::Fatal => "fatal".to_spvalue(),
            // SOPState::NotFound => "not_found".to_spvalue(),
            SOPState::Completed => "completed".to_spvalue(),
            // SOPState::Terminated => "terminated".to_spvalue(),
            SOPState::Cancelled => "cancelled".to_spvalue(),
            SOPState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl PlannerState {
    pub fn from_str(x: &str) -> PlannerState {
        match x {
            "found" => PlannerState::Found,
            "not_found" => PlannerState::NotFound,
            "ready" => PlannerState::Ready,
            _ => PlannerState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match self {
            PlannerState::Found => "found".to_spvalue(),
            PlannerState::NotFound => "not_found".to_spvalue(),
            PlannerState::Ready => "ready".to_spvalue(),
            PlannerState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl fmt::Display for PlanState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlanState::UNKNOWN => write!(f, "UNKNOWN"),
            // PlanState::Empty => write!(f, "empty"),
            PlanState::Initial => write!(f, "initial"),
            PlanState::Executing => write!(f, "executing"),
            // PlanState::Paused => write!(f, "paused"),
            PlanState::Failed => write!(f, "failed"),
            // PlanState::NotFound => write!(f, "not_found"),
            PlanState::Completed => write!(f, "completed"),
            PlanState::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl fmt::Display for SOPState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SOPState::UNKNOWN => write!(f, "UNKNOWN"),
            // SOPState::Empty => write!(f, "empty"),
            SOPState::Initial => write!(f, "initial"),
            SOPState::Executing => write!(f, "executing"),
            // SOPState::Paused => write!(f, "paused"),
            SOPState::Fatal => write!(f, "fatal"),
            // SOPState::NotFound => write!(f, "not_found"),
            SOPState::Completed => write!(f, "completed"),
            // SOPState::Terminated => write!(f, "terminated"),
            SOPState::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl fmt::Display for PlannerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlannerState::UNKNOWN => write!(f, "UNKNOWN"),
            PlannerState::Found => write!(f, "found"),
            PlannerState::NotFound => write!(f, "not_found"),
            PlannerState::Ready => write!(f, "ready"),
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


#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum RunnerState {
    Idle,
    Running,
    Stopped,
    Paused,
    UNKNOWN,
}

impl Default for RunnerState {
    fn default() -> Self {
        RunnerState::UNKNOWN
    }
}

impl RunnerState {
    pub fn from_str(x: &str) -> RunnerState {
        match x {
            "idle" => RunnerState::Idle,
            "running" => RunnerState::Running,
            "paused" => RunnerState::Paused,
            "stopped" => RunnerState::Stopped,
            _ => RunnerState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match self {
            RunnerState::Running => "running".to_spvalue(),
            RunnerState::Paused => "paused".to_spvalue(),
            RunnerState::Stopped => "stopped".to_spvalue(),
            RunnerState::Idle => "idle".to_spvalue(),
            RunnerState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl fmt::Display for RunnerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunnerState::UNKNOWN => write!(f, "UNKNOWN"),
            RunnerState::Running => write!(f, "running"),
            RunnerState::Paused => write!(f, "paused"),
            RunnerState::Stopped => write!(f, "stopped"),
            RunnerState::Idle => write!(f, "idle"),
        }
    }
}