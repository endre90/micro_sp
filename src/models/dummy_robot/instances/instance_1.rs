use super::*;

pub fn instance() -> ParamPlanningProblem {

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

    ParamPlanningProblem::new(
        "dummy_name",
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
        &vec![],
        &ParamPredicate::new(&vec![Predicate::TRUE]),
        &vec!(p1, p2)
    )
}