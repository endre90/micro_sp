#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{SPValue, SPValueType, SPVariable, State, ToSPValue};
use std::collections::{HashMap, HashSet};

fn john_doe_hashmap() -> HashMap<SPVariable, SPValue> {
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spval(), "Jack".to_spval()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spval(), "Crawford".to_spval()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spval(), 185.to_spval(), 190.to_spval()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spval(), false.to_spval()],
    );
    HashMap::from([
        (name, "John".to_spval()),
        (surname, "Doe".to_spval()),
        (height, 185.to_spval()),
        (weight, 80.to_spval()),
        (smart, true.to_spval()),
    ])
}

fn john_doe_state() -> State {
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spval(), "Jack".to_spval()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spval(), "Crawford".to_spval()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spval(), 185.to_spval(), 190.to_spval()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spval(), false.to_spval()],
    );
    State::new(&HashMap::from([
        (name, "John".to_spval()),
        (surname, "Doe".to_spval()),
        (height, 185.to_spval()),
        (weight, 80.to_spval()),
        (smart, true.to_spval()),
    ]))
}

fn john_doe_faulty() -> HashMap<SPVariable, SPValue> {
    let name = SPVariable::new(
        "name",
        &SPValueType::String,
        &vec!["John".to_spval(), "Jack".to_spval()],
    );
    let surname = SPVariable::new(
        "surname",
        &SPValueType::String,
        &vec!["Doe".to_spval(), "Crawford".to_spval()],
    );
    let height = SPVariable::new(
        "height",
        &SPValueType::Int32,
        &vec![180.to_spval(), 185.to_spval(), 190.to_spval()],
    );
    let weight = SPVariable::new(
        "weight",
        &SPValueType::Int32,
        &vec![80.to_spval(), 85.to_spval(), 90.to_spval()],
    );
    let smart = SPVariable::new(
        "smart",
        &SPValueType::Bool,
        &vec![true.to_spval(), false.to_spval()],
    );
    HashMap::from([
        (name, "John".to_spval()),
        (surname, "Doe".to_spval()),
        (height, 185.to_spval()),
        (weight, 81.to_spval()),
        (smart, true.to_spval()),
    ])
}

#[test]
fn test_state_new() {
    let john_doe = john_doe_hashmap();
    let new_state = State::new(&john_doe);
    assert_eq!(new_state.state, john_doe_state().state)
}

// #[test]
// #[should_panic]
// fn test_state_new_panic() {
//     let john_doe = john_doe_faulty();john_doe
//     State::new(&john_doe);
// }

// #[test]
// fn test_state_keys() {
//     let john_doe = john_doe();
//     let keys = State::keys(State::new(&john_doe));
//     let name = SPVariable::new(
//         "name",
//         &SPValueType::String,
//         &vec!["John".to_spval(), "Jack".to_spval()],
//     );
//     let surname = SPVariable::new(
//         "surname",
//         &SPValueType::String,
//         &vec!["Doe".to_spval(), "Crawford".to_spval()],
//     );
//     let height = SPVariable::new(
//         "height",
//         &SPValueType::Int32,
//         &vec![180.to_spval(), 185.to_spval(), 190.to_spval()],
//     );
//     let weight = SPVariable::new(
//         "weight",
//         &SPValueType::Int32,
//         &vec![80.to_spval(), 85.to_spval(), 90.to_spval()],
//     );
//     let smart = SPVariable::new(
//         "smart",
//         &SPValueType::Bool,
//         &vec![true.to_spval(), false.to_spval()],
//     );
//     assert_eq!(
//         keys,
//         HashSet::from_iter(vec!(name, surname, height, weight, smart))
//     )
// }

// #[test]
// fn test_state_names() {
//     let john_doe = john_doe();
//     let keys = State::names(State::new(&john_doe));
//     assert_eq!(
//         keys,
//         HashSet::from_iter(vec!(
//             "name".to_string(),
//             "surname".to_string(),
//             "height".to_string(),
//             "weight".to_string(),
//             "smart".to_string()
//         ))
//     )
// }

// #[test]
// fn test_state_contains() {
//     let john_doe = john_doe();
//     let surname = SPVariable::new(
//         "surname",
//         &SPValueType::String,
//         &vec!["Doe".to_spval(), "Crawford".to_spval()],
//     );
//     let job = SPVariable::new(
//         "job",
//         &SPValueType::String,
//         &vec!["Carpenter".to_spval(), "Waiter".to_spval()],
//     );
//     assert_eq!(true, State::new(&john_doe).contains(&surname));
//     assert_ne!(true, State::new(&john_doe.clone()).contains(&job));
// }

// #[test]
// fn test_state_contains_name() {
//     let john_doe = john_doe();
//     assert_eq!(true, State::new(&john_doe).contains_name("surname"));
//     assert_ne!(true, State::new(&john_doe).contains_name("job"));
// }

// #[test]
// fn test_state_get() {
//     let john_doe = john_doe();
//     let height = SPVariable::new(
//         "height",
//         &SPValueType::Int32,
//         &vec![180.to_spval(), 185.to_spval(), 190.to_spval()],
//     );
//     assert_eq!(185.to_spval(), State::new(&john_doe).get(&height));
//     assert_ne!(186.to_spval(), State::new(&john_doe).get(&height));
// }

// #[test]
// fn test_state_get_spval() {
//     let john_doe = john_doe();
//     State::new(&john_doe).get_spval("height");
// }

// #[test]
// fn test_state_get_spvar() {
//     let john_doe = john_doe();
//     let height = SPVariable::new(
//         "height",
//         &SPValueType::Int32,
//         &vec![180.to_spval(), 185.to_spval(), 190.to_spval()],
//     );
//     assert_eq!(height, State::new(&john_doe).get_spvar("height"));
// }

// #[test]
// #[should_panic]
// fn test_state_get_panic() {
//     let john_doe = john_doe();
//     let job = SPVariable::new(
//         "job",
//         &SPValueType::String,
//         &vec!["Carpenter".to_spval(), "Waiter".to_spval()],
//     );
//     State::new(&john_doe).get(&job);
// }

// #[test]
// #[should_panic]
// fn test_state_get_spval_panic() {
//     let john_doe = john_doe();
//     State::new(&john_doe).get_spval("job");
// }

// #[test]
// fn test_state_update() {
//     let john_doe = john_doe();
//     let old_state = State::new(&john_doe);
//     let new_state = old_state.clone().update("weight", &90.to_spval());
//     assert_ne!(old_state, new_state);
//     assert_eq!(90.to_spval(), new_state.clone().get_spval("weight"));
// }