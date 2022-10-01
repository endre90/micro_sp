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

#[test]
fn test_state_contains() {
    let john_doe = john_doe();
    assert_eq!(true, State::new(john_doe.clone()).contains("surname"));
    assert_ne!(true, State::new(john_doe.clone()).contains("job"));
}

#[test]
fn test_state_get() {
    let john_doe = john_doe();
    assert_eq!(
        185.to_spvalue(),
        State::new(john_doe.clone()).get("height")
    );
    assert_ne!(
        186.to_spvalue(),
        State::new(john_doe.clone()).get("height")
    );
}

#[test]
#[should_panic]
fn test_state_get_panic() {
    let john_doe = john_doe();
    State::new(john_doe).get("job");
}

#[test]
fn test_state_update() {
    let john_doe = john_doe();
    let old_state = State::new(john_doe.clone());
    let new_state = old_state.clone().update("weight", 87.5.to_spvalue());
    assert_ne!(old_state, new_state);
    assert_eq!(87.5.to_spvalue(), new_state.clone().get("weight"));
}

#[test]
fn test_state_updates() {
    let john_doe = john_doe();
    let old_state = State::new(john_doe.clone());
    let new_state = old_state.clone().updates(HashMap::from([
        ("weight".to_string(), 87.5.to_spvalue()),
        ("job".to_string(), "carpenter".to_spvalue()),
    ]));
    assert_ne!(old_state, new_state);
    assert_eq!(87.5.to_spvalue(), new_state.clone().get("weight"));
    assert_eq!("carpenter".to_spvalue(), new_state.get("job"));
}