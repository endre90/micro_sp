use micro_sp::{
    a, and, bv, eq, s, simple_transition_planner, t, v, Action, Predicate, SPValue, SPValueType,
    SPVariable, State, ToSPCommon, ToSPCommonVar, ToSPValue, Transition,
};

fn main() {
    let pos = v!("pos", &vec!("a", "b", "c"));
    let stat = v!("stat", &vec!("on", "off"));
    let grip = v!("grip", vec!("opened", "closed"));
    let occ = bv!("occ");
    let box1 = v!("box_1", vec!("at_a", "at_b", "at_c", "at_grip"));
    let box2 = v!("box_2", vec!("at_a", "at_b", "at_c", "at_grip"));
    let box3 = v!("box_3", vec!("at_a", "at_b", "at_c", "at_grip"));

    let mut transitions = vec![];
    transitions.push(t!(
        "turn_on",
        eq!(&stat.cr(), "off".cl()),
        vec!(a!(&stat, "on".cl()))
    ));
    transitions.push(t!(
        "turn_off",
        eq!(&stat.cr(), "on".cl()),
        vec!(a!(&stat, "off".cl()))
    ));
    for p1 in &pos.domain {
        for p2 in &pos.domain {
            if p1 != p2 {
                transitions.push(t!(
                    &format!("{}_to_{}", p1, p2),
                    and!(eq!(&stat.cr(), "on".cl()), eq!(&pos.cr(), p1.cl())),
                    vec!(a!(&pos, p2.cl()))
                ))
            }
        }
    }
    transitions.push(t!(
        "open_gripper",
        eq!(&grip.cr(), "closed".cl()),
        vec!(a!(&grip, "opened".cl()))
    ));
    transitions.push(t!(
        "close_gripper",
        eq!(&grip.cr(), "opened".cl()),
        vec!(a!(&grip, "closed".cl()))
    ));
    for b in vec![&box1, &box2, &box3] {
        for p in &pos.domain {
            transitions.push(t!(
                &format!("pick_{}_at_{}", b, p),
                and!(
                    eq!(&stat.cr(), "on".cl()),
                    eq!(&occ.cr(), false.cl()),
                    eq!(&pos.cr(), p.cl()),
                    eq!(&b.cr(), &format!("at_{p}").cl())
                ),
                vec!(a!(&occ, true.cl()), a!(b, &format!("at_grip").cl()))
            ))
        }
    }
    for b in vec![&box1, &box2, &box3] {
        for p in &pos.domain {
            transitions.push(t!(
                &format!("place_{}_at_{}", b, p),
                and!(
                    eq!(&stat.cr(), "on".cl()),
                    eq!(&occ.cr(), true.cl()),
                    eq!(&pos.cr(), p.cl()),
                    eq!(&b.cr(), &format!("at_grip").cl())
                ),
                vec!(a!(&occ, false.cl()), a!(b, &format!("at_{p}").cl()))
            ))
        }
    }

    // maybe extend each domain with and UNKNOWN, so that unititialized vars
    // are unknown rather that not in state
    let init = s!(vec!(
        (&stat, "off".to_spval()),
        (&grip, "closed".to_spval()),
        (&occ, false.to_spval()),
        (&pos, "c".to_spval()),
        (&box1, "at_a".to_spval()),
        (&box2, "at_b".to_spval()),
        (&box3, "at_b".to_spval())
    ));

    let goal = and!(
        eq!(&box1.cr(), "at_c".cl()),
        eq!(&box2.cr(), "at_c".cl()),
        eq!(&box3.cr(), "at_c".cl()),
        eq!(&grip.cr(), "closed".cl()),
        eq!(&stat.cr(), "off".cl()),
        eq!(&pos.cr(), "a".cl())
    );

    let result = simple_transition_planner(init, goal, transitions, 30);
    println!("{:?}", result.plan);
}