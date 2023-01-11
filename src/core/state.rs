use crate::{SPAssignment, SPValue, SPVariable};
use std::collections::{HashMap, HashSet};
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
    pub fn new(state: &HashMap<SPVariable, SPValue>) -> State {
        state.iter().for_each(|(k, v)| match k.domain.contains(v) {
            true => (),
            false => panic!("Value {:?} not in the domain of variable {:?}", v, k.name),
        });
        State {
            state: state
                .iter()
                .map(|(x, y)| {
                    (
                        x.name.clone(),
                        SPAssignment {
                            var: x.clone(),
                            val: y.clone(),
                        },
                    )
                })
                .collect::<HashMap<String, SPAssignment>>(),
        }
    }

    pub fn new_empty() -> State {
        State {
            state: HashMap::new(),
        }
    }

    pub fn from_vec(vec: &Vec<(SPVariable, SPValue)>) -> State {
        let mut state_map = HashMap::new();
        vec.iter().for_each(|(var, val)| {
            state_map.insert(var.clone(), val.clone());
        });
        State::new(&state_map)
    }

    // pub fn add(to_add: &(SPVariable, SPValue)) -> State {
    //     if to_add.0.domain.contains(to_add.1) {
    //         state.
    //     }
    //     state.iter().for_each(|(k, v)| match k.domain.contains(v) {
    //         true => (),
    //         false => panic!("Value {:?} not in the domain of variable {:?}", v, k.name),
    //     });
    //     State {
    //         state: state.to_owned(),
    //     }
    // }

    // pub fn add(state: &HashMap<SPVariable, SPValue>) -> State {
    //     state.iter().for_each(|(k, v)| match k.domain.contains(v) {
    //         true => (),
    //         false => panic!("Value {:?} not in the domain of variable {:?}", v, k.name),
    //     });
    //     State {
    //         state: state.to_owned(),
    //     }
    // }

    // pub fn names(self) -> HashSet<String> {
    //     self.state
    //         .keys()
    //         .into_iter()
    //         .map(|k| k.name.to_string())
    //         .collect::<HashSet<String>>()
    // }

    // pub fn keys(self) -> HashSet<SPVariable> {
    //     self.state
    //         .keys()
    //         .into_iter()
    //         .map(|k| k.to_owned())
    //         .collect::<HashSet<SPVariable>>()
    // }

    // pub fn contains(self, key: &str) -> bool {
    //     self.state.contains_key(key)
    // }

    pub fn add(self, assignment: &SPAssignment) -> State {
        match self.state.clone().get(&assignment.var.name) {
            Some(_) => panic!("already in state"),
            None => {
                let mut state = self.state.clone();
                state.insert(assignment.var.name.to_string(), assignment.clone());
                State { state }
            }
        }
    }

    // have to add check if value is in the domain
    pub fn update(self, name: &str, val: &SPValue) -> State {
        match self.state.clone().get(name) {
            Some(assignment) => match assignment.var.domain.contains(val) {
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
            None => panic!("Variable {} not in state.", name),
        }
        // let var = self.clone().get_spvar(var);
        // match var.domain.contains(val) {
        //     true => {
        //         let mut state = self.state.clone();
        //         state.insert(var.clone(), val.clone());
        //         State { state }
        //     },
        //     false => panic!("Value {} to update the variable {} is not in its domain.", val, var.name)
        // }
    }
}
