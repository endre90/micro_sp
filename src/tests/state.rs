#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{SPValue, State, ToSPValue};
use std::collections::{HashMap, HashSet};

fn john_doe() -> HashMap<String, SPValue> {
    HashMap::from([
        ("name".to_string(), "John".to_spvalue()),
        ("surname".to_string(), "Doe".to_spvalue()),
        ("height".to_string(), 185.to_spvalue()),
        ("weight".to_string(), 80.5.to_spvalue()),
        ("smart".to_string(), true.to_spvalue()),
    ])
}

#[test]
fn test_state_new() {
    let john_doe = john_doe();
    let new_state = State::new(john_doe.clone());
    assert_eq!(new_state.state, john_doe)
}

#[test]
fn test_state_keys() {
    let john_doe = john_doe();
    let keys = State::keys(State::new(john_doe.clone()));
    assert_eq!(
        keys,
        HashSet::from_iter(vec!(
            "name".to_string(),
            "surname".to_string(),
            "height".to_string(),
            "weight".to_string(),
            "smart".to_string()
        ))
    )
}
