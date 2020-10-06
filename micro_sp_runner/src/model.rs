use micro_sp_tools::*;

pub fn model() -> PlanningProblem {

    let act_pos = EnumVariable::new(
        "act_pos",
        &vec!["left", "right", "unknown", "dummy_value"],
        None,
        Some(&ControlKind::Measured),
    );

    let ref_pos = EnumVariable::new(
        "ref_pos",
        &vec!["left", "right", "dummy_value"],
        None,
        Some(&ControlKind::Command),
    );

    let act_stat = EnumVariable::new(
        "act_stat",
        &vec!["idle", "active", "unknown", "dummy_value"],
        None,
        Some(&ControlKind::Measured),
    );

    let ref_stat = EnumVariable::new(
        "ref_stat",
        &vec!["idle", "active", "dummy_value"],
        None,
        Some(&ControlKind::Command),
    );

    let act_left = Predicate::EQRL(act_pos.clone(), "left".to_string());
    let not_act_left = Predicate::NOT(Box::new(act_left.clone()));
    let act_right = Predicate::EQRL(act_pos.clone(), "right".to_string());
    let not_act_right = Predicate::NOT(Box::new(act_right.clone()));
    let ref_left = Predicate::EQRL(ref_pos.clone(), "left".to_string());
    let ref_right = Predicate::EQRL(ref_pos.clone(), "right".to_string());
    let activate = Predicate::EQRL(ref_stat.clone(), "active".to_string());
    let activated = Predicate::EQRL(act_stat.clone(), "active".to_string());
    let deactivate = Predicate::EQRL(ref_stat.clone(), "idle".to_string());
    let deactivated = Predicate::EQRL(act_stat.clone(), "idle".to_string());

    let t1 = Transition::new(
        "start_move_left",
        &Predicate::AND(vec![not_act_left.clone(), ref_right.clone()]),
        &ref_left,
    );

    let t2 = Transition::new(
        "start_move_right",
        &Predicate::AND(vec![not_act_right.clone(), ref_left.clone()]),
        &ref_right,
    );

    let t3 = Transition::new(
        "finish_move_left",
        &Predicate::AND(vec![not_act_left.clone(), ref_left.clone()]),
        &act_left,
    );

    let t4 = Transition::new(
        "finish_move_right",
        &Predicate::AND(vec![not_act_right.clone(), ref_right.clone()]),
        &act_right,
    );

    let t5 = Transition::new(
        "start_activate",
        &Predicate::AND(vec![deactivated.clone(), deactivate.clone()]),
        &activate,
    );

    let t6 = Transition::new(
        "finish_activate",
        &Predicate::AND(vec![deactivated.clone(), activate.clone()]),
        &activated,
    );

    let t7 = Transition::new(
        "start_deactivate",
        &Predicate::AND(vec![activated.clone(), activate.clone()]),
        &deactivate,
    );

    let t8 = Transition::new(
        "finish_deactivate",
        &Predicate::AND(vec![activated.clone(), deactivate.clone()]),
        &deactivated,
    );

    let problem = PlanningProblem::new(
        "prob1",
        &Predicate::AND(vec![act_left.clone(), ref_left, deactivated, deactivate]),
        &Predicate::AND(vec![activated, act_right]),
        &vec![t1, t2, t3, t4, t5, t6, t7, t8],
        &Predicate::TRUE,
        &12,
    );
    
    problem
}
