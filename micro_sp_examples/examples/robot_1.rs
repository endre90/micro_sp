use micro_sp_tools::*;
use micro_sp_runner::*;
use micro_sp_examples::robot::robot_1;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::channel;
use r2r::*;
use tokio::prelude::*;

#[tokio::main]
async fn main() {

    let robot_model = robot_1();
    let domain = vec!("left", "right", "home");

    let init = Predicate::AND(
        vec!(
            Predicate::EQRL(EnumVariable::new("act_robot_1_pose", "act_robot_1_pose", &domain, None, &ControlKind::None), String::from("left")),
            Predicate::EQRL(EnumVariable::new("ref_robot_1_pose", "ref_robot_1_pose", &domain, None, &ControlKind::None), String::from("left"))
        )
    );

    let goal = Predicate::EQRL(EnumVariable::new("act_robot_1_pose", "act_robot_1_pose", &domain, None, &ControlKind::None), String::from("right"));

    let problem = PlanningProblem::new("problem_1", &init, &goal, &robot_model, &Predicate::TRUE, &30);
    
    let result = incremental::incremental(&problem);

    // let _m = runner(&problem);

    println!("\n");
    println!("============================================");
    println!("              PLANNING RESULT               ");
    println!("============================================");
    for t in &result.trace{
 
        println!("source: {:?}", t.source);
        println!("trans: {:?}", t.trans);
        println!("sink: {:?}", t.sink);
        println!("============================================");
    }
    println!("               END OF RESULT                ");
    println!("============================================");
    println!("plan_found: {:?}", result.plan_found);
    println!("plan_lenght: {:?}", result.plan_length);
    println!("time_to_solve: {:?}", result.time_to_solve);
}