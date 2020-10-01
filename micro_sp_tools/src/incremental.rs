use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub update: Predicate,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct PlanningProblem {
    pub name: String,
    pub init: Predicate,
    pub goal: Predicate,
    pub trans: Vec<Transition>,
    pub ltl_specs: Predicate,
    pub max_steps: u32,
}

#[derive(Clone)]
pub struct KeepVariableValues<'ctx> {
    pub ctx: &'ctx ContextZ3,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrameStrings {
    pub source: Vec<String>,
    pub sink: Vec<String>,
    pub trans: String,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrameStates {
    pub source: State,
    pub sink: State,
    pub trans: String,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningResultStates {
    pub plan_found: bool,
    pub plan_length: u32,
    pub trace: Vec<PlanningFrameStates>,
    pub time_to_solve: std::time::Duration,
}

pub struct GetPlanningResultZ3<'ctx> {
    pub ctx: &'ctx ContextZ3,
    pub model: Z3_model,
    pub nr_steps: u32,
    pub frames: PlanningResultStates,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningResultStrings {
    pub plan_found: bool,
    pub plan_length: u32,
    pub trace: Vec<PlanningFrameStrings>,
    // pub raw_trace: Vec<PlanningFrame>,
    pub time_to_solve: std::time::Duration,
}

impl Transition {
    pub fn new(name: &str, guard: &Predicate, update: &Predicate) -> Transition {
        Transition {
            name: name.to_string(),
            guard: guard.to_owned(),
            update: update.to_owned(),
        }
    }
}

impl PlanningProblem {
    pub fn new(
        name: &str,
        init: &Predicate,
        goal: &Predicate,
        trans: &Vec<Transition>,
        ltl_specs: &Predicate,
        max_steps: &u32,
    ) -> PlanningProblem {
        PlanningProblem {
            name: name.to_string(),
            init: init.to_owned(),
            goal: goal.to_owned(),
            trans: trans.to_owned(),
            ltl_specs: ltl_specs.to_owned(),
            max_steps: max_steps.to_owned(),
        }
    }
}

impl<'ctx> KeepVariableValues<'ctx> {
    pub fn new(
        ctx: &'ctx ContextZ3,
        vars: &Vec<EnumVariable>,
        trans: &Transition,
        step: &u32,
    ) -> Z3_ast {
        let changed = get_predicate_vars(&trans.update);
        let unchanged = IterOps::difference(vars, &changed);
        let mut assert_vec = vec![];
        for u in unchanged {
            let sort = EnumSortZ3::new(
                &ctx,
                &u.r#type,
                u.domain.iter().map(|x| x.as_str()).collect(),
            );
            let v_1 = EnumVarZ3::new(
                &ctx,
                sort.r,
                format!("{}_s{}", u.name.to_string(), step).as_str(),
            );
            let v_2 = EnumVarZ3::new(
                &ctx,
                sort.r,
                format!("{}_s{}", u.name.to_string(), step - 1).as_str(),
            );
            assert_vec.push(EQZ3::new(&ctx, v_1, v_2));
        }

        ANDZ3::new(&ctx, assert_vec)
    }
}

pub fn incremental(prob: &PlanningProblem) -> PlanningResultStrings {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

    let problem_vars = get_problem_vars(&prob);

    SlvAssertZ3::new(
        &ctx,
        &slv,
        PredicateToAstZ3::new(&ctx, &prob.init, "state", &0),
    );

    SlvPushZ3::new(&ctx, &slv); // create backtracking point
    SlvAssertZ3::new(
        &ctx,
        &slv,
        PredicateToAstZ3::new(&ctx, &prob.ltl_specs, "specs", &0),
    );
    SlvAssertZ3::new(
        &ctx,
        &slv,
        PredicateToAstZ3::new(&ctx, &prob.goal, "specs", &0),
    );

    let now = Instant::now();
    let mut plan_found: bool = false;

    let mut step: u32 = 0;

    while step < prob.max_steps + 1 {
        step = step + 1;
        if SlvCheckZ3::new(&ctx, &slv) != 1 {
            SlvPopZ3::new(&ctx, &slv, 1);

            let mut all_trans = vec![];
            for t in &prob.trans {
                let name = format!("{}_t{}", &t.name, step);
                let guard = PredicateToAstZ3::new(&ctx, &t.guard, "guard", &(step - 1));
                let update = PredicateToAstZ3::new(&ctx, &t.update, "update", &(step));
                let keeps = KeepVariableValues::new(&ctx, &problem_vars, &t, &step);

                all_trans.push(ANDZ3::new(
                    &ctx,
                    vec![
                        EQZ3::new(
                            &ctx,
                            BoolVarZ3::new(&ctx, &BoolSortZ3::new(&ctx), name.as_str()),
                            BoolZ3::new(&ctx, true),
                        ),
                        guard,
                        update,
                        keeps,
                    ],
                ));
            }

            SlvAssertZ3::new(&ctx, &slv, ORZ3::new(&ctx, all_trans));
            SlvPushZ3::new(&ctx, &slv);
            SlvAssertZ3::new(
                &ctx,
                &slv,
                PredicateToAstZ3::new(&ctx, &prob.ltl_specs, "specs", &step),
            );
            SlvAssertZ3::new(
                &ctx,
                &slv,
                PredicateToAstZ3::new(&ctx, &prob.goal, "specs", &step),
            );
        } else {
            plan_found = true;
            break;
        }
    }

    let planning_time = now.elapsed();

    // let asserts = SlvGetAssertsZ3::new(&ctx, &slv);
    // let asrtvec = Z3AstVectorToVectorAstZ3::new(&ctx, asserts);
    // for asrt in asrtvec {
    //     println!("{}", AstToStringZ3::new(&ctx, asrt));
    // }
    // let cnf = GetCnfVectorZ3::new(&ctx, asrtvec);
    if plan_found == true {
        let model = SlvGetModelZ3::new(&ctx, &slv);
        let result = GetPlanningResultZ3::new(&ctx, model, step, planning_time, plan_found);
        result
    } else {
        let model = FreshModelZ3::new(&ctx);
        let result = GetPlanningResultZ3::new(&ctx, model, step, planning_time, plan_found);
        result
    }
}

impl PlanningFrameStrings {
    pub fn new(source: &Vec<&str>, sink: &Vec<&str>, trans: &str) -> PlanningFrameStrings {
        PlanningFrameStrings {
            source: source.iter().map(|x| x.to_string()).collect(),
            sink: sink.iter().map(|x| x.to_string()).collect(),
            trans: trans.to_string(),
        }
    }
}

impl<'ctx> GetPlanningResultZ3<'ctx> {
    pub fn new(
        ctx: &'ctx ContextZ3,
        model: Z3_model,
        nr_steps: u32,
        planning_time: std::time::Duration,
        plan_found: bool,
    ) -> PlanningResultStrings {
        let model_str = ModelToStringZ3::new(&ctx, model);
        let mut model_vec = vec![];

        let num = ModelGetNumConstsZ3::new(&ctx, model);
        let mut lines = model_str.lines();
        let mut i: u32 = 0;

        while i < num {
            model_vec.push(lines.next().unwrap_or(""));
            i = i + 1;
        }

        // println!("{:#?}", model_vec);

        let mut trace: Vec<PlanningFrameStrings> = vec![];
        // let mut raw_trace: Vec<PlanningFrame> = vec!();
        for i in 0..nr_steps {
            let mut frame: PlanningFrameStrings = PlanningFrameStrings::new(&vec![], &vec![], "");
            let mut raw_frame: PlanningFrameStrings = PlanningFrameStrings::new(&vec![], &vec![], "");
            for j in &model_vec {
                let sep: Vec<&str> = j.split(" -> ").collect();
                if sep[0].ends_with(&format!("_s{}", i)) {
                    // raw_frame.state.push(j.to_string());
                    let trimmed_state = sep[0].trim_end_matches(&format!("_s{}", i));
                    match sep[1] {
                        "false" => {
                            frame.sink.push(sep[0].to_string());
                            // raw_frame.state.push(j.to_string());
                        }
                        "true" => {
                            frame.sink.push(sep[0].to_string());
                            // raw_frame.state.push(j.to_string());
                        }
                        _ => {
                            frame.sink.push(format!("{} -> {}", trimmed_state, sep[1]));
                            // raw_frame.state.push(j.to_string());
                        }
                    }
                } else if sep[0].ends_with(&format!("_t{}", i)) && sep[1] == "true" {
                    let trimmed_trans = sep[0].trim_end_matches(&format!("_t{}", i));
                    frame.trans = trimmed_trans.to_string();
                    raw_frame.trans = sep[0].to_string();
                }
            }
            if model_vec.len() != 0 {
                trace.push(frame);
                // raw_trace.push(raw_frame);
            }
        }

        let mut new_trace = vec![];
        let mut new = trace.iter();
        let mut prev = vec![];

        'breakable: loop {
            let mut frame: PlanningFrameStrings = PlanningFrameStrings::new(&vec![], &vec![], "");

            match new.next() {
                Some(x) => {
                    frame.source = prev.clone();
                    frame.sink = x.sink.clone();
                    prev = x.sink.clone();
                    frame.trans = x.trans.clone();
                }
                None => break 'breakable,
            }

            new_trace.push(frame);
        }
        if new_trace.len() >= 1 {
            new_trace.drain(0..1);
        }

        PlanningResultStrings {
            plan_found: plan_found,
            plan_length: nr_steps - 1,
            trace: new_trace,
            time_to_solve: planning_time,
        }
    }
}

#[test]
fn test_incremental_1() {
    let act_pos = EnumVariable::new("act_pos", &vec!["left", "right"], None, None);
    let ref_pos = EnumVariable::new("ref_pos", &vec!["left", "right"], None, None);
    let act_left = Predicate::EQRL(act_pos.clone(), "left".to_string());
    let act_right = Predicate::EQRL(act_pos.clone(), "right".to_string());
    let ref_left = Predicate::EQRL(ref_pos.clone(), "left".to_string());
    let ref_right = Predicate::EQRL(ref_pos.clone(), "right".to_string());
    let t1 = Transition::new(
        "start_move_left",
        &Predicate::AND(vec![act_right.clone(), ref_right.clone()]),
        &ref_left,
    );
    let t2 = Transition::new(
        "start_move_right",
        &Predicate::AND(vec![act_left.clone(), ref_left.clone()]),
        &ref_right,
    );
    let t3 = Transition::new(
        "finish_move_left",
        &Predicate::AND(vec![act_right.clone(), ref_left.clone()]),
        &act_left,
    );
    let t4 = Transition::new(
        "finish_move_right",
        &Predicate::AND(vec![act_left.clone(), ref_right.clone()]),
        &act_right,
    );
    let problem = PlanningProblem::new(
        "prob1",
        &Predicate::AND(vec![act_left, ref_left]),
        &act_right,
        &vec![t1, t2, t3, t4],
        &Predicate::TRUE,
        &12,
    );
    let result = incremental(&problem);
    println!("{:?}", result);
}
