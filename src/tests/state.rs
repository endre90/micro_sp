#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{bv, bv_run, fv, fv_run, iv, iv_run, v, v_run, assign};
use crate::{SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, State, ToSPValue};
use std::collections::{HashMap, HashSet};

fn john_doe() -> Vec<(SPVariable, SPValue)> {
    let name = v!("name", vec!("John", "Jack"));
    let surname = v!("surname", vec!("Doe", "Crawford"));
    let height = iv!("height", vec!(180, 185, 190));
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv!("smart");

    vec![
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 80.0.to_spvalue()),
        (smart, true.to_spvalue()),
    ]
}

fn john_doe_faulty() -> Vec<(SPVariable, SPValue)> {
    let name = v!("name", vec!("John", "Jack"));
    let surname = v!("surname", vec!("Doe", "Crawford"));
    let height = iv!("height", vec!(180, 185, 190));
    let weight = fv!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv!("smart");

    vec![
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 81.0.to_spvalue()),
        (smart, true.to_spvalue()),
    ]
}

#[test]
fn test_state_new() {
    let new_state = State::new();
    assert_eq!(new_state.state.len(), 0)
}

#[test]
fn test_state_from_vec() {
    let john_doe = john_doe();
    let new_state = State::from_vec(&john_doe);
    assert_eq!(new_state.state.len(), 5)
}

#[test]
#[should_panic]
fn test_state_from_vec_panic() {
    let john_doe = john_doe();
    let new_state = State::from_vec(&john_doe);
    assert_eq!(new_state.state.len(), 6)
}

#[test]
fn test_state_get_value() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    assert_eq!(185.to_spvalue(), state.get_value("height"));
    assert_ne!(186.to_spvalue(), state.get_value("height"));
}

#[test]
fn test_state_get_all() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    assert_eq!(
        SPAssignment {
            var: iv!("height", vec!(180, 185, 190)),
            val: 185.to_spvalue()
        },
        state.get_all("height")
    );
    assert_ne!(
        SPAssignment {
            var: iv!("height", vec!(180, 185, 190)),
            val: 186.to_spvalue()
        },
        state.get_all("height")
    );
}

#[test]
fn test_state_contains() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    assert_eq!(true, state.contains("height"));
    assert_ne!(true, state.contains("wealth"));
}

#[test]
fn test_state_add_not_mutable() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    let wealth = iv!("wealth", vec!(1000, 2000));
    state.add(assign!(wealth, 2000.to_spvalue()));
    assert_ne!(state.state.len(), 6)
}

#[test]
fn test_state_add() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    let wealth = iv!("wealth", vec!(1000, 2000));
    let state = state.add(assign!(wealth, 2000.to_spvalue()));
    assert_eq!(state.state.len(), 6)
}

#[test]
#[should_panic]
fn test_state_add_already_exists() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    let wealth = iv!("height", vec!(1000, 2000));
    let state = state.add(assign!(wealth, 2000.to_spvalue()));
    assert_eq!(state.state.len(), 6)
}

#[test]
fn test_state_update() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    let state = state.update("height", 190.to_spvalue());
    assert_eq!(state.get_value("height"), 190.to_spvalue())
}

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
