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
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), "boolean", &domain, "true", "r"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), "boolean", &domain, "false", "r")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_b), "boolean", &domain, "true", "r"))
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
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), "boolean", &domain, "true", "b")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), "boolean", &domain, "true", "r")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), "boolean", &domain, "true", "g"))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), "boolean", &domain, "false", "b")),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), "boolean", &domain, "true", "g")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), "boolean", &domain, "false", "g"))
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
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), "boolean", &domain, "true", "g")),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), "boolean", &domain, "true", "r"))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), "boolean", &domain, "true", "b")),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), "boolean", &domain, "false", "g")),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), "boolean", &domain, "true", "g"))
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
