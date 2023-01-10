#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    a, eq, bfs_transition_planner, t, v, Action, Predicate, SPValue, State, ToSPCommon,
    ToSPCommonVar, ToSPValue, Transition, SPVariable, SPValueType
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_planning_simple() {
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    let s = State::new(&HashMap::from([(pos.clone(), "a".to_spval())]));

    let t1 = t!(
        "a_to_b",
        eq!(&pos.cr(), "a".cl()),
        vec!(a!(pos.clone(), "b".cl()))
    );
    let t2 = t!(
        "b_to_c",
        eq!(&pos.cr(), "b".cl()),
        vec!(a!(pos.clone(), "c".cl()))
    );
    let t3 = t!(
        "c_to_d",
        eq!(&pos.cr(), "c".cl()),
        vec!(a!(pos.clone(), "d".cl()))
    );
    let t4 = t!(
        "d_to_e",
        eq!(&pos.cr(), "d".cl()),
        vec!(a!(pos.clone(), "e".cl()))
    );
    let t5 = t!(
        "e_to_f",
        eq!(&pos.cr(), "e".cl()),
        vec!(a!(pos.clone(), "f".cl()))
    );
    let t6 = t!(
        "a_to_c",
        eq!(&pos.cr(), "a".cl()),
        vec!(a!(pos.clone(), "c".cl()))
    );
    let t7 = t!(
        "d_to_f",
        eq!(&pos.cr(), "d".cl()),
        vec!(a!(pos.clone(), "f".cl()))
    );

    let result = bfs_transition_planner(
        s.clone(),
        eq!(&pos.cr(), "f".cl()),
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

    let result = bfs_transition_planner(
        s.clone(),
        eq!(&pos.cr(), "a".cl()),
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

    let result = bfs_transition_planner(
        s.clone(),
        eq!(&pos.cr(), "f".cl()),
        vec![t1.clone(), t2.clone()],
        10,
    );
    assert_eq!(result.found, false);
    assert_eq!(result.length, 0);
    assert_eq!(result.plan, Vec::<&str>::new());
}
