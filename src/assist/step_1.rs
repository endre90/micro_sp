use std::{
    collections::HashMap,
    iter::Product,
    time::{Duration, Instant},
};

use rand::seq::SliceRandom;

use crate::{
    and, eq, simple_transition_planner, PlanningResult, Predicate, SPCommon, SPVariable,
    State, Transition, get_model_vars,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Step1Solution {
    pub solution: Vec<(State, State, PlanningResult)>, // init, goal, trace
    pub combination_coverage: f32,
    pub solution_coverage: f32,
    pub time: Duration,
}

pub fn step_1(
    model: Vec<Transition>,
    max_tries: usize,
    max_state_combinations: usize,
    max_solutions: usize,
    max_plan_lenght: usize,
) -> Step1Solution {
    let now = Instant::now();
    let vars = get_model_vars(&model);
    println!("{:?}", vars);
    let mut tried_init_states: Vec<State> = vec![];
    let mut tried_goal_states: Vec<State> = vec![];
    let mut nr_init_tries = 0;
    let mut nr_goal_tries = 0;
    let mut nr_init_combinations = 0;
    let mut nr_goal_combinations = 0;
    loop {
        match max_state_combinations <= nr_init_combinations || max_tries <= nr_init_tries {
            true => break,
            false => {
                let mut init_combination = HashMap::new();
                vars.iter()
                    .for_each(|v| match v.domain.choose(&mut rand::thread_rng()) {
                        Some(random_value) => {
                            init_combination.insert(v.clone(), random_value.clone());
                        }
                        None => panic!("Variable {:?} has no domain?", v.name),
                    });
                let random_init_state = State::new(&init_combination);
                match tried_init_states.contains(&random_init_state) {
                    true => (),
                    false => {
                        nr_init_combinations = nr_init_combinations + 1;
                        tried_init_states.push(random_init_state)
                    }
                }
                nr_init_tries = nr_init_tries + 1;
            }
        }
    }

    loop {
        match max_state_combinations <= nr_goal_combinations || max_tries <= nr_goal_tries {
            true => break,
            false => {
                let mut goal_combination = HashMap::new();
                vars.iter()
                    .for_each(|v| match v.domain.choose(&mut rand::thread_rng()) {
                        Some(random_value) => {
                            goal_combination.insert(v.clone(), random_value.clone());
                        }
                        None => panic!("Variable {:?} has no domain?", v.name),
                    });
                let random_goal_state = State::new(&goal_combination);
                match tried_goal_states.contains(&random_goal_state) {
                    true => (),
                    false => {
                        nr_goal_combinations = nr_goal_combinations + 1;
                        tried_goal_states.push(random_goal_state);
                    }
                }
                nr_goal_tries = nr_goal_tries + 1;
            }
        }
    }

    let mut nr_solutions = 0;
    let mut found_solutions = vec![];
    let nr = core::cmp::min(tried_init_states.len(), tried_goal_states.len());
    for x in 0..nr {
        for y in 0..nr {
            if max_solutions <= found_solutions.len() {
                break;
            }
            if tried_init_states[x] != tried_goal_states[y] {
                nr_solutions = nr_solutions + 1;
                let result = simple_transition_planner(
                    tried_init_states[x].clone(),
                    and!(tried_goal_states[y]
                        .state
                        .iter()
                        .map(|(var, val)| eq!(
                            SPCommon::SPVariable(var.clone()),
                            SPCommon::SPValue(val.clone())
                        ))
                        .collect::<Vec<Predicate>>()),
                    model.clone(),
                    max_plan_lenght,
                );
                if result.found {
                    found_solutions.push((
                        tried_init_states[x].clone(),
                        tried_goal_states[y].clone(),
                        result,
                    ));
                }
            }
        }
    }

    // calculate coverage
    let domain_sizes: Vec<usize> = vars.iter().map(|x| x.domain.len()).collect();
    // println!("domain_sizes: {:?}", domain_sizes);
    let state_space: usize = Product::product(domain_sizes.iter());
    // println!("state_space: {:?}", state_space);
    let max_coverage = state_space * state_space - state_space;
    // println!("max_coverage: {:?}", max_coverage);
    let coverage = tried_init_states.len() * tried_goal_states.len() - tried_goal_states.len();
    // println!("coverage: {:?}", coverage);
    let combination_coverage = (coverage as f32 / max_coverage as f32) * 100.0;
    let solution_coverage = (nr_solutions as f32 / max_coverage as f32) * 100.0;

    Step1Solution {
        solution: found_solutions,
        combination_coverage,
        solution_coverage,
        time: now.elapsed(),
    }
}