use super::*;
use std::time::Instant;
use std::time::Duration;
use z3_sys::*;
use z3_v2::*;

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
pub fn incremental(prob: &PlanningProblem, timeout: u64, tries: u64) -> PlanningResult {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

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
        step = step + 1;
        match SlvCheckZ3::new(&ctx, &slv) == 1 {
            false => {
                SlvPopZ3::new(&ctx, &slv, 1);
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
                                                format!("{}_t{}", &x.name, step).as_str(),
                                            ),
                                            BoolZ3::new(&ctx, true),
                                        ),
                                        predicate_to_ast(&ctx, &x.guard, step - 1),
                                        predicate_to_ast(&ctx, &x.update, step),
                                        keep_variable_values(
                                            &ctx,
                                            &get_problem_vars(&prob),
                                            &x,
                                            step,
                                        ),
                                    ],
                                )
                            })
                            .collect(),
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
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            FreshModelZ3::new(&ctx),
            "incremental",
            step,
            planning_time,
            plan_found,
        ),
    }
}
