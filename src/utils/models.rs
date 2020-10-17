use super::*;

pub struct Args {
    pub plan_only: bool,
    pub print_all: bool,
    pub problem: PlanningProblem,
}

/// Handle arguments
pub fn handle_args(args: &Vec<String>) -> Args {
    let mut mut_args = args.to_owned();
    mut_args.drain(0..1);

    let mut plan_only = false;
    let mut print_all = false;
    let mut problem_name = String::from("initial");

    match mut_args.len() {
        0 => (),
        1 => {
            problem_name = mut_args.drain(0..1).collect::<Vec<String>>()[0]
                .parse::<String>()
                .unwrap_or_default()
        }
        2 => {
            let arg1 = mut_args.drain(0..1).collect::<Vec<String>>()[0]
                .parse::<String>()
                .unwrap_or_default();
            let arg2 = mut_args.drain(0..1).collect::<Vec<String>>()[0]
                .parse::<String>()
                .unwrap_or_default();
            for arg in vec![arg1, arg2] {
                match arg.as_str() {
                    "plan-only" => plan_only = true,
                    "print-all" => print_all = true,
                    _ => problem_name = arg,
                }
            }
        }
        3 => {
            let arg1 = mut_args.drain(0..1).collect::<Vec<String>>()[0]
                .parse::<String>()
                .unwrap_or_default();
            let arg2 = mut_args.drain(0..1).collect::<Vec<String>>()[0]
                .parse::<String>()
                .unwrap_or_default();
            let arg3 = mut_args.drain(0..1).collect::<Vec<String>>()[0]
                .parse::<String>()
                .unwrap_or_default();
            for arg in vec![arg1, arg2, arg3] {
                match arg.as_str() {
                    "plan-only" => plan_only = true,
                    "print-all" => print_all = true,
                    _ => problem_name = arg,
                }
            }
        }
        _ => panic!("too many arguments"),
    }

    // yeah, parser exists now only for blocksworld...
    let problem = blocksworld::parser::parser(problem_name.as_str());
    // let problem = match problem_name.as_str() {
    //     "initial" => models::initial::raar_model(),
    //     "blocks_4_a" => models::blocksworld::instances::instance4::instance_4_a(),
    //     "blocks_4_b" => models::blocksworld::instances::instance4::instance_4_b(),
    //     "blocks_4_c" => models::blocksworld::instances::instance4::instance_4_c(),
    //     "blocks_5_a" => models::blocksworld::instances::instance5::instance_5_a(),
    //     _ => panic!("unknown model"),
    // };

    Args {
        plan_only: plan_only,
        print_all: print_all,
        problem: problem,
    }
}