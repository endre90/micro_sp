use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

/// A planning problem that is given to the incremental solver.
#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct PlanningProblem {
    pub name: String,
    pub init: Predicate,
    pub goal: Predicate,
    pub trans: Vec<Transition>,
    pub invar: Predicate,
    pub max_steps: u32,
    pub paradigm: Paradigm,
}

/// A frame holds states about what happens in a step.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrame {
    pub source: CompleteState,
    pub sink: CompleteState,
    pub trans: String,
}

/// A result is generated when the planner finds a satisfiable model.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningResult {
    pub plan_found: bool,
    pub plan_length: u32,
    pub trace: Vec<PlanningFrame>,
    pub time_to_solve: std::time::Duration,
}

impl PlanningProblem {
    /// Make a new planning problem from defined componenets.
    pub fn new(
        name: &str,
        init: &Predicate,
        goal: &Predicate,
        trans: &Vec<Transition>,
        invar: &Predicate,
        max_steps: &u32,
        paradigm: &Paradigm,
    ) -> PlanningProblem {
        PlanningProblem {
            name: name.to_string(),
            init: init.to_owned(),
            goal: goal.to_owned(),
            trans: trans.to_owned(),
            invar: invar.to_owned(),
            max_steps: max_steps.to_owned(),
            paradigm: paradigm.to_owned(),
        }
    }
}

/// When some varibels are updated in a transition, the other variables
/// from the problem should keep their values from the previous step.
pub fn keep_variable_values(
    ctx: &ContextZ3,
    vars: &Vec<EnumVariable>,
    trans: &Transition,
    step: &u32,
) -> Z3_ast {
    let changed = get_predicate_vars(&trans.update);
    let unchanged = IterOps::difference(vars, &changed);

    ANDZ3::new(
        &ctx,
        unchanged
            .iter()
            .map(|x| {
                let sort = EnumSortZ3::new(
                    &ctx,
                    &x.r#type,
                    x.domain.iter().map(|x| x.as_str()).collect(),
                );
                EQZ3::new(
                    &ctx,
                    EnumVarZ3::new(
                        &ctx,
                        sort.r,
                        format!("{}_s{}", x.name.to_string(), step).as_str(),
                    ),
                    EnumVarZ3::new(
                        &ctx,
                        sort.r,
                        format!("{}_s{}", x.name.to_string(), step - 1).as_str(),
                    ),
                )
            })
            .collect(),
    )
}

/// The incremental algorithm that calls z3 to find a plan.
///
/// Based on Gocht and Balyo's algorithm from 2017.
pub fn incremental(prob: &PlanningProblem) -> PlanningResult {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, &0));
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invar, &0));

    SlvPushZ3::new(&ctx, &slv); // create backtracking point
    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, &0));

    let now = Instant::now();
    let mut plan_found: bool = false;
    let mut step: u32 = 0;

    while step < prob.max_steps + 1 {
        step = step + 1;
        match SlvCheckZ3::new(&ctx, &slv) == 1 {
            false => {
                SlvPopZ3::new(&ctx, &slv, 1);
                SlvAssertZ3::new(
                    &ctx,
                    &slv,
                    ORZ3::new(
                        &ctx,
                        prob.trans
                            .iter()
                            .map(|x| {
                                ANDZ3::new(
                                    &ctx,
                                    vec![
                                        EQZ3::new(
                                            &ctx,
                                            BoolVarZ3::new(
                                                &ctx,
                                                &BoolSortZ3::new(&ctx),
                                                format!("{}_t{}", &x.name, step).as_str(),
                                            ),
                                            BoolZ3::new(&ctx, true),
                                        ),
                                        predicate_to_ast(&ctx, &x.guard, &(step - 1)),
                                        predicate_to_ast(&ctx, &x.update, &(step)),
                                        keep_variable_values(
                                            &ctx,
                                            &get_problem_vars(&prob),
                                            &x,
                                            &step,
                                        ),
                                    ],
                                )
                            })
                            .collect(),
                    ),
                );

                SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.invar, &step));
                SlvPushZ3::new(&ctx, &slv);
                SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, &step));

                // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
                // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
                // for asrt in asrtvec {
                //     println!("{}", AstToStringZ3::new(&ctx, asrt));
                // }
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
            SlvGetModelZ3::new(&ctx, &slv),
            step,
            planning_time,
            plan_found,
        ),
        false => get_planning_result(
            &ctx,
            &prob,
            FreshModelZ3::new(&ctx),
            step,
            planning_time,
            plan_found,
        ),
    }
}
