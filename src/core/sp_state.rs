use crate::{SPAssignment, SPValue, SPVariable, SPVariableType};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct State {
    pub state: HashMap<String, SPAssignment>,
}

impl Hash for State {
    fn hash<H: Hasher>(&self, s: &mut H) {
        self.state
            .keys()
            .into_iter()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
            .hash(s);
        self.state
            .values()
            .into_iter()
            .map(|x| x.var.to_owned())
            .collect::<Vec<SPVariable>>()
            .hash(s);
        self.state
            .values()
            .into_iter()
            .map(|x| x.val.to_owned())
            .collect::<Vec<SPValue>>()
            .hash(s);
    }
}

impl State {
    pub fn new() -> State {
        State {
            state: HashMap::new(),
        }
    }

    pub fn from_vec(vec: &Vec<(SPVariable, SPValue)>) -> State {
        let mut state = HashMap::new();
        vec.iter().for_each(|(var, val)| {
            state.insert(
                var.name.clone(),
                SPAssignment {
                    var: var.clone(),
                    val: val.clone(),
                },
            );
        });
        State { state }
    }

    pub fn add(&self, assignment: SPAssignment) -> State {
        match self.state.clone().get(&assignment.var.name) {
            Some(_) => panic!("already in state"),
            None => {
                let mut state = self.state.clone();
                state.insert(assignment.var.name.to_string(), assignment.clone());
                State { state }
            }
        }
    }

    pub fn get_value(&self, name: &str) -> SPValue {
        match self.state.clone().get(name) {
            None => panic!("Not in state!"),
            Some(x) => x.val.clone()
        }
    }

    pub fn get_all(&self, name: &str) -> SPAssignment {
        match self.state.clone().get(name) {
            None => panic!("Not in state!"),
            Some(x) => x.clone()
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.state.clone().contains_key(name)
    }

    pub fn update(&self, name: &str, val: SPValue) -> State {
        match self.state.clone().get(name) {
            Some(assignment) => match assignment.var.variable_type {
                SPVariableType::Planner => match assignment.var.domain.contains(&val) {
                    true => {
                        let mut state = self.state.clone();
                        state.insert(
                            name.to_string(),
                            SPAssignment {
                                var: assignment.var.clone(),
                                val: val.clone(),
                            },
                        );
                        State { state }
                    }
                    false => panic!(
                        "Value {} to update the variable {} is not in its domain.",
                        val, assignment.var.name
                    ),
                },
                SPVariableType::Runner => {
                    let mut state = self.state.clone();
                    state.insert(
                        name.to_string(),
                        SPAssignment {
                            var: assignment.var.clone(),
                            val: val.clone(),
                        },
                    );
                    State { state }
                }
            },
            None => panic!("Variable {} not in state.", name),
        }
    }
}