use super::*;
use std::time::Duration;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

pub fn skipping(
    prob: &PlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {

    let n = 10;

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

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());
        step = step + n;
        // step = match step {
        //     0 => step + 1,
        //     _ => step + 3
        // };
        match SlvCheckZ3::new(&ctx, &slv) == 1 {
            false => {
                SlvPopZ3::new(&ctx, &slv, 1);
                for s in (step-n)+1..=step {
                    if now.elapsed() > Duration::from_secs(timeout) {
                        break;
                    }
                    println!("s: {:?}", s);
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
                                    format!("{}_t{}_s{}", &x.name, s, s).as_str(),
                                ),
                                BoolZ3::new(&ctx, true),
                            );

                            trans_name_assignments.push(trans_assign);

                            ANDZ3::new(
                                &ctx,
                                vec![
                                    trans_assign,
                                    predicate_to_ast(&ctx, &x.guard, s - 1),
                                    predicate_to_ast(&ctx, &x.update, s),
                                    keep_variable_values(&ctx, &get_problem_vars(&prob), &x, s),
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
                                PBEQZ3::new(&ctx, trans_name_assignments.clone(), 1),
                            ],
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
            "skipping",
            step - (n-1),
            planning_time,
            plan_found,
            ModelSizeZ3::new(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            FreshModelZ3::new(&ctx),
            "skipping",
            step - (n-1),
            planning_time,
            plan_found,
            ModelSizeZ3::new(),
        ),
    }
}