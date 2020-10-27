use super::*;
use z3_sys::*;
use z3_v2::*;

/// Given a predicate, return a vector of variables that play a role in it.
pub fn get_predicate_vars(pred: &Predicate) -> Vec<EnumVariable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars(x)),
        Predicate::EQ(x) => s.push(x.var.clone()),
        Predicate::EQRR(x, y) => {
            s.push(x.clone());
            s.push(y.clone());
        },
        Predicate::PBEQ(x, _) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
    }
    s.sort();
    s.dedup();
    s
}

/// Given a parameterized predicate, return a vector of variables that play a role in it.
pub fn get_param_predicate_vars(ppred: &ParamPredicate) -> Vec<EnumVariable> {
    ppred.preds.iter().map(|x| get_predicate_vars(&x)).flatten().collect()
}

/// Given a planning problem, return a vector of all variables defined for that problem.
pub fn get_problem_vars(prob: &PlanningProblem) -> Vec<EnumVariable> {
    let mut s = Vec::new();
    for t in &prob.trans {
        s.extend(get_predicate_vars(&t.guard));
        s.extend(get_predicate_vars(&t.update));
    }
    s.extend(get_predicate_vars(&prob.init));
    s.extend(get_predicate_vars(&prob.goal));
    s.sort();
    s.dedup();
    s
}

/// After the incremental algorithm has found a model it is unrolled into a plan.
pub fn get_planning_result(
    ctx: &ContextZ3,
    prob: &PlanningProblem,
    model: Z3_model,
    nr_steps: u32,
    planning_time: std::time::Duration,
    plan_found: bool,
) -> PlanningResult {
    let model_str = ModelToStringZ3::new(&ctx, model);
    let model_vec: Vec<Vec<&str>> = model_str
        .lines()
        .map(|l| l.split(" -> ").collect())
        .collect();
    let vars = get_problem_vars(&prob);

    // println!("MODEL:");
    // for m in &model_vec {
    //     println!("{:?}", m);
    // }

    let mut trace: Vec<PlanningFrame> = vec![];
    for i in 0..nr_steps - 1 {
        let enum_vals_source: Vec<EnumValue> = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_s{}", i)))
            .map(|x| (x[0].trim_end_matches(&format!("_s{}", i)), x[1], i))
            .map(|x| (vars.iter().find(|y| y.name == x.0).unwrap(), x.1))
            .map(|x| EnumValue::new(&x.0, x.1, None))
            .collect();

        let enum_vals_sink: Vec<EnumValue> = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_s{}", i + 1)))
            .map(|x| (x[0].trim_end_matches(&format!("_s{}", i + 1)), x[1], i + 1))
            .map(|x| (vars.iter().find(|y| y.name == x.0).unwrap(), x.1))
            .map(|x| EnumValue::new(&x.0, x.1, None))
            .collect();

        let measured_source: Vec<EnumValue> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Measured)
            .map(|y| y.clone())
            .collect::<Vec<EnumValue>>();
        // let handshake_source: Vec<EnumValue> = enum_vals_source
        //     .iter()
        //     .filter(|x| x.var.kind == Kind::Handshake)
        //     .map(|y| y.clone())
        //     .collect::<Vec<EnumValue>>();
        let command_source: Vec<EnumValue> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Command)
            .map(|y| y.clone())
            .collect();
        let estimated_source: Vec<EnumValue> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Estimated)
            .map(|y| y.clone())
            .collect();

        let measured_sink: Vec<EnumValue> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Measured)
            .map(|y| y.clone())
            .collect();
        // let handshake_sink: Vec<EnumValue> = enum_vals_sink
        //     .iter()
        //     .filter(|x| x.var.kind == Kind::Handshake)
        //     .map(|y| y.clone())
        //     .collect();
        let command_sink: Vec<EnumValue> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Command)
            .map(|y| y.clone())
            .collect();
        let estimated_sink: Vec<EnumValue> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Estimated)
            .map(|y| y.clone())
            .collect();

        let trans = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_t{}", i + 1)))
            .map(|x| (x[0].trim_end_matches(&format!("_t{}", i + 1)), x[1], i + 1))
            .find(|x| x.1 == "true")
            .map(|z| z.0)
            .unwrap_or_default();

        trace.push(PlanningFrame {
            source: CompleteState::from_states(
                &State::new(&measured_source, &Kind::Measured),
                // &State::new(&handshake_source, &Kind::Handshake),
                &State::new(&command_source, &Kind::Command),
                &State::new(&estimated_source, &Kind::Estimated),
            ),
            trans: String::from(trans),
            sink: CompleteState::from_states(
                &State::new(&measured_sink, &Kind::Measured),
                // &State::new(&handshake_sink, &Kind::Handshake),
                &State::new(&command_sink, &Kind::Command),
                &State::new(&estimated_sink, &Kind::Estimated),
            ),
        });
    }
    match plan_found {
        true => PlanningResult {
            plan_found: plan_found,
            plan_length: nr_steps - 1,
            trace: trace,
            time_to_solve: planning_time,
        },
        false => PlanningResult {
            plan_found: plan_found,
            plan_length: 0,
            trace: vec![],
            time_to_solve: planning_time,
        },
    }
}

