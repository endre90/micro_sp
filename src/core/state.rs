use crate::SPValue;
use serde::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct State {
    pub state: HashMap<String, SPValue>,
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

    pub fn get(self, key: &str) -> Option<SPValue> {
        match self.state.get(key) {
            Some(value) => Some(value.to_owned()),
            None => None,
        }
    }

    pub fn update(self, change: HashMap<String, SPValue>) -> State {
        let mut state = self.state.clone();
        change.iter().for_each(|(k, v)| {
            state.insert(k.to_string(), v.clone());
        });
        State { state }
    }
}