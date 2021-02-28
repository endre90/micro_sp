use super::*;
use std::time::Instant;
use std::time::Duration;

pub fn subgoaling(
    prob: &ParamPlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let first_subgoal = ParamPlanningProblem::new(
        &prob.name,
        &prob.init,
        &ParamPredicate::new(&vec![prob.goal.preds[0].clone()]),
        &prob.trans,
        &prob.invars,
        &prob.params,
    );

    let first_result = incremental(&unparam(&first_subgoal), logic, timeout, tries);

    let failed_result = PlanningResult{
        name: prob.name.to_owned(),
        alg: String::from("subgoaling"),
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(timeout),
        model_size: 0
    };

    let now = Instant::now();

    let mut subresults = vec![first_result.clone()];
    let return_result = match now.elapsed() < Duration::from_secs(timeout) {
        true => recursive_subfn(&first_result, &prob, 0, logic, timeout, tries, &mut subresults),
        false => failed_result
    };
        
    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
        logic: &str,
        timeout: u64,
        tries: u64,
        subresults: &mut Vec<PlanningResult>,
    ) -> PlanningResult {
        if i < prob.goal.preds.len() as u64 - 1 {
            let i = i + 1;
            let mut goals = vec![];
            for j in 0..i + 1{
                goals.push(prob.goal.preds[j as usize].clone())
            }

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };
            
            let new_result = parameterized(
                &ParamPlanningProblem::new(
                    &prob.name,
                    &init,
                    &ParamPredicate::new(&goals),
                    &prob.trans,
                    &prob.invars,
                    &prob.params,
                ),
                &logic,
                timeout,
                tries,
            );

            subresults.push(new_result.clone());
            recursive_subfn(&new_result, &prob, i, logic, timeout, tries, subresults)
            
        } else {
            concatenate(&prob.name, "subgoaling_on_incremental", &subresults)        
        }
    }
    return_result
}

pub fn subgoaling_seq_skp1(
    prob: &ParamPlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let first_subgoal = ParamPlanningProblem::new(
        &prob.name,
        &prob.init,
        &ParamPredicate::new(&vec![prob.goal.preds[0].clone()]),
        &prob.trans,
        &prob.invars,
        &prob.params,
    );

    let first_result = sequential(&unparam(&first_subgoal), logic, timeout, tries);

    let failed_result = PlanningResult{
        name: prob.name.to_owned(),
        alg: String::from("subgoaling"),
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(timeout),
        model_size: 0
    };

    let now = Instant::now();

    let mut subresults = vec![first_result.clone()];
    let return_result = match now.elapsed() < Duration::from_secs(timeout) {
        true => recursive_subfn(&first_result, &prob, 0, logic, timeout, tries, &mut subresults),
        false => failed_result
    };
        
    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
        logic: &str,
        timeout: u64,
        tries: u64,
        subresults: &mut Vec<PlanningResult>,
    ) -> PlanningResult {
        if i < prob.goal.preds.len() as u64 - 1 {
            let i = i + 1;
            let mut goals = vec![];
            for j in 0..i + 1{
                goals.push(prob.goal.preds[j as usize].clone())
            }

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };
            
            let new_result = sequential(
                &unparam(
                    &ParamPlanningProblem::new(
                        &prob.name,
                        &init,
                        &ParamPredicate::new(&goals),
                        &prob.trans,
                        &prob.invars,
                        &prob.params,
                    )
                ),
                &logic,
                timeout,
                tries,
            );

            subresults.push(new_result.clone());
            recursive_subfn(&new_result, &prob, i, logic, timeout, tries, subresults)
            
        } else {
            concatenate(&prob.name, "subgoaling_on_sequential", &subresults)        
        }
    }
    return_result
}

pub fn subgoaling_seq_skp3(
    prob: &ParamPlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let first_subgoal = ParamPlanningProblem::new(
        &prob.name,
        &prob.init,
        &ParamPredicate::new(&vec![prob.goal.preds[0].clone()]),
        &prob.trans,
        &prob.invars,
        &prob.params,
    );

    let first_result = seq_skipping_c3(&unparam(&first_subgoal), logic, timeout, tries);

    let failed_result = PlanningResult{
        name: prob.name.to_owned(),
        alg: String::from("subgoaling"),
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(timeout),
        model_size: 0
    };

    let now = Instant::now();

    let mut subresults = vec![first_result.clone()];
    let return_result = match now.elapsed() < Duration::from_secs(timeout) {
        true => recursive_subfn(&first_result, &prob, 0, logic, timeout, tries, &mut subresults),
        false => failed_result
    };
        
    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
        logic: &str,
        timeout: u64,
        tries: u64,
        subresults: &mut Vec<PlanningResult>,
    ) -> PlanningResult {
        if i < prob.goal.preds.len() as u64 - 1 {
            let i = i + 1;
            let mut goals = vec![];
            for j in 0..i + 1{
                goals.push(prob.goal.preds[j as usize].clone())
            }

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };
            
            let new_result = seq_skipping_c3(
                &unparam(
                    &ParamPlanningProblem::new(
                        &prob.name,
                        &init,
                        &ParamPredicate::new(&goals),
                        &prob.trans,
                        &prob.invars,
                        &prob.params,
                    )
                ),
                &logic,
                timeout,
                tries,
            );

            subresults.push(new_result.clone());
            recursive_subfn(&new_result, &prob, i, logic, timeout, tries, subresults)
            
        } else {
            concatenate(&prob.name, "subgoaling_on_sequential", &subresults)        
        }
    }
    return_result
}

pub fn subgoaling_seq_skp5(
    prob: &ParamPlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let first_subgoal = ParamPlanningProblem::new(
        &prob.name,
        &prob.init,
        &ParamPredicate::new(&vec![prob.goal.preds[0].clone()]),
        &prob.trans,
        &prob.invars,
        &prob.params,
    );

    let first_result = seq_skipping_c5(&unparam(&first_subgoal), logic, timeout, tries);

    let failed_result = PlanningResult{
        name: prob.name.to_owned(),
        alg: String::from("subgoaling"),
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(timeout),
        model_size: 0
    };

    let now = Instant::now();

    let mut subresults = vec![first_result.clone()];
    let return_result = match now.elapsed() < Duration::from_secs(timeout) {
        true => recursive_subfn(&first_result, &prob, 0, logic, timeout, tries, &mut subresults),
        false => failed_result
    };
        
    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
        logic: &str,
        timeout: u64,
        tries: u64,
        subresults: &mut Vec<PlanningResult>,
    ) -> PlanningResult {
        if i < prob.goal.preds.len() as u64 - 1 {
            let i = i + 1;
            let mut goals = vec![];
            for j in 0..i + 1{
                goals.push(prob.goal.preds[j as usize].clone())
            }

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };
            
            let new_result = seq_skipping_c5(
                &unparam(
                    &ParamPlanningProblem::new(
                        &prob.name,
                        &init,
                        &ParamPredicate::new(&goals),
                        &prob.trans,
                        &prob.invars,
                        &prob.params,
                    )
                ),
                &logic,
                timeout,
                tries,
            );

            subresults.push(new_result.clone());
            recursive_subfn(&new_result, &prob, i, logic, timeout, tries, subresults)
            
        } else {
            concatenate(&prob.name, "subgoaling_on_sequential", &subresults)        
        }
    }
    return_result
}

pub fn subgoaling_seq_skp10(
    prob: &ParamPlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let first_subgoal = ParamPlanningProblem::new(
        &prob.name,
        &prob.init,
        &ParamPredicate::new(&vec![prob.goal.preds[0].clone()]),
        &prob.trans,
        &prob.invars,
        &prob.params,
    );

    let first_result = seq_skipping_c10(&unparam(&first_subgoal), logic, timeout, tries);

    let failed_result = PlanningResult{
        name: prob.name.to_owned(),
        alg: String::from("subgoaling"),
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(timeout),
        model_size: 0
    };

    let now = Instant::now();

    let mut subresults = vec![first_result.clone()];
    let return_result = match now.elapsed() < Duration::from_secs(timeout) {
        true => recursive_subfn(&first_result, &prob, 0, logic, timeout, tries, &mut subresults),
        false => failed_result
    };
        
    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
        logic: &str,
        timeout: u64,
        tries: u64,
        subresults: &mut Vec<PlanningResult>,
    ) -> PlanningResult {
        if i < prob.goal.preds.len() as u64 - 1 {
            let i = i + 1;
            let mut goals = vec![];
            for j in 0..i + 1{
                goals.push(prob.goal.preds[j as usize].clone())
            }

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };
            
            let new_result = seq_skipping_c10(
                &unparam(
                    &ParamPlanningProblem::new(
                        &prob.name,
                        &init,
                        &ParamPredicate::new(&goals),
                        &prob.trans,
                        &prob.invars,
                        &prob.params,
                    )
                ),
                &logic,
                timeout,
                tries,
            );

            subresults.push(new_result.clone());
            recursive_subfn(&new_result, &prob, i, logic, timeout, tries, subresults)
            
        } else {
            concatenate(&prob.name, "subgoaling_on_sequential", &subresults)        
        }
    }
    return_result
}

pub fn subgoaling_seq_skpq(
    prob: &ParamPlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let first_subgoal = ParamPlanningProblem::new(
        &prob.name,
        &prob.init,
        &ParamPredicate::new(&vec![prob.goal.preds[0].clone()]),
        &prob.trans,
        &prob.invars,
        &prob.params,
    );

    let first_result = seq_skipping_q(&unparam(&first_subgoal), logic, timeout, tries);

    let failed_result = PlanningResult{
        name: prob.name.to_owned(),
        alg: String::from("subgoaling"),
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(timeout),
        model_size: 0
    };

    let now = Instant::now();

    let mut subresults = vec![first_result.clone()];
    let return_result = match now.elapsed() < Duration::from_secs(timeout) {
        true => recursive_subfn(&first_result, &prob, 0, logic, timeout, tries, &mut subresults),
        false => failed_result
    };
        
    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
        logic: &str,
        timeout: u64,
        tries: u64,
        subresults: &mut Vec<PlanningResult>,
    ) -> PlanningResult {
        if i < prob.goal.preds.len() as u64 - 1 {
            let i = i + 1;
            let mut goals = vec![];
            for j in 0..i + 1{
                goals.push(prob.goal.preds[j as usize].clone())
            }

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };
            
            let new_result = seq_skipping_q(
                &unparam(
                    &ParamPlanningProblem::new(
                        &prob.name,
                        &init,
                        &ParamPredicate::new(&goals),
                        &prob.trans,
                        &prob.invars,
                        &prob.params,
                    )
                ),
                &logic,
                timeout,
                tries,
            );

            subresults.push(new_result.clone());
            recursive_subfn(&new_result, &prob, i, logic, timeout, tries, subresults)
            
        } else {
            concatenate(&prob.name, "subgoaling_on_sequential", &subresults)        
        }
    }
    return_result
}

pub fn subgoaling_seq_skpe(
    prob: &ParamPlanningProblem,
    logic: &str,
    timeout: u64,
    tries: u64,
) -> PlanningResult {
    let first_subgoal = ParamPlanningProblem::new(
        &prob.name,
        &prob.init,
        &ParamPredicate::new(&vec![prob.goal.preds[0].clone()]),
        &prob.trans,
        &prob.invars,
        &prob.params,
    );

    let first_result = seq_skipping_e(&unparam(&first_subgoal), logic, timeout, tries);

    let failed_result = PlanningResult{
        name: prob.name.to_owned(),
        alg: String::from("subgoaling"),
        plan_found: false,
        plan_length: 0,
        trace: vec!(),
        time_to_solve: Duration::from_secs(timeout),
        model_size: 0
    };

    let now = Instant::now();

    let mut subresults = vec![first_result.clone()];
    let return_result = match now.elapsed() < Duration::from_secs(timeout) {
        true => recursive_subfn(&first_result, &prob, 0, logic, timeout, tries, &mut subresults),
        false => failed_result
    };
        
    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
        logic: &str,
        timeout: u64,
        tries: u64,
        subresults: &mut Vec<PlanningResult>,
    ) -> PlanningResult {
        if i < prob.goal.preds.len() as u64 - 1 {
            let i = i + 1;
            let mut goals = vec![];
            for j in 0..i + 1{
                goals.push(prob.goal.preds[j as usize].clone())
            }

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };
            
            let new_result = seq_skipping_e(
                &unparam(
                    &ParamPlanningProblem::new(
                        &prob.name,
                        &init,
                        &ParamPredicate::new(&goals),
                        &prob.trans,
                        &prob.invars,
                        &prob.params,
                    )
                ),
                &logic,
                timeout,
                tries,
            );

            subresults.push(new_result.clone());
            recursive_subfn(&new_result, &prob, i, logic, timeout, tries, subresults)
            
        } else {
            concatenate(&prob.name, "subgoaling_on_sequential", &subresults)        
        }
    }
    return_result
}