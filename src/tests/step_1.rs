#![allow(unused_imports)]
#![allow(dead_code)]
use micro_sp::{
    a, and, eq, eq2, simple_transition_planner, step_1, t, v, Action, Predicate, SPCommon, SPValue,
    SPValueType, SPVariable, State, ToSPCommon, ToSPCommonVar, ToSPValue, ToSPVariable, Transition,
};
use std::collections::{HashMap, HashSet};

#[test]
fn test_step_1_1() {
    let stat = v!("stat", &vec!("on", "off"));
    let pos = v!("pos", &vec!("a", "b", "c", "d", "e", "f"));
    let s = State::new(&HashMap::from([
        (pos.clone(), "a".to_spval()),
        (stat.clone(), "off".to_spval()),
    ]));

    let t1 = t!(
        "a_to_b",
        and!(
            eq!("pos".to_comvar(&s), "a".to_comval()),
            eq!("stat".to_comvar(&s), "on".to_comval())
        ),
        vec!(a!(pos.clone(), "b".to_comval()))
    );
    // introduce fault here
    let t2 = t!(
        "b_to_c",
        and!(
            eq2!(&pos, "b".to_comval()),
            eq!("stat".to_comvar(&s), "on".to_comval())
        ),
        vec!(a!(pos.clone(), "a".to_comval()))
    );
    let t3 = t!(
        "c_to_d",
        and!(
            eq!("pos".to_comvar(&s), "c".to_comval()),
            eq!("stat".to_comvar(&s), "on".to_comval())
        ),
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

    let result = step_1(
        vec![pos, stat],
        vec![t1.clone(), t2.clone(), t3.clone(), t4.clone(), t5.clone()],
        100, // max_tries
        50,  // max_combinations
        5,   // max solutions
        20,  // max_plan_lenght
    );

    println!("combination coverage: {}%", result.combination_coverage);
    println!("solution coverage: {}%", result.solution_coverage);
    println!("results shown: {}", result.solution.len());
    println!("time to solve: {:?}", result.time);
    println!("-----------------------------");
    for r in result.solution {
        let mut inits =
            r.0.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        inits.sort();
        let mut goals =
            r.1.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        goals.sort();

        println!("init: {:?}", inits);
        println!("goal: {:?}", goals);
        println!("plan: {:?}", r.2.trace);
        println!("-----------------------------");
    }
}

#[test]
fn test_step_1_2() {
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

    let result = step_1(
        vec![pos, stat],
        transitions,
        100, // max_tries
        50,  // max_combinations
        5,   // max solutions
        20,  // max_plan_lenght
    );

    println!("combination coverage: {}%", result.combination_coverage);
    println!("solution coverage: {}%", result.solution_coverage);
    println!("results shown: {}", result.solution.len());
    println!("time to solve: {:?}", result.time);
    println!("-----------------------------");
    for r in result.solution {
        let mut inits =
            r.0.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        inits.sort();
        let mut goals =
            r.1.state
                .iter()
                .map(|(var, val)| format!("{} = {}", var.name, val))
                .collect::<Vec<String>>();
        goals.sort();

        println!("init: {:?}", inits);
        println!("goal: {:?}", goals);
        println!("plan: {:?}", r.2.trace);
        println!("-----------------------------");
    }
}
