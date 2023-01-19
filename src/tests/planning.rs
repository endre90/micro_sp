#![allow(unused_imports)]
#![allow(dead_code)]
use crate::{
    a, eq, bfs_transition_planner, t_plan, v, Action, Predicate, SPValue, State, ToSPWrapped,
    ToSPWrappedVar, ToSPValue, Transition, SPVariable, SPValueType, SPVariableType
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_planning_simple() {
    let pos = v!("pos", vec!("a", "b", "c", "d", "e", "f"));
    let s = State::from_vec(&vec!((pos.clone(), "a".to_spvalue())));
    
    let t1 = t_plan!("a_to_b", eq!(pos.wrap(), "a".wrap()), vec!(a!(pos.clone(), "b".wrap())));
    let t2 = t_plan!("b_to_c", eq!(pos.wrap(), "b".wrap()), vec!(a!(pos.clone(), "c".wrap())));
    let t3 = t_plan!("c_to_d", eq!(pos.wrap(), "c".wrap()), vec!(a!(pos.clone(), "d".wrap())));
    let t4 = t_plan!("d_to_e", eq!(pos.wrap(), "d".wrap()), vec!(a!(pos.clone(), "e".wrap())));
    let t5 = t_plan!("e_to_f", eq!(pos.wrap(), "e".wrap()), vec!(a!(pos.clone(), "f".wrap())));
    let t6 = t_plan!("a_to_c", eq!(pos.wrap(), "a".wrap()), vec!(a!(pos.clone(), "c".wrap())));
    let t7 = t_plan!("d_to_f", eq!(pos.wrap(), "d".wrap()), vec!(a!(pos.clone(), "f".wrap())));

    let result = bfs_transition_planner(
        s.clone(),
        eq!(pos.wrap(), "f".wrap()),
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
        eq!(&pos.wrap(), "a".wrap()),
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
        eq!(&pos.wrap(), "f".wrap()),
        vec![t1.clone(), t2.clone()],
        10,
    );
    assert_eq!(result.found, false);
    assert_eq!(result.length, 0);
    assert_eq!(result.plan, Vec::<&str>::new());
}
