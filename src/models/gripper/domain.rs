use super::*;

pub fn gripper_model_pure_booleans(
    rooms: &Vec<&str>,
    grippers: &Vec<&str>,
    balls: &Vec<&str>,
) -> (Vec<ParamTransition>, ParamPredicate) {

    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    for room_a in rooms {
        for room_b in rooms {
            if room_a != room_b { 
                move_transitions.push(
                    ptrans!(
                        &format!("move_from_{}_to_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_a), true, "r"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_a), false, "r")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_b), true, "r"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                pick_transitions.push(
                    ptrans!(
                        &format!("pick_{}_in_{}_with_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, "b")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "g"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), false, "b")),
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), true, "g")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), false, "g"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                drop_transitions.push(
                    ptrans!(
                        &format!("drop_{}_to_{}_from_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), true, "g")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, "b")),
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), false, "g")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "g"))
                        )
                    )
                )
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

pub fn gripper_model_pure_booleans_2(
    rooms: &Vec<&str>,
    grippers: &Vec<&str>,
    balls: &Vec<&str>,
) -> (Vec<ParamTransition>, ParamPredicate) {

    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    for room_a in rooms {
        for room_b in rooms {
            if room_a != room_b { 
                move_transitions.push(
                    ptrans!(
                        &format!("move_from_{}_to_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_a), true, "r"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_a), false, "r")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_b), true, "r"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                pick_transitions.push(
                    ptrans!(
                        &format!("pick_{}_in_{}_with_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, &format!("{}", ball))),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "g"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), false, &format!("{}", ball))),
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), true, "g")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), false, "g"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                drop_transitions.push(
                    ptrans!(
                        &format!("drop_{}_to_{}_from_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), true, "g")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, &format!("{}", ball))),
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), false, "g")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "g"))
                        )
                    )
                )
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

pub fn gripper_model_enumerated_booleans(
    rooms: &Vec<&str>,
    grippers: &Vec<&str>,
    balls: &Vec<&str>,
) -> (Vec<ParamTransition>, ParamPredicate) {
    let domain = vec!["true", "false"];

    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    for room_a in rooms {
        for room_b in rooms {
            if room_a != room_b { 
                move_transitions.push(
                    ptrans!(
                        &format!("move_from_{}_to_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), &domain, "true", "boolean","r"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), &domain, "false","boolean", "r")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_b), &domain, "true", "boolean","r"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                pick_transitions.push(
                    ptrans!(
                        &format!("pick_{}_in_{}_with_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), &domain, "true", "boolean", "b")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), &domain, "true", "boolean", "r")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), &domain, "true", "boolean", "g"))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), &domain, "false", "boolean", "b")),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), &domain, "true", "boolean", "g")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), &domain, "false", "boolean", "g"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                drop_transitions.push(
                    ptrans!(
                        &format!("drop_{}_to_{}_from_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), &domain, "true", "boolean", "g")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), &domain, "true", "boolean", "r"))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), &domain, "true", "boolean", "b")),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), &domain, "false", "boolean", "g")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), &domain, "true", "boolean", "g"))
                        )
                    )
                )
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

pub fn gripper_model_enumerated_booleans_2(
    rooms: &Vec<&str>,
    grippers: &Vec<&str>,
    balls: &Vec<&str>,
) -> (Vec<ParamTransition>, ParamPredicate) {
    let domain = vec!["true", "false"];

    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    for room_a in rooms {
        for room_b in rooms {
            if room_a != room_b { 
                move_transitions.push(
                    ptrans!(
                        &format!("move_from_{}_to_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), &domain, "true", "boolean","r"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), &domain, "false","boolean", "r")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_b), &domain, "true", "boolean","r"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                pick_transitions.push(
                    ptrans!(
                        &format!("pick_{}_in_{}_with_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), &domain, "true", "boolean", &format!("{}", ball))),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), &domain, "true", "boolean", "r")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), &domain, "true", "boolean", "g"))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), &domain, "false", "boolean", &format!("{}", ball))),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), &domain, "true", "boolean", "g")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), &domain, "false", "boolean", "g"))
                        )
                    )
                )
            }
        }
    }

    for room in rooms {
        for gripper in grippers {
            for ball in balls {
                drop_transitions.push(
                    ptrans!(
                        &format!("drop_{}_to_{}_from_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), &domain, "true", "boolean", "g")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), &domain, "true", "boolean", "r"))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), &domain, "true", "boolean", &format!("{}", ball))),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), &domain, "false", "boolean", "g")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), &domain, "true", "boolean", "g"))
                        )
                    )
                )
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

pub fn gripper_model_pure_enumeration(
    balls: &Vec<&str>,
) -> Vec<ParamTransition> {
    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    let rooms = vec!("rooma", "roomb");
    let grippers = vec!("left", "right");

    let ball_pos_domain = vec!("rooma", "roomb", "left", "right");
    let robot_pos_domain = vec!("rooma", "roomb");
    let gripper_domain = vec!("e", "f");

    for room_a in &rooms {
        for room_b in &rooms {
            if room_a != room_b { 
                move_transitions.push(
                    ptrans!(
                        &format!("move_from_{}_to_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_enum_assign_c!("robot_at", &robot_pos_domain, room_a, "rooms", "r"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!("robot_at", &robot_pos_domain, room_b, "rooms", "r"))
                        )
                    )
                )
            }
        }
    }

    for room in &rooms {
        for gripper in &grippers {
            for ball in balls {
                pick_transitions.push(
                    ptrans!(
                        &format!("pick_{}_in_{}_with_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("ball_{}_at", ball), &ball_pos_domain, room, "balls", "b")),
                            &pass!(&new_enum_assign_c!("robot_at", &robot_pos_domain, room, "rooms", "r")),
                            &pass!(&new_enum_assign_c!("gripper", &gripper_domain, "e", "grippers", "g"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("ball_{}_at", ball), &ball_pos_domain, gripper, "balls", "b")),
                            &pass!(&new_enum_assign_c!("gripper", &gripper_domain, "f", "grippers", "g"))
                        )
                    )
                )
            }
        }
    }

    for room in &rooms {
        for gripper in &grippers {
            for ball in balls {
                drop_transitions.push(
                    ptrans!(
                        &format!("drop_{}_to_{}_from_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("ball_{}_at", ball), &ball_pos_domain, gripper, "balls", "b")),
                            &pass!(&new_enum_assign_c!("robot_at", &robot_pos_domain, room, "rooms", "r")),
                            &pass!(&new_enum_assign_c!("gripper", &gripper_domain, "f", "grippers", "g"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("ball_{}_at", ball), &ball_pos_domain, room, "balls", "b")),
                            &pass!(&new_enum_assign_c!("gripper", &gripper_domain, "e", "grippers", "g"))
                        )
                    )
                )
            }
        }
    }

    let mut transitions = vec![];
    for t in vec![move_transitions, pick_transitions, drop_transitions] {
        transitions.extend(t)
    }

    // (transitions, Predicate::AND(invariants))
    transitions
}

#[test]
fn test_gripper() {
    let g_param = Parameter::new("g", &false);
    let r_param = Parameter::new("r", &false);
    let b_param = Parameter::new("b", &false);

    let ball_pos_domain = vec!("rooma", "roomb", "left", "right");
    let robot_pos_domain = vec!("rooma", "roomb");

    // let b1 = Parameter::new("ball1", &false);
    // let b2 = Parameter::new("ball2", &false);
    // let b3 = Parameter::new("ball3", &false);
    // let b4 = Parameter::new("ball4", &false);

    // let gripper_params = vec![b1, b2, b3, b4, g_param, r_param];
    let gripper_params = vec![g_param, b_param, r_param];
    let mut init_pred = vec!();
    let mut goal_pred = vec!();
    let balls = vec!["ball1", "ball2", "ball3", "ball4", "ball5", "ball6", "ball7", "ball8"];
    for b in &balls {
        init_pred.push(
            pass!(&new_enum_assign_c!(&format!("ball_{}_at", b), &ball_pos_domain, "rooma", "balls", "b"))
        );
        goal_pred.push(
            pass!(&new_enum_assign_c!(&format!("ball_{}_at", b), &ball_pos_domain, "roomb", "balls", "b"))
        );
    }

    let problem = ParamPlanningProblem::new(
        "prob1", 
        &ParamPredicate::new(
            &vec!(
                Predicate::AND(init_pred)
            )
        ), 
        &ParamPredicate::new(
            &vec!(
                Predicate::AND(goal_pred)
            )
        ),
        &gripper_model_pure_enumeration(&balls),
        &ParamPredicate::new(&vec!(Predicate::TRUE)),
        &50);

    let result = compositional(&problem, &gripper_params); //, 1200);
    pprint_result_trans_only(&result)
}