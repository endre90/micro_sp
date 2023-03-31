use crate::{SPAssignment, SPValue, SPVariable, SPVariableType};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{collections::HashMap, fmt};

/// Represents the current state of the system.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct State {
    pub state: HashMap<String, SPAssignment>,
}

/// The Hash trait is implemented on State in order to enable the comparison of different State instances using a hashing function.
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
    /// Creates and returns a new State instance.
    pub fn new() -> State {
        State {
            state: HashMap::new(),
        }
    }

    /// The from_vec function creates a new State object from a vector of (SPVariable, SPValue) tuples.
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

    /// Adds an SPAssignment to the State, returning a new State instance.
    pub fn add(&self, assignment: SPAssignment) -> State {
        match self.state.clone().get(&assignment.var.name) {
            Some(_) => panic!(
                "Variable {} already in state!",
                assignment.var.name.to_string()
            ),
            None => {
                let mut state = self.state.clone();
                state.insert(assignment.var.name.to_string(), assignment.clone());
                State { state }
            }
        }
    }

    /// Returns the value of the variable with the given name from the state, and panics if the variable is not in the state.
    pub fn get_value(&self, name: &str) -> SPValue {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.val.clone(),
        }
    }

    /// Returns the assignment of a variable in the state or panics if the variable is not found.
    pub fn get_all(&self, name: &str) -> SPAssignment {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.clone(),
        }
    }

    /// Checks whether a variable with the given name is contained in the state.
    pub fn contains(&self, name: &str) -> bool {
        self.state.clone().contains_key(name)
    }

    /// Updates the value of a variable in the state. If the variable is of type Runner, the value can be any. 
    /// If the value is not in the variable's domain, the update is ignored, unless the value is "unknown". 
    /// If the variable is not in the state, a panic occurs. Returns a new state with the updated value.
    pub fn update(&self, name: &str, val: SPValue) -> State {
        match self.state.clone().get(name) {
            Some(assignment) => match assignment.var.variable_type {
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
                _ => match assignment.var.domain.contains(&val) {
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
                    false => match val {
                        SPValue::Unknown => {
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
                        SPValue::String(x) => match x.as_str() {
                            "unknown" => self.clone(),
                            _ => {
                                println!("Value {} to update the variable {} is not in its domain. State not updated!", x, assignment.var.name);
                                self.clone()
                            }
                        },
                        _ => {
                            println!("Value {} to update the variable {} is not in its domain. State not updated!", val, assignment.var.name);
                            self.clone()
                        }
                    },
                },
            },
            None => panic!("Variable {} not in state.", name),
        }
    }
}

/// Displaying the State in a user-friendly way.
impl fmt::Display for State {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = {
            // let sorted = self.state.sort();
            let mut children: Vec<_> = self
                .state
                .iter()
                .map(|(k, v)| match &v.val {
                    SPValue::Array(_, some_array) => {
                        let mut sub_children: Vec<String> = vec![format!("    {}:", k)];
                        sub_children.extend(
                            some_array
                                .iter()
                                .map(|value| format!("        {}", value))
                                .collect::<Vec<String>>(),
                        );
                        format!("{}", sub_children.join("\n"))
                    }
                    _ => format!("    {}: {}", k, v.val),
                })
                .collect();
            children.sort();
            format!("{}", children.join("\n"))
        };

        write!(fmtr, "State: {{\n{}\n}}\n", &s)
    }
}
