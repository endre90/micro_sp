use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

pub enum Case {
    First,
    Central,
    Last,
    Zerolength,
}

/// Given a parameter list, return it with the next value activated.
pub fn activate_next(params: &Vec<Parameter>) -> Vec<Parameter> {
    let mut one_activated = false;
    params
        .iter()
        .map(|x| {
            if !x.value && !one_activated {
                one_activated = true;
                Parameter::new(x.name.as_str(), &true)
            } else {
                x.to_owned().to_owned()
            }
        })
        .collect()
}

/// Given a parameter list, return it with all values activated.
pub fn activate_all(params: &Vec<Parameter>) -> Vec<Parameter> {
    params
        .iter()
        .map(|x| Parameter::new(&x.name, &true))
        .collect()
}

/// Given a parameter list, return it with all values deactivated.
pub fn deactivate_all(params: &Vec<Parameter>) -> Vec<Parameter> {
    params
        .iter()
        .map(|x| Parameter::new(&x.name, &false))
        .collect()
}

/// Given a parameterized planning problem, return it with the next value activated.
pub fn activate_next_in_problem(prob: &ParamPlanningProblem) -> ParamPlanningProblem {
    ParamPlanningProblem {
        name: prob.name.to_owned(),
        init: prob.init.to_owned(),
        goal: prob.goal.to_owned(),
        trans: prob.trans.to_owned(),
        invars: prob.invars.to_owned(),
        params: activate_next(&prob.params)
    }
}

/// Given a parameterized planning problem, return it with all values activated.
pub fn activate_all_in_problem(prob: &ParamPlanningProblem) -> ParamPlanningProblem {
    ParamPlanningProblem {
        name: prob.name.to_owned(),
        init: prob.init.to_owned(),
        goal: prob.goal.to_owned(),
        trans: prob.trans.to_owned(),
        invars: prob.invars.to_owned(),
        params: activate_all(&prob.params)
    }
}

/// Given a parameterized planning problem, return it with all values deactivated.
pub fn deactivate_all_in_problem(prob: &ParamPlanningProblem) -> ParamPlanningProblem {
    ParamPlanningProblem {
        name: prob.name.to_owned(),
        init: prob.init.to_owned(),
        goal: prob.goal.to_owned(),
        trans: prob.trans.to_owned(),
        invars: prob.invars.to_owned(),
        params: deactivate_all(&prob.params)
    }
}

/// Generate and solve the refined concat-th problem of a result
pub fn generate_and_solve(
    case: &Case,
    inh: &State,
    prob: &ParamPlanningProblem,
    res: &PlanningResult,
    params: &Vec<Parameter>,
    level: u64,
    concat: u64,
    timeout: u64,
    max_steps: u64,
) -> PlanningResult {
    let res = parameterized(
        &ParamPlanningProblem {
            name: format!("problem_l{:?}_c{:?}", level, concat),
            init: match case {
                Case::First => prob.init.to_owned(),
                Case::Central => state_to_param_predicate(&inh),
                Case::Last => state_to_param_predicate(&inh),
                Case::Zerolength => prob.init.to_owned(),
            },
            goal: match case {
                Case::First => state_to_param_predicate(&res.trace[concat.to_owned() as usize + 1].source),
                Case::Central => state_to_param_predicate(&res.trace[concat.to_owned() as usize + 1].source),
                Case::Last => prob.goal.to_owned(),
                Case::Zerolength => prob.goal.to_owned(),
            },
            trans: prob.trans.to_owned(),
            invars: prob.invars.to_owned(),
            params: params.to_owned()
        },
        &params,
        timeout,
        max_steps
    );
    match res.plan_found {
        true => {
            println!("SUBPLAN");
            pprint_result(&res);
            res
        }
        // Maybe handle this differently, like return an empty plan
        false => panic!("Error 66a7001a-67f1-4876-9928-b90b6aa55936: No plan found."),
    }
}

/// Concatenate all results in a level.
pub fn concatenate(results: &Vec<PlanningResult>) -> PlanningResult {
    PlanningResult {
        plan_found: results.iter().all(|x| x.plan_found),
        plan_length: results.iter().map(|x| x.plan_length).sum(),
        trace: results
            .iter()
            .map(|x| x.trace.to_owned())
            .flatten()
            .collect(),
        time_to_solve: results.iter().map(|x| x.time_to_solve).sum(),
    }
}

// pub fn compositional(
//     prob: &ParamPlanningProblem,
//     params: &Vec<Parameter>,
//     timeout: u64,
// ) -> PlanningResult {
//     let one = activate_next(&deactivate_all(&params));
//     let first_res = parameterized(&prob, &one, timeout);
//     println!("LEVEL: {:?}", 0);
//     println!("PARAMS: {:?}", one);
//     pprint_result(&first_res);
//     let return_result = recursive(&first_res, &prob, &one, &0, timeout);

//     fn recursive(
//         res: &PlanningResult,
//         prob: &ParamPlanningProblem,
//         params: &Vec<Parameter>,
//         level: &u32,
//         timeout: u64
//     ) -> PlanningResult {
//         let level = level + 1;
//         let mut final_result: PlanningResult = res.to_owned();
//         if !params.iter().all(|x| x.value) {
//             if res.plan_found {
//                 let mut level_subresults = vec![];
//                 let mut inh = State::empty();
//                 let mut concat: u32 = 0;
//                 let act = activate_next(&params);
//                 println!("LEVEL: {:?}", level);
//                 println!("PARAMS: {:?}", params);
//                 if res.plan_length != 0 {
//                     for i in 0..=res.trace.len() - 1 {
//                         println!("CONCAT: {:?}", concat);
//                         if i == 0 {
//                             println!("FIRST CASE");
//                             let next = generate_and_solve(
//                                 &Case::First,
//                                 &inh,
//                                 &prob,
//                                 &res,
//                                 &act,
//                                 &level,
//                                 &concat,
//                                 timeout
//                             );
//                             level_subresults.push(next.to_owned());
//                             match next.trace.last() {
//                                 Some(x) => inh = x.sink.clone(),
//                                 None => panic!("Error cb10dd80-f6dd-4ae1-9119-116d8ba09dfa: No tail in the plan.")
//                             }
//                             concat = concat + 1;
//                         } else if i == res.trace.len() - 1 {
//                             println!("LAST CASE");
//                             let next = generate_and_solve(
//                                 &Case::Last,
//                                 &inh,
//                                 &prob,
//                                 &res,
//                                 &act,
//                                 &level,
//                                 &concat,
//                                 timeout
//                             );
//                             level_subresults.push(next.to_owned());
//                             concat = concat + 1;
//                         } else {
//                             println!("CENTRAL CASE");
//                             let next = generate_and_solve(
//                                 &Case::Central,
//                                 &inh,
//                                 &prob,
//                                 &res,
//                                 &act,
//                                 &level,
//                                 &concat,
//                                 timeout
//                             );
//                             level_subresults.push(next.to_owned());
//                             match next.trace.last() {
//                                 Some(x) => inh = x.sink.clone(),
//                                 None => panic!("Error cb10dd80-f6dd-4ae1-9119-116d8ba09dfa: No tail in the plan.")
//                             }
//                             concat = concat + 1;
//                         }
//                     }
//                     let level_result = concatenate(&level_subresults);
//                     // for ls in level_subresults {
//                     //     pprint_result(&ls.result)
//                     // }
//                     println!("CONCATENATED");
//                     pprint_result(&level_result);
//                     recursive(&level_result, &prob, &act, &level, timeout);
//                 } else {
//                     println!("ZEROLENGTH");
//                     let act = activate_next(&params);
//                     let next = generate_and_solve(
//                         &Case::Zerolength,
//                         &inh,
//                         &prob,
//                         &res,
//                         &act,
//                         &level,
//                         &concat,
//                         timeout
//                     );
//                     level_subresults.push(next.to_owned());
//                 }
//                 final_result = recursive(&res, &prob, &act, &level, timeout);
//             }
//         }
//         final_result
//     }
//     return_result
// }

pub fn compositional(prob: &ParamPlanningProblem, params: &Vec<Parameter>, timeout: u64, max_steps: u64) -> PlanningResult {
    let return_result = match params.iter().all(|x| !x.value) {
        true => {
            let first_params = activate_next(params);
            let first_result = parameterized(&prob, &first_params, timeout, max_steps);
            recursive_subfn(&first_result, &prob, &params, 0, timeout, max_steps)
        }
        false => {
            let first_result = parameterized(&prob, &params, timeout, max_steps);
            recursive_subfn(&first_result, &prob, &params, 0, timeout, max_steps)
        }
    };

    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        params: &Vec<Parameter>,
        level: u64,
        timeout: u64,
        max_steps: u64
    ) -> PlanningResult {
        let level = level + 1;
        let mut final_result: PlanningResult = result.to_owned();
        if !params.iter().all(|x| x.value) {
            if result.plan_found {
                let mut inheritance = State::empty();
                let mut level_subresults = vec![];
                let activated_params = activate_next(&params);
                let mut concat: u32 = 0;
                if result.plan_length != 0 {
                    for i in 0..=result.trace.len() - 1 {
                        if i == 0 {
                            let next_prob = ParamPlanningProblem::new(
                                &format!("problem_l{:?}_c{:?}", level, concat),
                                &prob.init,
                                &state_to_param_predicate(&result.trace[i + 1].source),
                                &prob.trans,
                                &prob.invars,
                                &prob.params
                            );
                            let next_result = parameterized(&next_prob, &activated_params, timeout, max_steps);
                            if next_result.plan_found {
                                level_subresults.push(next_result.to_owned());
                                match next_result.trace.last() {
                                    Some(x) => inheritance = x.sink.clone(),
                                    None => panic!("Error cb10dd80-f6dd-4ae1-9119-116d8ba09dfa: No tail in the plan.")
                                }
                            } else {
                                panic!("Error 66a7001a-67f1-4876-9928-b90b6aa55936: No plan found.")
                            }
                            concat = concat + 1;
                        } else if i == result.trace.len() - 1 {
                            let next_prob = ParamPlanningProblem::new(
                                &format!("problem_l{:?}_c{:?}", level, concat),
                                &state_to_param_predicate(&inheritance),
                                &prob.goal,
                                &prob.trans,
                                &prob.invars,
                                &prob.params
                            );
                            let next_result = parameterized(&next_prob, &activated_params, timeout, max_steps);
                            if next_result.plan_found {
                                level_subresults.push(next_result.clone());
                            } else {
                                panic!("Error b22dd6ed-cded-4424-89d6-b828c62aa0a1: No plan found.")
                            }
                            concat = concat + 1;
                        } else {
                            let next_prob = ParamPlanningProblem::new(
                                &format!("problem_l{:?}_c{:?}", level, concat),
                                &state_to_param_predicate(&inheritance),
                                &state_to_param_predicate(&result.trace[i + 1].source),
                                &prob.trans,
                                &prob.invars,
                                &prob.params
                            );
                            let next_result = parameterized(&next_prob, &activated_params, timeout, max_steps);
                            if next_result.plan_found {
                                level_subresults.push(next_result.to_owned());
                                match next_result.trace.last() {
                                    Some(x) => inheritance = x.sink.clone(),
                                    None => panic!("Error cb10dd80-f6dd-4ae1-9119-116d8ba09dfa: No tail in the plan.")
                                }
                            } else {
                                panic!("Error 66a7001a-67f1-4876-9928-b90b6aa55936: No plan found.")
                            }
                            concat = concat + 1;
                        }
                    }
                } else {
                    // have to investigate this step more... now it feels like a hack
                    let activated_params = activate_next(&params);
                    let next_prob = ParamPlanningProblem::new(
                        &format!("problem_l{:?}_c{:?}", level, concat),
                        &prob.init,
                        &prob.goal,
                        &prob.trans,
                        &prob.invars,
                        &prob.params
                    );
                    let next_result = parameterized(&next_prob, &activated_params, timeout, max_steps);
                    if next_result.plan_found {
                        level_subresults.push(next_result.to_owned());
                    } else {
                        panic!("Error 6e797cad-58f4-423d-8837-10521a986cfb: No plan found.")
                    }
                }
                let level_result = concatenate(&level_subresults);
                final_result = recursive_subfn(&level_result, &prob, &activated_params, level, timeout, max_steps);
            }
        }
        final_result
    }
    return_result
}
