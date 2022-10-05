use std::time::{Duration, Instant};

use crate::{
    a, and, eq, simple_transition_planner, Action, PlanningResult, Predicate, SPCommon, SPVariable,
    State, ToSPCommon, ToSPValue, Transition,
};

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Default)]
pub struct TransitionHints {
    // pub found: bool,
    pub length: usize,
    pub hints: Vec<PlanningResult>,
    pub time: Duration,
}

pub fn step_2(
    init: State,
    goal: Predicate,
    vars: Vec<SPVariable>,
    model: Vec<Transition>,
    max_tries: usize,
    max_solutions: usize,
) -> TransitionHints {
    let now = Instant::now();
    let mut solutions: Vec<PlanningResult> = vec![];
    // let mut index = 0;
    let mut tries = 0;
    let mut nr_solutions = 0;
    'main_loop: loop {
        let mut repaired_model = model.clone();
        let mut repaired_init = init.clone();
        let mut repaired_goal = goal.clone();
        for _trans in &model {
            // first remove one of the transitions to see if it causes the problem
            if repaired_model.len() == 0 {
                break 'main_loop TransitionHints {
                    length: nr_solutions,
                    hints: solutions,
                    time: now.elapsed(),
                };
            }
            repaired_model.remove(0);
            // index = index + 1;
            // enforce that the other transitions have to be taken with extra boolean variables
            let repaired_model_2: Vec<Transition> = repaired_model
                .iter()
                .map(|t| {
                    // let new_t = t.clone();
                    let added_var = SPVariable::new(
                        &t.name,
                        &crate::SPValueType::Bool,
                        &vec![true.to_spval(), false.to_spval()],
                    );
                    let new_guard = and!(
                        t.guard,
                        eq!(SPCommon::SPVariable(added_var.clone()), false.to_comval())
                    );
                    let mut new_actions = t.actions.clone();
                    new_actions.push(a!(added_var.clone(), true.to_comval()));
                    repaired_init
                        .state
                        .insert(added_var.clone(), false.to_spval());
                    repaired_goal = and!(
                        repaired_goal,
                        eq!(SPCommon::SPVariable(added_var.clone()), true.to_comval())
                    );
                    Transition::new(&t.name, new_guard, new_actions)
                })
                .collect();
            // generate a transition instead of the one that was removed using the variable domains?
            for var in &vars {
                for val1 in &var.domain {
                    for val2 in &var.domain {
                        if val1 != val2 {
                            let mut repaired_model_3 = repaired_model_2.clone();
                            repaired_model_3.push(Transition::new(
                                "FIX",
                                eq!(
                                    SPCommon::SPVariable(var.clone()),
                                    SPCommon::SPValue(val1.clone())
                                ),
                                vec![a!(var.clone(), SPCommon::SPValue(val2.clone()))],
                            ));
                            let result = simple_transition_planner(
                                repaired_init.clone(),
                                repaired_goal.clone(),
                                repaired_model_3,
                                20,
                            );
                            tries = tries + 1;
                            if result.found {
                                nr_solutions = nr_solutions + 1;
                                solutions.push(result);
                                if nr_solutions >= max_solutions {
                                    break 'main_loop TransitionHints {
                                        length: nr_solutions,
                                        hints: solutions,
                                        time: now.elapsed(),
                                    };
                                }
                            }
                            if tries > max_tries {
                                break 'main_loop TransitionHints {
                                    length: nr_solutions,
                                    hints: solutions,
                                    time: now.elapsed(),
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}
