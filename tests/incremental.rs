use lib::*;

#[test]
pub fn test_raar_incremental_1() {
    let stat_domain = vec!["idle", "active", "unknown", "dummy_value"];
    let pos_domain = vec!["left", "right", "unknown", "dummy_value"];

    let act_pos = EnumVariable::new("act_pos", &pos_domain, "pos", None, &Kind::Measured);

    let ref_pos = EnumVariable::new("ref_pos", &pos_domain, "pos", None, &Kind::Command);

    let act_stat = EnumVariable::new("act_stat", &stat_domain, "stat", None, &Kind::Measured);

    let ref_stat = EnumVariable::new("ref_stat", &stat_domain, "stat", None, &Kind::Command);

    let act_ref_pos = EnumVariable::new("act_ref_pos", &pos_domain, "pos", None, &Kind::Measured);

    let act_ref_stat =
        EnumVariable::new("act_ref_stat", &stat_domain, "stat", None, &Kind::Measured);

    let act_left = Predicate::EQ(EnumValue::new(&act_pos, "left", None));
    let not_act_left = Predicate::NOT(Box::new(act_left.clone()));
    let act_right = Predicate::EQ(EnumValue::new(&act_pos, "right", None));
    let not_act_right = Predicate::NOT(Box::new(act_right.clone()));

    let act_idle = Predicate::EQ(EnumValue::new(&act_stat, "idle", None));
    let not_act_idle = Predicate::NOT(Box::new(act_idle.clone()));
    let act_active = Predicate::EQ(EnumValue::new(&act_stat, "active", None));
    let not_act_active = Predicate::NOT(Box::new(act_active.clone()));

    let ref_left = Predicate::EQ(EnumValue::new(&ref_pos, "left", None));
    let not_ref_left = Predicate::NOT(Box::new(ref_left.clone()));
    let ref_right = Predicate::EQ(EnumValue::new(&ref_pos, "right", None));
    let not_ref_right = Predicate::NOT(Box::new(ref_right.clone()));

    let ref_idle = Predicate::EQ(EnumValue::new(&ref_stat, "idle", None));
    let not_ref_idle = Predicate::NOT(Box::new(ref_idle.clone()));
    let ref_active = Predicate::EQ(EnumValue::new(&ref_stat, "active", None));
    let not_ref_active = Predicate::NOT(Box::new(ref_active.clone()));

    let act_ref_left = Predicate::EQ(EnumValue::new(&act_ref_pos, "left", None));
    let not_act_ref_left = Predicate::NOT(Box::new(act_ref_left.clone()));
    let act_ref_right = Predicate::EQ(EnumValue::new(&act_ref_pos, "right", None));
    let not_act_ref_right = Predicate::NOT(Box::new(act_ref_right.clone()));

    let act_ref_idle = Predicate::EQ(EnumValue::new(&act_ref_stat, "idle", None));
    let not_act_ref_idle = Predicate::NOT(Box::new(act_ref_idle.clone()));
    let act_ref_active = Predicate::EQ(EnumValue::new(&act_ref_stat, "active", None));
    let not_act_ref_active = Predicate::NOT(Box::new(act_ref_active.clone()));

    let t1 = Transition::new(
        "start_activate",
        &Predicate::AND(vec![not_act_active.clone(), not_act_ref_active.clone()]),
        &Predicate::AND(vec![ref_active.clone(), act_ref_active.clone()]),
    );

    let t2 = Transition::new(
        "finish_activate",
        &Predicate::AND(vec![not_act_active.clone(), act_ref_active.clone()]),
        &Predicate::AND(vec![act_active.clone(), act_ref_active.clone()]),
    );

    let t3 = Transition::new(
        "start_deactivate",
        &Predicate::AND(vec![not_act_idle.clone(), not_act_ref_idle.clone()]),
        &Predicate::AND(vec![ref_idle.clone(), act_ref_idle.clone()]),
    );

    let t4 = Transition::new(
        "finish_deactivate",
        &Predicate::AND(vec![not_act_active.clone(), act_ref_idle.clone()]),
        &Predicate::AND(vec![act_idle.clone(), act_ref_idle.clone()]),
    );

    let t5 = Transition::new(
        "start_move_left",
        &Predicate::AND(vec![
            not_act_left.clone(),
            not_act_ref_left.clone(),
            act_active.clone(),
            act_ref_active.clone(),
        ]),
        &Predicate::AND(vec![ref_left.clone(), act_ref_left.clone()]),
    );

    let t6 = Transition::new(
        "finish_move_left",
        &Predicate::AND(vec![
            not_act_left.clone(),
            act_ref_left.clone(),
            act_active.clone(),
            act_ref_active.clone(),
        ]),
        &Predicate::AND(vec![
            act_left.clone(),
            ref_left.clone(),
            act_ref_left.clone(),
        ]),
    );

    let t7 = Transition::new(
        "start_move_right",
        &Predicate::AND(vec![
            not_act_right.clone(),
            not_act_ref_right.clone(),
            act_active.clone(),
            act_ref_active.clone(),
        ]),
        &Predicate::AND(vec![ref_right.clone(), act_ref_right.clone()]),
    );

    let t8 = Transition::new(
        "finish_move_right",
        &Predicate::AND(vec![
            not_act_right.clone(),
            act_ref_right.clone(),
            act_active.clone(),
            act_ref_active.clone(),
        ]),
        &Predicate::AND(vec![
            act_right.clone(),
            ref_right.clone(),
            act_ref_right.clone(),
        ]),
    );

    let problem = PlanningProblem::new(
        "prob1",
        &Predicate::AND(vec![
            act_idle.clone(),
            act_ref_idle.clone(),
            act_left.clone(),
            act_ref_left.clone(),
        ]),
        &Predicate::AND(vec![
            act_active.clone(),
            ref_active.clone(),
            act_right.clone(),
            ref_right.clone(),
        ]),
        &vec![t1, t2, t3, t4, t5, t6, t7, t8],
        &12,
        &Paradigm::Raar,
    );
    let result = incremental(&problem);
    pprint_result(&result)
}

#[test]
pub fn blocks_model_test() {
    let domain = vec!["true", "false"];
    let boolean = "boolean";
    let hand = EnumVariable::new("hand", &domain, boolean, None, &Kind::Command);
    let hand_empty = Predicate::EQ(EnumValue::new(&hand, "true", None));
    let hand_full = Predicate::EQ(EnumValue::new(&hand, "false", None));

    let blocks = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in &blocks {
        pick_up_transitions.push(Transition::new(
            &format!("pick_up_{}", block),
            &Predicate::AND(vec![
                hand_empty.clone(),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        boolean,
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
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
            ]),
            &Predicate::AND(vec![
                hand_full.clone(),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("ontable_{}", block),
                        &domain,
                        boolean,
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
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
            ]),
        ))
    }

    for block in &blocks {
        put_down_transitions.push(Transition::new(
            &format!("put_down_{}", block),
            &Predicate::EQ(EnumValue::new(
                &EnumVariable::new(
                    &format!("holding_{}", block),
                    &domain,
                    boolean,
                    None,
                    &Kind::Command,
                ),
                "true",
                None,
            )),
            &Predicate::AND(vec![
                hand_empty.clone(),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &domain,
                        boolean,
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
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding_{}", block),
                        &domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )),
            ]),
        ))
    }

    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                stack_transitions.push(Transition::new(
                    &format!("stack_{}_on_{}", b1, b2),
                    &Predicate::AND(vec![
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b2),
                                &domain,
                                boolean,
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
                                boolean,
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &Predicate::AND(vec![
                        hand_empty.clone(),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b2),
                                &domain,
                                boolean,
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
                                boolean,
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
                                boolean,
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
                                boolean,
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

    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                unstack_transitions.push(Transition::new(
                    &format!("unstack_{}_from_{}", b1, b2),
                    &Predicate::AND(vec![
                        hand_empty.clone(),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_on_{}", b1, b2),
                                &domain,
                                boolean,
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
                                boolean,
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &Predicate::AND(vec![
                        hand_full.clone(),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b2),
                                &domain,
                                boolean,
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
                                boolean,
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
                                boolean,
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
                                boolean,
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

    let mut clear_predicates = vec![];
    for x in vec!["C", "F"] {
        clear_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("clear_{}", x),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut ontable_predicates = vec![];
    for x in vec!["I", "F"] {
        ontable_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("ontable_{}", x),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut on_predicates = vec![];
    for (b1, b2) in vec![
        ("C", "E"),
        ("E", "J"),
        ("J", "B"),
        ("B", "G"),
        ("G", "H"),
        ("H", "A"),
        ("A", "D"),
        ("D", "I"),
    ] {
        on_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on_{}", b1, b2),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let initial = Predicate::AND(vec![
        Predicate::AND(clear_predicates),
        Predicate::AND(ontable_predicates),
        Predicate::AND(on_predicates),
        hand_empty,
    ]);

    let mut goal_on_predicates = vec![];
    for (b1, b2) in vec![
        ("D", "C"),
        ("C", "F"),
        ("F", "J"),
        ("J", "E"),
        ("E", "H"),
        ("H", "B"),
        ("B", "A"),
        ("A", "G"),
        ("G", "I"),
    ] {
        goal_on_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on_{}", b1, b2),
                &domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
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

    let goal = Predicate::AND(goal_on_predicates);
    let problem = PlanningProblem::new(
        "blocks_world",
        &initial,
        &goal,
        &transitions,
        &50,
        &Paradigm::Raar,
    );

    let result = incremental(&problem);
    pprint_result(&result)
}
