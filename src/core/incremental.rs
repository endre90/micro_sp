use super::*;
use std::time::Duration;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

// // can't think of anything smarter now currently.
// // this method merges "mergable" transitions
// pub fn stupid_preprocess(trans: &Vec<Transition>) -> Vec<Transition> {
//     for t in trans {
//         println!("OLD {:?}", t.name);
//     }
//     let mut new_trans = vec![];
//     for t1 in trans {
//         new_trans.push(t1.clone());
//         for t2 in trans {
//             if t1 != t2 {
//                 new_trans.push(Transition::new(
//                     &format!("{}_AND_{}", t1.name, t2.name),
//                     &Predicate::AND(vec![t1.guard.clone(), t2.guard.clone()]),
//                     &Predicate::AND(vec![t2.update.clone(), t1.update.clone()]),
//                 ))
//             }
//         }
//     }
//     for t in &new_trans {
//         println!("NEW {:?}", t.name);
//     }
//     new_trans
// }

// // can't think of anything smarter now currently.
// // this method merges "mergable" transitions
// pub fn a_bit_better_preprocess(trans: &Vec<Transition>) -> Vec<Transition> {
//     for t in trans {
//         println!("OLD {:?}", t.name);
//     }
//     let mut new_trans = vec![];
//     for t1 in trans {
//         let v1 = get_predicate_vars(&t1.update);
//         new_trans.push(t1.clone());
//         for t2 in trans {
//             let v2 = get_predicate_vars(&t2.update);
//             if t1 != t2 {
//                 let effect_vars_delta = IterOps::intersect(v1.clone(), v2);
//                 if effect_vars_delta.len() == 0 {
//                     new_trans.push(Transition::new(
//                         &format!("{}_AND_{}", t1.name, t2.name),
//                         &Predicate::AND(vec![t1.guard.clone(), t2.guard.clone()]),
//                         &Predicate::AND(vec![t1.update.clone(), t2.update.clone()]),
//                     ))
//                 } 
//             }
//         }
//     }
//     for t in &new_trans {
//         println!("NEW {:?}", t.name);
//     }
//     new_trans
// }

// can't think of anything smarter now currently.
// this method merges "mergable" transitions
// two transitions can be merged if both guards can hold and the effects are non interfeering
// pub fn a_bit_better_preprocess(trans: &Vec<Transition>) -> Vec<Transition> {
//     for t in trans {
//         println!("OLD {:?}", t.name);
//     }
//     let mut new_trans = vec!();
//     for t1 in trans {
//         let v1 = get_predicate_vars(&t1.update);
//         let a1 = get_predicate_assigns(&t1.update);
//         new_trans.push(t1.clone());
//         for t2 in trans {
//             if t1 != t2 {
//                 let v2 = get_predicate_vars(&t2.update);
//                 let a2 = get_predicate_assigns(&t2.update);
//                 let effect_vars_delta = IterOps::intersect(v1.clone(), v2);
//                 let effect_assigns_delta = IterOps::intersect(a1.clone(), a2);
//                 if effect_assigns_delta.len() == 0 && effect_vars_delta.len() != 0 {
//                     ()
//                 } else if effect_assigns_delta.len() == 0 && effect_vars_delta.len() == 0 {
//                     new_trans.push(
//                         Transition::new(
//                             &format!("{}_AND_{}", t1.name, t2.name),
//                             &Predicate::AND(vec!(t1.guard.clone(), t2.guard.clone())),
//                             &Predicate::AND(vec!(t1.update.clone(), t2.update.clone()))
//                         )
//                     )
//                 } else if effect_assigns_delta.len() != 0 {
//                     if effect_vars_delta.len() == 0 {
//                         panic!("impossible")
//                     } else if effect_vars_delta.len() != 0 {

//                     }
//                 }
//                 if effect_vars_delta.len() == 0 {
//                     new_trans.push(
//                         Transition::new(
//                             &format!("{}_AND_{}", t1.name, t2.name),
//                             &Predicate::AND(vec!(t1.guard.clone(), t2.guard.clone())),
//                             &Predicate::AND(vec!(t1.update.clone(), t2.update.clone()))
//                         )
//                     )
//                 } else if {

//                 }
//             }
//         }
//     }
//     for t in &new_trans {
//         println!("NEW {:?}", t);
//     }
//     new_trans
// }

/// When some varibels are updated in a transition, the other variables
/// from the problem should keep their values from the previous step.
pub fn keep_variable_values(
    ctx: &ContextZ3,
    vars: &Vec<Variable>,
    trans: &Transition,
    step: u64,
) -> Z3_ast {
    let changed = get_predicate_vars(&trans.update);
    let unchanged = IterOps::difference(vars, &changed);

    // println!("CHANGED: {:?}", changed);
    // println!("UNCHANGED: {:?}", unchanged);

    ANDZ3::new(
        &ctx,
        unchanged
            .iter()
            .map(|x| match x.value_type {
                SPValueType::Bool => {
                    let sort = BoolSortZ3::new(&ctx);
                    EQZ3::new(
                        &ctx,
                        BoolVarZ3::new(
                            &ctx,
                            &sort,
                            format!("{}_s{}", x.name.to_string(), step).as_str(),
                        ),
                        BoolVarZ3::new(
                            &ctx,
                            &sort,
                            format!("{}_s{}", x.name.to_string(), step - 1).as_str(),
                        ),
                    )
                }
                SPValueType::String => {
                    let sort = EnumSortZ3::new(
                        &ctx,
                        &x.r#type,
                        x.domain
                            .iter()
                            .map(|y| match y {
                                SPValue::Bool(_) => {
                                    panic!("can't assign boolean value to enum type variable!")
                                }
                                SPValue::String(z) => z.as_str(),
                            })
                            .collect(),
                    );
                    EQZ3::new(
                        &ctx,
                        EnumVarZ3::new(
                            &ctx,
                            sort.r,
                            format!("{}_s{}", x.name.to_string(), step).as_str(),
                        ),
                        EnumVarZ3::new(
                            &ctx,
                            sort.r,
                            format!("{}_s{}", x.name.to_string(), step - 1).as_str(),
                        ),
                    )
                }
            })
            .collect(),
    )
}

/// The incremental algorithm that calls z3 to find a plan.
///
/// Based on Gocht and Balyo's algorithm from 2017.
pub fn incremental(
    prob: &PlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let params = ParamsZ3::new(&ctx);
    let slv = match logic {
        "default" => SolverZ3::new(&ctx),
        "qffd" => SolverForLogicZ3::new(&ctx, "QF_FD"),
        _ => panic!("unknown logic!"),
    };
    AddUIntParamToParamsZ3::new(&ctx, params, "timeout", (timeout * 1000) as u32);
    SolverSetParamsZ3::new(&ctx, &slv, params);

    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, 0));
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, 0));

    SlvPushZ3::new(&ctx, &slv); // create backtracking point
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, 0));

    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;

    // let stupid_preprocess_now = Instant::now();
    // let trans = stupid_preprocess(&prob.trans);
    // println!("stupid_preprocessing: {:?}", stupid_preprocess_now.elapsed());

    // let prob_vars = get_problem_vars(&prob);

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());
        step = step + 1;
        match SlvCheckZ3::new(&ctx, &slv) == 1 {
            false => {
                SlvPopZ3::new(&ctx, &slv, 1);

                let mut trans_name_assignments: Vec<Z3_ast> = vec![];

                let trans_assignments: Vec<Z3_ast> = prob
                    .trans
                    .iter()
                    .map(|x| {
                        let trans_assign = EQZ3::new(
                            &ctx,
                            BoolVarZ3::new(
                                &ctx,
                                &BoolSortZ3::new(&ctx),
                                format!("{}_t{}_s{}", &x.name, step, step).as_str(),
                            ),
                            BoolZ3::new(&ctx, true),
                        );

                        trans_name_assignments.push(trans_assign);

                        ANDZ3::new(
                            &ctx,
                            vec![
                                trans_assign,
                                predicate_to_ast(&ctx, &x.guard, step - 1),
                                predicate_to_ast(&ctx, &x.update, step),
                                // NOTZ3::new(&ctx, ANDZ3::new(&ctx, vec!(predicate_to_ast(&ctx, &x.guard, step), predicate_to_ast(&ctx, &x.update, step)))),
                                keep_variable_values(&ctx, &get_problem_vars(&prob), &x, step),
                            ],
                        )
                    })
                    .collect();

                SlvAssertZ3::new(
                    &ctx,
                    &slv,
                    ANDZ3::new(
                        &ctx,
                        vec![
                            ORZ3::new(&ctx, trans_assignments.clone()),
                            // ORZ3::new(&ctx, prob.trans.iter().map(|x| keep_variable_values(&ctx, &prob_vars, &x, step)).collect()),
                            PBEQZ3::new(&ctx, trans_name_assignments.clone(), 1),
                        ],
                    ),
                );

                SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, step));
                SlvPushZ3::new(&ctx, &slv);
                SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, step));

                // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
                // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
                // for asrt in asrtvec {
                //     println!("{}", AstToStringZ3::new(&ctx, asrt));
                // }

            }
            true => {
                plan_found = true;
                break;
            }
        }
    }

    

    let planning_time = now.elapsed();

    match plan_found {
        true => get_planning_result(
            &ctx,
            &prob,
            SlvGetModelZ3::new(&ctx, &slv),
            "incremental",
            step,
            planning_time,
            plan_found,
            ModelSizeZ3::new(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            FreshModelZ3::new(&ctx),
            "incremental",
            step,
            planning_time,
            plan_found,
            ModelSizeZ3::new(),
        ),
    }
}


// // old incremental algorithm, slower than the new one
// pub fn incremental(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
//     let cfg = ConfigZ3::new();
//     let ctx = ContextZ3::new(&cfg);
//     let params = ParamsZ3::new(&ctx);
//     let slv = match logic {
//         "default" => SolverZ3::new(&ctx),
//         "sat" => SolverFromTacticZ3::new(&ctx, "sat"),
//         "simple_smt" => {
//             let solver = SimpleSolverZ3::new(&ctx);
//             AddBoolParamToParamsZ3::new(&ctx, params, "smt.auto_config", false);
//             AddBoolParamToParamsZ3::new(&ctx, params, "smt.mbqi", false);
//             AddBoolParamToParamsZ3::new(&ctx, params, "smt.ematching", false);
//             solver
//         },
//         "QF_UF" => SolverForLogicZ3::new(&ctx, "QF_UF"),
//         "QF_FD" => SolverForLogicZ3::new(&ctx, "QF_FD"),
//         "QF_BV" => SolverForLogicZ3::new(&ctx, "QF_BV"),
//         _ => panic!("unknown logic!")
//     };
//     AddUIntParamToParamsZ3::new(&ctx, params, "timeout", (timeout*1000) as u32);
//     SolverSetParamsZ3::new(&ctx, &slv, params);

//     SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, 0));
//     SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, 0));

//     SlvPushZ3::new(&ctx, &slv); // create backtracking point
//     SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, 0));

//     let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
//     let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
//     for asrt in asrtvec {
//         println!("{}", AstToStringZ3::new(&ctx, asrt));
//     }
//     let now = Instant::now();
//     let mut plan_found: bool = false;
//     let mut step: u64 = 0;
//     while now.elapsed() < Duration::from_secs(timeout) && step < tries {
//         println!("elapsed: {:?}", now.elapsed());
//         step = step + 1;
//         match SlvCheckZ3::new(&ctx, &slv) == 1 {
//             false => {
//                 SlvPopZ3::new(&ctx, &slv, 1);
//                 // make a list of assignments to track transitions
//                 let trans_name_assignments: Vec<Predicate> = prob
//                     .trans
//                     .iter()
//                     .map(|x| {
//                         pass!(&new_bool_assign_e!(
//                             format!("{}_t{}", &x.name, step).as_str(),
//                             true
//                         ))
//                     })
//                     .collect();
//                 SlvAssertZ3::new(
//                     &ctx,
//                     &slv,
//                     ORZ3::new(
//                         &ctx,
//                         prob.trans
//                             .iter()
//                             .map(|x| {
//                                 ANDZ3::new(
//                                     &ctx,
//                                     vec![
//                                         EQZ3::new(
//                                             &ctx,
//                                             BoolVarZ3::new(
//                                                 &ctx,
//                                                 &BoolSortZ3::new(&ctx),
//                                                 format!("{}_t{}_s{}", &x.name, step, step).as_str(),
//                                             ),
//                                             BoolZ3::new(&ctx, true),
//                                         ),
//                                         predicate_to_ast(
//                                             &ctx,
//                                             &Predicate::PBEQ(trans_name_assignments.clone(), 1),
//                                             // &Predicate::OR(trans_name_assignments.clone()),
//                                             step,
//                                         ),
//                                         predicate_to_ast(&ctx, &x.guard, step - 1),
//                                         predicate_to_ast(&ctx, &x.update, step),
//                                         keep_variable_values(
//                                             &ctx,
//                                             &get_problem_vars(&prob),
//                                             &x,
//                                             step,
//                                         ),
//                                     ],
//                                 )
//                             })
//                             .collect(),
//                     ),
//                 );
//                 SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, step));
//                 SlvPushZ3::new(&ctx, &slv);
//                 SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, step));
//                 // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
//                 // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
//                 // for asrt in asrtvec {
//                 //     println!("{}", AstToStringZ3::new(&ctx, asrt));
//                 // }
//             }
//             true => {
//                 plan_found = true;
//                 break;
//             }
//         }
//     }
//     let planning_time = now.elapsed();
//     match plan_found {
//         true => get_planning_result(
//             &ctx,
//             &prob,
//             SlvGetModelZ3::new(&ctx, &slv),
//             "incremental",
//             step,
//             planning_time,
//             plan_found,
//             ModelSizeZ3::new(),
//         ),
//         false => get_planning_result(
//             &ctx,
//             &prob,
//             FreshModelZ3::new(&ctx),
//             "incremental",
//             step,
//             planning_time,
//             plan_found,
//             ModelSizeZ3::new(),
//         ),
//     }
// }