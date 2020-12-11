use super::*;
use std::time::Duration;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

// /// The basic sequential planning algorithm.
// pub fn sequential(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
//     let now = Instant::now();
//     let mut plan_found: bool = false;
//     let mut step: u64 = 0;

//     let mut result = PlanningResult {
//         name: prob.name.to_owned(),
//         alg: String::from("sequential"),
//         plan_found : false,
//         plan_length: 0,
//         trace: vec!(),
//         time_to_solve: Duration::from_secs(0),
//         model_size: 123456789 as u64
//     };

//     while now.elapsed() < Duration::from_secs(timeout) && step < tries {
//         println!("elapsed: {:?}", now.elapsed());
//         let cfg = ConfigZ3::new();
//         let ctx = ContextZ3::new(&cfg);
//         // let slv = SolverZ3::new(&ctx);
//         let params = ParamsZ3::new(&ctx);

//         let slv = match logic {
//             "default" => SolverZ3::new(&ctx),
//             // "smt" => SolverFromTacticZ3::new(&ctx, "smt"),
//             "QF_UF" => SolverForLogicZ3::new(&ctx, "QF_UF"),
//             "QF_FD" => SolverForLogicZ3::new(&ctx, "QF_FD"),
//             // "QF_BV" => SolverForLogicZ3::new(&ctx, "QF_BV"),
//             _ => panic!("unknown logic!")
//         };
        
//         AddUIntParamToParamsZ3::new(&ctx, params, "timeout", (timeout*1000) as u32);

//         SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, 0));
//         SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, 0));
//         SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, step));
//         for s in 0..step {
//             // println!("s: {:?}", s);
//             SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, s + 1));

//             // make a list of assignments to track transitions
//             let trans_name_assignments: Vec<Predicate> = prob
//             .trans
//             .iter()
//             .map(|x| {
//                 pass!(&new_bool_assign_e!(
//                     format!("{}_t{}", &x.name, step).as_str(),
//                     true
//                 ))
//             })
//             .collect();

//             SlvAssertZ3::new(
//                 &ctx,
//                 &slv,
//                 ORZ3::new(
//                     &ctx,
//                     prob.trans
//                         .iter()
//                         .map(|x| {
//                             ANDZ3::new(
//                                 &ctx,
//                                 vec![
//                                     EQZ3::new(
//                                         &ctx,
//                                         BoolVarZ3::new(
//                                             &ctx,
//                                             &BoolSortZ3::new(&ctx),
//                                             format!("{}_t{}_s{}", &x.name, s, s).as_str(),
//                                         ),
//                                         BoolZ3::new(&ctx, true),
//                                     ),
//                                     predicate_to_ast(
//                                         &ctx,
//                                         &Predicate::PBEQ(trans_name_assignments.clone(), 1),
//                                         s,
//                                     ),
//                                     predicate_to_ast(&ctx, &x.guard, s - 1),
//                                     predicate_to_ast(&ctx, &x.update, s),
//                                     keep_variable_values(
//                                         &ctx,
//                                         &get_problem_vars(&prob),
//                                         &x,
//                                         s,
//                                     ),
//                                 ],
//                             )
//                         })
//                         .collect(),
//                 ),
//             );
//         }
//         step = step + 1;
//         // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
//         // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
//         // for asrt in asrtvec {
//         //     println!("{}", AstToStringZ3::new(&ctx, asrt));
//         // }
//         match SlvCheckZ3::new(&ctx, &slv) == 1 {
//             false => (),
//             true => {
//                 plan_found = true;
//                 result = get_planning_result(
//                     &ctx,
//                     &prob,
//                     SlvGetModelZ3::new(&ctx, &slv),
//                     "sequential",
//                     step,
//                     now.elapsed(),
//                     plan_found,
//                     ModelSizeZ3::new()
//                 );
//                 break;
//             }
//         }
//     }
//     result
// }


/// The basic sequential planning algorithm.
pub fn sequential(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
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
        model_size: 12345678 as u64
    };

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());
        let cfg = ConfigZ3::new();
        let ctx = ContextZ3::new(&cfg);
        let slv = match logic {
            "default" => SolverZ3::new(&ctx),
            // "smt" => SolverFromTacticZ3::new(&ctx, "smt"),
            "QF_UF" => SolverForLogicZ3::new(&ctx, "QF_UF"),
            "QF_FD" => SolverForLogicZ3::new(&ctx, "QF_FD"),
            // "QF_BV" => SolverForLogicZ3::new(&ctx, "QF_BV"),
            _ => panic!("unknown logic!")
        };
        let params = ParamsZ3::new(&ctx);
        AddUIntParamToParamsZ3::new(&ctx, params, "timeout", (timeout*1000) as u32);
        SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, 0));
        SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, 0));
        SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, step));
        for s in 0..=step {

            // make a list of assignments to track transitions
            let trans_name_assignments: Vec<Predicate> = prob
            .trans
            .iter()
            .map(|x| {
                pass!(&new_bool_assign_e!(
                    format!("{}_t{}", &x.name, s).as_str(),
                    true
                ))
            })
            .collect();

            // println!("s: {:?}", s);
            SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invars, s));
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
                                            format!("{}_t{}_s{}", &x.name, s, s).as_str(),
                                        ),
                                        BoolZ3::new(&ctx, true),
                                    ),
                                    predicate_to_ast(
                                        &ctx,
                                        &Predicate::PBEQ(trans_name_assignments.clone(), 1),
                                        step,
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
        }
        step = step + 1;
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
                    "sequential",
                    step,
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