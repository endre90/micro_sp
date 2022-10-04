use crate::{SPValue, SPVariable};
use serde::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct State {
    pub state: HashMap<SPVariable, SPValue>,
}

impl Hash for State {
    fn hash<H: Hasher>(&self, s: &mut H) {
        self.state
            .keys()
            .into_iter()
            .map(|x| x.to_owned())
            .collect::<Vec<SPVariable>>()
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
    pub fn new(state: &HashMap<SPVariable, SPValue>) -> State {
        state.iter().for_each(|(k, v)| match k.domain.contains(v) {
            true => (),
            false => panic!("Value {:?} not in the domain of variable {:?}", v, k.name),
        });
        State {
            state: state.to_owned(),
        }
    }

    pub fn names(self) -> HashSet<String> {
        self.state
            .keys()
            .into_iter()
            .map(|k| k.name.to_string())
            .collect::<HashSet<String>>()
    }

    pub fn keys(self) -> HashSet<SPVariable> {
        self.state
            .keys()
            .into_iter()
            .map(|k| k.to_owned())
            .collect::<HashSet<SPVariable>>()
    }

    pub fn contains(self, key: &SPVariable) -> bool {
        self.state.contains_key(key)
    }

    pub fn contains_name(self, key: &str) -> bool {
        let mut map = HashMap::new();
        self.state.iter().for_each(|(k, v)| {
            map.insert(k.name.clone(), v);
        });
        map.contains_key(key)
    }

    pub fn get_spval(self, key: &str) -> SPValue {
        let mut map = HashMap::new();
        self.state.iter().for_each(|(k, v)| {
            map.insert(k.name.clone(), v.clone());
        });
        match map.get(key) {
            Some(value) => value.to_owned(),
            None => panic!("Variable {key} not found in the state."),
        }
    }

    pub fn get_spvar(self, key: &str) -> SPVariable {
        match self.state.iter().find(|(k, _)| k.name == key) {
            Some((var, _)) => var.to_owned(),
            None => panic!("Variable {key} not found in the state."),
        }
    }

    // Need to panic because if both keys are not
    // found in EQ, None == None
    pub fn get(self, key: &SPVariable) -> SPValue {
        match self.state.get(key) {
            Some(value) => value.to_owned(),
            None => panic!("Variable {} not found in the state.", key.name),
        }
    }

    // have to add check if value is in the domain
    pub fn update(self, var: &str, val: &SPValue) -> State {
        let var = self.clone().get_spvar(var);
        match var.domain.contains(val) {
            true => {
                let mut state = self.state.clone();
                state.insert(var.clone(), val.clone());
                State { state }
            },
            false => panic!("Value {} to update the variable {} is not in its domain.", val, var.name)
        }
        
    }

    // pub fn updates(self, changes: HashMap<String, SPValue>) -> State {
    //     let mut state = self.state.clone();
    //     changes.iter().for_each(|(k, v)| {
    //         state.insert(k.to_string(), v.clone());
    //     });
    //     State { state }
    // }
}
