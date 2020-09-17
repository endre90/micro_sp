use micro_sp_tools::*;
use micro_sp_runner::*;

pub fn robot_1() -> Vec<Transition> {

    let ref_pos = "ref_robot_1_pose";
    let act_pos = "act_robot_1_pose";
    let mut move_to_transitions = vec!();
    let ref_robot_pos_domain = vec!("left", "home", "right");
    let act_robot_pos_domain = vec!("left", "home", "right", "unknown");
    // let transition_state_domain = vec!("idle", "exec");
    for rpd in &ref_robot_pos_domain {
        move_to_transitions.push(
            Transition::new(
                &format!("start_move_to_{}", rpd),
                &Predicate::AND(
                    vec!(
                        // Predicate::EQRL(EnumVariable::new(&format!("start_move_to_{}", rpd),, &format!("start_move_to_{}", rpd), &transition_state_domain, None, &ControlKind::None), String::from("executing")),
                        Predicate::NOT(
                            Box::new(Predicate::EQRL(EnumVariable::new(ref_pos, ref_pos, &ref_robot_pos_domain, None, &ControlKind::Control), String::from(rpd.to_owned()))
                            )
                        ),
                        Predicate::NOT(
                            Box::new(Predicate::EQRL(EnumVariable::new(act_pos, act_pos, &act_robot_pos_domain, None, &ControlKind::Control), String::from(rpd.to_owned()))
                            )
                        )
                    )
                ),
                &Predicate::AND(
                    vec!(
                        Predicate::EQRL(EnumVariable::new(ref_pos, ref_pos, &ref_robot_pos_domain, None, &ControlKind::Control), String::from(rpd.to_owned()))
                    )
                )
            )
        );
        move_to_transitions.push(
            Transition::new(
                &format!("finish_move_to_{}", rpd),
                &Predicate::AND(
                    vec!(
                        // Predicate::EQRL(EnumVariable::new(&format!("start_move_to_{}", rpd),, &format!("start_move_to_{}", rpd), &transition_state_domain, None, &ControlKind::None), String::from("executing")),
                        Predicate::EQRL(EnumVariable::new(ref_pos, ref_pos, &ref_robot_pos_domain, None, &ControlKind::Control), String::from(rpd.to_owned())),
                        Predicate::NOT(
                            Box::new(Predicate::EQRL(EnumVariable::new(act_pos, act_pos, &act_robot_pos_domain, None, &ControlKind::Control), String::from(rpd.to_owned()))
                            )
                        )
                    )
                ),
                &Predicate::AND(
                    vec!(
                        Predicate::EQRL(EnumVariable::new(act_pos, act_pos, &act_robot_pos_domain, None, &ControlKind::Control), String::from(rpd.to_owned()))
                    )
                )
            )
        )
    }

    move_to_transitions
}