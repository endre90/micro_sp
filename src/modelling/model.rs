use serde::{Deserialize, Serialize};

use crate::{Operation, SPAssignment, SPVariable, SPVariableType, State, ToSPValue, Transition, Resource};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub state: State,
    // pub variables: Vec<SPVariable>,
    pub auto_transitions: Vec<Transition>,
    pub auto_operations: Vec<Operation>,
    pub operations: Vec<Operation>,
    pub resources: Vec<Resource>
}

impl Model {
    pub fn new(
        name: &str,
        state: State,
        auto_transitions: Vec<Transition>,
        auto_operations: Vec<Operation>,
        operations: Vec<Operation>,
        resources: Vec<Resource>
    ) -> Model {
        let mut state_with_op = state.clone();
        for op in &operations {
            match state.contains(&op.name) {
                false => {
                    state_with_op.state.insert(
                        op.name.clone(),
                        SPAssignment::new(
                            SPVariable::new(
                                &op.name,
                                SPVariableType::Runner,
                                crate::SPValueType::String,
                                vec!["initial".to_spvalue(), "executing".to_spvalue()],
                            ),
                            "initial".to_spvalue(),
                        ),
                    );
                }
                true => panic!("A variable already named as the operation '{}' exists.", op.name),
            }
        }

        Model {
            name: name.to_string(),
            state: state_with_op.clone(),
            // variables: state_with_op
            //     .state
            //     .iter()
            //     .map(|(_, assignment)| assignment.var.clone())
            //     .collect(),
            auto_transitions,
            auto_operations,
            operations,
            resources
        }
    }
}
