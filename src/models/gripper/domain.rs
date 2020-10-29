use super::*;

pub fn gripper_model_enumerated_booleans(
    rooms: &Vec<&str>,
    grippers: &Vec<&str>,
    balls: &Vec<&str>,
) -> (Vec<ParamTransition>, ParamPredicate) {
    let domain = vec!["true", "false"];

    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    let  g_param = Parameter::new("g", &true);
    let  r_param = Parameter::new("r", &true);
    let  b_param = Parameter::new("b", &true);

    for room_a in rooms {
        for room_b in rooms {
            if room_a != room_b {
                move_transitions.push(ParamTransition::new(
                    &format!("move_from_{}_to_{}", room_a, room_b),
                    &ParamPredicate::new(&vec![Predicate::SET(EnumValue::new(
                        &EnumVariable::new(
                            &format!("at-robby_{}", room_a),
                            &domain,
                            "boolean",
                            // Some(&r_param),
                            None,
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    ))]),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("at-robby_{}", room_b),
                                &domain,
                                "boolean",
                                // Some(&r_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("at-robby_{}", room_a),
                                &domain,
                                "boolean",
                                // Some(&r_param),
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
    
    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                pick_transitions.push(ParamTransition::new(
                    &format!(
                        "pick_{}_in_{}_with_{}_gripper",
                        ball, room, gripper
                    ),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("at_{}_{}", ball, room),
                                &domain,
                                "boolean",
                                //  Some(&b_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("at-robby_{}", room),
                                &domain,
                                "boolean",
                                //  Some(&r_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("free_{}", gripper),
                                &domain,
                                "boolean",
                                // Some(&g_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("at_{}_{}", ball, room),
                                &domain,
                                "boolean",
                                // Some(&b_param),
                                None,
                                &Kind::Command,
                            ),
                            "false",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_carry_{}", gripper, ball),
                                &domain,
                                "boolean",
                                // Some(&b_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("free_{}", gripper),
                                &domain,
                                "boolean",
                                // Some(&g_param),
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

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                drop_transitions.push(ParamTransition::new(
                    &format!(
                        "drop_{}_to_{}_from_{}_gripper",
                        ball, room, gripper
                    ),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_carry_{}", gripper, ball),
                                &domain,
                                "boolean",
                                // Some(&b_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("at-robby_{}", room),
                                &domain,
                                "boolean",
                                // Some(&r_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("at_{}_{}", ball, room),
                                &domain,
                                "boolean",
                                // Some(&b_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("free_{}", gripper),
                                &domain,
                                "boolean",
                                // Some(&g_param),
                                None,
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("{}_carry_{}", gripper, ball),
                                &domain,
                                "boolean",
                                // Some(&b_param),
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
    for t in vec![move_transitions, pick_transitions, drop_transitions] {
        transitions.extend(t)
    }

    // (transitions, Predicate::AND(invariants))
    (transitions, ParamPredicate::new(&vec!(Predicate::TRUE)))
}
