#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    a, eq, and, simple_transition_planner, TransitionHints, hint_transition, t, v, Action, Predicate, SPValue, State, ToSPCommon,
    ToSPCommonVar, ToSPValue, ToSPVariable, Transition, SPVariable, SPValueType, step_1
};
use std::collections::{HashMap, HashSet};

// TransitionHints, hint_transition

#[test]
fn test_step_2() {
    let stat = v!("stat", &vec!("on".to_spval(), "off".to_spval()));
    let pos = v!("pos", &vec!("a".to_spval(), "b".to_spval(), "c".to_spval(), "d".to_spval(), "e".to_spval(), "f".to_spval()));
    // let s = State::new(state)
    let s = State::new(&HashMap::from([(pos.clone(), "a".to_spval()), (stat.clone(), "off".to_spval())]));

    let t1 = t!(
        "a_to_b",
        and!(eq!("pos".to_comvar(&s), "a".to_comval()), eq!("stat".to_comvar(&s), "on".to_comval())),
        vec!(a!(pos.clone(), "b".to_comval()))
    );
    let t2 = t!(
        "b_to_c_faulty",
        and!(eq!("pos".to_comvar(&s), "b".to_comval()), eq!("stat".to_comvar(&s), "on".to_comval())),
        vec!(a!(pos.clone(), "a".to_comval()))
    );
    let t3 = t!(
        "c_to_d",
        and!(eq!("pos".to_comvar(&s), "c".to_comval()), eq!("stat".to_comvar(&s), "on".to_comval())),
        vec!(a!(pos.clone(), "d".to_comval()))
    );
    let t4 = t!(
        "turn_on",
        eq!("stat".to_comvar(&s), "off".to_comval()),
        vec!(a!(stat.clone(), "on".to_comval()))
    );
    let t5 = t!(
        "turn_off",
        eq!("stat".to_comvar(&s), "on".to_comval()),
        vec!(a!(stat.clone(), "off".to_comval()))
    );

    let result = hint_transition(
        s.clone(),
        and!(eq!("pos".to_comvar(&s), "d".to_comval()), eq!("stat".to_comvar(&s), "off".to_comval())),
        vec!(pos),
        vec![
            t1.clone(),
            t2.clone(),
            t3.clone(),
            t4.clone(),
            t5.clone()
        ],
        100,
        1,
    );
    println!("Length: {:?}", result.length);
    println!("Time: {:?}", result.time);
    for r in result.hints {
        println!("{:?}", r)
    }
}