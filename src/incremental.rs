use super::*;
use std::time::Instant;
use z3_sys::*;
use z3_v2::*;

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct PlanningProblem {
    pub name: String,
    pub init: Predicate,
    pub goal: Predicate,
    pub trans: Vec<Transition>,
    pub max_steps: u32,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrame {
    pub source: CompleteState,
    pub sink: CompleteState,
    pub trans: String,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningResult {
    pub plan_found: bool,
    pub plan_length: u32,
    pub trace: Vec<PlanningFrame>,
    pub time_to_solve: std::time::Duration,
}

impl PlanningProblem {
    pub fn new(
        name: &str,
        init: &Predicate,
        goal: &Predicate,
        trans: &Vec<Transition>,
        max_steps: &u32,
    ) -> PlanningProblem {
        PlanningProblem {
            name: name.to_string(),
            init: init.to_owned(),
            goal: goal.to_owned(),
            trans: trans.to_owned(),
            max_steps: max_steps.to_owned(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrameStrings {
    pub source: Vec<String>,
    pub sink: Vec<String>,
    pub trans: String,
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrameStates {
    pub source: CompleteState,
    pub sink: CompleteState,
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

pub fn incremental(prob: &PlanningProblem) -> PlanningResultStrings {
    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

    SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.init, &0));

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

                SlvPushZ3::new(&ctx, &slv);
                SlvAssertZ3::new(&ctx, &slv, predicate_to_ast(&ctx, &prob.goal, &step));
            }
            true => {
                plan_found = true;
                break;
            }
        }
    }

    let planning_time = now.elapsed();

    match plan_found {
        true => GetPlanningResultZ3::new(
            &ctx,
            SlvGetModelZ3::new(&ctx, &slv),
            step,
            planning_time,
            plan_found,
        ),
        false => GetPlanningResultZ3::new(
            &ctx,
            FreshModelZ3::new(&ctx),
            step,
            planning_time,
            plan_found,
        ),
    }
}

// rewrite this one day... merge string to state from utils with this
// pub fn get_planning_result(ctx: &ContextZ3, model: Z3_model, nr_steps: u32, planning_time: std::time::Duration, plan_found: bool) -> PlanningResult {
//     let model_str = ModelToStringZ3::new(&ctx, model);
//         let mut model_vec = vec![];

//         let num = ModelGetNumConstsZ3::new(&ctx, model);
//         let mut lines = model_str.lines();
//         let mut i: u32 = 0;

//         while i < num {
//             model_vec.push(lines.next().unwrap_or(""));
//             i = i + 1;
//         }

//         // println!("{:#?}", model_vec);

//         let mut trace: Vec<PlanningFrameStrings> = vec![];
//         // let mut raw_trace: Vec<PlanningFrame> = vec!();
//         for i in 0..nr_steps {
//             let mut frame: PlanningFrameStrings = PlanningFrameStrings::new(&vec![], &vec![], "");
//             let mut raw_frame: PlanningFrameStrings =
//                 PlanningFrameStrings::new(&vec![], &vec![], "");
//             for j in &model_vec {
//                 let sep: Vec<&str> = j.split(" -> ").collect();
//                 if sep[0].ends_with(&format!("_s{}", i)) {
//                     // raw_frame.state.push(j.to_string());
//                     let trimmed_state = sep[0].trim_end_matches(&format!("_s{}", i));
//                     match sep[1] {
//                         "false" => {
//                             frame.sink.push(sep[0].to_string());
//                             // raw_frame.state.push(j.to_string());
//                         }
//                         "true" => {
//                             frame.sink.push(sep[0].to_string());
//                             // raw_frame.state.push(j.to_string());
//                         }
//                         _ => {
//                             frame.sink.push(format!("{} -> {}", trimmed_state, sep[1]));
//                             // raw_frame.state.push(j.to_string());
//                         }
//                     }
//                 } else if sep[0].ends_with(&format!("_t{}", i)) && sep[1] == "true" {
//                     let trimmed_trans = sep[0].trim_end_matches(&format!("_t{}", i));
//                     frame.trans = trimmed_trans.to_string();
//                     raw_frame.trans = sep[0].to_string();
//                 }
//             }
//             if model_vec.len() != 0 {
//                 trace.push(frame);
//                 // raw_trace.push(raw_frame);
//             }
//         }

//         let mut new_trace = vec![];
//         let mut new = trace.iter();
//         let mut prev = vec![];

//         'breakable: loop {
//             let mut frame: PlanningFrameStrings = PlanningFrameStrings::new(&vec![], &vec![], "");

//             match new.next() {
//                 Some(x) => {
//                     frame.source = prev.clone();
//                     frame.sink = x.sink.clone();
//                     prev = x.sink.clone();
//                     frame.trans = x.trans.clone();
//                 }
//                 None => break 'breakable,
//             }

//             new_trace.push(frame);
//         }
//         if new_trace.len() >= 1 {
//             new_trace.drain(0..1);
//         }

//         PlanningResult {
//             plan_found: plan_found,
//             plan_length: nr_steps - 1,
//             trace: new_trace,
//             time_to_solve: planning_time,
//         }

// }

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
            let mut raw_frame: PlanningFrameStrings =
                PlanningFrameStrings::new(&vec![], &vec![], "");
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

// #[test]
// fn test_incremental_1() {

//     let act_pos = EnumVariable::new("act_pos", &vec!["left", "right"], None, &Kind::Measured);
//     let ref_pos = EnumVariable::new("ref_pos", &vec!["left", "right"], None, &Kind::Command);
//     let act_left = Predicate::EQRL(act_pos.clone(), "left".to_string());
//     let act_right = Predicate::EQRL(act_pos.clone(), "right".to_string());
//     let ref_left = Predicate::EQRL(ref_pos.clone(), "left".to_string());
//     let ref_right = Predicate::EQRL(ref_pos.clone(), "right".to_string());

//     let t1 = Transition::new(
//         "start_move_left",
//         &Predicate::AND(vec![act_right.clone(), ref_right.clone()]),
//         &ref_left
//     );
//     let t2 = Transition::new(
//         "start_move_right",
//         &Predicate::AND(vec![act_left.clone(), ref_left.clone()]),
//         &ref_right
//     );
//     let t3 = Transition::new(
//         "finish_move_left",
//         &Predicate::AND(vec![act_right.clone(), ref_left.clone()]),
//         &act_left
//     );
//     let t4 = Transition::new(
//         "finish_move_right",
//         &Predicate::AND(vec![act_left.clone(), ref_right.clone()]),
//         &act_right
//     );
//     let problem = PlanningProblem::new(
//         "prob1",
//         &Predicate::AND(vec![act_left, ref_left]),
//         &act_right,
//         &vec![t1, t2, t3, t4],
//         &Predicate::TRUE,
//         &12,
//     );
//     let result = incremental(&problem);
//     println!("{:?}", result);
// }
