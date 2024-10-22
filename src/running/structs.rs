use crate::*;

use std::fmt::{self};

#[derive(Debug, Clone, PartialEq)]
pub enum RunnerState {
    Idle,
    Running,
    Paused,
    Stopped,
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