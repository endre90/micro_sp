use super::*;

pub fn blocksworld_model_enumerated_booleans_invariants(blocks: &Vec<&str>) -> (Vec<Transition>, Predicate) {
    let domain = vec!["true", "false"];

    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in blocks {
        pick_up_transitions.push(Transition::new(
            &format!("pick_up_{}", block),
            &pand!(
                &pass!(&new_enum_assign_c!(
                    &format!("clear_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
            ),
            &pand!(
                &pass!(&new_enum_assign_c!(
                    &format!("clear_{}", block),
                    &domain,
                    "false"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("ontable_{}", block),
                    &domain,
                    "false"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("holding_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("hand_empty"),
                    &domain,
                    "false"
                ))
            ),
        ))
    }

    for block in blocks {
        put_down_transitions.push(Transition::new(
            &format!("put_down_{}", block),
            &pass!(&new_enum_assign_c!(
                &format!("holding_{}", block),
                &domain,
                "true"
            )),
            &pand!(
                &pass!(&new_enum_assign_c!(
                    &format!("holding_{}", block),
                    &domain,
                    "false"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("clear_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("ontable_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
            ),
        ))
    }

    for b1 in blocks {
        for b2 in blocks {
            if b1 != b2 {
                stack_transitions.push(Transition::new(
                    &format!("stack_{}_on_{}", b1, b2),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("holding_{}", b1),
                            &domain,
                            "true"
                        ))
                    ),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b2),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("holding_{}", b1),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b1),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
                    ),
                ))
            }
        }
    }

    for b1 in blocks {
        for b2 in blocks {
            if b1 != b2 {
                unstack_transitions.push(Transition::new(
                    &format!("unstack_{}_from_{}", b1, b2),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b1),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
                    ),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("holding_{}", b1),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b1),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("hand_empty"),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            "false"
                        ))
                    ),
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
    // 5. [x] if block is on table, it is not on a block
    let mut invariants = vec![];
    let mut holding = vec![];
    for b1 in blocks {
        holding.push(pass!(&new_enum_assign_c!(
            &format!("holding_{}", b1),
            &domain,
            "true"
        )));
        for b2 in blocks {
            if b1 != b2 {
                invariants.push(pnot!(&pand!(
                    &pass!(&new_enum_assign_c!(
                        &format!("{}_on_{}", b1, b2),
                        &domain,
                        "true"
                    )),
                    &pass!(&new_enum_assign_c!(
                        &format!("{}_on_{}", b2, b1),
                        &domain,
                        "true"
                    ))
                )))
            }
        }
    }

    for b1 in blocks {
        let mut local_vec = vec![];
        for b2 in blocks {
            if b1 != b2 {
                local_vec.push(pass!(&new_enum_assign_c!(
                    &format!("{}_on_{}", b1, b2),
                    &domain,
                    "true"
                )))
            }
        }

        invariants.push(pnot!(&pand!(
            &pass!(&new_enum_assign_c!(
                &format!("ontable_{}", b1),
                &domain,
                "true"
            )),
            &Predicate::OR(local_vec.clone())
        )));

        invariants.push(Predicate::OR(vec![
            Predicate::PBEQ(local_vec.clone(), 1),
            Predicate::PBEQ(local_vec, 0),
        ]))
    }

    invariants.push(pnot!(&pand!(
        &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true")),
        &Predicate::OR(holding.clone())
    )));

    invariants.push(Predicate::OR(vec![
        Predicate::PBEQ(holding.clone(), 1),
        Predicate::PBEQ(holding, 0),
    ]));

    (transitions, Predicate::AND(invariants))
}

pub fn blocksworld_model_enumerated_booleans_explicit(blocks: &Vec<&str>) -> (Vec<Transition>, Predicate) {
    let domain = vec!["true", "false"];

    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in blocks {
        pick_up_transitions.push(Transition::new(
            &format!("pick_up_{}", block),
            &pand!(
                &pass!(&new_enum_assign_c!(
                    &format!("clear_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
            ),
            &pand!(
                &pass!(&new_enum_assign_c!(
                    &format!("clear_{}", block),
                    &domain,
                    "false"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("ontable_{}", block),
                    &domain,
                    "false"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("holding_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("hand_empty"),
                    &domain,
                    "false"
                ))
            ),
        ))
    }

    for block in blocks {
        put_down_transitions.push(Transition::new(
            &format!("put_down_{}", block),
            &pass!(&new_enum_assign_c!(
                &format!("holding_{}", block),
                &domain,
                "true"
            )),
            &pand!(
                &pass!(&new_enum_assign_c!(
                    &format!("holding_{}", block),
                    &domain,
                    "false"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("clear_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(
                    &format!("ontable_{}", block),
                    &domain,
                    "true"
                )),
                &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
            ),
        ))
    }

    for b1 in blocks {
        for b2 in blocks {
            if b1 != b2 {
                stack_transitions.push(Transition::new(
                    &format!("stack_{}_on_{}", b1, b2),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("holding_{}", b1),
                            &domain,
                            "true"
                        ))
                    ),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b2),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("holding_{}", b1),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b1),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
                    ),
                ))
            }
        }
    }

    for b1 in blocks {
        for b2 in blocks {
            if b1 != b2 {
                unstack_transitions.push(Transition::new(
                    &format!("unstack_{}_from_{}", b1, b2),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b1),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true"))
                    ),
                    &pand!(
                        &pass!(&new_enum_assign_c!(
                            &format!("holding_{}", b1),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b2),
                            &domain,
                            "true"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("clear_{}", b1),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("hand_empty"),
                            &domain,
                            "false"
                        )),
                        &pass!(&new_enum_assign_c!(
                            &format!("{}_on_{}", b1, b2),
                            &domain,
                            "false"
                        ))
                    ),
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
    // 5. [x] if block is on table, it is not on a block
    let mut invariants = vec![];
    let mut holding = vec![];
    for b1 in blocks {
        holding.push(pass!(&new_enum_assign_c!(
            &format!("holding_{}", b1),
            &domain,
            "true"
        )));
        for b2 in blocks {
            if b1 != b2 {
                invariants.push(pnot!(&pand!(
                    &pass!(&new_enum_assign_c!(
                        &format!("{}_on_{}", b1, b2),
                        &domain,
                        "true"
                    )),
                    &pass!(&new_enum_assign_c!(
                        &format!("{}_on_{}", b2, b1),
                        &domain,
                        "true"
                    ))
                )))
            }
        }
    }

    for b1 in blocks {
        let mut local_vec = vec![];
        for b2 in blocks {
            if b1 != b2 {
                local_vec.push(pass!(&new_enum_assign_c!(
                    &format!("{}_on_{}", b1, b2),
                    &domain,
                    "true"
                )))
            }
        }

        invariants.push(pnot!(&pand!(
            &pass!(&new_enum_assign_c!(
                &format!("ontable_{}", b1),
                &domain,
                "true"
            )),
            &Predicate::OR(local_vec.clone())
        )));

        invariants.push(Predicate::OR(vec![
            Predicate::PBEQ(local_vec.clone(), 1),
            Predicate::PBEQ(local_vec, 0),
        ]))
    }

    invariants.push(pnot!(&pand!(
        &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true")),
        &Predicate::OR(holding.clone())
    )));

    invariants.push(Predicate::OR(vec![
        Predicate::PBEQ(holding.clone(), 1),
        Predicate::PBEQ(holding, 0),
    ]));

    (transitions, Predicate::AND(invariants))
}

// pub fn blocksworld_model_booleans(blocks: &Vec<&str>) -> (Vec<Transition>, Predicate)
