use super::*;
use std::time::Duration;
use std::time::Instant;
use z3_sys::*;
use micro_z3_rust::*;

/// When some varibels are updated in a transition, the other variables
/// from the problem should keep their values from the previous step.
pub fn keep_variable_values(
    ctx: &Z3_context,
    vars: &Vec<Variable>,
    trans: &Transition,
    step: u64,
) -> Z3_ast {
    let changed = get_predicate_vars(&trans.update);
    let unchanged = IterOps::difference(vars, &changed);

    // println!("CHANGED: {:?}", changed);
    // println!("UNCHANGED: {:?}", unchanged);

    and_z3(
        &ctx,
        &unchanged
            .iter()
            .map(|x| match x.value_type {
                SPValueType::Bool => {
                    let bool_sort = new_bool_sort_z3(&ctx);
                    eq_z3(
                        &ctx,
                        &new_var_z3(
                            &ctx,
                            &bool_sort,
                            format!("{}_s{}", x.name.to_string(), step).as_str(),
                        ),
                        &new_var_z3(
                            &ctx,
                            &bool_sort,
                            format!("{}_s{}", x.name.to_string(), step - 1).as_str(),
                        )
                    )
                }
                SPValueType::String => {
                    let enum_sort = new_enum_sort_z3(
                        &ctx,
                        &x.r#type,
                        &x.domain
                            .iter()
                            .map(|y| match y {
                                SPValue::Bool(_) => {
                                    panic!("can't assign boolean value to enum type variable!")
                                }
                                SPValue::String(z) => z.as_str(),
                            })
                            .collect(),
                    );
                    eq_z3(
                        &ctx,
                        &new_var_z3(
                            &ctx,
                            &enum_sort.0,
                            format!("{}_s{}", x.name.to_string(), step).as_str(),
                        ),
                        &new_var_z3(
                            &ctx,
                            &enum_sort.0,
                            format!("{}_s{}", x.name.to_string(), step - 1).as_str(),
                        )
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
    let cfg = new_config_z3();
    let ctx = new_context_z3(&cfg);
    let params = params_z3(&ctx);
    let slv = match logic {
        "default" => new_solver_z3(&ctx),
        "qffd" => new_solver_for_logic_z3(&ctx, "QF_FD"),
        _ => panic!("unknown logic!")
    };
    add_uint_param_z3(&ctx, &params, "timeout", (timeout * 1000) as u32);
    solver_set_params_z3(&ctx, &slv, &params);

    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.init, 0));
    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, 0));

    solver_push_z3(&ctx, &slv); // create backtracking point
    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.goal, 0));

    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());
        step = step + 1;
        match solver_check_z3(&ctx, &slv) == 1 {
            false => {
                solver_pop_z3(&ctx, &slv, 1);

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
                                format!("{}_t{}_s{}", &x.name, step, step).as_str(),
                            ),
                            &new_bool_value_z3(&ctx, true)
                        );

                        trans_name_assignments.push(trans_assign);

                        and_z3(
                            &ctx,
                            &vec![
                                trans_assign,
                                predicate_to_ast(&ctx, &x.guard, step - 1),
                                predicate_to_ast(&ctx, &x.update, step),
                                keep_variable_values(&ctx, &get_problem_vars(&prob), &x, step),
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

                solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, step));
                solver_push_z3(&ctx, &slv);
                solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.goal, step));

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
            &solver_get_model_z3(&ctx, &slv),
            "incremental",
            step,
            planning_time,
            plan_found,
            get_model_size_z3(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            &new_fresh_model_z3(&ctx),
            "incremental",
            step,
            planning_time,
            plan_found,
            get_model_size_z3()
        ),
    }
}