#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    a, eq, simple_transition_planner, t, v, Action, Predicate, SPValue, State, ToSPCommon,
    ToSPCommonVar, ToSPValue, ToSPVariable, Transition, SPVariable, SPValueType
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_planning_simple() {
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    // let s = State::new(state)
    let s = State::new(&HashMap::from([(pos.clone(), "a".to_spval())]));

    let t1 = t!(
        "a_to_b",
        eq!("pos".to_comvar(&s), "a".to_comval()),
        vec!(a!(pos.clone(), "b".to_comval()))
    );
    let t2 = t!(
        "b_to_c",
        eq!("pos".to_comvar(&s), "b".to_comval()),
        vec!(a!(pos.clone(), "c".to_comval()))
    );
    let t3 = t!(
        "c_to_d",
        eq!("pos".to_comvar(&s), "c".to_comval()),
        vec!(a!(pos.clone(), "d".to_comval()))
    );
    let t4 = t!(
        "d_to_e",
        eq!("pos".to_comvar(&s), "d".to_comval()),
        vec!(a!(pos.clone(), "e".to_comval()))
    );
    let t5 = t!(
        "e_to_f",
        eq!("pos".to_comvar(&s), "e".to_comval()),
        vec!(a!(pos.clone(), "f".to_comval()))
    );
    let t6 = t!(
        "a_to_c",
        eq!("pos".to_comvar(&s), "a".to_comval()),
        vec!(a!(pos.clone(), "c".to_comval()))
    );
    let t7 = t!(
        "d_to_f",
        eq!("pos".to_comvar(&s), "d".to_comval()),
        vec!(a!(pos.clone(), "f".to_comval()))
    );

    let result = simple_transition_planner(
        s.clone(),
        eq!("pos".to_comvar(&s), "f".to_comval()),
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
    assert_eq!(result.plan, vec!("a_to_c", "c_to_d", "d_to_f"));

    let result = simple_transition_planner(
        s.clone(),
        eq!("pos".to_comvar(&s), "a".to_comval()),
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
    assert_eq!(result.plan, Vec::<&str>::new());

    let result = simple_transition_planner(
        s.clone(),
        eq!("pos".to_comvar(&s), "f".to_comval()),
        vec![t1.clone(), t2.clone()],
        10,
    );
    assert_eq!(result.found, false);
    assert_eq!(result.length, 0);
    assert_eq!(result.plan, Vec::<&str>::new());
}
