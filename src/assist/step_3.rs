use std::collections::HashSet;

use crate::{simple_transition_planner, Predicate, State, Transition};

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

pub fn step_3(
    valid_combinations: Vec<(State, Predicate)>,
    model: Vec<Transition>,
    max_plan_lenght: usize,
) -> HashSet<Transition> {
    let all_transitions = model
        .iter()
        .map(|t| t.name.clone())
        .collect::<HashSet<String>>();
    let mut taken_transitions = HashSet::new();
    let mut unsuccessful_combinations = vec!();
    for comb in valid_combinations {
        let result = simple_transition_planner(comb.0, comb.1, model.clone(), max_plan_lenght);
        match &result.found {
            true => result.trace.iter().for_each(|t| {
                taken_transitions.insert(t.clone());
            }),
            false => unsuccessful_combinations.push(comb),
        }
    }
    let not_taken_transitions = all_transitions
        .difference(&taken_transitions)
        .map(|x| x.to_owned())
        .collect::<HashSet<String>>();

    // now the idea is to distil transition hints from these cores:
    // not_taken_transitions and unsuccessful_combinations, let's see...

    for uc in unsuccessful_combinations {

    }



    not_taken_transitions
}



// #[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Default)]
// pub struct TransitionHints {
//     // pub found: bool,
//     pub length: usize,
//     pub hints: Vec<PlanningResult>,
//     pub time: Duration,
// }

// old step 2
// pub fn step_2(
//     init: State,
//     goal: Predicate,
//     vars: Vec<SPVariable>,
//     model: Vec<Transition>,
//     max_tries: usize,
//     max_solutions: usize,
// ) -> TransitionHints {
//     let now = Instant::now();
//     let mut solutions: Vec<PlanningResult> = vec![];
//     // let mut index = 0;
//     let mut tries = 0;
//     let mut nr_solutions = 0;
//     'main_loop: loop {
//         let mut repaired_model = model.clone();
//         let mut repaired_init = init.clone();
//         let mut repaired_goal = goal.clone();
//         for _trans in &model {
//             // first remove one of the transitions to see if it causes the problem
//             if repaired_model.len() == 0 {
//                 break 'main_loop TransitionHints {
//                     length: nr_solutions,
//                     hints: solutions,
//                     time: now.elapsed(),
//                 };
//             }
//             repaired_model.remove(0);
//             // index = index + 1;
//             // enforce that the other transitions have to be taken with extra boolean variables
//             let repaired_model_2: Vec<Transition> = repaired_model
//                 .iter()
//                 .map(|t| {
//                     // let new_t = t.clone();
//                     let added_var = SPVariable::new(
//                         &t.name,
//                         &crate::SPValueType::Bool,
//                         &vec![true.to_spval(), false.to_spval()],
//                     );
//                     let new_guard = and!(
//                         t.guard,
//                         eq!(SPCommon::SPVariable(added_var.clone()), false.to_comval())
//                     );
//                     let mut new_actions = t.actions.clone();
//                     new_actions.push(a!(added_var.clone(), true.to_comval()));
//                     repaired_init
//                         .state
//                         .insert(added_var.clone(), false.to_spval());
//                     repaired_goal = and!(
//                         repaired_goal,
//                         eq!(SPCommon::SPVariable(added_var.clone()), true.to_comval())
//                     );
//                     Transition::new(&t.name, new_guard, new_actions)
//                 })
//                 .collect();
//             // generate a transition instead of the one that was removed using the variable domains?
//             for var in &vars {
//                 for val1 in &var.domain {
//                     for val2 in &var.domain {
//                         if val1 != val2 {
//                             let mut repaired_model_3 = repaired_model_2.clone();
//                             repaired_model_3.push(Transition::new(
//                                 "FIX",
//                                 eq!(
//                                     SPCommon::SPVariable(var.clone()),
//                                     SPCommon::SPValue(val1.clone())
//                                 ),
//                                 vec![a!(var.clone(), SPCommon::SPValue(val2.clone()))],
//                             ));
//                             let result = simple_transition_planner(
//                                 repaired_init.clone(),
//                                 repaired_goal.clone(),
//                                 repaired_model_3,
//                                 20,
//                             );
//                             tries = tries + 1;
//                             if result.found {
//                                 nr_solutions = nr_solutions + 1;
//                                 solutions.push(result);
//                                 if nr_solutions >= max_solutions {
//                                     break 'main_loop TransitionHints {
//                                         length: nr_solutions,
//                                         hints: solutions,
//                                         time: now.elapsed(),
//                                     };
//                                 }
//                             }
//                             if tries > max_tries {
//                                 break 'main_loop TransitionHints {
//                                     length: nr_solutions,
//                                     hints: solutions,
//                                     time: now.elapsed(),
//                                 };
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
