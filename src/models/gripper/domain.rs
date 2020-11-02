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

    for room_a in rooms {
        for room_b in rooms {
            if room_a != room_b { 
                move_transitions.push(
                    ptrans!(
                        &format!("move_from_{}_to_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), "boolean", &domain, "true", ("r", &true)))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_a), "boolean", &domain, "false", ("r", &true))),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room_b), "boolean", &domain, "true", ("r", &true)))
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
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), "boolean", &domain, "true", ("b", &true))),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), "boolean", &domain, "true", ("r", &true))),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), "boolean", &domain, "true", ("g", &true)))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), "boolean", &domain, "false", ("b", &true))),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), "boolean", &domain, "true", ("g", &true))),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), "boolean", &domain, "false", ("g", &true)))
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
                        &format!("drop_{}_to_{}_from_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), "boolean", &domain, "true", ("g", &true))),
                            &pass!(&new_enum_assign_c!(&format!("at-robby_{}", room), "boolean", &domain, "true", ("r", &true)))

                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}_{}", ball, room), "boolean", &domain, "true", ("b", &true))),
                            &pass!(&new_enum_assign_c!(&format!("{}_carry_{}", gripper, ball), "boolean", &domain, "false", ("g", &true))),
                            &pass!(&new_enum_assign_c!(&format!("free_{}", gripper), "boolean", &domain, "true", ("g", &true)))
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
