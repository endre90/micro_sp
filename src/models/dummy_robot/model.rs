use super::*;

pub fn model1(instance: &str) -> ParamPlanningProblem {

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
                &Predicate::TRUE,
                &vec!(p1, p2)
            )
        },
        _ => panic!("No such instance")
    }
}

pub fn model2(instance: &str) -> ParamPlanningProblem {

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

    let act_grip_pos = enum_m!(
        "act_grip_pos",
        vec!("open", "closed", "unknown", "dummy_value"),
        "grip_pos",
        "p3"
    );
    let act_grip_stat = enum_m!(
        "act_grip_stat",
        vec!("idle", "active", "unknown", "dummy_value"),
        "grip_stat",
        "p4"
    );
    let ref_grip_pos = enum_c!(
        "ref_girp_pos",
        vec!("open", "closed", "unknown", "dummy_value"),
        "grip_pos",
        "p3"
    );
    let ref_grip_stat = enum_c!(
        "ref_grip_stat",
        vec!("idle", "active", "unknown", "dummy_value"),
        "grip_stat",
        "p4"
    );

    let act_pos_dummy = pass!(&enum_assign!(act_pos, "dummy_value"));
    let act_stat_dummy = pass!(&enum_assign!(act_stat, "dummy_value"));
    let act_grip_pos_dummy = pass!(&enum_assign!(act_grip_pos, "dummy_value"));
    let act_grip_stat_dummy = pass!(&enum_assign!(act_grip_stat, "dummy_value"));

    let not_any_measured_dummy = pnot!(&por!(&act_pos_dummy, &act_stat_dummy, &act_grip_pos_dummy, &act_grip_stat_dummy));

    // robot
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

    //gripper
    let grip_act_open = pass!(&enum_assign!(act_grip_pos, "open"));
    let grip_not_act_open = pnot!(&grip_act_open);
    let grip_act_closed = pass!(&enum_assign!(act_grip_pos, "closed"));
    let grip_not_act_closed = pnot!(&grip_act_closed);

    let grip_act_idle = pass!(&enum_assign!(act_grip_stat, "idle"));
    let grip_not_act_idle = pnot!(&grip_act_idle);
    let grip_act_active = pass!(&enum_assign!(act_grip_stat, "active"));
    let grip_not_act_active = pnot!(&grip_act_active);

    let grip_ref_open = pass!(&enum_assign!(ref_grip_pos, "open"));
    let grip_not_ref_open = pnot!(&grip_ref_open);
    let grip_ref_closed = pass!(&enum_assign!(ref_grip_pos, "closed"));
    let grip_not_ref_closed = pnot!(&grip_ref_closed);

    let grip_ref_idle = pass!(&enum_assign!(ref_grip_stat, "idle"));
    let grip_not_ref_idle = pnot!(&grip_ref_idle);
    let grip_ref_active = pass!(&enum_assign!(ref_grip_stat, "active"));
    let grip_not_ref_active = pnot!(&grip_ref_active);

    //robot
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

    //gripper
    let t9 = ParamTransition::new(
        "gripper_start_activate",
        &ParamPredicate::new(&vec![grip_not_act_active.clone(), grip_not_ref_active.clone()]),
        &ParamPredicate::new(&vec![grip_ref_active.clone()]),
    );

    let t10 = ParamTransition::new(
        "gripper_finish_activate",
        &ParamPredicate::new(&vec![grip_not_act_active.clone(), grip_ref_active.clone()]),
        &ParamPredicate::new(&vec![grip_act_active.clone()]),
    );
  
    let t11 = ParamTransition::new(
        "gripper_start_deactivate",
        &ParamPredicate::new(&vec![grip_not_act_idle.clone(), grip_not_ref_idle.clone()]),
        &ParamPredicate::new(&vec![grip_ref_idle.clone()]),
    );

    let t12 = ParamTransition::new(
        "gripper_finish_deactivate",
        &ParamPredicate::new(&vec![grip_not_act_idle.clone(), grip_ref_idle.clone()]),
        &ParamPredicate::new(&vec![grip_act_idle.clone()]),
    );

    let t13 = ParamTransition::new(
        "gripper_start_open",
        &ParamPredicate::new(&vec![
            grip_not_act_open.clone(),
            grip_not_ref_open.clone(),
            grip_act_active.clone(),
            grip_ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![grip_ref_open.clone()]),
    );

    let t14 = ParamTransition::new(
        "gripper_finish_open",
        &ParamPredicate::new(&vec![
            grip_not_act_open.clone(),
            grip_ref_open.clone(),
            grip_act_active.clone(),
            grip_ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![grip_act_open.clone()]),
    );

    let t15 = ParamTransition::new(
        "gripper_start_close",
        &ParamPredicate::new(&vec![
            grip_not_act_closed.clone(),
            grip_not_ref_closed.clone(),
            grip_act_active.clone(),
            grip_ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![grip_ref_closed.clone()]),
    );
   
    let t16 = ParamTransition::new(
        "gripper_finish_close",
        &ParamPredicate::new(&vec![
            grip_not_act_closed.clone(),
            grip_ref_closed.clone(),
            grip_act_active.clone(),
            grip_ref_active.clone(),
        ]),
        &ParamPredicate::new(&vec![grip_act_closed.clone()]),
    );

    let transitions = vec![t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13, t14, t15, t16];
 
    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &true);
    let p3 = Parameter::new("p3", &true);
    let p4 = Parameter::new("p4", &true);

    match instance {
        "instance_1" => {
            ParamPlanningProblem::new(
                instance, 
                &ParamPredicate::new(&vec![
                    act_idle.clone(),
                    ref_idle.clone(),
                    act_left.clone(),
                    ref_left.clone(),
                    grip_act_idle.clone(),
                    grip_ref_idle.clone(),
                    grip_act_open.clone(),
                    grip_ref_open.clone(),
                ]),
                &ParamPredicate::new(&vec![
                    act_active.clone(),
                    ref_active.clone(),
                    act_right.clone(),
                    ref_right.clone(),
                    grip_act_active.clone(),
                    grip_ref_active.clone(),
                    grip_act_closed.clone(),
                    grip_ref_closed.clone(),
                ]),
                &transitions,
                &Predicate::TRUE,
                &vec!(p1, p2, p3, p4)
            )
        },
        _ => panic!("No such instance")
    }
}