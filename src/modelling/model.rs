use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// use crate::{Operation, State, Transition};
use crate::*;
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub state: State,
    pub auto_transitions: Vec<Transition>,
    pub operations: Vec<Operation>
}

impl Model {
    pub fn new(
        name: &str,
        state: State,
        auto_transitions: Vec<Transition>,
        operations: Vec<Operation>,
    ) -> Model {
        Model {
            name: name.to_string(),
            state: state.clone(),
            auto_transitions,
            operations,
        }
    }

    // TODO: test relax function
    pub fn relax(self, vars: &Vec<String>) -> Model {
        let r_operations = self
            .operations
            .iter()
            .map(|op| op.clone().relax(vars))
            .collect();
        let r_auto_transitions = self
            .auto_transitions
            .iter()
            .map(|t| t.clone().relax(vars))
            .collect();
        let mut r_state = HashMap::new();
        self.state
            .state
            .iter()
            .for_each(|(k, v)| match vars.contains(&k) {
                false => {
                    r_state.insert(k.clone(), v.clone());
                }
                true => (),
            });
        Model {
            name: self.name,
            state: State { state: r_state },
            auto_transitions: r_auto_transitions,
            operations: r_operations
        }
    }
}