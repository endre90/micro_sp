use crate::*;
use serde::{Deserialize, Serialize};

/// A model contains behavior that defines what a system is capable of doing.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub auto_transitions: Vec<Transition>,
    pub sops: Vec<SOPStruct>,
    pub operations: Vec<Operation>,
}

impl Model {
    pub fn new(
        name: &str,
        auto_transitions: Vec<Transition>,
        sops: Vec<SOPStruct>,
        operations: Vec<Operation>,
    ) -> Model {
        Model {
            name: name.to_string(),
            auto_transitions,
            sops: sops
                .iter()
                .map(|sop| SOPStruct {
                    id: sop.id.clone(),
                    sop: uniquify_sop_operations(sop.sop.clone()),
                })
                .collect(),
            operations: operations
                .iter()
                .map(|o| Operation {
                    name: format!("operation_{}", o.name),
                    timeout_ms: o.timeout_ms,
                    retries: o.retries,
                    preconditions: o.preconditions.clone(),
                    postconditions: o.postconditions.clone(),
                    fail_transitions: o.fail_transitions.clone(),
                    timeout_transitions: o.timeout_transitions.clone(),
                    reset_transitions: o.reset_transitions.clone(),
                    state: o.state.clone(),
                })
                .collect(),
        }
    }

    // TODO: test relax function
    // pub fn relax(self, vars: &Vec<String>) -> Model {
    //     let r_operations = self
    //         .operations
    //         .iter()
    //         .map(|op| op.clone().relax(vars))
    //         .collect();
    //     let r_auto_transitions = self
    //         .auto_transitions
    //         .iter()
    //         .map(|t| t.clone().relax(vars))
    //         .collect();
    //     let mut r_state = HashMap::new();
    //     self.state
    //         .state
    //         .iter()
    //         .for_each(|(k, v)| match vars.contains(&k) {
    //             false => {
    //                 r_state.insert(k.clone(), v.clone());
    //             }
    //             true => (),
    //         });
    //     Model {
    //         name: self.name,
    //         state: State { state: r_state },
    //         auto_transitions: r_auto_transitions,
    //         operations: r_operations
    //     }
    // }
}
