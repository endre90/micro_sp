// the idea here is to try to generate transitions that can correct the model
// by replacing faulty transitions. using a set of provided valid init states
// and goal predicates, and the model which contains errors, we try to solve
// each problem for the init/goal combination. if a plan is found, we save the
// used transitions in it as "valid" in a list. in step 2, we have just returned
// the difference of all transitions and these "valid" transitions so that we
// can indicate to which transitions could be faulty.

// in step 3, the idea is to generate transitions which can be used together with
// the previously annotated "valid" transitions to search for a plan. First of all,
// we should test if the generated transition(s) can help solve the unsuccessfull
// init/goal combination, and later test for all combinations.
// Do it stepwise actually, for each provided init/goal combination generate hints
// and from there select which ones are ok and not ok, do so for each init/goal combination.
// Save the ok ones in the taken transitions list and save the not ok one in the disabled
// transition lists so that they are disable for the future. Do so until the model is comlete?

// in step 4, we solve for all previously valid combinations. if we have managed to get
// the correct transition for the specific init/goal combination, it is time to test it
// together with the other init/goal combinations. we do this iteratively, since it could be
// that we generated a correct transition for the current set of init/goal combinations but
// that that transition is not correct for a future set, so we might have to re move it.
// so, after every iteration, we have to try to solve everything again and make new "valid"
// transitions lists. we do this until we have unsuccessfull init/goal combinations
// we do this iteratively,
// i.e. we reinforce (correct) the model until we can solve all init/goal combinations.
// we have to find the common treansitions which work for every init/goal combination.

// new insights into step 3:
// use step 2 until the set of untriggered transitions stops shrinking
// or until it seems boring to continue shringking?
// then call this step to try to generate the smallest ammount of transitions
// that will satisfy all the valid initial/goal combinations

// first try to generate one transition that satisfies the valid initial/goal combinations
// if it fails to do so for all combinations, discard it and try to generate two different transitions that try to do that
// keep a vector of tried and/or taken and/or failed (discarded) transitions so that different ones are generated next time

// we might have to manually say something like: no, 1 transition is bad, I want 2, now I want 3 and so on...

// don't try to replace any "faulty" transitions, rather add the generated ones into the model,
// but keep removing the generated ones in the next iterations

// on one side show these transitions, and on the other side show the names of transitions that were not taken

use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

use crate::{
    get_model_vars, simple_transition_planner, Action, Predicate, SPCommon, SPVariable, State,
    Transition,
};

pub fn generate_random_transition(name: &str, vars: &Vec<SPVariable>) -> Option<Transition> {
    let mut guard_vec = vec![];
    let mut action_vec = vec![];
    vars.iter()
        .for_each(|v| match v.domain.choose(&mut rand::thread_rng()) {
            Some(random_value) => {
                guard_vec.push((v.clone(), SPCommon::SPValue(random_value.clone())));
            }
            None => panic!("Variable {:?} has no domain?", v.name),
        });
    vars.iter()
        .for_each(|v| match v.domain.choose(&mut rand::thread_rng()) {
            Some(random_value) => {
                action_vec.push((v.clone(), SPCommon::SPValue(random_value.clone())));
            }
            None => panic!("Variable {:?} has no domain?", v.name),
        });
    if guard_vec != action_vec {
        let guard = Predicate::AND(
            guard_vec
                .iter()
                .map(|(var, val)| {
                    Predicate::EQ(SPCommon::SPVariable(var.to_owned()), val.to_owned())
                })
                .collect(),
        );
        let actions = action_vec
            .iter()
            .map(|(var, val)| Action {
                var: var.to_owned(),
                common: val.to_owned(),
            })
            .collect();
        Some(Transition::new(name, guard, actions))
    } else {
        None
    }
}

pub fn step_3(
    valid_combinations: Vec<(State, Predicate)>,
    model: Vec<Transition>,
    max_plan_lenght: usize,
    increment_tries: usize
) -> Vec<Transition> {
    let mut tried_transitions = model.clone();
    let vars = get_model_vars(&model);
    
    loop {
        let new_trans = generate_random_transition("FIX", &vars);
        match new_trans {
            Some(t) => {
                if !tried_transitions.contains(&t) {
                    tried_transitions.push(t);
                    'inner: for (init, goal) in valid_combinations {
                        let result = simple_transition_planner(
                            init,
                            goal,
                            tried_transitions.clone(),
                            max_plan_lenght,
                        );
                        if !result.found {
                            break 'inner;
                        }
                    }
                }
            }
            None => (),
        }
    }

    let all_transitions = model
        .iter()
        .map(|t| (t.name.clone(), t.clone()))
        .collect::<HashMap<String, Transition>>();
    let mut taken_transitions = HashSet::new();
    for comb in valid_combinations {
        let result = simple_transition_planner(comb.0, comb.1, model.clone(), max_plan_lenght);
        match &result.found {
            true => result.trace.iter().for_each(|t| {
                taken_transitions.insert(t.clone());
            }),
            false => (),
        }
    }
    let not_taken_transitions = all_transitions
        .difference(&taken_transitions)
        .map(|x| x.to_owned())
        .collect::<HashSet<String>>();
    let mut to_return = not_taken_transitions
        .iter()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    to_return.sort();
    to_return
}
