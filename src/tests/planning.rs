#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    a, eq, simple_transition_planner, t, Action, Predicate, SPValue, State, ToSPValue, ToVal,
    ToVar, Transition,
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_planning_simple() {
    let s = State::new(HashMap::from([("pos".to_string(), "a".to_spvalue())]));

    let t1 = t!(
        "a_to_b",
        eq!("pos".to_var(), "a".to_val()),
        vec!(a!("pos", "b".to_val()))
    );
    let t2 = t!(
        "b_to_c",
        eq!("pos".to_var(), "b".to_val()),
        vec!(a!("pos", "c".to_val()))
    );
    let t3 = t!(
        "c_to_d",
        eq!("pos".to_var(), "c".to_val()),
        vec!(a!("pos", "d".to_val()))
    );
    let t4 = t!(
        "d_to_e",
        eq!("pos".to_var(), "d".to_val()),
        vec!(a!("pos", "e".to_val()))
    );
    let t5 = t!(
        "e_to_f",
        eq!("pos".to_var(), "e".to_val()),
        vec!(a!("pos", "f".to_val()))
    );
    let t6 = t!(
        "a_to_c",
        eq!("pos".to_var(), "a".to_val()),
        vec!(a!("pos", "c".to_val()))
    );
    let t7 = t!(
        "d_to_f",
        eq!("pos".to_var(), "d".to_val()),
        vec!(a!("pos", "f".to_val()))
    );

    let result = simple_transition_planner(
        s.clone(),
        eq!("pos".to_var(), "f".to_val()),
        vec![
            t1.clone(),
            t2.clone(),
            t3.clone(),
            t4.clone(),
            t5.clone(),
            t6.clone(),
            t7.clone(),
        ],
        10,
    );
    assert_eq!(result.found, true);
    assert_eq!(result.length, 3);
    assert_eq!(result.trace, vec!("a_to_c", "c_to_d", "d_to_f"));

    let result = simple_transition_planner(
        s.clone(),
        eq!("pos".to_var(), "a".to_val()),
        vec![
            t1.clone(),
            t2.clone(),
            t3.clone(),
            t4.clone(),
            t5.clone(),
            t6.clone(),
            t7.clone(),
        ],
        10,
    );
    assert_eq!(result.found, true);
    assert_eq!(result.length, 0);
    assert_eq!(result.trace, Vec::<&str>::new());

    let result = simple_transition_planner(
        s.clone(),
        eq!("pos".to_var(), "f".to_val()),
        vec![t1.clone(), t2.clone()],
        10,
    );
    assert_eq!(result.found, false);
    assert_eq!(result.length, 0);
    assert_eq!(result.trace, Vec::<&str>::new());
}