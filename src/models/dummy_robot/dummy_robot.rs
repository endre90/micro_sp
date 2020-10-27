use super::*;

pub fn raar_model() -> PlanningProblem {
    // "dummy_value" should be a special, reserved value used for initialization
    // so that a plan can't be found before the acual values come in

    let stat_domain = vec!["idle", "active", "unknown", "dummy_value"];
    let pos_domain = vec!["left", "right", "unknown", "dummy_value"];

    let act_pos = EnumVariable::new("act_pos", &pos_domain, "pos", None, &Kind::Measured);

    let ref_pos = EnumVariable::new("ref_pos", &pos_domain, "pos", None, &Kind::Command);

    let act_stat = EnumVariable::new("act_stat", &stat_domain, "stat", None, &Kind::Measured);

    let ref_stat = EnumVariable::new("ref_stat", &stat_domain, "stat", None, &Kind::Command);

    // let act_ref_pos = EnumVariable::new("act_ref_pos", &pos_domain, "pos", None, &Kind::Handshake);

    // let act_ref_stat =
    // EnumVariable::new("act_ref_stat", &stat_domain, "stat", None, &Kind::Handshake);

    let ref_pos_dummy = Predicate::EQ(EnumValue::new(&ref_pos, "dummy_value", None));
    let ref_stat_dummy = Predicate::EQ(EnumValue::new(&ref_pos, "dummy_value", None));
    let act_pos_dummy = Predicate::EQ(EnumValue::new(&act_pos, "dummy_value", None));
    let act_stat_dummy = Predicate::EQ(EnumValue::new(&act_pos, "dummy_value", None));
    // let act_ref_pos_dummy = Predicate::EQ(EnumValue::new(&act_pos, "dummy_value", None));
    // let act_ref_stat_dummy = Predicate::EQ(EnumValue::new(&act_pos, "dummy_value", None));

    let not_ref_pos_dummy = Predicate::NOT(Box::new(ref_pos_dummy.clone()));
    let not_ref_stat_dummy = Predicate::NOT(Box::new(ref_stat_dummy.clone()));
    let not_act_pos_dummy = Predicate::NOT(Box::new(act_pos_dummy.clone()));
    let not_act_stat_dummy = Predicate::NOT(Box::new(act_stat_dummy.clone()));
    // let not_act_ref_pos_dummy = Predicate::NOT(Box::new(act_ref_pos_dummy.clone()));
    // let not_act_ref_stat_dummy = Predicate::NOT(Box::new(act_ref_stat_dummy.clone()));

    // if any of the measured vars has the value "dummy_value", not_any_measured_dummy is false
    let not_any_measured_dummy = Predicate::NOT(Box::new(Predicate::OR(vec![
        act_pos_dummy.clone(),
        act_stat_dummy.clone(),
        // act_ref_pos_dummy.clone(),
        // act_ref_stat_dummy.clone(),
    ])));

    let all_dummy = Predicate::AND(vec![
        ref_pos_dummy.clone(),
        ref_stat_dummy.clone(),
        act_pos_dummy.clone(),
        act_stat_dummy.clone(),
        // act_ref_pos_dummy.clone(),
        // act_ref_stat_dummy.clone(),
    ]);

    let act_left = Predicate::EQ(EnumValue::new(&act_pos, "left", None));
    let not_act_left = Predicate::NOT(Box::new(act_left.clone()));
    let act_right = Predicate::EQ(EnumValue::new(&act_pos, "right", None));
    let not_act_right = Predicate::NOT(Box::new(act_right.clone()));

    let act_idle = Predicate::EQ(EnumValue::new(&act_stat, "idle", None));
    let not_act_idle = Predicate::NOT(Box::new(act_idle.clone()));
    let act_active = Predicate::EQ(EnumValue::new(&act_stat, "active", None));
    let not_act_active = Predicate::NOT(Box::new(act_active.clone()));

    let ref_left = Predicate::EQ(EnumValue::new(&ref_pos, "left", None));
    let not_ref_left = Predicate::NOT(Box::new(ref_left.clone()));
    let ref_right = Predicate::EQ(EnumValue::new(&ref_pos, "right", None));
    let not_ref_right = Predicate::NOT(Box::new(ref_right.clone()));

    let ref_idle = Predicate::EQ(EnumValue::new(&ref_stat, "idle", None));
    let not_ref_idle = Predicate::NOT(Box::new(ref_idle.clone()));
    let ref_active = Predicate::EQ(EnumValue::new(&ref_stat, "active", None));
    let not_ref_active = Predicate::NOT(Box::new(ref_active.clone()));

    // let act_ref_left = Predicate::EQ(EnumValue::new(&act_ref_pos, "left", None));
    // let not_act_ref_left = Predicate::NOT(Box::new(act_ref_left.clone()));
    // let act_ref_right = Predicate::EQ(EnumValue::new(&act_ref_pos, "right", None));
    // let not_act_ref_right = Predicate::NOT(Box::new(act_ref_right.clone()));

    // let act_ref_idle = Predicate::EQ(EnumValue::new(&act_ref_stat, "idle", None));
    // let not_act_ref_idle = Predicate::NOT(Box::new(act_ref_idle.clone()));
    // let act_ref_active = Predicate::EQ(EnumValue::new(&act_ref_stat, "active", None));
    // let not_act_ref_active = Predicate::NOT(Box::new(act_ref_active.clone()));

    let t1 = Transition::new(
        "start_activate",
        &Predicate::AND(vec![
            not_act_active.clone(),
            not_ref_active.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQRR(act_pos.clone(), ref_pos.clone()),
        ]),
        &Predicate::AND(vec![not_act_active.clone(), ref_active.clone()]),
    );

    let t2 = Transition::new(
        "finish_activate",
        &Predicate::AND(vec![
            not_act_active.clone(),
            ref_active.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQRR(act_pos.clone(), ref_pos.clone()),
        ]),
        &Predicate::AND(vec![act_active.clone(), ref_active.clone()]),
    );

    let t3 = Transition::new(
        "start_deactivate",
        &Predicate::AND(vec![
            not_act_idle.clone(),
            not_ref_idle.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQRR(act_pos.clone(), ref_pos.clone()),
        ]),
        &Predicate::AND(vec![ref_idle.clone(), ref_idle.clone()]),
    );

    let t4 = Transition::new(
        "finish_deactivate",
        &Predicate::AND(vec![
            not_act_active.clone(),
            ref_idle.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQRR(act_pos.clone(), ref_pos.clone()),
        ]),
        &Predicate::AND(vec![act_idle.clone(), ref_idle.clone()]),
    );

    let t5 = Transition::new(
        "start_move_left",
        &Predicate::AND(vec![
            not_act_left.clone(),
            not_ref_left.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQRR(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &Predicate::AND(vec![ref_left.clone(), not_act_left.clone()]),
    );

    let t6 = Transition::new(
        "finish_move_left",
        &Predicate::AND(vec![
            not_act_left.clone(),
            ref_left.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQRR(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &Predicate::AND(vec![act_left.clone(), ref_left.clone()]),
    );

    let t7 = Transition::new(
        "start_move_right",
        &Predicate::AND(vec![
            not_act_right.clone(),
            not_ref_right.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQRR(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &Predicate::AND(vec![ref_right.clone(), not_act_right.clone()]),
    );

    let t8 = Transition::new(
        "finish_move_right",
        &Predicate::AND(vec![
            not_act_right.clone(),
            ref_right.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQRR(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &Predicate::AND(vec![act_right.clone(), ref_right.clone()]),
    );

    let problem = PlanningProblem::new(
        "prob1",
        &Predicate::AND(vec![
            not_act_active.clone(),
            not_ref_active.clone(),
            act_left.clone(),
            ref_left.clone(),
        ]),
        &Predicate::AND(vec![
            act_active.clone(),
            ref_active.clone(),
            act_right.clone(),
            ref_right.clone(),
        ]),
        &vec![t1, t2, t3, t4, t5, t6, t7, t8],
        &Predicate::TRUE,
        &12,
        &Paradigm::Raar,
    );
    problem
}
