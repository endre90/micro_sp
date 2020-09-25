use serde::{Deserialize, Serialize};
use arrayvec::ArrayString;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct KeyValuePair {
    pub key: ArrayString::<[u8; 20]>,
    pub value: ArrayString::<[u8; 20]>,
}   

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct State {
    pub pairs: Vec<KeyValuePair>
}   

impl KeyValuePair {
    pub fn new(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            key: match key.len() > 20 {
                false => ArrayString::<[_; 20]>::from(key).unwrap_or_default(),
                true => panic!("variable name too big")
            },
            value: match value.len() > 20 {
                false => ArrayString::<[_; 20]>::from(value).unwrap_or_default(),
                true => panic!("value too big")
            }
        }
    }
}   

impl State {
    pub fn new(pairs: &Vec<KeyValuePair>) -> State {
        State {
            pairs: pairs.iter().map(|x| x.to_owned()).collect()
        }
    }
}