use crate::SPValue;
use serde::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct State {
    pub state: HashMap<String, SPValue>,
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
            .map(|x| x.to_owned())
            .collect::<Vec<SPValue>>()
            .hash(s);
    }
}

impl State {
    pub fn new(state: HashMap<String, SPValue>) -> State {
        State { state }
    }

    pub fn keys(self) -> HashSet<String> {
        self.state
            .keys()
            .into_iter()
            .map(|k| k.to_string())
            .collect::<HashSet<String>>()
    }

    pub fn contains(self, key: &str) -> bool {
        self.state.contains_key(key)
    }

    // Need to panic because if both keys are not
    // found in EQ, None == None
    pub fn get(self, key: &str) -> SPValue {
        match self.state.get(key) {
            Some(value) => value.to_owned(),
            None => panic!("Variable {key} not found in the state."),
        }
    }

    pub fn update(self, var: &str, val: SPValue) -> State {
        let mut state = self.state.clone();
        state.insert(var.to_string(), val.clone());
        State { state }
    }

    pub fn updates(self, changes: HashMap<String, SPValue>) -> State {
        let mut state = self.state.clone();
        changes.iter().for_each(|(k, v)| {
            state.insert(k.to_string(), v.clone());
        });
        State { state }
    }
}
