use super::*;

#[test]
fn test_gripper() {
    let g_param = Parameter::new("g", &true);
    let r_param = Parameter::new("r", &true);
    let b_param = Parameter::new("b", &true);

    // let params = vec!(g_param, b_param, r_param);

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
    let balls = vec!["ball1", "ball2", "ball3", "ball4"]; //, "ball7", "ball8"];
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
        &models::gripper::domain::gripper_model_pure_enumeration(&balls),
        &ParamPredicate::new(&vec!(Predicate::TRUE)),
        &gripper_params);

    // let result = compositional(&problem, &gripper_params); //, 1200);
    let result = parameterized(&problem, 1200, 30);
    pprint_result_trans_only(&result)
}