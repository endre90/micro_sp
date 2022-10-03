#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{SPValue, SPValueType, SPVariable, State, ToSPValue};
use std::collections::{HashMap, HashSet};

fn john_doe() -> HashMap<SPVariable, SPValue> {
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spvalue(), "Jack".to_spvalue()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spvalue(), "Crawford".to_spvalue()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spvalue(), 185.to_spvalue(), 190.to_spvalue()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spvalue(), 85.to_spvalue(), 90.to_spvalue()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spvalue(), false.to_spvalue()],
    );
    HashMap::from([
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 80.to_spvalue()),
        (smart, true.to_spvalue()),
    ])
}

fn john_doe_faulty() -> HashMap<SPVariable, SPValue> {
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spvalue(), "Jack".to_spvalue()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spvalue(), "Crawford".to_spvalue()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spvalue(), 185.to_spvalue(), 190.to_spvalue()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spvalue(), 85.to_spvalue(), 90.to_spvalue()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spvalue(), false.to_spvalue()],
    );
    HashMap::from([
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 81.to_spvalue()),
        (smart, true.to_spvalue()),
    ])
}

#[test]
fn test_state_new() {
    let john_doe = john_doe();
    let new_state = State::new(&john_doe);
    assert_eq!(new_state.state, john_doe)
}

#[test]
#[should_panic]
fn test_state_new_panic() {
    let john_doe = john_doe_faulty();
    State::new(&john_doe);
}

#[test]
fn test_state_keys() {
    let john_doe = john_doe();
    let keys = State::keys(State::new(&john_doe));
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spvalue(), "Jack".to_spvalue()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spvalue(), "Crawford".to_spvalue()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spvalue(), 185.to_spvalue(), 190.to_spvalue()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spvalue(), 85.to_spvalue(), 90.to_spvalue()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spvalue(), false.to_spvalue()],
    );
    assert_eq!(
        keys,
        HashSet::from_iter(vec!(name, surname, height, weight, smart))
    )
}

#[test]
fn test_state_names() {
    let john_doe = john_doe();
    let keys = State::names(State::new(&john_doe));
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
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spvalue(), "Crawford".to_spvalue()],
    );
    let job = SPVariable::new(
        "job",
        &SPValueType::String,
        &vec!["Carpenter".to_spvalue(), "Waiter".to_spvalue()],
    );
    assert_eq!(true, State::new(&john_doe).contains(&surname));
    assert_ne!(true, State::new(&john_doe.clone()).contains(&job));
}

#[test]
fn test_state_contains_name() {
    let john_doe = john_doe();
    assert_eq!(true, State::new(&john_doe).contains_name("surname"));
    assert_ne!(true, State::new(&john_doe).contains_name("job"));
}

#[test]
fn test_state_get() {
    let john_doe = john_doe();
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spvalue(), 185.to_spvalue(), 190.to_spvalue()],
    );
    assert_eq!(185.to_spvalue(), State::new(&john_doe).get(&height));
    assert_ne!(186.to_spvalue(), State::new(&john_doe).get(&height));
}

#[test]
fn test_state_get_val() {
    let john_doe = john_doe();
    State::new(&john_doe).get_val("height");
}

#[test]
fn test_state_get_var() {
    let john_doe = john_doe();
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spvalue(), 185.to_spvalue(), 190.to_spvalue()],
    );
    assert_eq!(height, State::new(&john_doe).get_var("height"));
}

#[test]
#[should_panic]
fn test_state_get_panic() {
    let john_doe = john_doe();
    let job = SPVariable::new(
        "job",
        &SPValueType::String,
        &vec!["Carpenter".to_spvalue(), "Waiter".to_spvalue()],
    );
    State::new(&john_doe).get(&job);
}

#[test]
#[should_panic]
fn test_state_get_val_panic() {
    let john_doe = john_doe();
    State::new(&john_doe).get_val("job");
}

#[test]
fn test_state_update() {
    let john_doe = john_doe();
    let old_state = State::new(&john_doe);
    let new_state = old_state.clone().update("weight", &87.to_spvalue());
    assert_ne!(old_state, new_state);
    assert_eq!(87.to_spvalue(), new_state.clone().get_val("weight"));
}

// #[test]
// fn test_state_updates() {
//     let john_doe = john_doe();
//     let old_state = State::new(john_doe.clone());
//     let new_state = old_state.clone().updates(HashMap::from([
//         ("weight".to_string(), 87.to_spvalue()),
//         ("job".to_string(), "carpenter".to_spvalue()),
//     ]));
//     assert_ne!(old_state, new_state);
//     assert_eq!(87.to_spvalue(), new_state.clone().get("weight"));
//     assert_eq!("carpenter".to_spvalue(), new_state.get("job"));
// }
