use super::*;
use std::time::Duration;
use std::time::Instant;
use z3_sys::*;
use micro_z3_rust::*;

pub fn skipping_c3(
    prob: &PlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {

    // let n: u32 = 3;
    let n: u64 = 3;

    let cfg = new_config_z3();
    let ctx = new_context_z3(&cfg);
    let params = params_z3(&ctx);
    let slv = match logic {
        "default" => new_solver_z3(&ctx),
        "qffd" => new_solver_for_logic_z3(&ctx, "QF_FD"),
        _ => panic!("unknown logic!"),
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
    // let mut increment: u32 = 0; // quadratic

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());

        step = step + n;         // const

        // step = match step {         // exponential
        //     0 => step + 1,          // exponential
        //     _ => step * n as u64    // exponential
        // };                          // exponential

        // increment = increment + 1;       // quadratic
        // step = increment.pow(n).into();  // quadratic

        match solver_check_z3(&ctx, &slv) == 1 {
            false => {
                solver_pop_z3(&ctx, &slv, 1);
                for s in (step - n) + 1..=step {                        // const
                // for s in (increment-1).pow(n) as u64 + 1..=step {     // quadratic
                    // for s in (step/2) + 1..=step {                       // exponential
                    if now.elapsed() > Duration::from_secs(timeout) {
                        break;
                    }
                    
                    println!("s: {:?}", s);
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
                                &new_bool_value_z3(&ctx, true)
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
                                or_z3(&ctx, &trans_assignments.clone()),
                                pbeq_z3(&ctx, &trans_name_assignments.clone(), 1),
                            ],
                        ),
                    );

                    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, s));
                }
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
            "skipping",
            step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            &new_fresh_model_z3(&ctx),
            "skipping",
            step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3()
        ),
    }
}

pub fn skipping_c5(
    prob: &PlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {

    let n: u64 = 5;

    let cfg = new_config_z3();
    let ctx = new_context_z3(&cfg);
    let params = params_z3(&ctx);
    let slv = match logic {
        "default" => new_solver_z3(&ctx),
        "qffd" => new_solver_for_logic_z3(&ctx, "QF_FD"),
        _ => panic!("unknown logic!"),
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
    // let mut increment: u32 = 0; // quadratic

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());

        step = step + n;         // const

        // step = match step {         // exponential
        //     0 => step + 1,          // exponential
        //     _ => step * n as u64    // exponential
        // };                          // exponential

        // increment = increment + 1;       // quadratic
        // step = increment.pow(n).into();  // quadratic

        match solver_check_z3(&ctx, &slv) == 1 {
            false => {
                solver_pop_z3(&ctx, &slv, 1);
                for s in (step - n) + 1..=step {                        // const
                // for s in (increment-1).pow(n) as u64 + 1..=step {     // quadratic
                    // for s in (step/2) + 1..=step {                       // exponential
                    if now.elapsed() > Duration::from_secs(timeout) {
                        break;
                    }
                    
                    println!("s: {:?}", s);
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
                                &new_bool_value_z3(&ctx, true)
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
                                or_z3(&ctx, &trans_assignments.clone()),
                                pbeq_z3(&ctx, &trans_name_assignments.clone(), 1),
                            ],
                        ),
                    );

                    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, s));
                }
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
            "skipping",
            step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            &new_fresh_model_z3(&ctx),
            "skipping",
            step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3()
        ),
    }
}

pub fn skipping_c10(
    prob: &PlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {

    let n: u64 = 10;

    let cfg = new_config_z3();
    let ctx = new_context_z3(&cfg);
    let params = params_z3(&ctx);
    let slv = match logic {
        "default" => new_solver_z3(&ctx),
        "qffd" => new_solver_for_logic_z3(&ctx, "QF_FD"),
        _ => panic!("unknown logic!"),
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
    // let mut increment: u32 = 0; // quadratic

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());

        step = step + n;         // const

        // step = match step {         // exponential
        //     0 => step + 1,          // exponential
        //     _ => step * n as u64    // exponential
        // };                          // exponential

        // increment = increment + 1;       // quadratic
        // step = increment.pow(n).into();  // quadratic

        match solver_check_z3(&ctx, &slv) == 1 {
            false => {
                solver_pop_z3(&ctx, &slv, 1);
                for s in (step - n) + 1..=step {                        // const
                // for s in (increment-1).pow(n) as u64 + 1..=step {     // quadratic
                    // for s in (step/2) + 1..=step {                       // exponential
                    if now.elapsed() > Duration::from_secs(timeout) {
                        break;
                    }
                    
                    println!("s: {:?}", s);
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
                                &new_bool_value_z3(&ctx, true)
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
                                or_z3(&ctx, &trans_assignments.clone()),
                                pbeq_z3(&ctx, &trans_name_assignments.clone(), 1),
                            ],
                        ),
                    );

                    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, s));
                }
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
            "skipping",
            step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            &new_fresh_model_z3(&ctx),
            "skipping",
            step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3()
        ),
    }
}

pub fn skipping_q(
    prob: &PlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {

    let n: u32 = 2;

    let cfg = new_config_z3();
    let ctx = new_context_z3(&cfg);
    let params = params_z3(&ctx);
    let slv = match logic {
        "default" => new_solver_z3(&ctx),
        "qffd" => new_solver_for_logic_z3(&ctx, "QF_FD"),
        _ => panic!("unknown logic!"),
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
    let mut increment: u32 = 0; // quadratic

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());

        // step = step + n;         // const

        // step = match step {         // exponential
        //     0 => step + 1,          // exponential
        //     _ => step * n as u64    // exponential
        // };                          // exponential

        increment = increment + 1;       // quadratic
        step = increment.pow(n).into();  // quadratic

        match solver_check_z3(&ctx, &slv) == 1 {
            false => {
                solver_pop_z3(&ctx, &slv, 1);
                // for s in (step - n) + 1..=step {                        // const
                for s in (increment-1).pow(n) as u64 + 1..=step {     // quadratic
                    // for s in (step/2) + 1..=step {                       // exponential
                    if now.elapsed() > Duration::from_secs(timeout) {
                        break;
                    }
                    
                    println!("s: {:?}", s);
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
                                &new_bool_value_z3(&ctx, true)
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
                                or_z3(&ctx, &trans_assignments.clone()),
                                pbeq_z3(&ctx, &trans_name_assignments.clone(), 1),
                            ],
                        ),
                    );

                    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, s));
                }
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
            "skipping",
            // step - (n-1),                                                    // const
            step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            &new_fresh_model_z3(&ctx),
            "skipping",
            // step - (n-1),                                                    // const
            step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            // step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3()
        ),
    }
}

pub fn skipping_e(
    prob: &PlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {

    let n: u32 = 2;

    let cfg = new_config_z3();
    let ctx = new_context_z3(&cfg);
    let params = params_z3(&ctx);
    let slv = match logic {
        "default" => new_solver_z3(&ctx),
        "qffd" => new_solver_for_logic_z3(&ctx, "QF_FD"),
        _ => panic!("unknown logic!"),
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
    // let mut increment: u32 = 0; // quadratic

    while now.elapsed() < Duration::from_secs(timeout) && step < tries {
        println!("elapsed: {:?}", now.elapsed());

        // step = step + n;         // const

        step = match step {         // exponential
            0 => step + 1,          // exponential
            _ => step * n as u64    // exponential
        };                          // exponential

        // increment = increment + 1;       // quadratic
        // step = increment.pow(n).into();  // quadratic

        match solver_check_z3(&ctx, &slv) == 1 {
            false => {
                solver_pop_z3(&ctx, &slv, 1);
                // for s in (step - n) + 1..=step {                        // const
                // for s in (increment-1).pow(n) as u64 + 1..=step {     // quadratic
                    for s in (step/2) + 1..=step {                       // exponential
                    if now.elapsed() > Duration::from_secs(timeout) {
                        break;
                    }
                    
                    println!("s: {:?}", s);
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
                                &new_bool_value_z3(&ctx, true)
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
                                or_z3(&ctx, &trans_assignments.clone()),
                                pbeq_z3(&ctx, &trans_name_assignments.clone(), 1),
                            ],
                        ),
                    );

                    solver_assert_z3(&ctx, &slv, &predicate_to_ast(&ctx, &prob.invars, s));
                }
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
            "skipping",
            // step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3(),
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            &new_fresh_model_z3(&ctx),
            "skipping",
            // step - (n-1),                                                    // const
            // step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
            step/2 + 1,                                                         // exponential
            planning_time,
            plan_found,
            get_model_size_z3()
        ),
    }
}

pub fn seq_skipping_c3(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;
    let n: u64 = 3;

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

            println!("s: {:?}", s);
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

        step = step + n;

        match solver_check_z3(&ctx, &slv) == 1 {
            false => (),
            true => {
                plan_found = true;
                result = get_planning_result(
                    &ctx,
                    &prob,
                    &solver_get_model_z3(&ctx, &slv),
                    "skipping_on_sequential",
                    (step - n) + 1,
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


pub fn seq_skipping_c5(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;
    let n: u64 = 5;

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

            println!("s: {:?}", s);
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

        step = step + n;

        match solver_check_z3(&ctx, &slv) == 1 {
            false => (),
            true => {
                plan_found = true;
                result = get_planning_result(
                    &ctx,
                    &prob,
                    &solver_get_model_z3(&ctx, &slv),
                    "skipping_on_sequential",
                    (step - n) + 1,
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

pub fn seq_skipping_c10(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;
    let n: u64 = 10;

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

            println!("s: {:?}", s);
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

        step = step + n;

        match solver_check_z3(&ctx, &slv) == 1 {
            false => (),
            true => {
                plan_found = true;
                result = get_planning_result(
                    &ctx,
                    &prob,
                    &solver_get_model_z3(&ctx, &slv),
                    "skipping_on_sequential",
                    (step - n) + 1,
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

pub fn seq_skipping_q(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;
    let n: u32 = 2;
    let mut increment: u32 = 0; // quadratic

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

            println!("s: {:?}", s);
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

        increment = increment + 1;       // quadratic
        step = increment.pow(n).into();  // quadratic

        match solver_check_z3(&ctx, &slv) == 1 {
            false => (),
            true => {
                plan_found = true;
                result = get_planning_result(
                    &ctx,
                    &prob,
                    &solver_get_model_z3(&ctx, &slv),
                    "skipping_on_sequential",
                    step - ((increment).pow(n) - (increment-1).pow(n)) as u64 + 1,   // quadratic
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


pub fn seq_skipping_e(prob: &PlanningProblem, logic: &str, timeout: u64, tries: u64) -> PlanningResult {
    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u64 = 0;
    let n: u32 = 2;

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

            println!("s: {:?}", s);
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

        step = match step {         // exponential
            0 => step + 1,          // exponential
            _ => step * n as u64    // exponential
        };                          // exponential

        match solver_check_z3(&ctx, &slv) == 1 {
            false => (),
            true => {
                plan_found = true;
                result = get_planning_result(
                    &ctx,
                    &prob,
                    &solver_get_model_z3(&ctx, &slv),
                    "skipping_on_sequential",
                    step/2 + 1, 
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
