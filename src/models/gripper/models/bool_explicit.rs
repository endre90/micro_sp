use crate::models::gripper::models::bool_explicit_parser::parser;
use super::*;

pub fn model(name: &str) -> ParamPlanningProblem {

    let rooms = vec!("rooma", "roomb");
    let grippers = vec!("left", "right");
    
    let (parsed, balls) = parser(name);

    println!("balls {:?}", balls);

    let mut move_transitions = vec![];
    let mut pick_transitions = vec![];
    let mut drop_transitions = vec![];

    for room_a in &rooms {
        for room_b in &rooms {
            if room_a != room_b { 
                move_transitions.push(
                    ptrans!(
                        &format!("move_from_{}_to_{}", room_a, room_b),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_a), true, "r")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room_b), false, "r"))
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

    for room in &rooms {
        for gripper in &grippers {
            for ball in &balls {
                pick_transitions.push(
                    ptrans!(
                        &format!("pick_{}_in_{}_with_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, String::from(ball))),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "g"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), false, String::from(ball))),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r")),
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), true, "g")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), false, "g"))
                        )
                    )
                )
            }
        }
    }

    for room in &rooms {
        for gripper in &grippers {
            for ball in &balls {
                drop_transitions.push(
                    ptrans!(
                        &format!("drop_{}_to_{}_from_{}_gripper", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("{}_carry_{}", gripper, ball), true, "g")),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r"))

                        ),
                        &ppred!(
                            &pass!(&new_bool_assign_c!(&format!("at_{}_{}", ball, room), true, String::from(ball))),
                            &pass!(&new_bool_assign_c!(&format!("at-robby_{}", room), true, "r")),
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

    let r = Parameter::new("r", &true);
    let g = Parameter::new("g", &true);
    let b: Vec<Parameter> = balls.iter().map(|x| Parameter::new(&String::from(x), &true)).collect();
    // let b = Parameter::new("b", &true);

    let mut params = vec!();
    for p in vec!(vec!(g), b, vec!(r)) {
        params.extend(p)
    }

    let problem = ParamPlanningProblem::new(
        &format!("gripper_bool_explicit_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::TRUE,
        &params
    );

    problem
}