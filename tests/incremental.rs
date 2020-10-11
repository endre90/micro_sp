use lib::*;

#[test]
fn test_incremental_1() {
    let act_pos = EnumVariable::new(
        "act_pos",
        &vec!["left", "right", "unknown", "dummy_value"],
        None,
        &Kind::Measured,
    );

    let ref_pos = EnumVariable::new(
        "ref_pos",
        &vec!["left", "right", "dummy_value"],
        None,
        &Kind::Command,
    );

    let act_stat = EnumVariable::new(
        "act_stat",
        &vec!["idle", "active", "unknown", "dummy_value"],
        None,
        &Kind::Measured,
    );

    let ref_stat = EnumVariable::new(
        "ref_stat",
        &vec!["idle", "active", "dummy_value"],
        None,
        &Kind::Command,
    );

    let act_left = Predicate::EQ(EnumValue::new(&act_pos, "left", None));
    let not_act_left = Predicate::NOT(Box::new(act_left.clone()));
    let act_right = Predicate::EQ(EnumValue::new(&act_pos, "right", None));
    let not_act_right = Predicate::NOT(Box::new(act_right.clone()));
    let ref_left = Predicate::EQ(EnumValue::new(&ref_pos, "left", None));
    let ref_right = Predicate::EQ(EnumValue::new(&ref_pos, "right", None));
    let activate = Predicate::EQ(EnumValue::new(&ref_stat, "active", None));
    let activated = Predicate::EQ(EnumValue::new(&act_stat, "active", None));
    let deactivate = Predicate::EQ(EnumValue::new(&ref_stat, "idle", None));
    let deactivated = Predicate::EQ(EnumValue::new(&act_stat, "idle", None));

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
        &12,
    );

    let result = incremental(&problem);
    pprint_result(&result)
}