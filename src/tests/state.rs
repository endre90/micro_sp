#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    av_command, av_estimated, av_measured, av_runner, bv_command, bv_estimated, bv_measured,
    bv_runner, fv_command, fv_estimated, fv_measured, fv_runner, iv_command, iv_estimated,
    iv_measured, iv_runner, v_command, v_estimated, v_measured, v_runner, assign
};
use crate::{SPAssignment, SPValue, SPValueType, SPVariable, SPVariableType, State, ToSPValue, Transition};
use std::collections::{HashMap, HashSet};

fn john_doe() -> Vec<(SPVariable, SPValue)> {
    let name = v_estimated!("name", vec!("John", "Jack"));
    let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
    let height = iv_estimated!("height", vec!(180, 185, 190));
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv_estimated!("smart");

    vec![
        (name, "John".to_spvalue()),
        (surname, "Doe".to_spvalue()),
        (height, 185.to_spvalue()),
        (weight, 80.0.to_spvalue()),
        (smart, true.to_spvalue()),
    ]
}

fn john_doe_faulty() -> Vec<(SPVariable, SPValue)> {
    let name = v_estimated!("name", vec!("John", "Jack"));
    let surname = v_estimated!("surname", vec!("Doe", "Crawford"));
    let height = iv_estimated!("height", vec!(180, 185, 190));
    let weight = fv_estimated!("weight", vec!(80.0, 82.5, 85.0));
    let smart = bv_estimated!("smart");

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
fn test_state_display() {
    let john_doe = john_doe();
    let new_state = State::from_vec(&john_doe);
    print!("{}", new_state)
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
            var: iv_estimated!("height", vec!(180, 185, 190)),
            val: 185.to_spvalue()
        },
        state.get_all("height")
    );
    assert_ne!(
        SPAssignment {
            var: iv_estimated!("height", vec!(180, 185, 190)),
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
    let wealth = iv_estimated!("wealth", vec!(1000, 2000));
    state.add(assign!(wealth, 2000.to_spvalue()));
    assert_ne!(state.state.len(), 6)
}

#[test]
fn test_state_add() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    let wealth = iv_estimated!("wealth", vec!(1000, 2000));
    let state = state.add(assign!(wealth, 2000.to_spvalue()));
    assert_eq!(state.state.len(), 6)
}

#[test]
#[should_panic]
fn test_state_add_already_exists() {
    let john_doe = john_doe();
    let state = State::from_vec(&john_doe);
    let wealth = iv_estimated!("height", vec!(1000, 2000));
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
// #[should_panic]
// fn test_state_update_panic() {
//     let john_doe = john_doe();
//     let state = State::from_vec(&john_doe);
//     state.update("height", 123.to_spvalue());
// }