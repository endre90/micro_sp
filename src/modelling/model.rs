use crate::*;
use serde::{Deserialize, Serialize};

/// A model contains behavior that defines what a system is capable of doing.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub auto_transitions: Vec<Transition>,
    pub auto_operations: Vec<Operation>,
    pub mutexed_auto_operations: Vec<Operation>, // allow only one operation to be run at a time
    pub sops: Vec<SOPStruct>,
    pub operations: Vec<Operation>,
}

impl Model {
    pub fn new(
        name: &str,
        auto_transitions: Vec<Transition>,
        auto_operations: Vec<Operation>,
        mutexed_auto_operations: Vec<Operation>,
        sops: Vec<SOPStruct>,
        operations: Vec<Operation>,
    ) -> Model {
        Model {
            name: name.to_string(),
            auto_transitions,
            auto_operations: auto_operations
                .iter()
                .map(|o| Operation {
                    name: format!("op_{}", o.name),
                    timeout_executing_ms: o.timeout_executing_ms,
                    timeout_disabled_ms: o.timeout_disabled_ms,
                    failure_retries: o.failure_retries,
                    timeout_retries: o.timeout_retries,
                    can_be_bypassed: o.can_be_bypassed,
                    preconditions: o.preconditions.clone(),
                    postconditions: o.postconditions.clone(),
                    failure_transitions: o.failure_transitions.clone(),
                    timeout_transitions: o.timeout_transitions.clone(),
                    bypass_transitions: o.bypass_transitions.clone(),
                    cancel_transitions: o.cancel_transitions.clone(),
                    state: o.state.clone(),
                })
                .collect(),
            mutexed_auto_operations: mutexed_auto_operations
                .iter()
                .map(|o| Operation {
                    name: format!("op_{}", o.name),
                    timeout_executing_ms: o.timeout_executing_ms,
                    timeout_disabled_ms: o.timeout_disabled_ms,
                    failure_retries: o.failure_retries,
                    timeout_retries: o.timeout_retries,
                    can_be_bypassed: o.can_be_bypassed,
                    preconditions: o.preconditions.clone(),
                    postconditions: o.postconditions.clone(),
                    failure_transitions: o.failure_transitions.clone(),
                    timeout_transitions: o.timeout_transitions.clone(),
                    bypass_transitions: o.bypass_transitions.clone(),
                    cancel_transitions: o.cancel_transitions.clone(),
                    state: o.state.clone(),
                })
                .collect(),
            sops,
            operations: operations
                .iter()
                .map(|o| Operation {
                    name: format!("op_{}", o.name),
                    timeout_executing_ms: o.timeout_executing_ms,
                    timeout_disabled_ms: o.timeout_disabled_ms,
                    failure_retries: o.failure_retries,
                    timeout_retries: o.timeout_retries,
                    can_be_bypassed: o.can_be_bypassed,
                    preconditions: o.preconditions.clone(),
                    postconditions: o.postconditions.clone(),
                    failure_transitions: o.failure_transitions.clone(),
                    timeout_transitions: o.timeout_transitions.clone(),
                    bypass_transitions: o.bypass_transitions.clone(),
                    cancel_transitions: o.cancel_transitions.clone(),
                    state: o.state.clone(),
                })
                .collect(),
        }
    }

}
