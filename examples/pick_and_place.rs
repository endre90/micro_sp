use micro_sp::{
    a, and, bv, eq, s, bfs_transition_planner, t, v, Action, Predicate, SPValue, SPValueType, SPVariableType,
    SPVariable, State, ToSPWrapped, ToSPWrappedVar, ToSPValue, Transition,
};

fn main() {
    let pos = v!("pos", vec!("a", "b", "c"));
    let stat = v!("stat", vec!("on", "off"));
    let grip = v!("grip", vec!("opened", "closed"));
    let occ = bv!("occ");
    let box1 = v!("box_1", vec!("at_a", "at_b", "at_c", "at_grip"));
    let box2 = v!("box_2", vec!("at_a", "at_b", "at_c", "at_grip"));
    let box3 = v!("box_3", vec!("at_a", "at_b", "at_c", "at_grip"));

    let mut transitions = vec![];
    transitions.push(t!(
        "turn_on",
        eq!(&stat.wrap(), "off".wrap()),
        vec!(a!(&stat, "on".wrap()))
    ));
    transitions.push(t!(
        "turn_off",
        eq!(&stat.wrap(), "on".wrap()),
        vec!(a!(&stat, "off".wrap()))
    ));
    for p1 in &pos.domain {
        for p2 in &pos.domain {
            if p1 != p2 {
                transitions.push(t!(
                    &format!("{}_to_{}", p1, p2),
                    and!(eq!(&stat.wrap(), "on".wrap()), eq!(&pos.wrap(), p1.wrap())),
                    vec!(a!(&pos, p2.wrap()))
                ))
            }
        }
    }
    transitions.push(t!(
        "open_gripper",
        eq!(&grip.wrap(), "closed".wrap()),
        vec!(a!(&grip, "opened".wrap()))
    ));
    transitions.push(t!(
        "close_gripper",
        eq!(&grip.wrap(), "opened".wrap()),
        vec!(a!(&grip, "closed".wrap()))
    ));
    for b in vec![&box1, &box2, &box3] {
        for p in &pos.domain {
            transitions.push(t!(
                &format!("pick_{}_at_{}", b, p),
                and!(
                    eq!(&stat.wrap(), "on".wrap()),
                    eq!(&occ.wrap(), false.wrap()),
                    eq!(&pos.wrap(), p.wrap()),
                    eq!(&b.wrap(), &format!("at_{p}").wrap())
                ),
                vec!(a!(&occ, true.wrap()), a!(b, &format!("at_grip").wrap()))
            ))
        }
    }
    for b in vec![&box1, &box2, &box3] {
        for p in &pos.domain {
            transitions.push(t!(
                &format!("place_{}_at_{}", b, p),
                and!(
                    eq!(&stat.wrap(), "on".wrap()),
                    eq!(&occ.wrap(), true.wrap()),
                    eq!(&pos.wrap(), p.wrap()),
                    eq!(&b.wrap(), &format!("at_grip").wrap())
                ),
                vec!(a!(&occ, false.wrap()), a!(b, &format!("at_{p}").wrap()))
            ))
        }
    }

    let init = s!(vec!(
        (&stat, "off".to_spvalue()),
        (&grip, "closed".to_spvalue()),
        (&occ, false.to_spvalue()),
        (&pos, "c".to_spvalue()),
        (&box1, "at_a".to_spvalue()),
        (&box2, "at_b".to_spvalue()),
        (&box3, "at_b".to_spvalue())
    ));

    let goal = and!(
        eq!(&box1.wrap(), "at_c".wrap()),
        eq!(&box2.wrap(), "at_c".wrap()),
        eq!(&box3.wrap(), "at_c".wrap()),
        eq!(&grip.wrap(), "closed".wrap()),
        eq!(&stat.wrap(), "off".wrap()),
        eq!(&pos.wrap(), "a".wrap())
    );

    let result = bfs_transition_planner(init, goal, transitions, 30);
    println!("{:?}", result.plan);
}