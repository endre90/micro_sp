use super::*;

pub fn blocksworld_model(blocks: &Vec<&str>) -> (Vec<Transition>, Predicate) {
    let domain = vec!["true", "false"];

    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in blocks {
        pick_up_transitions.push(Transition::new(
            &format!("pick_up_{}", block),
            &Predicate::AND(vec![
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("ontable_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("hand_empty"),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
            ]),
            &Predicate::AND(vec![
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("ontable_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("hand_empty"),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
            ]),
        ))
    }

    for block in blocks {
        put_down_transitions.push(Transition::new(
            &format!("put_down_{}", block),
            &Predicate::EQ(EnumValue::new(
                &EnumVariable::new(
                    &format!("holding_{}", block),
                    &domain,
                    "boolean",
                    None,
                    &Kind::Command,
                ),
                "true",
                None,
            )),
            &Predicate::AND(vec![
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("ontable_{}", block),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("hand_empty"),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
            ]),
        ))
    }

    for b1 in blocks {
        for b2 in blocks {
            if b1 != b2 {
                stack_transitions.push(Transition::new(
                    &format!("stack_{}_on_{}", b1, b2),
                    &Predicate::AND(vec![
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b2),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("holding_{}", b1),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &Predicate::AND(vec![
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("holding_{}", b1),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "false",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b2),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "false",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b1),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_on_{}", b1, b2),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("hand_empty"),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                ))
            }
        }
    }

    for b1 in blocks {
        for b2 in blocks {
            if b1 != b2 {
                unstack_transitions.push(Transition::new(
                    &format!("unstack_{}_from_{}", b1, b2),
                    &Predicate::AND(vec![
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_on_{}", b1, b2),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b1),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("hand_empty"),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &Predicate::AND(vec![
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("hand_empty"),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "false",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b2),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b1),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "false",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("holding_{}", b1),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_on_{}", b1, b2),
                                &domain,
                                "boolean",
                                None,
                                &Kind::Command,
                            ),
                            "false",
                            None,
                        )),
                    ]),
                ))
            }
        }
    }

    let mut transitions = vec![];
    for t in vec![
        pick_up_transitions,
        put_down_transitions,
        stack_transitions,
        unstack_transitions,
    ] {
        transitions.extend(t)
    }

    // Added invariants to make it work:
    // 1. [x] block can't be on another block if that block is on the first block
    // 2. [x] if holding any block, the gripper can't be empty
    // 3. [x] at most one block can be held
    // 4. [x] a block can't simultaneously be on several different blocks
    // 5. [ ] if block is on table, it is not on a block
    let mut invariants = vec![];
    let mut holding = vec![];
    for b1 in blocks {
        holding.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("holding_{}", b1),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )));
        for b2 in blocks {
            if b1 != b2 {
                invariants.push(Predicate::NOT(Box::new(Predicate::AND(vec![
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            "boolean",
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on_{}", b2, b1),
                            &domain,
                            "boolean",
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                ]))))
            }
        }
    }

    for b1 in blocks {
        let mut local_vec = vec![];
        for b2 in blocks {
            if b1 != b2 {
                local_vec.push(Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("{}_on_{}", b1, b2),
                        &domain,
                        "boolean",
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )))
            }
        }

        invariants.push(Predicate::NOT(Box::new(Predicate::AND(vec![
            Predicate::EQ(EnumValue::new(
                &EnumVariable::new(
                    &format!("ontable_{}", b1),
                    &domain,
                    "boolean",
                    None,
                    &Kind::Command,
                ),
                "true",
                None,
            )),
            Predicate::OR(local_vec.clone()),
        ]))));

        invariants.push(Predicate::OR(vec![
            Predicate::PBEQ(local_vec.clone(), 1),
            Predicate::PBEQ(local_vec, 0),
        ]))
    }

    // for b in &blocks {
    //     invariants.push(Predicate::AND(vec![]))
    // }

    invariants.push(Predicate::NOT(Box::new(Predicate::AND(vec![
        Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("hand_empty"),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )),
        Predicate::OR(holding.clone()),
    ]))));

    invariants.push(Predicate::OR(vec![
        Predicate::PBEQ(holding.clone(), 1),
        Predicate::PBEQ(holding, 0),
    ]));

    (transitions, Predicate::AND(invariants))
}
