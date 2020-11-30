use super::*;
use std::time::Instant;
use std::time::Duration;
use z3_sys::*;
use z3_v2::*;

/// merge non-interfering transitions and append them to the transition list
/// condition: no same variables in the guard or effect
/// not efficient, try to do this smart in the uniparallel algorithm
pub fn merge_transitions(trans: &Vec<Transition>) -> Vec<Transition> {
    let mut merge_trans: Vec<Transition> = vec!();
    for t1 in trans {
        let mut mergeable_with_t1 = vec!();
        mergeable_with_t1.push(t1);
        let t1_guard_vars = get_predicate_vars(&t1.guard);
        let t1_effect_vars = get_predicate_vars(&t1.update);
        // println!("t1_guard_vars: {:?}", t1_guard_vars);
        for t2 in trans {
            if t1 != t2 {
                let t2_guard_vars = get_predicate_vars(&t2.guard);
                let t2_effect_vars = get_predicate_vars(&t2.update);
                if t1_guard_vars.clone().intersect(t2_guard_vars.clone()).len() == 0 && 
                    t1_effect_vars.clone().intersect(t2_effect_vars.clone()).len() == 0 &&
                    mergeable_with_t1.iter().all(|x| get_predicate_vars(&x.guard).intersect(t2_guard_vars.clone()).len() == 0 && get_predicate_vars(&x.update).intersect(t2_effect_vars.clone()).len() == 0)
                {
                    // println!("t2_guard_vars: {:?}", t2_guard_vars);
                    mergeable_with_t1.push(t2)
                }
            }
        }
        merge_trans.push(
            Transition::new(
                &mergeable_with_t1.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>().join("_and_"),
                &Predicate::AND(mergeable_with_t1.iter().map(|x| x.guard.to_owned()).collect()),
                &Predicate::AND(mergeable_with_t1.iter().map(|x| x.update.to_owned()).collect())
            )
        )
    }
    for m in &merge_trans {
        println!("{}", m.name)
    }
    merge_trans
}

/// The universal step semantics parallelism based on non-interfeering transitions.
pub fn uniparallel(prob: &PlanningProblem, timeout: u64, tries: u64) -> PlanningResult {

    let merged_problem = PlanningProblem::new(
        &prob.name, 
        &prob.init, 
        &prob.goal, 
        &merge_transitions(&prob.trans), 
        &prob.invars
    );

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &merged_problem.init, 0));
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &merged_problem.invars, 0));

    SlvPushZ3::new(&ctx, &slv); // create backtracking point
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &merged_problem.goal, 0));

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
                        merged_problem.trans
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

                SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &merged_problem.invars, step));
                SlvPushZ3::new(&ctx, &slv);
                SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &merged_problem.goal, step));

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
            &merged_problem,
            SlvGetModelZ3::new(&ctx, &slv),
            "uniparallel",
            step,
            planning_time,
            plan_found,
            ModelSizeZ3::new()
        ),
        false => get_planning_result(
            &ctx,
            &merged_problem,
            FreshModelZ3::new(&ctx),
            "uniparallel",
            step,
            planning_time,
            plan_found,
            ModelSizeZ3::new()
        ),
    }
}
