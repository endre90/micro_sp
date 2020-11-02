use super::*;
use z3_sys::*;
use z3_v2::*;
use std::str::FromStr;

/// Given a predicate, return a vector of variables that play a role in it.
pub fn get_predicate_vars(pred: &Predicate) -> Vec<Variable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars(x)),
        Predicate::ASS(x) => s.push(x.var.clone()),
        Predicate::EQ(x, y) => {
            s.push(x.clone());
            s.push(y.clone());
        }
        Predicate::PBEQ(x, _) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
    }
    s.sort();
    s.dedup();
    s
}

/// Given a parameterized predicate, return a vector of variables that play a role in it.
pub fn get_param_predicate_vars(ppred: &ParamPredicate) -> Vec<Variable> {
    ppred
        .preds
        .iter()
        .map(|x| get_predicate_vars(&x))
        .flatten()
        .collect()
}

pub fn get_model_vars(trans: &Vec<Transition>) -> Vec<Variable> {
    let mut s = Vec::new();
    for t in trans {
        s.extend(get_predicate_vars(&t.guard));
        s.extend(get_predicate_vars(&t.update));
    }
    s.sort();
    s.dedup();
    s
}

pub fn get_param_model_vars(trans: &Vec<ParamTransition>) -> Vec<Variable> {
    let mut s = Vec::new();
    for t in trans {
        s.extend(get_param_predicate_vars(&t.guard));
        s.extend(get_param_predicate_vars(&t.update));
    }
    s.sort();
    s.dedup();
    s
}

/// Given a planning problem, return a vector of all variables defined for that problem.
pub fn get_problem_vars(prob: &PlanningProblem) -> Vec<Variable> {
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

// /// Collect the state as a vector of predicates.
// pub fn state_to_predicate_vector(state: &State) -> Vec<Predicate> {
//     state
//         .vec
//         .iter()
//         .map(|x| {
//             Predicate::SET(EnumValue::new(
//                 &EnumVariable::new(
//                     &x.var.name,
//                     &x.var.domain.iter().map(|x| x.as_str()).collect(),
//                     &x.var.r#type,
//                     Some(&x.var.param),
//                     &x.var.kind,
//                 ),
//                 &x.val,
//                 Some(&x.lifetime),
//             ))
//         })
//         .collect::<Vec<Predicate>>()
// }

// /// Generate a predicate from a given state as a conjunction of values.
// pub fn state_to_predicate(state: &State) -> Predicate {
//     Predicate::AND(state_to_predicate_vector(&state))
// }

// /// Generate a parameterized predicate from a given state.
// pub fn state_to_param_predicate(state: &State) -> ParamPredicate {
//     ParamPredicate::new(&state_to_predicate_vector(&state))
// }

// /// Generate a predicate from a complete state as a conjunction of values.
// pub fn complete_state_to_predicate(state: &CompleteState) -> Predicate {
//     Predicate::AND(vec![
//         state_to_predicate(&state.measured),
//         state_to_predicate(&state.command),
//         state_to_predicate(&state.estimated),
//     ])
// }

// /// Generate a parameterized predicate from a complete state.
// pub fn complete_state_to_param_predicate(state: &CompleteState) -> ParamPredicate {
//     ParamPredicate::new(&vec![
//         state_to_predicate(&state.measured),
//         state_to_predicate(&state.command),
//         state_to_predicate(&state.estimated),
//     ])
// }

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

    // println!("{:#?}", model_vec);

    let mut trace: Vec<PlanningFrame> = vec![];
    for i in 0..nr_steps - 1 {
        let enum_vals_source: Vec<Assignment> = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_s{}", i)))
            .map(|x| (x[0].trim_end_matches(&format!("_s{}", i)), x[1], i))
            .map(|x| (vars.iter().find(|y| y.name == x.0).unwrap(), x.1))
            .map(|x| match x.0.value_type {
                SPValueType::Bool => Assignment::new(&x.0, &bool::from_str(x.1).unwrap().to_spvalue(), None),
                SPValueType::String => Assignment::new(&x.0, &String::from(x.1).to_spvalue(), None)
            })
            .collect();

        let enum_vals_sink: Vec<Assignment> = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_s{}", i + 1)))
            .map(|x| (x[0].trim_end_matches(&format!("_s{}", i + 1)), x[1], i + 1))
            .map(|x| (vars.iter().find(|y| y.name == x.0).unwrap(), x.1))
            .map(|x| match x.0.value_type {
                SPValueType::Bool => Assignment::new(&x.0, &bool::from_str(x.1).unwrap().to_spvalue(), None),
                SPValueType::String => Assignment::new(&x.0, &String::from(x.1).to_spvalue(), None)
            })
            .collect();

        let measured_source: Vec<Assignment> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Measured)
            .map(|y| y.clone())
            .collect::<Vec<Assignment>>();
        let command_source: Vec<Assignment> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Command)
            .map(|y| y.clone())
            .collect();
        let estimated_source: Vec<Assignment> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Estimated)
            .map(|y| y.clone())
            .collect();

        let measured_sink: Vec<Assignment> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Measured)
            .map(|y| y.clone())
            .collect();

        let command_sink: Vec<Assignment> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Command)
            .map(|y| y.clone())
            .collect();
        let estimated_sink: Vec<Assignment> = enum_vals_sink
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

        let mut source = vec!();
        for i in vec!(measured_source, command_source, estimated_source) {
            source.extend(i)
        }

        let mut sink = vec!();
        for i in vec!(measured_sink, command_sink, estimated_sink) {
            sink.extend(i)
        }

        trace.push(PlanningFrame {
            source: State::from_vec(&source),
            trans: String::from(trans),
            sink: State::from_vec(&sink)
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
