use micro_sp_tools::*;

pub fn model() -> PlanningProblem {
    let act_pos = EnumVariable::new(
        "act_pos",
        "act_pos",
        &vec!["left", "right"],
        None,
        Some(&ControlKind::Measured),
    );
    let ref_pos = EnumVariable::new(
        "ref_pos",
        "ref_pos",
        &vec!["left", "right"],
        None,
        Some(&ControlKind::Command),
    );
    let act_left = Predicate::EQRL(act_pos.clone(), "left".to_string());
    let act_right = Predicate::EQRL(act_pos.clone(), "right".to_string());
    let ref_left = Predicate::EQRL(ref_pos.clone(), "left".to_string());
    let ref_right = Predicate::EQRL(ref_pos.clone(), "right".to_string());
    let t1 = Transition::new(
        "start_move_left",
        &Predicate::AND(vec![act_right.clone(), ref_right.clone()]),
        &ref_left,
    );
    let t2 = Transition::new(
        "start_move_right",
        &Predicate::AND(vec![act_left.clone(), ref_left.clone()]),
        &ref_right,
    );
    let t3 = Transition::new(
        "finish_move_left",
        &Predicate::AND(vec![act_right.clone(), ref_left.clone()]),
        &act_left,
    );
    let t4 = Transition::new(
        "finish_move_right",
        &Predicate::AND(vec![act_left.clone(), ref_right.clone()]),
        &act_right,
    );
    let problem = PlanningProblem::new(
        "prob1",
        &Predicate::AND(vec![act_left, ref_left]),
        &act_right,
        &vec![t1, t2, t3, t4],
        &Predicate::TRUE,
        &12,
    );
    problem
}
