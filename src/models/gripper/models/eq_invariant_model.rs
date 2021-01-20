use crate::models::gripper::models::eq_invariant_parser::parser;
use super::*;

#[allow(dead_code)]
pub fn model(name: &str) -> ParamPlanningProblem {

    let (parsed, objects) = parser(name);

    let mut transitions = vec![];

    let rooms = objects.get("room").unwrap_or(&vec!()).to_vec();
    let balls = objects.get("ball").unwrap_or(&vec!()).to_vec();
    let grippers = objects.get("gripper").unwrap_or(&vec!()).to_vec();

    let mut ball_domain = vec!();
    ball_domain.extend(rooms.clone());
    ball_domain.extend(grippers.clone());
    
    for room in &rooms {
        transitions.push(
            ParamTransition::new(
                &format!("move_to_{}", room),
                &ppred!(
                    &pnot!(
                        &pass!(&new_enum_assign_c!("at-robby", &rooms, &format!("{}", room), "c", "c"))
                    )
                ),
                &ppred!(
                    &pass!(&new_enum_assign_c!("at-robby", &rooms, &format!("{}", room), "c", "c"))
                )
            )
        )
    }

    for room in &rooms {
        for gripper in &grippers {
            for ball in &balls {
                transitions.push(
                    ParamTransition::new(
                        &format!("pick_{}_{}_{}", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!("at-robby", &rooms, &format!("{}", room), "c", "c")),
                            &pass!(&new_enum_assign_c!(&format!("at_{}", ball), &ball_domain, &format!("{}", room), "c", "c")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}", ball), &ball_domain, &format!("{}", gripper), "c", "c")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), false, "c"))
                        )
                    )
                )
            }
        }
    }


    for room in &rooms {
        for gripper in &grippers {
            for ball in &balls {
                transitions.push(
                    ptrans!(
                        &format!("drop_{}_{}_{}", ball, room, gripper),
                        &ppred!(
                            &pass!(&new_enum_assign_c!("at-robby", &rooms, &format!("{}", room), "c", "c")),
                            &pass!(&new_enum_assign_c!(&format!("at_{}", ball), &ball_domain, &format!("{}", gripper), "c", "c"))
                        ),
                        &ppred!(
                            &pass!(&new_enum_assign_c!(&format!("at_{}", ball), &ball_domain, &format!("{}", room), "c", "c")),
                            &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "c"))
                        )
                    )
                )
            }
        }
    }

    let mut invariants = vec!();

    // 1. if a gripper carries a ball, it is not free
    // 2. the ball can't be in two rooms simultaneously

    for gripper in &grippers {    
        for ball in &balls {
            // 1. if the ball is at a gripper, it is not free
            invariants.push(
                pnot!(
                    &pand!(
                        &pass!(&new_enum_assign_c!(&format!("at_{}", ball), &ball_domain, &format!("{}", gripper), "c", "c")),
                        &pass!(&new_bool_assign_c!(&format!("free_{}", gripper), true, "c"))
                    )
                )
            );
        }
    }

    let c = Parameter::new("c", &true);

    let problem = ParamPlanningProblem::new(
        &format!("gripper_eq_invariant_{}", parsed.name.as_str()), 
        &parsed.init,
        &parsed.goal,
        &transitions,
        &Predicate::AND(invariants),
        &vec!(c)
    );

    problem
}