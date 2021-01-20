use super::*;
use std::time::Duration;
use std::time::Instant;
use z3_sys::*;
use micro_z3_rust::*;

/// The basic sequential planning algorithm.
pub fn sequential(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;

    let mut result = PlanningResult {
        name: prob.name.to_owned(),
        alg: String::from("sequential"),
        plan_found : plan_found,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(0),
        model_size: 12345678 as u64
    };

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());
        let cfg = new_config_z3();
        let ctx = new_context_z3(&cfg);
        let params = params_z3(&ctx);

        let slv = match logic {
            "default" => new_solver_z3(&ctx),
            "qffd" => new_solver_for_logic_z3(&ctx, "QF_FD"),
            _ => panic!("unknown logic!")
        };

        add_uint_param_z3(&ctx, &params, "timeout", (timeout*1000) as u32);

        solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.init, 0));
        solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, 0));
        solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.goal, step));

        for s in 0..=step {

            let mut trans_name_assignments: Vec<Z3_ast> = vec![];

            let trans_assignments: Vec<Z3_ast> = prob
                .trans
                .iter()
                .map(|x| {
                    let trans_assign = eq_z3(
                        &ctx,
                        &new_var_z3(
                            &ctx,
                            &new_bool_sort_z3(&ctx),
                            format!("{}_t{}_s{}", &x.name, s, s).as_str(),
                        ),
                        &new_bool_value_z3(&ctx, true),
                    );

                    trans_name_assignments.push(trans_assign);

                    and_z3(
                        &ctx,
                        &vec![
                            trans_assign,
                            predicate_to_ast(&ctx, &x.guard, s - 1),
                            predicate_to_ast(&ctx, &x.update, s),
                            keep_variable_values(&ctx, &get_problem_vars(&prob), &x, s),
                        ],
                    )
                })
                .collect();

            solver_assert_z3(
                &ctx,
                &slv,
                &and_z3(
                    &ctx,
                    &vec![
                        or_z3(&ctx, &trans_assignments),
                        pbeq_z3(&ctx, &trans_name_assignments, 1),
                    ],
                ),
            );
        }

        step = step + 1;

        match solver_check_z3(&ctx, &slv) == 1 {
            false => (),
            true => {
                plan_found = true;
                result = get_planning_result(
                    &ctx,
                    &prob,
                    &solver_get_model_z3(&ctx, &slv),
                    "sequential",
                    step,
                    now.elapsed(),
                    plan_found,
                    get_model_size_z3()
                );
                break;
            }
        }
    }
    result
}