use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

// pub fn parameterized()

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