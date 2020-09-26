use std::time::{Instant};
use z3_sys::*;
use z3_v2::*;
use super::*;

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct Transition {
    pub name: String,
    pub guard: Predicate,
    pub update: Predicate
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub struct PlanningProblem {
    pub name: String,
    pub init: Predicate,
    pub goal: Predicate,
    pub trans: Vec<Transition>,
    pub ltl_specs: Predicate,
    pub max_steps: u32
}

#[derive(Clone)]
pub struct KeepVariableValues<'ctx> {
    pub ctx: &'ctx ContextZ3
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningFrame {
    pub source: Vec<String>,
    pub sink: Vec<String>,
    pub trans: String,
}

pub struct GetPlanningResultZ3<'ctx> {
    pub ctx: &'ctx ContextZ3,
    pub model: Z3_model,
    pub nr_steps: u32,
    pub frames: PlanningResult
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct PlanningResult {
    pub plan_found: bool,
    pub plan_length: u32,
    pub trace: Vec<PlanningFrame>,
    // pub raw_trace: Vec<PlanningFrame>,
    pub time_to_solve: std::time::Duration,
}

impl Transition {
    pub fn new(name: &str, guard: &Predicate, update: &Predicate) -> Transition {
        Transition { name: name.to_string(),
                     guard: guard.to_owned(),
                     update: update.to_owned() }
    }
}

impl PlanningProblem {
    pub fn new(name: &str, init: &Predicate, goal: &Predicate, trans: &Vec<Transition>,
        ltl_specs: &Predicate, max_steps: &u32) -> PlanningProblem {
        PlanningProblem {
            name: name.to_string(),
            init: init.to_owned(),
            goal: goal.to_owned(),
            trans: trans.to_owned(),
            ltl_specs: ltl_specs.to_owned(),
            max_steps: max_steps.to_owned()
        }
    }
}

impl <'ctx> KeepVariableValues<'ctx> {
    pub fn new(ctx: &'ctx ContextZ3, vars: &Vec<EnumVariable>, trans: &Transition, step: &u32) -> Z3_ast {
        let changed = GetPredicateVars::new(&trans.update);
        let unchanged = IterOps::difference(vars, &changed);
        let mut assert_vec = vec!();
        for u in unchanged {
            let sort = EnumSortZ3::new(&ctx, &u.r#type, u.domain.iter().map(|x| x.as_str()).collect());
            let v_1 = EnumVarZ3::new(&ctx, sort.r, format!("{}_s{}", u.name.to_string(), step).as_str());
            let v_2 = EnumVarZ3::new(&ctx, sort.r, format!("{}_s{}", u.name.to_string(), step - 1).as_str());
            assert_vec.push(EQZ3::new(&ctx, v_1, v_2));
        }

        ANDZ3::new(&ctx, assert_vec)
    }
}

pub fn incremental(prob: &PlanningProblem) -> PlanningResult {

    let cfg = ConfigZ3::new();
    let ctx = ContextZ3::new(&cfg);
    let slv = SolverZ3::new(&ctx);

    let problem_vars = GetProblemVars::new(&prob);

    SlvAssertZ3::new(&ctx, &slv, PredicateToAstZ3::new(&ctx, &prob.init, "state", &0));

    SlvPushZ3::new(&ctx, &slv); // create backtracking point
    SlvAssertZ3::new(&ctx, &slv, PredicateToAstZ3::new(&ctx, &prob.ltl_specs, "specs", &0));
    SlvAssertZ3::new(&ctx, &slv, PredicateToAstZ3::new(&ctx, &prob.goal, "specs", &0));

    let now = Instant::now();
    let mut plan_found: bool = false;

    let mut step: u32 = 0;

    while step < prob.max_steps + 1 {
        step = step + 1;
        if SlvCheckZ3::new(&ctx, &slv) != 1 {
            SlvPopZ3::new(&ctx, &slv, 1);

            let mut all_trans = vec!();
            for t in &prob.trans {
                let name = format!("{}_t{}", &t.name, step);
                let guard = PredicateToAstZ3::new(&ctx, &t.guard, "guard", &(step - 1));
                let update = PredicateToAstZ3::new(&ctx, &t.update, "update", &(step));
                let keeps = KeepVariableValues::new(&ctx, &problem_vars, &t, &step);

                all_trans.push(ANDZ3::new(&ctx, 
                    vec!(EQZ3::new(&ctx, 
                        BoolVarZ3::new(&ctx, &BoolSortZ3::new(&ctx), name.as_str()), 
                        BoolZ3::new(&ctx, true)),
                    guard, update, keeps)));
            }

            SlvAssertZ3::new(&ctx, &slv, ORZ3::new(&ctx, all_trans));
            
            SlvPushZ3::new(&ctx, &slv);
            SlvAssertZ3::new(&ctx, &slv, PredicateToAstZ3::new(&ctx, &prob.ltl_specs, "specs", &step));
            SlvAssertZ3::new(&ctx, &slv, PredicateToAstZ3::new(&ctx, &prob.goal, "specs", &step));
            
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

impl PlanningFrame {
    pub fn new(source: &Vec<&str>, sink: &Vec<&str>, trans: &str) -> PlanningFrame {
        PlanningFrame {
            source: source.iter().map(|x| x.to_string()).collect(),
            sink: sink.iter().map(|x| x.to_string()).collect(),
            trans: trans.to_string()
        }
    }
}

impl <'ctx> GetPlanningResultZ3<'ctx> {
    pub fn new(ctx: &'ctx ContextZ3, model: Z3_model, nr_steps: u32, 
    planning_time: std::time::Duration, plan_found: bool) -> PlanningResult {
        let model_str = ModelToStringZ3::new(&ctx, model);
        let mut model_vec = vec!();

        let num = ModelGetNumConstsZ3::new(&ctx, model);
        let mut lines = model_str.lines();
        let mut i: u32 = 0;

        while i < num {
            model_vec.push(lines.next().unwrap_or(""));
            i = i + 1;
        }

        // println!("{:#?}", model_vec);

        let mut trace: Vec<PlanningFrame> = vec!();
        // let mut raw_trace: Vec<PlanningFrame> = vec!();
        
        for i in 0..nr_steps {
            let mut frame: PlanningFrame = PlanningFrame::new(&vec!(), &vec!(), "");
            let mut raw_frame: PlanningFrame = PlanningFrame::new(&vec!(), &vec!(), "");
            for j in &model_vec {
                let sep: Vec<&str> = j.split(" -> ").collect();
                if sep[0].ends_with(&format!("_s{}", i)){
                    // raw_frame.state.push(j.to_string());
                    let trimmed_state = sep[0].trim_end_matches(&format!("_s{}", i));
                    match sep[1] {
                        "false" => {
                            frame.sink.push(sep[0].to_string());
                            // raw_frame.state.push(j.to_string());
                        },
                        "true" => {
                            frame.sink.push(sep[0].to_string());
                            // raw_frame.state.push(j.to_string());
                        },
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

        let mut new_trace = vec!();
        let mut new = trace.iter();
        let mut prev = vec!();

        'breakable: loop {
            let mut frame: PlanningFrame = PlanningFrame::new(&vec!(), &vec!(), "");

            match new.next() {
                Some(x) => {
                    frame.source = prev.clone();
                    frame.sink = x.sink.clone();
                    prev = x.sink.clone();
                    frame.trans = x.trans.clone();
                },
                None => break 'breakable
            }

            new_trace.push(frame);
        }
        if new_trace.len() >= 1 {
            new_trace.drain(0..1);
        }

        PlanningResult {
            plan_found: plan_found,
            plan_length: nr_steps - 1,
            trace: new_trace,
            time_to_solve: planning_time,
        }
    }
}

#[test]
fn test_incremental_1(){

    let max_steps: u32 = 60;

    let color_domain = vec!("red", "green");
    let pose_domain = vec!("buffer", "home", "table");
    let stat_domain = vec!("active", "idle");
    let buffer_domain = vec!("cube", "ball", "empty");
    let gripper_domain = vec!("cube", "ball", "empty");
    let table_domain = vec!("cube", "ball", "empty");

    let color = EnumVariable::new("color", "color", &color_domain, None, &ControlKind::None);

    let act_pos = EnumVariable::new("act_pos", "pose", &pose_domain, None, &ControlKind::None);
    let ref_pos = EnumVariable::new("ref_pos", "pose", &pose_domain, None, &ControlKind::None);

    let act_stat = EnumVariable::new("act_stat", "status", &stat_domain, None, &ControlKind::None);
    let ref_stat = EnumVariable::new("ref_stat", "status", &stat_domain, None, &ControlKind::None);

    let buffer = EnumVariable::new("buffer_cube", "buffer", &buffer_domain, None, &ControlKind::None);
    let gripper = EnumVariable::new("gripper_cube", "gripper", &gripper_domain, None, &ControlKind::None);
    let table = EnumVariable::new("table_cube", "table", &table_domain, None, &ControlKind::None);

    // colors
    let red = Predicate::EQRL(color.clone(), String::from("red"));
    let green = Predicate::EQRL(color.clone(), String::from("green"));

    // act stat predicates
    let stat_active = Predicate::EQRL(act_stat.clone(), String::from("active"));
    let stat_idle = Predicate::EQRL(act_stat.clone(), String::from("idle"));
    let not_stat_active = Predicate::NOT(Box::new(stat_active.clone()));
    let not_stat_idle = Predicate::NOT(Box::new(stat_idle.clone()));

    // ref stat predicates
    let set_stat_active = Predicate::EQRL(ref_stat.clone(), String::from("active"));
    let set_stat_idle = Predicate::EQRL(ref_stat.clone(), String::from("idle"));
    let not_set_stat_active = Predicate::NOT(Box::new(set_stat_active.clone()));
    let not_set_stat_idle = Predicate::NOT(Box::new(set_stat_idle.clone()));

    // act pos predicates
    let pos_buffer = Predicate::EQRL(act_pos.clone(), String::from("buffer"));
    let pos_table = Predicate::EQRL(act_pos.clone(), String::from("table"));
    let pos_home = Predicate::EQRL(act_pos.clone(), String::from("home"));
    let not_pos_buffer = Predicate::NOT(Box::new(pos_buffer.clone()));
    let not_pos_table = Predicate::NOT(Box::new(pos_table.clone()));
    let not_pos_home = Predicate::NOT(Box::new(pos_home.clone()));

    // ref pos predicates
    let set_pos_buffer = Predicate::EQRL(ref_pos.clone(), String::from("buffer"));
    let set_pos_table = Predicate::EQRL(ref_pos.clone(), String::from("table"));
    let set_pos_home = Predicate::EQRL(ref_pos.clone(), String::from("home"));
    let not_set_pos_buffer = Predicate::NOT(Box::new(set_pos_buffer.clone()));
    let not_set_pos_table = Predicate::NOT(Box::new(set_pos_table.clone()));
    let not_set_pos_home = Predicate::NOT(Box::new(set_pos_home.clone()));

    // act buffer predicates
    let buffer_cube = Predicate::EQRL(buffer.clone(), String::from("cube"));
    let buffer_ball = Predicate::EQRL(buffer.clone(), String::from("ball"));
    let buffer_empty = Predicate::EQRL(buffer.clone(), String::from("empty"));
    let _not_buffer_cube = Predicate::NOT(Box::new(buffer_cube.clone()));
    let _not_buffer_ball = Predicate::NOT(Box::new(buffer_ball.clone()));
    let _not_buffer_empty = Predicate::NOT(Box::new(buffer_empty.clone()));
    
    // act gripper predicates
    let gripper_cube = Predicate::EQRL(gripper.clone(), String::from("cube"));
    let gripper_ball = Predicate::EQRL(gripper.clone(), String::from("ball"));
    let gripper_empty = Predicate::EQRL(gripper.clone(), String::from("empty"));
    let _not_gripper_cube = Predicate::NOT(Box::new(gripper_cube.clone()));
    let _not_gripper_ball = Predicate::NOT(Box::new(gripper_ball.clone()));
    let _not_gripper_empty = Predicate::NOT(Box::new(gripper_empty.clone()));

    // act table predicates
    let table_cube = Predicate::EQRL(table.clone(), String::from("cube"));
    let table_ball = Predicate::EQRL(table.clone(), String::from("ball"));
    let table_empty = Predicate::EQRL(table.clone(), String::from("empty"));
    let _not_table_cube = Predicate::NOT(Box::new(table_cube.clone()));
    let _not_table_ball = Predicate::NOT(Box::new(table_ball.clone()));
    let _not_table_empty = Predicate::NOT(Box::new(table_empty.clone()));

    // are ref == act predicates
    let pos_stable = Predicate::EQRR(act_pos.clone(), ref_pos.clone());
    let stat_stable = Predicate::EQRR(act_stat.clone(), ref_stat.clone());
    let _not_pos_stable = Predicate::EQRR(act_pos.clone(), ref_pos.clone());
    let _not_stat_stable = Predicate::EQRR(act_stat.clone(), ref_stat.clone());

    let t1 = Transition::new(
        "start_activate", 
        &Predicate::AND(
            vec!(
                not_stat_active.clone(),
                not_set_stat_active.clone()
            )
        ),
        &set_stat_active
    );

    let t2 = Transition::new(
        "finish_activate", 
        &Predicate::AND(
            vec!(
                set_stat_active.clone(),
                not_stat_active.clone()
            )
        ),
        &stat_active
    );

    let t3 = Transition::new(
        "start_deactivate", 
        &Predicate::AND(
            vec!(
                not_stat_idle.clone(),
                not_set_stat_idle.clone()
            )
        ),
        &set_stat_idle
    );

    let t4 = Transition::new(
        "finish_deactivate", 
        &Predicate::AND(
            vec!(
                not_stat_idle.clone(),
                set_stat_idle.clone()
            )
        ),
        &stat_idle
    );

    let t5 = Transition::new(
        "start_move_to_buffer",
        &Predicate::AND(
            vec!(
                stat_active.clone(),
                set_stat_active.clone(),
                pos_stable.clone(),
                not_pos_buffer.clone(),
                not_set_pos_buffer.clone()
            )
        ),
        &set_pos_buffer
    );

    let t6 = Transition::new(
        "finish_move_to_buffer",
        &Predicate::AND(
            vec!(
                stat_active.clone(),
                set_stat_active.clone(),
                not_pos_buffer.clone(),
                set_pos_buffer.clone()
            )
        ),
        &pos_buffer
    );

    let t7 = Transition::new(
        "start_move_to_table",
        &Predicate::AND(
            vec!(
                stat_active.clone(),
                set_stat_active.clone(),
                pos_stable.clone(),
                not_pos_table.clone(),
                not_set_pos_table.clone()
            )
        ),
        &set_pos_table
    );

    let t8 = Transition::new(
        "finish_move_to_table",
        &Predicate::AND(
            vec!(
                stat_active.clone(),
                set_stat_active.clone(),
                not_pos_table.clone(),
                set_pos_table.clone()
            )
        ),
        &pos_table
    );

    let t9 = Transition::new(
        "start_move_to_home",
        &Predicate::AND(
            vec!(
                stat_active.clone(),
                set_stat_active.clone(),
                pos_stable.clone(),
                not_pos_home.clone(),
                not_set_pos_home.clone()
            )
        ),
        &set_pos_home
    );

    let t10 = Transition::new(
        "finish_move_to_home",
        &Predicate::AND(
            vec!(
                stat_active.clone(),
                set_stat_active.clone(),
                not_pos_home.clone(),
                set_pos_home.clone()
            )
        ),
        &pos_home
    );

    let t11 = Transition::new(
        "take_cube_from_buffer",
        &Predicate::AND(
            vec!(
                buffer_cube.clone(),
                stat_active.clone(),
                set_stat_active.clone(),
                pos_buffer.clone(),
                set_pos_buffer.clone()
            )
        ),
        &Predicate::AND(
            vec!(
                gripper_cube.clone(),
                table_empty.clone(),
                buffer_empty.clone()
            )
        )
    );

    let t12 = Transition::new(
        "take_cube_from_table",
        &Predicate::AND(
            vec!(
                table_cube.clone(),
                stat_active.clone(),
                set_stat_active.clone(),
                pos_table.clone(),
                set_pos_table.clone()
            )
        ),
        &Predicate::AND(
            vec!(
                gripper_cube.clone(),
                table_empty.clone(),
                buffer_empty.clone()
            )
        )
    );

    let t13 = Transition::new(
        "leave_cube_at_buffer",
        &Predicate::AND(
            vec!(
                gripper_cube.clone(),
                stat_active.clone(),
                set_stat_active.clone(),
                pos_buffer.clone(),
                set_pos_buffer.clone()
            )
        ),
        &Predicate::AND(
            vec!(
                gripper_empty.clone(),
                table_empty.clone(),
                buffer_cube.clone()
            )
        )
    );

    let t14 = Transition::new(
        "leave_cube_at_table",
        &Predicate::AND(
            vec!(
                gripper_cube.clone(),
                stat_active.clone(),
                set_stat_active.clone(),
                pos_table.clone(),
                set_pos_table.clone()
            )
        ),
        &Predicate::AND(
            vec!(
                gripper_empty.clone(),
                table_cube.clone(),
                buffer_empty.clone()
            )
        )
    );

    let init = Predicate::AND(
        vec!(
            pos_stable.clone(),
            pos_buffer.clone(),
            stat_stable.clone(),
            stat_idle.clone(),
            table_cube.clone()
        )
    );

    let goal1 = Predicate::AND(
        vec!(
            pos_table.clone(),
            stat_idle.clone(),
            buffer_cube.clone(),
            red.clone()
        )
    );

    let goal2 = Predicate::AND(
        vec!(
            pos_buffer.clone(),
            stat_idle.clone(),
            table_cube.clone()
        )
    );

    let goal3 = Predicate::AND(
        vec!(
            pos_table.clone(),
            stat_idle.clone(),
            buffer_cube.clone(),
            green.clone()
        )
    );

    // let goals = vec!((&goal1, None), (&goal2, None)); //, (&goal3, None));

    let trans = vec!(t1, t2, t3, t4, t5, t6, t7, t8, t9, t10, t11, t12, t13, t14);

    // let problem = MultGoalsPlanningProblem::new("problem_1", &init, &goals, &trans, &specs, &max_steps);

    let problem = PlanningProblem::new("problem_1", &init, &goal1, &trans, &Predicate::TRUE, &max_steps);
    
    // let result = MultGoalsIncremental::new(&problem);

    let result = incremental(&problem);

    let forb = Predicate::AND(
        vec!(
            gripper_ball.clone(),
            pos_buffer.clone(),
        )
    );

    // let safe = VerifySafety::new(&problem, &result, &forb);

    // println!("safe: {:?}", safe);
    println!("plan_found: {:?}", result.plan_found);
    println!("plan_lenght: {:?}", result.plan_length);
    println!("time_to_solve: {:?}", result.time_to_solve);
    println!("trace: ");

    for t in &result.trace{
 
        println!("source: {:?}", t.source);
        // println!("state_pred: {:?}", StateToPredicate::new(&t.sink.iter().map(|x| x.as_str()).collect(), &problem));
        println!("trans: {:?}", t.trans);
        println!("sink: {:?}", t.sink);
        println!("=========================");
    }
}