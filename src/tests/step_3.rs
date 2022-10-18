#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    a, and, eq, eq2, s, simple_transition_planner, step_3, t, v, Action, Predicate, SPCommon,
    SPValue, SPValueType, SPVariable, State, ToSPCommon, ToSPCommonVar, ToSPValue, ToSPVariable,
    Transition,
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_step_3() {
    let stat = v!("stat", &vec!("on", "off"));
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    let s = State::new(&HashMap::from([
        (pos.clone(), "a".to_spval()),
        (stat.clone(), "off".to_spval()),
    ]));

    let mut transitions = vec![];

    transitions.push(t!(
        "a_to_b",
        and!(
            eq!("pos".to_comvar(&s), "a".to_comval()),
            eq!("stat".to_comvar(&s), "on".to_comval())
        ),
        vec!(a!(pos.clone(), "b".to_comval()))
    ));
    transitions.push(t!(
        "b_to_c",
        and!(
            eq!("pos".to_comvar(&s), "b".to_comval()),
            eq!("stat".to_comvar(&s), "on".to_comval())
        ),
        vec!(a!(pos.clone(), "a".to_comval()))
    ));
    transitions.push(t!(
        "c_to_d",
        and!(
            eq!("pos".to_comvar(&s), "c".to_comval()),
            eq!("stat".to_comvar(&s), "on".to_comval())
        ),
        vec!(a!(pos.clone(), "d".to_comval()))
    ));
    transitions.push(t!(
        "turn_on",
        eq!("stat".to_comvar(&s), "off".to_comval()),
        vec!(a!(stat.clone(), "on".to_comval()))
    ));
    transitions.push(t!(
        "turn_off",
        eq!("stat".to_comvar(&s), "on".to_comval()),
        vec!(a!(stat.clone(), "off".to_comval()))
    ));

    // valid init/goal combinations
    let mut comb = vec![];

    // TODO: have to introduce don't cares in the initial state
    comb.push((
        s!([
            (pos.clone(), "a".to_spval()),
            (stat.clone(), "off".to_spval())
        ]),
        and!(
            eq!("pos".to_comvar(&s), "d".to_comval()),
            eq!("stat".to_comvar(&s), "off".to_comval())
        ),
    ));
    comb.push((
        s!([
            (stat.clone(), "off".to_spval()),
            (pos.clone(), "a".to_spval())
        ]),
        and!(
            eq!("pos".to_comvar(&s), "a".to_comval()),
            eq!("stat".to_comvar(&s), "on".to_comval())
        )
        
    ));
    comb.push((
        s!([
            (pos.clone(), "a".to_spval()),
            (stat.clone(), "off".to_spval())
        ]),
        eq!("pos".to_comvar(&s), "b".to_comval()),
    ));

    // let not_taken_transitions = step_3(comb.clone(), transitions.clone(), 20, 2, 50);
    // println!("not taken: {:?}", not_taken_transitions);

    // at this point not taken: {"turn_off", "b_to_c", "c_to_d"}
    comb.push((
        s!([
            (pos.clone(), "b".to_spval()),
            (stat.clone(), "on".to_spval())
        ]),
        eq!("pos".to_comvar(&s), "c".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "c".to_spval()),
            (stat.clone(), "on".to_spval())
        ]),
        eq!("pos".to_comvar(&s), "d".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "c".to_spval()),
            (stat.clone(), "on".to_spval())
        ]),
        eq!("stat".to_comvar(&s), "off".to_comval()),
    ));

    // at this point not taken {"b_to_c"}, but since we see that it is added as a valid combination, the error is there

    let generated = step_3(comb, transitions, 20, 3, 1000);
    for g in generated {
        println!("guard: {:?}", g.guard);
        println!("actions: ");
        for a in g.actions {
            println!("  {:?}", a)
        }
    }
}


#[test]
fn test_step_3_2() {
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    let s = State::new(&HashMap::from([
        (pos.clone(), "a".to_spval())
    ]));

    let mut transitions = vec![];

    transitions.push(t!(
        "a_to_b",
        eq!("pos".to_comvar(&s), "a".to_comval()),
        vec!(a!(pos.clone(), "b".to_comval()))
    ));
    transitions.push(t!(
        "b_to_c",
        eq!("pos".to_comvar(&s), "b".to_comval()),
        vec!(a!(pos.clone(), "a".to_comval()))
    ));
    transitions.push(t!(
        "c_to_d",
        eq!("pos".to_comvar(&s), "c".to_comval()),
        vec!(a!(pos.clone(), "d".to_comval()))
    ));

    // valid init/goal combinations
    let mut comb = vec![];

    // TODO: have to introduce don't cares in the initial state
    comb.push((
        s!([
            (pos.clone(), "a".to_spval()),
        ]),
        eq!("pos".to_comvar(&s), "b".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "a".to_spval()),
        ]),
        eq!("pos".to_comvar(&s), "c".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "a".to_spval()),
        ]),
        eq!("pos".to_comvar(&s), "d".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "b".to_spval()),
        ]),
        eq!("pos".to_comvar(&s), "c".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "b".to_spval()),
        ]),
        eq!("pos".to_comvar(&s), "d".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "c".to_spval()),
        ]),
        eq!("pos".to_comvar(&s), "d".to_comval()),
    ));

    // at this point not taken {"b_to_c"}, but since we see that it is added as a valid combination, the error is there

    let generated = step_3(comb, transitions, 20, 3, 1000);
    for g in generated {
        println!("guard: {:?}", g.guard);
        println!("actions: ");
        for a in g.actions {
            println!("  {:?}", a)
        }
    }
}