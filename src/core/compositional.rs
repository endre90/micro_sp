// use super::*;
// use std::time::Instant;
// use z3_sys::*;
// use z3_v2::*;

// pub enum Case {
//     First,
//     Central,
//     Last,
// }

// /// Given a parameter list, return it with the next value activated.
// pub fn activate_next(params: &Vec<Parameter>) -> Vec<Parameter> {
//     let mut one_activated = false;
//     params
//         .iter()
//         .map(|x| {
//             if !x.value && !one_activated {
//                 one_activated = true;
//                 Parameter::new(x.name.as_str(), &true)
//             } else {
//                 x.to_owned().to_owned()
//             }
//         })
//         .collect()
// }

// /// Given a parameter list, return it with all values deactivated.
// pub fn deactivate_all(params: &Vec<Parameter>) -> Vec<Parameter> {
//     params
//         .iter()
//         .map(|x| Parameter::new(&x.name, &false))
//         .collect()
// }

// /// Given a parameterized planning problem, return it with the next value activated.
// pub fn activate_next_in_problem(prob: &ParamPlanningProblem) -> ParamPlanningProblem {
//     ParamPlanningProblem {
//         name: prob.name.to_owned(),
//         init: prob.init.to_owned(),
//         goal: prob.goal.to_owned(),
//         trans: prob.trans.to_owned(),
//         invars: prob.invars.to_owned(),
//         max_steps: prob.max_steps.to_owned(),
//         // params: activate_next(&prob.params),
//     }
// }

// /// Given a parameterized planning problem, return it with the next value activated.
// pub fn deactivate_all_in_problem(prob: &ParamPlanningProblem) -> ParamPlanningProblem {
//     ParamPlanningProblem {
//         name: prob.name.to_owned(),
//         init: prob.init.to_owned(),
//         goal: prob.goal.to_owned(),
//         trans: prob.trans.to_owned(),
//         invars: prob.invars.to_owned(),
//         max_steps: prob.max_steps.to_owned(),
//         // params: prob
//         //     .params
//         //     .iter()
//         //     .map(|x| Parameter {
//         //         name: x.name.to_owned(),
//         //         value: false,
//         //     })
//         //     .collect::<Vec<Parameter>>(),
//     }
// }

// /// Generate and solve the concat-th problem of a result
// pub fn generate_and_solve(
//     case: &Case,
//     inh: &CompleteState,
//     prob: &ParamPlanningProblem,
//     res: &ParamPlanningResult,
//     params: &Vec<Parameter>,
//     level: &u32,
//     concat: &u32,
// ) -> ParamPlanningResult {
//     println!("GENANDSOLVE_PARAMS: {:?}", params);
//     let res = parameterized(
//         &ParamPlanningProblem {
//             name: format!("problem_l{:?}_c{:?}", level, concat),
//             init: match case {
//                 Case::First => prob.init.to_owned(),
//                 Case::Central => complete_state_to_param_predicate(&inh),
//                 Case::Last => complete_state_to_param_predicate(&inh),
//             },
//             goal: match case {
//                 Case::First => complete_state_to_param_predicate(
//                     &res.result.trace[*concat as usize + 1].source,
//                 ),
//                 Case::Central => complete_state_to_param_predicate(
//                     &res.result.trace[*concat as usize + 1].source,
//                 ),
//                 Case::Last => prob.goal.to_owned(),
//             },
//             trans: prob.trans.to_owned(),
//             invars: prob.invars.to_owned(),
//             max_steps: prob.max_steps,
//         },
//         &params,
//         // &level,
//         // &concat,
//     );
//     match res.result.plan_found {
//         true => {
//             println!("SUBPLAN");
//             pprint_result(&res.result);
//             res
//         }
//         // Maybe handle this differently, like return an empty plan
//         false => panic!("Error 66a7001a-67f1-4876-9928-b90b6aa55936: No plan found."),
//     }
// }

// /// Concatenate all results in a level.
// pub fn concatenate(results: &Vec<ParamPlanningResult>) -> ParamPlanningResult {
//     ParamPlanningResult {
//         result: PlanningResult {
//             plan_found: results.iter().all(|x| x.result.plan_found),
//             plan_length: results.iter().map(|x| x.result.plan_length).sum(),
//             trace: results
//                 .iter()
//                 .map(|x| x.result.trace.to_owned())
//                 .flatten()
//                 .collect(),
//             time_to_solve: results.iter().map(|x| x.result.time_to_solve).sum(),
//         },
//         // level: results[0].level,
//         // concat: 123456789,
//     }
// }

// pub fn compositional(prob: &ParamPlanningProblem, params: &Vec<Parameter>) -> PlanningResult {
//     let one = activate_next(&deactivate_all(&params));
//     let return_result = recursive(&parameterized(&prob, &one), &prob, &one, &0);

//     fn recursive(
//         res: &ParamPlanningResult,
//         prob: &ParamPlanningProblem,
//         params: &Vec<Parameter>,
//         level: &u32,
//     ) -> ParamPlanningResult {
//         let level = level + 1;
//         println!("LEVEL: {:?}", level);
//         println!("PARAMS: {:?}", params);
//         let mut final_result: ParamPlanningResult = res.to_owned();
//         if !params.iter().all(|x| x.value) {
//             if res.result.plan_found {
//                 let mut level_subresults = vec![];
//                 let mut inh = CompleteState::empty();
//                 let mut concat: u32 = 0;
//                 if res.result.plan_length != 0 {
//                     let act = activate_next(&params);
//                     for i in 0..=res.result.trace.len() - 1 {
//                         println!("CONCAT: {:?}", concat);
//                         if i == 0 {
//                             let next = generate_and_solve(&Case::First, &inh, &prob, &res, &act, &level, &concat);
//                             level_subresults.push(next.to_owned());
//                             match next.result.trace.last() {
//                                 Some(x) => inh = x.sink.clone(),
//                                 None => panic!("Error cb10dd80-f6dd-4ae1-9119-116d8ba09dfa: No tail in the plan.")
//                             }
//                             concat = concat + 1;
//                         } else if i == res.result.trace.len() - 1 {
//                             let next = generate_and_solve(&Case::Last, &inh, &prob, &res, &act, &level, &concat);
//                             level_subresults.push(next.to_owned());
//                             concat = concat + 1;
//                         } else {
//                             let next = generate_and_solve(&Case::Central, &inh, &prob, &res, &act, &level, &concat);
//                             level_subresults.push(next.to_owned());
//                             match next.result.trace.last() {
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
//                     pprint_result(&level_result.result);
//                     final_result = recursive(&level_result, &prob, &act, &level);
//                 }
//             }
//         }
//         final_result
//     }
//     return_result.result
// }
