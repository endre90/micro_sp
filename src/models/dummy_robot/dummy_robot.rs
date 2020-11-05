use super::*;

pub fn model() -> PlanningProblem {
    let act_pos = enum_m!(
        "act_pos",
        vec!("left", "right", "unknown", "dummy_value"),
        "pos"
    );
    let act_stat = enum_m!(
        "act_stat",
        vec!("idle", "active", "unknown", "dummy_value"),
        "stat"
    );
    let ref_pos = enum_c!(
        "ref_pos",
        vec!("left", "right", "unknown", "dummy_value"),
        "pos"
    );
    let ref_stat = enum_c!(
        "ref_stat",
        vec!("idle", "active", "unknown", "dummy_value"),
        "stat"
    );

    let act_pos_dummy = Predicate::ASS(enum_assign!(act_pos, "dummy_value"));
    let act_stat_dummy = Predicate::ASS(enum_assign!(act_stat, "dummy_value"));

    let not_any_measured_dummy = Predicate::NOT(Box::new(Predicate::OR(vec![
        act_pos_dummy.clone(),
        act_stat_dummy.clone(),
    ])));

    let act_left = Predicate::ASS(enum_assign!(act_pos, "left"));
    let not_act_left = Predicate::NOT(Box::new(act_left.clone()));
    let act_right = Predicate::ASS(enum_assign!(act_pos, "right"));
    let not_act_right = Predicate::NOT(Box::new(act_right.clone()));

    let act_idle = Predicate::ASS(enum_assign!(act_stat, "idle"));
    let not_act_idle = Predicate::NOT(Box::new(act_idle.clone()));
    let act_active = Predicate::ASS(enum_assign!(act_stat, "active"));
    let not_act_active = Predicate::NOT(Box::new(act_active.clone()));

    let ref_left = Predicate::ASS(enum_assign!(ref_pos, "left"));
    let not_ref_left = Predicate::NOT(Box::new(ref_left.clone()));
    let ref_right = Predicate::ASS(enum_assign!(ref_pos, "right"));
    let not_ref_right = Predicate::NOT(Box::new(ref_right.clone()));

    let ref_idle = Predicate::ASS(enum_assign!(ref_stat, "idle"));
    let not_ref_idle = Predicate::NOT(Box::new(ref_idle.clone()));
    let ref_active = Predicate::ASS(enum_assign!(ref_stat, "active"));
    let not_ref_active = Predicate::NOT(Box::new(ref_active.clone()));

    let t1 = Transition::new(
        "start_activate",
        &Predicate::AND(vec![
            not_act_active.clone(),
            not_ref_active.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
        ]),
        &Predicate::AND(vec![not_act_active.clone(), ref_active.clone()]),
    );

    let t2 = Transition::new(
        "finish_activate",
        &Predicate::AND(vec![
            not_act_active.clone(),
            ref_active.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
        ]),
        &Predicate::AND(vec![act_active.clone(), ref_active.clone()]),
    );

    let t3 = Transition::new(
        "start_deactivate",
        &Predicate::AND(vec![
            not_act_idle.clone(),
            not_ref_idle.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
        ]),
        &Predicate::AND(vec![ref_idle.clone(), ref_idle.clone()]),
    );

    let t4 = Transition::new(
        "finish_deactivate",
        &Predicate::AND(vec![
            not_act_active.clone(),
            ref_idle.clone(),
            not_any_measured_dummy.clone(),
            Predicate::EQ(act_pos.clone(), ref_pos.clone()),
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
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
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
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
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
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
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
            Predicate::EQ(act_stat.clone(), ref_stat.clone()),
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
    );
    problem
}

pub fn param_model() -> (ParamPlanningProblem, Vec<Parameter>) {
    let act_pos = enum_m!(
        "act_pos",
        vec!("left", "right", "unknown", "dummy_value"),
        "pos",
        "p2"
    );
    let act_stat = enum_m!(
        "act_stat",
        vec!("idle", "active", "unknown", "dummy_value"),
        "stat",
        "p1"
    );
    let ref_pos = enum_c!(
        "ref_pos",
        vec!("left", "right", "unknown", "dummy_value"),
        "pos",
        "p2"
    );
    let ref_stat = enum_c!(
        "ref_stat",
        vec!("idle", "active", "unknown", "dummy_value"),
        "stat",
        "p1"
    );

    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &true);
 
    let act_pos_dummy = pass!(&enum_assign!(act_pos, "dummy_value"));
    let act_stat_dummy = pass!(&enum_assign!(act_stat, "dummy_value"));

    let not_any_measured_dummy = pnot!(&por!(&act_pos_dummy, &act_stat_dummy));

    let act_left = pass!(&enum_assign!(act_pos, "left"));
    let not_act_left = pnot!(&act_left);
    let act_right = pass!(&enum_assign!(act_pos, "right"));
    let not_act_right = pnot!(&act_right);

    let act_idle = pass!(&enum_assign!(act_stat, "idle"));
    let not_act_idle = pnot!(&act_idle);
    let act_active = pass!(&enum_assign!(act_stat, "active"));
    let not_act_active = pnot!(&act_active);

    let ref_left = pass!(&enum_assign!(ref_pos, "left"));
    let not_ref_left = pnot!(&ref_left);
    let ref_right = pass!(&enum_assign!(ref_pos, "right"));
    let not_ref_right = pnot!(&ref_right);

    let ref_idle = pass!(&enum_assign!(ref_stat, "idle"));
    let not_ref_idle = pnot!(&ref_idle);
    let ref_active = pass!(&enum_assign!(ref_stat, "active"));
    let not_ref_active = pnot!(&ref_active);

    let t1 = ParamTransition::new(
        "start_activate",
        &ParamPredicate::new(
            &vec!(
                not_act_active.clone(),
                not_ref_active.clone()
            )
        ),
        &ParamPredicate::new(
            &vec!(
                ref_active.clone()
            )
        )
    );

    // let t1 = ParamTransition::new(
    //     "start_activate",
    //     &ParamPredicate::new(&vec![
    //         not_act_active.clone(),
    //         not_ref_active.clone(),
    //         not_any_measured_dummy.clone(),
    //         Predicate::EQ(act_pos.clone(), ref_pos.clone()),
    //     ]),
    //     &ParamPredicate::new(&vec![not_act_active.clone(), ref_active.clone()]),
    // );

    let t2 = ParamTransition::new(
        "finish_activate",
        &ParamPredicate::new(
            &vec!(
                not_act_active.clone(),
                ref_active.clone()
            )
        ),
        &ParamPredicate::new(
            &vec!(
                act_active.clone()
            )
        )
    );
        
    // let t2 = ParamTransition::new(
    //     "finish_activate",
    //     &ParamPredicate::new(&vec![
    //         not_act_active.clone(),
    //         ref_active.clone(),
    //         not_any_measured_dummy.clone(),
    //         Predicate::EQ(act_pos.clone(), ref_pos.clone()),
    //     ]),
    //     &ParamPredicate::new(&vec![act_active.clone(), ref_active.clone()]),
    // );
        
    let t3 = ParamTransition::new(
        "start_deactivate",
        &ParamPredicate::new(
            &vec!(
                not_act_idle.clone(),
                not_ref_idle.clone()
            )
        ),
        &ParamPredicate::new(
            &vec!(
                ref_idle.clone()
            )
        )
    );

    // let t3 = ParamTransition::new(
    //     "start_deactivate",
    //     &ParamPredicate::new(&vec![
    //         not_act_idle.clone(),
    //         not_ref_idle.clone(),
    //         not_any_measured_dummy.clone(),
    //         Predicate::EQ(act_pos.clone(), ref_pos.clone()),
    //     ]),
    //     &ParamPredicate::new(&vec![ref_idle.clone(), ref_idle.clone()]),
    // );

    let t4 = ParamTransition::new(
        "finish_deactivate",
        &ParamPredicate::new(
            &vec!(
                not_act_idle.clone(),
                ref_idle.clone()
            )
        ),
        &ParamPredicate::new(
            &vec!(
                act_idle.clone()
            )
        )
    );
        
    // let t4 = ParamTransition::new(
    //     "finish_deactivate",
    //     &ParamPredicate::new(&vec![
    //         not_act_active.clone(),
    //         ref_idle.clone(),
    //         not_any_measured_dummy.clone(),
    //         Predicate::EQ(act_pos.clone(), ref_pos.clone()),
    //     ]),
    //     &ParamPredicate::new(&vec![act_idle.clone(), ref_idle.clone()]),
    // );

    let t5 = ParamTransition::new(
        "start_move_left",
        &ParamPredicate::new(&vec![
            not_act_left.clone(),
            not_ref_left.clone(),
            act_active.clone(),
            ref_active.clone()
        ]),
        &ParamPredicate::new(&vec![ref_left.clone()]),
    );
        
    // let t5 = ParamTransition::new(
    //     "start_move_left",
    //     &ParamPredicate::new(&vec![
    //         not_act_left.clone(),
    //         not_ref_left.clone(),
    //         act_active.clone(),
    //         ref_active.clone(),
    //         Predicate::EQ(act_stat.clone(), ref_stat.clone()),
    //         not_any_measured_dummy.clone(),
    //     ]),
    //     &ParamPredicate::new(&vec![ref_left.clone(), not_act_left.clone()]),
    // );

    let t6 = ParamTransition::new(
        "finish_move_left",
        &ParamPredicate::new(&vec![
            not_act_left.clone(),
            ref_left.clone(),
            act_active.clone(),
            ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![act_left.clone()]),
    );
        
    // let t6 = ParamTransition::new(
    //     "finish_move_left",
    //     &ParamPredicate::new(&vec![
    //         not_act_left.clone(),
    //         ref_left.clone(),
    //         act_active.clone(),
    //         ref_active.clone(),
    //         Predicate::EQ(act_stat.clone(), ref_stat.clone()),
    //         not_any_measured_dummy.clone(),
    //     ]),
    //     &ParamPredicate::new(&vec![act_left.clone(), ref_left.clone()]),
    // );

    let t7 = ParamTransition::new(
        "start_move_right",
        &ParamPredicate::new(&vec![
            not_act_right.clone(),
            not_ref_right.clone(),
            act_active.clone(),
            ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![ref_right.clone()]),
    );
        
    // let t7 = ParamTransition::new(
    //     "start_move_right",
    //     &ParamPredicate::new(&vec![
    //         not_act_right.clone(),
    //         not_ref_right.clone(),
    //         act_active.clone(),
    //         ref_active.clone(),
    //         Predicate::EQ(act_stat.clone(), ref_stat.clone()),
    //         not_any_measured_dummy.clone(),
    //     ]),
    //     &ParamPredicate::new(&vec![ref_right.clone(), not_act_right.clone()]),
    // );

    let t8 = ParamTransition::new(
        "finish_move_right",
        &ParamPredicate::new(&vec![
            not_act_right.clone(),
            ref_right.clone(),
            act_active.clone(),
            ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![act_right.clone()]),
    );
        
    // let t8 = ParamTransition::new(
    //     "finish_move_right",
    //     &ParamPredicate::new(&vec![
    //         not_act_right.clone(),
    //         ref_right.clone(),
    //         act_active.clone(),
    //         ref_active.clone(),
    //         Predicate::EQ(act_stat.clone(), ref_stat.clone()),
    //         not_any_measured_dummy.clone(),
    //     ]),
    //     &ParamPredicate::new(&vec![act_right.clone(), ref_right.clone()]),
    // );
        
    let problem = ParamPlanningProblem::new(
        "prob1",
        &ParamPredicate::new(&vec![
            act_idle.clone(),
            ref_idle.clone(),
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
        &12
    );
    (problem, vec!(p2, p1))   
}