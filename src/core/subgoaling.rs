use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

pub fn subgoaling(
    prob: &ParamPlanningProblem,
    alg: &str,
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

    let first_result = match alg {
        "seq" => sequential(&unparam(&first_subgoal), timeout, tries),
        "inc" => incremental(&unparam(&first_subgoal), timeout, tries),
        "comp" => unimplemented!(),
        _ => panic!("impossible")
    };

    let mut subresults = vec![first_result.clone()];
    pprint_result(&first_result);
    println!("{:?}", subresults.len());
    let return_result =
        recursive_subfn(&first_result, &prob, 0, timeout, tries, &mut subresults);

    fn recursive_subfn(
        result: &PlanningResult,
        prob: &ParamPlanningProblem,
        i: u64,
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
            // for g in &goals {
            //     println!("GOAL {:?}", g);
            // }
            

            let init = match result.trace.len() == 0 {
                false => match &result.trace.last() {
                    Some(x) => {
                        state_to_param_predicate(&x.sink)
                        // let sink = x.sink.clone();
                        // let asdd = state_to_param_predicate(&sink);
                        // asdd
                    },
                    None => panic!("no tail in the plan"),
                },
                true => prob.init.clone(),
            };

            // for j in &init.preds {
            //     println!("INITIAL {:?}", j);
            // }
            
            let new_result = parameterized(
                &ParamPlanningProblem::new(
                    &prob.name,
                    &init,
                    &ParamPredicate::new(&goals),
                    &prob.trans,
                    &prob.invars,
                    &prob.params,
                ),
                timeout,
                tries,
            );
            pprint_result(&new_result);
            subresults.push(new_result.clone());
            // println!("{:?}", subresults.len());
            recursive_subfn(&new_result, &prob, i, timeout, tries, subresults)
        } else {
            concatenate(&subresults)
        }
    }
    return_result
}