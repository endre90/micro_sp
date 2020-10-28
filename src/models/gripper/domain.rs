use super::*;

pub fn gripper_model_enumerated_booleans(
    rooms: &Vec<&str>,
    grippers: &Vec<&str>,
    balls: &Vec<&str>,
) -> (Vec<ParamTransition>, Predicate) {
    let domain = vec!["true", "false"];

    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    let  g_param = Parameter::new("g", &false);
    let  r_param = Parameter::new("r", &false);
    let  b_param = Parameter::new("b", &false);
    for room_a in rooms {
        for room_b in rooms {
            if room_a != room_b {
                move_transitions.push(ParamTransition::new(
                    &format!("move_from_{}_to_{}", room_a, room_b),
                    &ParamPredicate::new(&vec![Predicate::SET(EnumValue::new(
                        &EnumVariable::new(
                            &format!("robot_at_{}", room_a),
                            &domain,
                            "boolean",
                            Some(&r_param),
                            &Kind::Command,
                        ),
                        "true",
                        None,
                    ))]),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("robot_at_{}", room_b),
                                &domain,
                                "boolean",
                                Some(&r_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("robot_at_{}", room_a),
                                &domain,
                                "boolean",
                                Some(&r_param),
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
                        "pick_ball_{}_in_room_{}_with_gripper_{}",
                        ball, room, gripper
                    ),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("ball_{}_at_room_{}", ball, room),
                                &domain,
                                "boolean",
                                 Some(&b_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("robot_at_{}", room),
                                &domain,
                                "boolean",
                                 Some(&r_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("gripper_{}_free", gripper),
                                &domain,
                                "boolean",
                                Some(&g_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("ball_{}_at_room_{}", ball, room),
                                &domain,
                                "boolean",
                                Some(&b_param),
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
                                Some(&b_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("gripper_{}_free", gripper),
                                &domain,
                                "boolean",
                                Some(&g_param),
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
                        "drop_ball_{}_to_room_{}_from_gripper_{}",
                        ball, room, gripper
                    ),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("carry_{}", ball),
                                &domain,
                                "boolean",
                                Some(&b_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("robot_at_{}", room),
                                &domain,
                                "boolean",
                                Some(&r_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                    ]),
                    &ParamPredicate::new(&vec![
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("ball_{}_at_room_{}", ball, room),
                                &domain,
                                "boolean",
                                Some(&b_param),
                                &Kind::Command,
                            ),
                            "true",
                            None,
                        )),
                        Predicate::SET(EnumValue::new(
                            &EnumVariable::new(
                                &format!("gripper_{}_free", gripper),
                                &domain,
                                "boolean",
                                Some(&g_param),
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
                                Some(&b_param),
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
    (transitions, Predicate::TRUE)
}
