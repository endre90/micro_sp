use super::*;
use std::time::Duration;
use std::time::Instant;
// use z3_sys::*;
use z3_v2::*;

/// Rintanens exponential on top of sequential.
pub fn seqexponential(prob: &PlanningProblem, timeout: u64, tries: u64) -> PlanningResult {
    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;

    let mut result = PlanningResult {
        name: prob.name.to_owned(),
        alg: String::from("sequential"),
        plan_found : false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(0),
        model_size: 12345689 as u64
    };

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());
        let cfg = ConfigZ3::new();
        let ctx = ContextZ3::new(&cfg);
        let params = ParamsZ3::new(&ctx);
        let slv = SolverZ3::new(&ctx);
        AddUIntParamToParamsZ3::new(&ctx, params, "timeout", (timeout*1000) as u32);
        SolverSetParamsZ3::new(&ctx, &slv, params);
        SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, 0));
        SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, 0));
        SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, step));
        for s in 0..step {
            // quick fix
            if now.elapsed() > Duration::from_secs(timeout) {
                break;
            }
            println!("s: {:?}", s);
            SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, s + 1));
            SlvAssertZ3::new(
                &ctx,
                &slv,
                ORZ3::new(
                    &ctx,
                    prob.trans
                        .iter()
                        .map(|x| {
                            ANDZ3::new(
                                &ctx,
                                vec![
                                    EQZ3::new(
                                        &ctx,
                                        BoolVarZ3::new(
                                            &ctx,
                                            &BoolSortZ3::new(&ctx),
                                            format!("{}_t{}", &x.name, s + 1).as_str(),
                                        ),
                                        BoolZ3::new(&ctx, true),
                                    ),
                                    predicate_to_ast(&ctx, &x.guard, s),
                                    predicate_to_ast(&ctx, &x.update, s + 1),
                                    keep_variable_values(
                                        &ctx,
                                        &get_problem_vars(&prob),
                                        &x,
                                        s + 1,
                                    ),
                                ],
                            )
                        })
                        .collect(),
                ),
            );
        }
        step = step*2;
        if step == 0 {
            step = step + 1
        }
        // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
        // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
        // for asrt in asrtvec {
        //     println!("{}", AstToStringZ3::new(&ctx, asrt));
        // }
        match SlvCheckZ3::new(&ctx, &slv) == 1 {
            false => (),
            true => {
                plan_found = true;
                result = get_planning_result(
                    &ctx,
                    &prob,
                    SlvGetModelZ3::new(&ctx, &slv),
                    "exponential_on_sequential",
                    step/2 + 1,
                    now.elapsed(),
                    plan_found,
                    ModelSizeZ3::new()
                );
                break;
            }
        }
    }
    result
}

/// Rintanens exponential on top of incremental.
pub fn incexponential(prob: &PlanningProblem, timeout: u64, tries: u64) -> PlanningResult {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let params = ParamsZ3::new(&ctx);
    let slv = SolverZ3::new(&ctx);
    AddUIntParamToParamsZ3::new(&ctx, params, "timeout", (timeout*1000) as u32);

    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, 0));
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, 0));

    SlvPushZ3::new(&ctx, &slv); // create backtracking point
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, 0));

    // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
    // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
    // for asrt in asrtvec {
    //     println!("{}", AstToStringZ3::new(&ctx, asrt));
    // }

    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());
        step = match step {
            0 => step + 1,
            _ => step * 2
        };
        match SlvCheckZ3::new(&ctx, &slv) == 1 {
            false => {
                SlvPopZ3::new(&ctx, &slv, 1);
                for s in (step/2)+ 1..=step {
                    // quick fix
                    if now.elapsed() > Duration::from_secs(timeout) {
                        break;
                    }
                    println!("s: {:?}", s);
                    SlvAssertZ3::new(
                        &ctx,
                        &slv,
                        ORZ3::new(
                            &ctx,
                            prob.trans
                                .iter()
                                .map(|x| {
                                    ANDZ3::new(
                                        &ctx,
                                        vec![
                                            EQZ3::new(
                                                &ctx,
                                                BoolVarZ3::new(
                                                    &ctx,
                                                    &BoolSortZ3::new(&ctx),
                                                    format!("{}_t{}", &x.name, s).as_str(),
                                                ),
                                                BoolZ3::new(&ctx, true),
                                            ),
                                            predicate_to_ast(&ctx, &x.guard, s - 1),
                                            predicate_to_ast(&ctx, &x.update, s),
                                            keep_variable_values(
                                                &ctx,
                                                &get_problem_vars(&prob),
                                                &x,
                                                s,
                                            ),
                                        ],
                                    )
                                })
                                .collect(),
                        ),
                    );
                    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, s));
                }
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
            "exponential_on_incremental",
            step/2 + 1,
            planning_time,
            plan_found,
            ModelSizeZ3::new()
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            FreshModelZ3::new(&ctx),
            "exponential_on_incremental",
            step/2 + 1,
            planning_time,
            plan_found,
            ModelSizeZ3::new()
        ),
    }
}

// /// Rintanens exponential on top of incremental with plan lenght minimization?
// pub fn minincexponential(prob: &PlanningProblem, timeout: u64, tries: u64) -> PlanningResult {
//     let cfg = ConfigZ3::new();
//     let ctx = ContextZ3::new(&cfg);
//     let slv = SolverZ3::new(&ctx);
//     let opt = OptimizerZ3::new(&ctx);

//     SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, 0));
//     SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, 0));

//     SlvPushZ3::new(&ctx, &slv); // create backtracking point
//     SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, 0));

//     // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
//     // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
//     // for asrt in asrtvec {
//     //     println!("{}", AstToStringZ3::new(&ctx, asrt));
//     // }

//     let now = Instant::now();
//     let mut plan_found: bool = false;
//     let mut step: u64 = 0;

//     while now.elapsed() < Duration::from_secs(timeout) && step < tries {
//         println!("elapsed: {:?}", now.elapsed());
//         step = match step {
//             0 => step + 1,
//             _ => step * 2
//         };
//         match SlvCheckZ3::new(&ctx, &slv) == 1 {
//             false => {
//                 SlvPopZ3::new(&ctx, &slv, 1);
//                 for s in (step/2)+ 1..=step {
//                     println!("s: {:?}", s);
//                     SlvAssertZ3::new(
//                         &ctx,
//                         &slv,
//                         ORZ3::new(
//                             &ctx,
//                             prob.trans
//                                 .iter()
//                                 .map(|x| {
//                                     ANDZ3::new(
//                                         &ctx,
//                                         vec![
//                                             EQZ3::new(
//                                                 &ctx,
//                                                 BoolVarZ3::new(
//                                                     &ctx,
//                                                     &BoolSortZ3::new(&ctx),
//                                                     format!("{}_t{}", &x.name, s).as_str(),
//                                                 ),
//                                                 BoolZ3::new(&ctx, true),
//                                             ),
//                                             predicate_to_ast(&ctx, &x.guard, s - 1),
//                                             predicate_to_ast(&ctx, &x.update, s),
//                                             keep_variable_values(
//                                                 &ctx,
//                                                 &get_problem_vars(&prob),
//                                                 &x,
//                                                 s,
//                                             ),
//                                         ],
//                                     )
//                                 })
//                                 .collect(),
//                         ),
//                     );
//                     SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, s));
//                 }
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
//             "exponential",
//             step/2 + 1,
//             planning_time,
//             plan_found,
//         ),
//         false => get_planning_result(
//             &ctx,
//             &prob,
//             FreshModelZ3::new(&ctx),
//             "exponential",
//             step/2 + 1,
//             planning_time,
//             plan_found,
//         ),
//     }
// }
