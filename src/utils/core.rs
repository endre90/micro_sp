use super::*;
use std::str::FromStr;
use z3_sys::*;
use micro_z3_rust::*;

/// Given a planning result, remove sections that lead back to an already visited state.
/// Actually, have to do this iterativelly since removing a loop might remove part of another one.
/// So, recursively just keep finding and removing the biggest loop until there are no loops.
pub fn remove_loops(result: &PlanningResult) -> PlanningResult {
    let mut mut_result = result.clone();
    let mut duplicates: Vec<(State, usize, usize)> = vec!();
    let mut cleaned_trace: Vec<PlanningFrame> = vec!();

    // find the first and the last occurence of every duplicated state in a trace
    for tr in &result.trace {
        let start = match result.trace.iter().position(|x| x.source == tr.source) {
            Some(y) => y as usize,
            None => 12345
        };
        let finish = match result.trace.iter().rposition(|x| x.source == tr.source) {
            Some(y) => y as usize,
            None => 12345
        };
        if start != finish && start != 12345 && finish != 12345 {
            if !duplicates.iter().any(|x| x.0 == tr.source) {
                duplicates.push((tr.source.to_owned(), start, finish))
            }   
        }
    }

    duplicates.sort();
    duplicates.dedup();

    // if there are loops, find the biggest one and remove it
    if duplicates.len() != 0 {
        let mut biggest_loop: (State, usize, usize) = duplicates[0].clone();
        for d in &duplicates {
            if d.2 - d.1 >= biggest_loop.2 - biggest_loop.1 {
                biggest_loop = d.to_owned();
                cleaned_trace = result.trace.iter().clone().map(|x| x.to_owned()).collect();
            }
        }
        
        println!("{:?}", duplicates.iter().map(|x| (x.1, x.2)).collect::<Vec<(usize, usize)>>());
        println!("{:?}", (biggest_loop.1..biggest_loop.2));
        cleaned_trace.drain(biggest_loop.1..biggest_loop.2).for_each(drop);  
        duplicates.clear();

        mut_result = remove_loops(
            &PlanningResult {
                name: result.name.to_owned(),
                alg: result.alg.to_owned(),
                plan_found: result.plan_found,
                plan_length: cleaned_trace.len() as u64,
                trace: cleaned_trace,
                time_to_solve: result.time_to_solve,
                model_size: result.model_size
            }
        );
    } 
    mut_result.to_owned()
}

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

/// Given a predicate, return a vector of assignments in the predicate.
pub fn get_predicate_assigns(pred: &Predicate) -> Vec<Assignment> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_assigns(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_assigns(p))),
        Predicate::NOT(x) => s.extend(get_predicate_assigns(x)),
        Predicate::ASS(x) => s.push(x.clone()),
        Predicate::EQ(_x, _y) => (),
        Predicate::PBEQ(x, _) => s.extend(x.iter().flat_map(|p| get_predicate_assigns(p))),
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

/// Collect the state as a vector of predicates.
pub fn assignment_vector_to_predicate_vector(vec: &Vec<Assignment>) -> Vec<Predicate> {
    vec.iter()
        .map(|x| {
            Predicate::ASS(Assignment::new(
                &Variable::new(
                    &x.var.name,
                    &x.var.value_type,
                    &x.var.domain.iter().map(|x| x.to_owned()).collect(),
                    Some(&x.var.param),
                    Some(&x.var.r#type),
                    Some(&x.var.kind),
                ),
                &x.val,
                Some(&x.lifetime),
            ))
        })
        .collect::<Vec<Predicate>>()
}

/// Generate a predicate from a given state as a conjunction of assignments.
pub fn state_to_predicate(state: &State) -> Predicate {
    let mut pred = vec![];
    for i in vec![&state.measured, &state.command, &state.estimated] {
        pred.extend(assignment_vector_to_predicate_vector(&i))
    }
    Predicate::AND(pred)
}

/// Generate a parameterized predicate from a given state.
pub fn state_to_param_predicate(state: &State) -> ParamPredicate {
    let mut pred = vec![];
    for i in vec![&state.measured, &state.command, &state.estimated] {
        pred.extend(assignment_vector_to_predicate_vector(&i))
    }
    ParamPredicate::new(&pred)
}

/// Convert a parameterized planning problem to a regular planning problem.
pub fn unparam(prob: &ParamPlanningProblem) -> PlanningProblem {
    let activated = activate_all_in_problem(&prob);
    PlanningProblem::new(
        &activated.name,
        &generate_predicate(&activated.init, &activated.params),
        &generate_predicate(&activated.goal, &activated.params),
        &activated
            .trans
            .iter()
            .map(|x| generate_transition(x, &activated.params))
            .collect(),
        &activated.invars,
    )
}

/// After the incremental algorithm has found a model it is unrolled into a plan.
pub fn get_planning_result(
    ctx: &Z3_context,
    prob: &PlanningProblem,
    model: &Z3_model,
    alg: &str,
    nr_steps: u64,
    planning_time: std::time::Duration,
    plan_found: bool,
    model_size: u64
) -> PlanningResult {
    let model_str = model_to_string_z3(&ctx, model);
    let model_vec: Vec<Vec<&str>> = model_str
        .lines()
        .map(|l| l.split(" -> ").collect())
        .collect();
    let vars = get_problem_vars(&prob);

    // for m in &model_vec {
    //     // if m[1] == "true"{
    //         println!("{:?}", m);
    // //     }
    // }
    

    let mut trace: Vec<PlanningFrame> = vec![];
    for i in 0..nr_steps - 1 {
        let mut enum_vals_source = vec![];
        for v in &vars {
            for m in &model_vec {
                if m[0].ends_with(&format!("_s{}", i)) {
                    let trimmed = m[0].trim_end_matches(&format!("_s{}", i));
                    if v.name == trimmed {
                        match v.value_type {
                            SPValueType::Bool => enum_vals_source.push(Assignment::new(
                                &v,
                                &bool::from_str(m[1]).unwrap().to_spvalue(),
                                None,
                            )),
                            SPValueType::String => enum_vals_source.push(Assignment::new(
                                &v,
                                &String::from(m[1]).to_spvalue(),
                                None,
                            )),
                        }
                    }
                }
            }
        }

        let mut enum_vals_sink = vec![];
        for v in &vars {
            for m in &model_vec {
                if m[0].ends_with(&format!("_s{}", i + 1)) {
                    let trimmed = m[0].trim_end_matches(&format!("_s{}", i + 1));
                    if v.name == trimmed {
                        match v.value_type {
                            SPValueType::Bool => enum_vals_sink.push(Assignment::new(
                                &v,
                                &bool::from_str(m[1]).unwrap().to_spvalue(),
                                None,
                            )),
                            SPValueType::String => enum_vals_sink.push(Assignment::new(
                                &v,
                                &String::from(m[1]).to_spvalue(),
                                None,
                            )),
                        }
                    }
                }
            }
        }

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

        // let trans = model_vec
        //     .iter()
        //     .filter(|x| x[0].ends_with(&format!("_t{}_s{}", i + 1, i + 1)))
        //     .map(|x| (x[0].trim_end_matches(&format!("_t{}_s{}", i + 1, i + 1)), x[1], i + 1))
        //     .find(|x| x.1 == "true")
        //     .map(|z| z.0)
        //     .unwrap_or_default();

        let trans = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_t{}_s{}", i + 1, i + 1)))
            .map(|x| (x[0].trim_end_matches(&format!("_t{}_s{}", i + 1, i + 1)), x[1], i + 1))
            .filter(|x| x.1 == "true")
            .map(|z| z.0.to_string())
            .collect();

        let mut source = vec![];
        for i in vec![measured_source, command_source, estimated_source] {
            source.extend(i)
        }

        let mut sink = vec![];
        for i in vec![measured_sink, command_sink, estimated_sink] {
            sink.extend(i)
        }

        trace.push(PlanningFrame {
            source: State::from_vec(&source),
            trans: trans,
            sink: State::from_vec(&sink),
        });
    }
    match plan_found {
        true => PlanningResult {
            name: prob.name.clone(),
            alg: alg.to_owned(),
            plan_found: plan_found,
            plan_length: nr_steps - 1,
            trace: trace,
            time_to_solve: planning_time,
            model_size: model_size
        },
        false => PlanningResult {
            name: prob.name.clone(),
            plan_found: plan_found,
            alg: alg.to_owned(),
            plan_length: 0,
            trace: vec![],
            time_to_solve: planning_time,
            model_size: model_size
        },
    }
}
