use crate::*;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RunnerState {
    Idle,
    Executing,
    Paused,
    Planning,
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
            "executing" => RunnerState::Executing,
            "paused" => RunnerState::Paused,
            "planning" => RunnerState::Planning,
            _ => RunnerState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        match self {
            RunnerState::Idle => "idle".to_spvalue(),
            RunnerState::Executing => "executing".to_spvalue(),
            RunnerState::Paused => "paused".to_spvalue(),
            RunnerState::Planning => "planning".to_spvalue(),
            RunnerState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl fmt::Display for RunnerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunnerState::UNKNOWN => write!(f, "UNKNOWN"),
            RunnerState::Idle => write!(f, "idle"),
            RunnerState::Executing => write!(f, "executing"),
            RunnerState::Paused => write!(f, "paused"),
            RunnerState::Planning => write!(f, "planning"),
        }
    }
}