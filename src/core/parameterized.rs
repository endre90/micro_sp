use super::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;



/// This helps with adding and removing predicates to a conjunction
/// before sending the problem to the incremental algorithm.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct ParamPredicate {
    pub preds: Vec<Predicate>,
}

impl ParamPredicate {
    /// Make a new parameterized predicate that is basically a vector of
    /// predicates that are marked with parameters. These parameters will
    /// turn the predicate on/off when it is generated for the incremental
    /// algorithm based on the value of the parameter in the current level.
    pub fn new(preds: &Vec<Predicate>) -> ParamPredicate {
        ParamPredicate {
            preds: preds.iter().map(|x| x.to_owned()).collect(),
        }
    }
}

/// A parameterized transition is basically a collection of guard and
/// update predicates, which are turned on/off when the actual transition
/// is being generated for the incremental algorithm.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct ParamTransition {
    pub name: String,
    pub guard: ParamPredicate,
    pub update: ParamPredicate,
}

impl ParamTransition {
    pub fn new(name: &str, guard: &ParamPredicate, update: &ParamPredicate) -> ParamTransition {
        ParamTransition {
            name: name.to_string(),
            guard: guard.to_owned(),
            update: update.to_owned(),
        }
    }
}

/// A parameterized planning problem that allows turning on/off certain
/// parts before generating the real problem and sending it to the
/// incremental algorithm.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct ParamPlanningProblem {
    pub name: String,
    pub init: ParamPredicate,
    pub goal: ParamPredicate,
    pub trans: Vec<ParamTransition>,
    pub invars: Predicate,
    pub params: Vec<Parameter>
}

impl ParamPlanningProblem {
    pub fn new(
        name: &str,
        init: &ParamPredicate,
        goal: &ParamPredicate,
        trans: &Vec<ParamTransition>,
        invars: &Predicate,
        params: &Vec<Parameter>
    ) -> ParamPlanningProblem {
        ParamPlanningProblem {
            name: name.to_string(),
            init: init.to_owned(),
            goal: goal.to_owned(),
            trans: trans.to_owned(),
            invars: invars.to_owned(),
            params: params.to_owned()
        }
    }
}

/// Given a parameterized predicate and the vector of activation parameters,
/// generate a predicate as a conjunction of predicates that are activated.
pub fn generate_predicate(ppred: &ParamPredicate, params: &Vec<Parameter>) -> Predicate {
    let activated: Vec<Parameter> = params
        .iter()
        .filter(|x| x.value)
        .map(|x| x.to_owned())
        .collect();
    Predicate::AND(
        ppred
            .preds
            .iter()
            .filter(|x| {
                get_predicate_vars(&x)
                    .iter()
                    .any(|y| activated.contains(&y.param))
            })
            .map(|x| x.to_owned())
            .collect(),
    )
}

/// Given a parameterized trtansition and the vector of activation parameters,
/// generate the transition guard and update as a conjunction of predicates in
/// the parameterized transition that are activated.
pub fn generate_transition(ptrans: &ParamTransition, params: &Vec<Parameter>) -> Transition {
    Transition::new(
        &ptrans.name,
        &generate_predicate(&ptrans.guard, &params),
        &generate_predicate(&ptrans.update, &params),
    )
}

/// Generates the problem from a parameterized problem and solves it with the incremental algorithm.
pub fn parameterized(
    prob: &ParamPlanningProblem,
    timeout: u64,
    max_steps: u64
) -> PlanningResult {
        incremental(&PlanningProblem::new(
            &prob.name,
            &generate_predicate(&prob.init, &prob.params),
            &generate_predicate(&prob.goal, &prob.params),
            &prob
                .trans
                .iter()
                .map(|x| generate_transition(x, &prob.params))
                .collect(),
            &prob.invars,
            // &generate_predicate(&prob.invars, &prob.params)
        ),
        timeout,
        max_steps
    )
}

// /// Generates the problem from a parameterized problem and solves it with the incremental algorithm.
// pub fn parameterized(
//     prob: &ParamPlanningProblem,
//     params: &Vec<Parameter>,
//     timeout: u64,
//     max_steps: u64
// ) -> PlanningResult {
//         incremental(&PlanningProblem::new(
//             &prob.name,
//             &generate_predicate(&prob.init, &params),
//             &generate_predicate(&prob.goal, &params),
//             &prob
//                 .trans
//                 .iter()
//                 .map(|x| generate_transition(x, &params))
//                 .collect(),
//             &generate_predicate(&prob.invars, &params)
//         ),
//         timeout,
//         max_steps
//     )
// }