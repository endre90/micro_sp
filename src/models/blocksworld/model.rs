use super::*;

// macro_rules! new_enum_assign_c {
//     ($name:expr, $domain:expr, $val:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr) => { ... };
//     ($name:expr, $domain:expr, $val:expr, $r#type:expr, $param:expr, $life:expr) => { ... };
// }

pub fn enum_bool_with_invariants(name: &str) -> ParamPlanningProblem {
    let domain = vec!["true", "false"];

    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in blocks {
        pick_up_transitions.push(
            ParamTransition::new(
                &format!("pick_up_{}", block),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("clear_{}", block), &domain, "true", "block", "clear")),
                    &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true", "hand", "hand"))
                ),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("clear_{}", block), &domain, "false", "block", "clear")),
                    &pass!(&new_enum_assign_c!(&format!("ontable_{}", block), &domain, "false", "block", "ontable")),
                    &pass!(&new_enum_assign_c!(&format!("holding_{}", block), &domain, "true", "block", "holding")),
                    &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "false", "hand", "hand"))
                )
            )
        )
    }

    for block in blocks {
        put_down_transitions.push(
            ParamTransition::new(
                &format!("put_down_{}", block),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("holding_{}", block), &domain, "true", "block", "holding")),
                ),
                &ppred!(
                    &pass!(&new_enum_assign_c!(&format!("holding_{}", block), &domain, "false", "block", "holding")),
                    &pass!(&new_enum_assign_c!(&format!("clear_{}", block), &domain, "true", "block", "clear")),
                    &pass!(&new_enum_assign_c!(&format!("ontable_{}", block), &domain, "true", "block", "ontable")),
                    &pass!(&new_enum_assign_c!(&format!("hand_empty"), &domain, "true", "hand", "hand"))
                )
            )
        )
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