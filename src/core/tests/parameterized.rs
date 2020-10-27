use super::*;

#[test]
fn test_new_parameter() {
    assert_eq!(
        Parameter::new("some_name", &false),
        Parameter {
            name: String::from("some_name"),
            value: false
        }
    )
}

#[test]
fn test_none_parameter() {
    assert_eq!(
        Parameter::none(),
        Parameter {
            name: String::from("NONE"),
            value: true
        }
    )
}

#[test]
fn test_new_param_predicate() {
    let pp = ParamPredicate::new(&vec![
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "var1_m",
                &vec!["a", "b", "c"],
                "t1",
                Some(&Parameter::new("p1", &true)),
                &Kind::Measured,
            ),
            "a",
            None,
        )),
        Predicate::SET(EnumValue::new(
            &EnumVariable::new(
                "var1_c",
                &vec!["a", "b", "c"],
                "t1",
                Some(&Parameter::new("p1", &true)),
                &Kind::Command,
            ),
            "b",
            None,
        )),
    ]);
    println!("{:?}", pp);
}

#[test]
fn test_generate_predicate() {
    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &false);
    let d = vec!["a", "b", "c"];

    let var1_m = EnumVariable::new("var1_m", &d, "t1", Some(&p1), &Kind::Measured);
    let var1_c = EnumVariable::new("var1_c", &d, "t1", Some(&p1), &Kind::Command);
    let var2_m = EnumVariable::new("var2_m", &d, "t2", Some(&p2), &Kind::Measured);
    let var2_c = EnumVariable::new("var2_c", &d, "t2", Some(&p2), &Kind::Command);

    let pp = ParamPredicate::new(&vec![
        Predicate::SET(EnumValue::new(&var1_m, "a", None)),
        Predicate::SET(EnumValue::new(&var1_c, "b", None)),
        Predicate::SET(EnumValue::new(&var2_m, "c", None)),
        Predicate::SET(EnumValue::new(&var2_c, "a", None)),
    ]);

    let params = vec![p1, p2];
    println!("generated {:?}", generate_predicate(&pp, &params));
}

#[test]
fn test_parameterized() {
    let stat_domain = vec!["idle", "active", "unknown", "dummy_value"];
    let pos_domain = vec!["left", "right", "unknown", "dummy_value"];

    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &false);

    let act_pos = EnumVariable::new("act_pos", &pos_domain, "pos", Some(&p1), &Kind::Measured);
    let ref_pos = EnumVariable::new("ref_pos", &pos_domain, "pos", Some(&p1), &Kind::Command);
    let act_stat = EnumVariable::new("act_stat", &stat_domain, "stat", Some(&p2), &Kind::Measured);
    let ref_stat = EnumVariable::new("ref_stat", &stat_domain, "stat", Some(&p2), &Kind::Command);

    let act_pos_dummy = Predicate::SET(EnumValue::new(&act_pos, "dummy_value", None));
    let act_stat_dummy = Predicate::SET(EnumValue::new(&act_pos, "dummy_value", None));

    let not_any_measured_dummy = Predicate::NOT(Box::new(Predicate::OR(vec![
        act_pos_dummy.clone(),
        act_stat_dummy.clone(),
    ])));

    let act_left = Predicate::SET(EnumValue::new(&act_pos, "left", None));
    let not_act_left = Predicate::NOT(Box::new(act_left.clone()));
    let act_right = Predicate::SET(EnumValue::new(&act_pos, "right", None));
    let not_act_right = Predicate::NOT(Box::new(act_right.clone()));

    let act_idle = Predicate::SET(EnumValue::new(&act_stat, "idle", None));
    let not_act_idle = Predicate::NOT(Box::new(act_idle.clone()));
    let act_active = Predicate::SET(EnumValue::new(&act_stat, "active", None));
    let not_act_active = Predicate::NOT(Box::new(act_active.clone()));

    let ref_left = Predicate::SET(EnumValue::new(&ref_pos, "left", None));
    let not_ref_left = Predicate::NOT(Box::new(ref_left.clone()));
    let ref_right = Predicate::SET(EnumValue::new(&ref_pos, "right", None));
    let not_ref_right = Predicate::NOT(Box::new(ref_right.clone()));

    let ref_idle = Predicate::SET(EnumValue::new(&ref_stat, "idle", None));
    let not_ref_idle = Predicate::NOT(Box::new(ref_idle.clone()));
    let ref_active = Predicate::SET(EnumValue::new(&ref_stat, "active", None));
    let not_ref_active = Predicate::NOT(Box::new(ref_active.clone()));

    let t1 = ParamTransition::new(
        "start_activate",
        &ParamPredicate::new(&vec![
            not_act_active.clone(),
            not_ref_active.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
        ]),
        &ParamPredicate::new(&vec![not_act_active.clone(), ref_active.clone()]),
    );

    let t2 = ParamTransition::new(
        "finish_activate",
        &ParamPredicate::new(&vec![
            not_act_active.clone(),
            ref_active.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
        ]),
        &ParamPredicate::new(&vec![act_active.clone(), ref_active.clone()]),
    );

    let t3 = ParamTransition::new(
        "start_deactivate",
        &ParamPredicate::new(&vec![
            not_act_idle.clone(),
            not_ref_idle.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
        ]),
        &ParamPredicate::new(&vec![ref_idle.clone(), ref_idle.clone()]),
    );

    let t4 = ParamTransition::new(
        "finish_deactivate",
        &ParamPredicate::new(&vec![
            not_act_active.clone(),
            ref_idle.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
        ]),
        &ParamPredicate::new(&vec![act_idle.clone(), ref_idle.clone()]),
    );

    let t5 = ParamTransition::new(
        "start_move_left",
        &ParamPredicate::new(&vec![
            not_act_left.clone(),
            not_ref_left.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &ParamPredicate::new(&vec![ref_left.clone(), not_act_left.clone()]),
    );

    let t6 = ParamTransition::new(
        "finish_move_left",
        &ParamPredicate::new(&vec![
            not_act_left.clone(),
            ref_left.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &ParamPredicate::new(&vec![act_left.clone(), ref_left.clone()]),
    );

    let t7 = ParamTransition::new(
        "start_move_right",
        &ParamPredicate::new(&vec![
            not_act_right.clone(),
            not_ref_right.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &ParamPredicate::new(&vec![ref_right.clone(), not_act_right.clone()]),
    );

    let t8 = ParamTransition::new(
        "finish_move_right",
        &ParamPredicate::new(&vec![
            not_act_right.clone(),
            ref_right.clone(),
            act_active.clone(),
            ref_active.clone(),
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
            not_any_measured_dummy.clone(),
        ]),
        &ParamPredicate::new(&vec![act_right.clone(), ref_right.clone()]),
    );

    let problem = ParamPlanningProblem::new(
        "prob1",
        &ParamPredicate::new(&vec![
            not_act_active.clone(),
            not_ref_active.clone(),
            act_left.clone(),
            ref_left.clone(),
        ]),
        &ParamPredicate::new(&vec![
            act_active.clone(),
            ref_active.clone(),
            act_right.clone(),
            ref_right.clone(),
        ]),
        &vec![t1, t2, t3, t4, t5, t6, t7, t8],
        &ParamPredicate::new(&vec!(Predicate::TRUE)),
        &12,
        &vec!(p1, p2),
    );
    let result = parameterized(&problem, &5, &12);
    pprint_result(&result.result)
}
