use serde::{Deserialize, Serialize};

use crate::{Operation, SPAssignment, SPVariable, SPVariableType, State, ToSPValue, Transition};

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct OperationModel {
    pub name: String,
    pub initial_state: State,
    pub variables: Vec<SPVariable>,
    pub transitions: Vec<Transition>,
    pub operations: Vec<Operation>,
}

impl OperationModel {
    pub fn new(
        name: &str,
        initial_state: State,
        transitions: Vec<Transition>,
        operations: Vec<Operation>,
    ) -> OperationModel {
        let mut state_with_op = initial_state.clone();
        for op in &operations {
            match initial_state.contains(&op.name) {
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

        OperationModel {
            name: name.to_string(),
            initial_state: state_with_op.clone(),
            variables: state_with_op
                .state
                .iter()
                .map(|(_, assignment)| assignment.var.clone())
                .collect(),
            transitions,
            operations,
        }
    }
}
