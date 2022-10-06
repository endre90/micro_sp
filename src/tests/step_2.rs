#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    a, and, eq, eq2, s, simple_transition_planner, step_2, t, v, Action, Predicate, SPCommon,
    SPValue, SPValueType, SPVariable, State, ToSPCommon, ToSPCommonVar, ToSPValue, ToSPVariable,
    Transition,
};
use std::collections::{HashMap, HashSet};

// TransitionHints, hint_transition

#[test]
fn test_step_2() {
    let stat = v!("stat", &vec!("on", "off"));
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    // let s = State::new(state)
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
        eq!("stat".to_comvar(&s), "on".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "a".to_spval()),
            (stat.clone(), "off".to_spval())
        ]),
        eq!("pos".to_comvar(&s), "b".to_comval()),
    ));

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

    let not_taken_transitions = step_2(comb, transitions, 20);
    println!("not taken: {:?}", not_taken_transitions);
}

#[test]
fn test_step_2_2() {
    let stat = v!("stat", &vec!("on", "off"));
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    let s = State::new(&HashMap::from([
        (pos.clone(), "a".to_spval()),
        (stat.clone(), "off".to_spval()),
    ]));

    // introduce fault here
    let mut transitions = vec![];
    for pos1 in &pos.domain {
        for pos2 in &pos.domain {
            if pos1 != pos2 {
                transitions.push(t!(
                    &format!("{}_to_{}", &pos1.value_as_string(), &pos2.value_as_string()),
                    and!(
                        eq2!(&pos, pos1.value_as_string().to_comval()),
                        eq2!(&stat, "on".to_comval())
                    ),
                    vec!(a!(pos.clone(), pos1.value_as_string().to_comval()))
                ))
            }
        }
    }

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
        eq!("stat".to_comvar(&s), "on".to_comval()),
    ));
    comb.push((
        s!([
            (pos.clone(), "a".to_spval()),
            (stat.clone(), "off".to_spval())
        ]),
        eq!("pos".to_comvar(&s), "b".to_comval()),
    ));

    // even now we see that a_to_b failed to the problem is in the "to" transitions
    // at this point not taken: {"b_to_d", "e_to_d", "e_to_b", "turn_off", "f_to_c", "d_to_f", "c_to_e", "d_to_b", "c_to_f", "c_to_b",
    //"e_to_c", "a_to_f", "c_to_d", "e_to_a", "a_to_b", "f_to_e", "a_to_e", "a_to_d", "b_to_c", "b_to_f", "e_to_f", "b_to_a", "d_to_e",
    //"f_to_b", "d_to_c", "b_to_e", "c_to_a", "f_to_d", "a_to_c", "d_to_a", "f_to_a"}
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

    // at this point not taken: {"b_to_d", "e_to_d", "e_to_b", "f_to_c", "d_to_f", "c_to_e", "d_to_b", "c_to_f", "c_to_b",
    //"e_to_c", "a_to_f", "c_to_d", "e_to_a", "a_to_b", "f_to_e", "a_to_e", "a_to_d", "b_to_c", "b_to_f", "e_to_f", "b_to_a", "d_to_e",
    //"f_to_b", "d_to_c", "b_to_e", "c_to_a", "f_to_d", "a_to_c", "d_to_a", "f_to_a"}
    // which means that the error is in the way we defined all the "to" transitions

    let not_taken_transitions = step_2(comb, transitions, 20);
    println!("not taken: {:?}", not_taken_transitions);
}
