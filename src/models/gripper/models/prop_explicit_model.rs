use crate::models::gripper::models::prop_explicit_parser::parser;
use super::*;

pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, objects) = parser(name);

    let mut transitions = vec![];

    let rooms = objects.get("room").unwrap_or(&vec!()).to_vec();
    let balls = objects.get("ball").unwrap_or(&vec!()).to_vec();
    let grippers = objects.get("gripper").unwrap_or(&vec!()).to_vec();

    // (:action move
    //     :parameters  (?from ?to)
    //     :precondition (and  (room ?from) (room ?to) (at-robby ?from))
    //     :effect (and  (at-robby ?to)
    //           (not (at-robby ?from))))

    for room_a in &rooms {
        for room_b in &rooms {
            if room_a != room_b { 
                transitions.push(
                    ptrans!(
                        &format!("move_{}_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_a), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_a), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_b), true, "c"))
                        )
                    )
                )
            }
        }
    }

    // (:action pick
    //     :parameters (?obj ?room ?gripper)
    //     :precondition  (and  (ball ?obj) (room ?room) (gripper ?gripper)
    //              (at ?obj ?room) (at-robby ?room) (free ?gripper))
    //     :effect (and (carry ?obj ?gripper)
    //          (not (at ?obj ?room)) 
    //          (not (free ?gripper))))

    for room in &rooms {
        for gripper in &grippers {
            for ball in &balls {
                transitions.push(
                    ptrans!(
                        &format!("pick_{}_{}_{}", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "c"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("carry_{}_{}", ball, gripper), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), false, "c"))
                        )
                    )
                )
            }
        }
    }

    // (:action drop
    //     :parameters  (?obj  ?room ?gripper)
    //     :precondition  (and  (ball ?obj) (room ?room) (gripper ?gripper)
    //              (carry ?obj ?gripper) (at-robby ?room))
    //     :effect (and (at ?obj ?room)
    //          (free ?gripper)
    //          (not (carry ?obj ?gripper)))))

    for room in &rooms {
        for gripper in &grippers {
            for ball in &balls {
                transitions.push(
                    ptrans!(
                        &format!("drop_{}_{}_{}", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("carry_{}_{}", ball, gripper), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "c"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, "c")),
                            &pass!(&new_bool_assign_c!(&format!("carry_{}_{}", ball, gripper), false, "c")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "c"))
                        )
                    )
                )
            }
        }
    }

    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("gripper_bool_explicit_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::TRUE,
        &vec!(c)
    );

    problem
}