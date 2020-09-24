use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Pair {
    pub key: String,
    pub value: String,
}   

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Pairs {
    pub pairs: Vec<Pair>
}   

impl Pair {
    pub fn new(key: &str, value: &str) -> Pair {
        Pair {
            key: key.to_string(),
            value: value.to_string()
        }
    }
}   

impl Pairs {
    pub fn new(pairs: &Vec<Pair>) -> Pairs {
        Pairs {
            pairs: pairs.iter().map(|x| x.to_owned()).collect()
        }
    }
}