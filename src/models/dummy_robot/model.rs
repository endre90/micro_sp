use super::*;

pub fn model(instance: &str) -> ParamPlanningProblem {

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
        &ParamPredicate::new(&vec![not_act_active.clone(), not_ref_active.clone()]),
        &ParamPredicate::new(&vec![ref_active.clone()]),
    );

    let t2 = ParamTransition::new(
        "finish_activate",
        &ParamPredicate::new(&vec![not_act_active.clone(), ref_active.clone()]),
        &ParamPredicate::new(&vec![act_active.clone()]),
    );
  
    let t3 = ParamTransition::new(
        "start_deactivate",
        &ParamPredicate::new(&vec![not_act_idle.clone(), not_ref_idle.clone()]),
        &ParamPredicate::new(&vec![ref_idle.clone()]),
    );

    let t4 = ParamTransition::new(
        "finish_deactivate",
        &ParamPredicate::new(&vec![not_act_idle.clone(), ref_idle.clone()]),
        &ParamPredicate::new(&vec![act_idle.clone()]),
    );

    let t5 = ParamTransition::new(
        "start_move_left",
        &ParamPredicate::new(&vec![
            not_act_left.clone(),
            not_ref_left.clone(),
            act_active.clone(),
            ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![ref_left.clone()]),
    );

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

    let transitions = vec![t1, t2, t3, t4, t5, t6, t7, t8];
    let invariants = ParamPredicate::new(&vec!(Predicate::TRUE));
 
    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &true);

    match instance {
        "instance_1" => {
            ParamPlanningProblem::new(
                instance, 
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
                &transitions,
                &invariants,
                &vec!(p1, p2)
            )
        },
        _ => panic!("No such instance")
    }
}