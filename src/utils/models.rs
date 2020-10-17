use super::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct ArgsCLI {
    #[structopt(long, parse(try_from_str), default_value = "true")]
    pub plan_only: bool,
    #[structopt(long, parse(try_from_str), default_value = "false")]
    pub print_all: bool,
    #[structopt(long)]
    pub problem: String,
    #[structopt(long)]
    pub instance: String,
}

pub struct Args {
    pub plan_only: bool,
    pub print_all: bool,
    pub problem: PlanningProblem,
}

pub fn handle_args_2() -> Args {
    let args = ArgsCLI::from_args();
    Args {
        plan_only: args.plan_only,
        print_all: args.print_all,
        problem: match args.problem.as_str() {
            "initial" => models::initial::raar_model(),
            "blocksworld" => blocksworld::parser::parser(args.instance.as_str()),
            _ => panic!("unknown problem")
        },
    }
}