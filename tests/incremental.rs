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
        &Predicate::TRUE,
        &12,
        &Paradigm::Raar,
    );
    let result = incremental(&problem);
    pprint_result(&result)
}

#[test]
pub fn blocks_model_test() {
    let tf_domain = vec!["true", "false"];
    let on_domain = vec!["GRIPPER", "TABLE", "A", "B"]; //, "C", "D", "E", "F", "G", "H", "I", "J"];
    let holding_domain = vec!["EMPTY", "A", "B"]; //, "C", "D", "E", "F", "G", "H", "I", "J"];
    let holding = "holding";
    let position = "position";
    let boolean = "boolean";

    let blocks = vec!["A", "B"]; //, "C", "D", "E", "F", "G", "H", "I", "J"];
    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in &blocks {
        pick_up_transitions.push(Transition::new(
            &format!("pick_up_{}", block),
            &Predicate::AND(vec![
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &tf_domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "true",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("{}_on", block),
                        &on_domain,
                        position,
                        None,
                        &Kind::Command,
                    ),
                    "TABLE",
                    None,
                )),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding"),
                        &holding_domain,
                        holding,
                        None,
                        &Kind::Command,
                    ),
                    "EMPTY",
                    None,
                )),
            ]),
            &Predicate::AND(vec![
                Predicate::NOT(Box::new(Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("{}_on", block),
                        &on_domain,
                        position,
                        None,
                        &Kind::Command,
                    ),
                    "TABLE",
                    None,
                )))),
                Predicate::NOT(Box::new(Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("clear_{}", block),
                        &tf_domain,
                        boolean,
                        None,
                        &Kind::Command,
                    ),
                    "false",
                    None,
                )))),
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding"),
                        &holding_domain,
                        holding,
                        None,
                        &Kind::Command,
                    ),
                    block,
                    None,
                )),
            ]),
        ))
    }

    for block in &blocks {
        if block != &"TABLE" {
            put_down_transitions.push(Transition::new(
                &format!("put_down_{}", block),
                &Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &format!("holding"),
                        &holding_domain,
                        holding,
                        None,
                        &Kind::Command,
                    ),
                    block,
                    None,
                )),
                &Predicate::AND(vec![
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("holding"),
                            &holding_domain,
                            holding,
                            None,
                            &Kind::Command,
                        ),
                        "EMPTY",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("clear_{}", block),
                            &tf_domain,
                            boolean,
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on", block),
                            &on_domain,
                            position,
                            None,
                            &Kind::Command,
                        ),
                        "TABLE",
                        None,
                    )),
                ]),
            ))
        }
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
                                &tf_domain,
                                boolean,
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("holding"),
                                &holding_domain,
                                holding,
                                None,
                                &Kind::Command,
                            ),
                            b1,
                            None,
                        )),
                    ]),
                    &Predicate::AND(vec![
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("holding"),
                                &holding_domain,
                                holding,
                                None,
                                &Kind::Command,
                            ),
                            "EMPTY",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b2),
                                &tf_domain,
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
                                &tf_domain,
                                boolean,
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_on", b1),
                                &on_domain,
                                position,
                                None,
                                &Kind::Command,
                            ),
                            b2,
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
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("holding"),
                                &holding_domain,
                                holding,
                                None,
                                &Kind::Command,
                            ),
                            "EMPTY",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_on", b1),
                                &on_domain,
                                position,
                                None,
                                &Kind::Command,
                            ),
                            b2,
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("clear_{}", b1),
                                &tf_domain,
                                boolean,
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
                                &format!("clear_{}", b2),
                                &tf_domain,
                                boolean,
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("holding"),
                                &holding_domain,
                                holding,
                                None,
                                &Kind::Command,
                            ),
                            b1,
                            None,
                        )),
                        Predicate::NOT(Box::new(Predicate::EQ(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_on", b1),
                                &on_domain,
                                position,
                                None,
                                &Kind::Command,
                            ),
                            b2,
                            None,
                        )))),
                    ]),
                ))
            }
        }
    }

    let mut clear_predicates = vec![];
    for x in vec!["B"] {
        clear_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("clear_{}", x),
                &tf_domain,
                boolean,
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut ontable_predicates = vec![];
    for x in vec!["A"] {
        ontable_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on", x),
                &on_domain,
                position,
                None,
                &Kind::Command,
            ),
            "TABLE",
            None,
        )))
    }

    let mut on_predicates = vec![];
    for (b1, b2) in vec![
        ("A", "B")
        // ("C", "E"),
        // ("E", "J"),
        // ("J", "B"),
        // ("B", "G"),
        // ("G", "H"),
        // ("H", "A"),
        // ("A", "D"),
        // ("D", "I"),
    ] {
        on_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on", b2),
                &on_domain,
                position,
                None,
                &Kind::Command,
            ),
            b1,
            None,
        )))
    }

    let initial = Predicate::AND(vec![
        Predicate::AND(clear_predicates),
        Predicate::AND(ontable_predicates),
        Predicate::AND(on_predicates),
        Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("holding"),
                &holding_domain,
                holding,
                None,
                &Kind::Command,
            ),
            "EMPTY",
            None,
        )),
    ]);

    let mut goal_on_predicates = vec![];
    for (b1, b2) in vec![
        // ("D", "C"),
        // ("C", "F"),
        // ("F", "J"),
        // ("J", "E"),
        // ("E", "H"),
        // ("H", "B"),
        // ("B", "A"),
        // ("A", "G"),
        // ("G", "I"),
        ("B", "A"),
    ] {
        goal_on_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("{}_on", b2),
                &on_domain,
                position,
                None,
                &Kind::Command,
            ),
            b1,
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

    let mut invariants = vec![];
    for b1 in &blocks {
        for b2 in &blocks {
            if b1 != b2 {
                invariants.push(Predicate::NOT(Box::new(Predicate::AND(vec![
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on", b1),
                            &on_domain,
                            position,
                            None,
                            &Kind::Command,
                        ),
                        b2,
                        None,
                    )),
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &format!("{}_on", b2),
                            &on_domain,
                            position,
                            None,
                            &Kind::Command,
                        ),
                        b1,
                        None,
                    )),
                ]))))
            }
        }
    }

    // for b in &blocks {
    //     invariants.push(Predicate::AND(vec![]))
    // }

    let goal = Predicate::AND(goal_on_predicates);
    let problem = PlanningProblem::new(
        "blocks_world",
        &initial,
        &goal,
        &transitions,
        &Predicate::AND(invariants),
        &50,
        &Paradigm::Raar,
    );

    let result = incremental(&problem);
    pprint_result(&result)
}

#[test]
pub fn blocks_model_test_2() {
    let domain = vec!["true", "false"];

    let blocks = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    let mut pick_up_transitions = vec![];
    let mut put_down_transitions = vec![];
    let mut stack_transitions = vec![];
    let mut unstack_transitions = vec![];

    for block in &blocks {
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

    for block in &blocks {
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

    for b1 in &blocks {
        for b2 in &blocks {
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

    // explicitly have to say that others are not clear?
    let mut clear_predicates = vec![];
    let clear_vec = vec!["C", "F"];
    let unclear_vec = IterOps::difference(blocks.clone(), clear_vec.clone());
    println!("{:?}", unclear_vec);

    for x in clear_vec {
        clear_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("clear_{}", x),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    for x in unclear_vec {
        clear_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("clear_{}", x),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "false",
            None,
        )))
    }

    let mut ontable_predicates = vec![];
    for x in vec!["I", "F"] {
        ontable_predicates.push(Predicate::EQ(EnumValue::new(
            &EnumVariable::new(
                &format!("ontable_{}", x),
                &domain,
                "boolean",
                None,
                &Kind::Command,
            ),
            "true",
            None,
        )))
    }

    let mut on_predicates = vec![];
    for (b1, b2) in vec![
        ("A", "B")
        // ("C", "E"),
        // ("E", "J"),
        // ("J", "B"),
        // ("B", "G"),
        // ("G", "H"),
        // ("H", "A"),
        // ("A", "D"),
        // ("D", "I"),
    ] {
        on_predicates.push(Predicate::EQ(EnumValue::new(
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

    let initial = Predicate::AND(vec![
        Predicate::AND(clear_predicates),
        Predicate::AND(ontable_predicates),
        Predicate::AND(on_predicates),
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
    ]);

    let mut goal_on_predicates = vec![];
    for (b1, b2) in vec![
        // ("D", "C"),
        // ("C", "F"),
        // ("F", "J"),
        // ("J", "E"),
        // ("E", "H"),
        // ("H", "B"),
        // ("B", "A"),
        // ("A", "G"),
        // ("G", "I"),
        ("B", "A"),
    ] {
        goal_on_predicates.push(Predicate::EQ(EnumValue::new(
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
    for b1 in &blocks {
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
        for b2 in &blocks {
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

    for b1 in &blocks {
        let mut local_vec = vec![];
        for b2 in &blocks {
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

    let goal = Predicate::AND(goal_on_predicates);
    let problem = PlanningProblem::new(
        "blocks_world",
        &initial,
        &goal,
        &transitions,
        &Predicate::AND(invariants),
        &50,
        &Paradigm::Raar,
    );

    let result = incremental(&problem);
    pprint_result(&result);
    pprint_result_trans_only(&result);
}
