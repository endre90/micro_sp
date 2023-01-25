use crate::{SPAssignment, SPValue, SPVariable, SPVariableType};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{collections::HashMap, fmt};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
            None => panic!("Variable {} Not in state!", name),
            Some(x) => x.val.clone(),
        }
    }

    pub fn get_all(&self, name: &str) -> SPAssignment {
        match self.state.clone().get(name) {
            None => panic!("Variable {} not in state!", name),
            Some(x) => x.clone(),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.state.clone().contains_key(name)
    }

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
                    false => panic!(
                        "Value {} to update the variable {} is not in its domain.",
                        val, assignment.var.name
                    ),
                },
                
            },
            None => panic!("Variable {} not in state.", name),
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = {
            // let sorted = self.state.sort();
            let children: Vec<_> = self
                .state
                .iter()
                .map(|(k, v)| format!("    {}: {}", k, v.val))
                .collect();
            format!("{}", children.join("\n"))
        };

        write!(fmtr, "State: {{\n{}\n}}\n", &s)
    }
}

// maybe can be easier like this?
// #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// pub struct State {
//     pub state: HashMap<String, SPValue>,
// }

// impl Hash for State {
//     fn hash<H: Hasher>(&self, s: &mut H) {
//         self.state
//             .keys()
//             .into_iter()
//             .map(|x| x.to_owned())
//             .collect::<Vec<String>>()
//             .hash(s);
//         self.state
//             .values()
//             .into_iter()
//             .map(|x| x.to_owned())
//             .collect::<Vec<SPValue>>()
//             .hash(s);
//     }
// }

// impl State {
//     pub fn new() -> State {
//         State {
//             state: HashMap::new(),
//         }
//     }

//     pub fn from_vec(vec: &Vec<(String, SPValue)>) -> State {
//         let mut state = HashMap::new();
//         vec.iter().for_each(|(var, val)| {
//             state.insert(
//                 var.clone(),
//                 val.clone(),
//             );
//         });
//         State { state }
//     }

//     pub fn add(&self, assignment: (&str, SPValue)) -> State {
//         match self.state.clone().get(assignment.0) {
//             Some(_) => panic!("already in state"),
//             None => {
//                 let mut state = self.state.clone();
//                 state.insert(assignment.0.to_string(), assignment.1.clone());
//                 State { state }
//             }
//         }
//     }

//     pub fn get_value(&self, name: &str) -> SPValue {
//         match self.state.clone().get(name) {
//             None => panic!("Variable {} Not in state!", name),
//             Some(x) => x.clone()
//         }
//     }

//     pub fn contains(&self, name: &str) -> bool {
//         self.state.clone().contains_key(name)
//     }

//     pub fn update(&self, name: &str, val: SPValue) -> State {
//         match self.state.clone().get(name) {
//             Some(value) => match assignment.var.variable_type {
//                 SPVariableType::Planner => match assignment.var.domain.contains(&val) {
//                     true => {
//                         let mut state = self.state.clone();
//                         state.insert(
//                             name.to_string(),
//                             val.clone(),
//                         );
//                         State { state }
//                     }
//                     false => panic!(
//                         "Value {} to update the variable {} is not in its domain.",
//                         val, assignment.var.name
//                     ),
//                 },
//                 SPVariableType::Runner => {
//                     let mut state = self.state.clone();
//                     state.insert(
//                         name.to_string(),
//                         SPAssignment {
//                             var: assignment.var.clone(),
//                             val: val.clone(),
//                         },
//                     );
//                     State { state }
//                 }
//             },
//             None => panic!("Variable {} not in state.", name),
//         }
//     }
// }

// impl fmt::Display for State {
//     fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let s: String = {
//             // let sorted = self.state.sort();
//             let children: Vec<_> = self.state.iter().map(|(k, v)| format!("    {}: {}", k, v.val)).collect();
//             format!("{}", children.join("\n"))
//         };

//         write!(fmtr, "State: {{\n{}\n}}\n", &s)
//     }
// }
