use super::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

/// Variables with the same parameter belong to the same group during compositional planning.
/// As such, they will be included in the model together after the next refinement.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: bool,
}

impl Parameter {
    /// Make a new paremeter that will enable or disable variables during compositional planning.
    pub fn new(name: &str, value: &bool) -> Parameter {
        Parameter {
            name: name.to_owned(),
            value: *value,
        }
    }
    /// Make a dummy parameter that will include variables in every step during compositional
    /// planning, or for incremental planning where no parameter is needed.
    pub fn none() -> Parameter {
        Parameter {
            name: "NONE".to_owned(),
            value: true,
        }
    }
}

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
    pub invars: ParamPredicate, // should this be parameterized?
    pub max_steps: u32,
    pub params: Vec<Parameter>,
    // pub paradigm: Paradigm,
}

impl ParamPlanningProblem {
    pub fn new(
        name: &str,
        init: &ParamPredicate,
        goal: &ParamPredicate,
        trans: &Vec<ParamTransition>,
        invars: &ParamPredicate,
        max_steps: &u32,
        params: &Vec<Parameter>,
        // paradigm: &Paradigm,
    ) -> ParamPlanningProblem {
        ParamPlanningProblem {
            name: name.to_string(),
            init: init.to_owned(),
            goal: goal.to_owned(),
            trans: trans.to_owned(),
            invars: invars.to_owned(),
            max_steps: max_steps.to_owned(),
            params: params.iter().map(|x| x.clone()).collect(),
            // paradigm: paradigm.to_owned(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct ParamPlanningResult {
    pub result: PlanningResult,
    pub level: u32,
    pub concat: u32,
}

// impl GeneratePredicate {
//     pub fn new(params: &Vec<&Parameter>, ppred: &ParamPredicate) -> Predicate {
//         let mut p_own = params.to_owned();
//         let default_param = Parameter::default();
//         p_own.push(&default_param);
//         let mut pred_vec = vec![];
//         for pred in &ppred.preds {
//             let pred_vars: Vec<EnumVariable> = GetPredicateVars::new(&pred);
//             for param in &p_own {
//                 if pred_vars.iter().any(|x| x.param.name == param.name) && param.value {
//                     pred_vec.push(pred.to_owned())
//                 }
//             }
//         }
//         pred_vec.sort();
//         pred_vec.dedup();
//         Predicate::AND(pred_vec)
//     }
// }

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

#[test]
fn test_generate_predicate() {
    let p1 = Parameter::new("p1", &true);
    let p2 = Parameter::new("p2", &false);
    
    let d = vec!["a", "b", "c"];

    let var1_m = EnumVariable::new("var1_m", &d, "t1", Some(&p1), &Kind::Measured);
    let var1_c = EnumVariable::new("var1_c", &d, "t1", Some(&p1), &Kind::Command);
    let var2_m = EnumVariable::new("var2_m", &d, "t2", Some(&p2), &Kind::Measured);
    let var2_c = EnumVariable::new("var2_c", &d, "t2", Some(&p2), &Kind::Command);

    let pp = ParamPredicate::new(&vec![
        Predicate::EQ(EnumValue::new(&var1_m, "a", None)),
        Predicate::EQ(EnumValue::new(&var1_c, "b", None)),
        Predicate::EQ(EnumValue::new(&var2_m, "c", None)),
        Predicate::EQ(EnumValue::new(&var2_c, "a", None)),
    ]);

    let params = vec![p1, p2];
    println!("generated {:?}", generate_predicate(&pp, &params));
}

// pub fn parameterized(pprob: &ParamPlanningProblem, params: &Vec<Parameter>) -> ParamPlanningResult {
// }

// impl ParamIncremental {
//     pub fn new(prob: &ParamPlanningProblem, params: &Vec<&Parameter>, level: &u32, concat: &u32) -> ParamPlanningResult {
//         let generated_init = GeneratePredicate::new(&params, &prob.init);
//         let generated_goals = GeneratePredicate::new(&params, &prob.goal);
//         let generated_trans = GenerateTransitions::new(&params, &prob.trans);

//         let generated_prob = PlanningProblem::new(
//             prob.name.as_str(),
//             &generated_init,
//             &generated_goals,
//             &generated_trans,
//             &prob.ltl_specs,
//             &prob.max_steps
//         );

//         let inc_result = Incremental::new(&generated_prob);

//         ParamPlanningResult {
//             plan_found: inc_result.plan_found,
//             plan_length: inc_result.plan_length,
//             level: *level,
//             concat: *concat,
//             trace: inc_result.trace,
//             time_to_solve: inc_result.time_to_solve
//         }
//     }
// }
