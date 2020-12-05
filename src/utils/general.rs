use super::*;
use itertools::sorted;
use structopt::StructOpt;

use std::time::Duration;
use std::time::Instant;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct ArgsCLI {
    /// Online planning and acting
    #[structopt(long, short = "r", parse(try_from_str), default_value = "false")]
    pub run: bool,
    /// Planning algorithm
    #[structopt(long, short = "a", default_value = "inc")]
    pub alg: String,
    /// Generate dummy driver (inverse micro_sp)
    #[structopt(long, short = "d", parse(try_from_str), default_value = "false")]
    pub dummy: bool,
    /// Print the states in the frames
    #[structopt(long, short = "p", parse(try_from_str), default_value = "false")]
    pub print: bool,
    /// Save the result to a file
    #[structopt(long, short = "f", parse(try_from_str), default_value = "false")]
    pub filesave: bool,
    /// The name of the model to run
    #[structopt(long, short = "m")]
    pub model: String,
    /// The variant of the model to run
    #[structopt(long, short = "v")]
    pub variant: String,
    /// The name of the instance to run
    #[structopt(long, short = "i")]
    pub instance: String,
    /// Timeout in seconds
    #[structopt(long, short = "t", parse(try_from_str), default_value = "300")]
    pub timeout: u64,
    /// Limit the number of steps
    #[structopt(long, short = "s", parse(try_from_str), default_value = "1000")]
    pub max_steps: u64,
    /// Specialize the solver
    #[structopt(long, short = "l", parse(try_from_str), default_value = "default")]
    pub logic: String,
}

pub struct Args {
    pub run: bool,
    pub print: bool,
    pub filesave: bool,
    pub alg: String,
    pub dummy: bool,
    pub model: ParamPlanningProblem,
    pub timeout: u64,
    pub max_steps: u64,
    pub logic: String
}

pub fn handle_args() -> Args {
    let args = ArgsCLI::from_args();
    Args {
        run: args.run,
        print: args.print,
        filesave: args.filesave,
        alg: args.alg,
        dummy: args.dummy,
        model: match args.model.as_str() {
            "dummy_robot" => match args.variant.as_str() {
                "model1" => dummy_robot::model::model1(args.instance.as_str()),
                "model2" => dummy_robot::model::model2(args.instance.as_str()),
                _ => panic!("unknown problem"),
            }
            "blocksworld" => match args.variant.as_str() {
                "bool_explicit" => {
                    blocksworld::models::bool_explicit::model(args.instance.as_str())
                }
                "bool_invariant" => {
                    blocksworld::models::bool_invariants::model(args.instance.as_str())
                }
                "enum_invariant" => {
                    blocksworld::models::enum_invariants::model(args.instance.as_str())
                }
                _ => panic!("unknown problem"),
            },
            "gripper" => match args.variant.as_str() {
                "bool_explicit" => {
                    gripper::models::bool_explicit::model(args.instance.as_str())
                },
                _ => panic!("unknown problem"),
            },
            _ => panic!("unknown problem"),
        },
        timeout: args.timeout,
        max_steps: args.max_steps,
        logic: args.logic
    }
}

pub trait IterOps<T, I>: IntoIterator<Item = T>
where
    I: IntoIterator<Item = T>,
    T: PartialEq,
{
    fn intersect(self, other: I) -> Vec<T>;
    fn difference(self, other: I) -> Vec<T>;
}

impl<T, I> IterOps<T, I> for I
where
    I: IntoIterator<Item = T>,
    T: PartialEq,
{
    /// Gets the intersection of two vectors.
    fn intersect(self, other: I) -> Vec<T> {
        let mut common = Vec::new();
        let mut v_other: Vec<_> = other.into_iter().collect();

        for e1 in self.into_iter() {
            if let Some(pos) = v_other.iter().position(|e2| e1 == *e2) {
                common.push(e1);
                v_other.remove(pos);
            }
        }

        common
    }

    /// Gets the diff of two vectors.
    fn difference(self, other: I) -> Vec<T> {
        let mut diff = Vec::new();
        let mut v_other: Vec<_> = other.into_iter().collect();

        for e1 in self.into_iter() {
            if let Some(pos) = v_other.iter().position(|e2| e1 == *e2) {
                v_other.remove(pos);
            } else {
                diff.push(e1);
            }
        }

        diff.append(&mut v_other);
        diff
    }
}

/// Pretty print a planning result.
pub fn pprint_result(result: &PlanningResult) -> () {
    println!("======================================================");
    println!("                   PLANNING RESULT                    ");
    println!("======================================================");
    println!("name: {:?}", result.name);
    println!("found: {:?}", result.plan_found);
    println!("lenght: {:?}", result.plan_length);
    println!("time: {:?}", result.time_to_solve);
    println!("size: {:?} MB", result.model_size / 1000000);
    println!("======================================================");
    for t in 0..result.trace.len() {
        println!("frame: {:?}", t);
        println!("trans: {:?}", result.trace[t].trans);
        println!("------------------------------------------------------");
        println!(
            "       | measured:  {:?}",
            sorted(result.trace[t].source.measured.iter().map(|x| format!(
                "{} -> {}",
                x.var.name,
                match x.val.clone() {
                    SPValue::Bool(x) => format!("{}", x),
                    SPValue::String(x) => format!("{}", x),
                }
            )))
            .collect::<Vec<String>>()
        );
        // println!(
        //     "source | handshake: {:?}",
        //     sorted(
        //         result.trace[t]
        //             .source
        //             .handshake
        //             .vec
        //             .iter()
        //             .map(|x| format!("{} -> {}", x.var.name, x.val))
        //     )
        //     .collect::<Vec<String>>()
        // );
        println!(
            "source | command:   {:?}",
            sorted(result.trace[t].source.command.iter().map(|x| format!(
                "{} -> {}",
                x.var.name,
                match x.val.clone() {
                    SPValue::Bool(x) => format!("{}", x),
                    SPValue::String(x) => format!("{}", x),
                }
            )))
            .collect::<Vec<String>>()
        );
        println!(
            "       | estimated: {:?}",
            sorted(result.trace[t].source.estimated.iter().map(|x| format!(
                "{} -> {}",
                x.var.name,
                match x.val.clone() {
                    SPValue::Bool(x) => format!("{}", x),
                    SPValue::String(x) => format!("{}", x),
                }
            )))
            .collect::<Vec<String>>()
        );
        println!("------------------------------------------------------");
        println!(
            "       | measured:  {:?}",
            sorted(result.trace[t].sink.measured.iter().map(|x| format!(
                "{} -> {}",
                x.var.name,
                match x.val.clone() {
                    SPValue::Bool(x) => format!("{}", x),
                    SPValue::String(x) => format!("{}", x),
                }
            )))
            .collect::<Vec<String>>()
        );
        // println!(
        //     " sink  | handshake: {:?}",
        //     sorted(
        //         result.trace[t]
        //             .sink
        //             .handshake
        //             .vec
        //             .iter()
        //             .map(|x| format!("{} -> {}", x.var.name, x.val))
        //     )
        //     .collect::<Vec<String>>()
        // );
        println!(
            " sink  | command:   {:?}",
            sorted(result.trace[t].sink.command.iter().map(|x| format!(
                "{} -> {}",
                x.var.name,
                match x.val.clone() {
                    SPValue::Bool(x) => format!("{}", x),
                    SPValue::String(x) => format!("{}", x),
                }
            )))
            .collect::<Vec<String>>()
        );
        println!(
            "       | estimated: {:?}",
            sorted(result.trace[t].sink.estimated.iter().map(|x| format!(
                "{} -> {}",
                x.var.name,
                match x.val.clone() {
                    SPValue::Bool(x) => format!("{}", x),
                    SPValue::String(x) => format!("{}", x),
                }
            )))
            .collect::<Vec<String>>()
        );
        println!("======================================================");
    }
    println!("                    END OF RESULT                     ");
    println!("======================================================");
    let delooped = remove_loops(&result);
}

/// Pretty print a planning result.
pub fn pprint_result_trans_only(result: &PlanningResult) -> () {
    println!("======================================================");
    println!("                   PLANNING RESULT                    ");
    println!("======================================================");
    println!("name: {:?}", result.name);
    println!("algo: {:?}", result.alg);
    println!("found: {:?}", result.plan_found);
    println!("lenght: {:?}", result.plan_length);
    println!("time: {:?}", result.time_to_solve);
    println!("size: {:?} MB", result.model_size / 1000000);
    println!("======================================================");
    for t in 0..result.trace.len() {
        // println!("frame: {:?}", t);
        println!("{:?}: {:?}", t, result.trace[t].trans);
        println!("------------------------------------------------------");
    }
    println!("                    END OF RESULT                     ");
    println!("======================================================");

    let now = Instant::now();
    let delooped = remove_loops(&result);

    println!("======================================================");
    println!("              DELOOPED PLANNING RESULT                ");
    println!("======================================================");
    println!("name: {:?}", delooped.name);
    println!("algo: {:?}", delooped.alg);
    println!("found: {:?}", delooped.plan_found);
    println!("lenght: {:?}", delooped.plan_length);
    println!("time: {:?}", delooped.time_to_solve);
    println!("delooping_time: {:?}", now.elapsed());
    println!("size: {:?} MB", delooped.model_size / 1000000);
    println!("======================================================");
    for t in 0..delooped.trace.len() {
        // println!("frame: {:?}", t);
        println!("{:?}: {:?}", t, delooped.trace[t].trans);
        println!("------------------------------------------------------");
    }
    println!("                    END OF RESULT                     ");
    println!("======================================================");

}



// /// Pretty print a planning result to a file.
pub fn pprint_result_to_file(result: &PlanningResult) -> () {
    let filename = format!("{}_{}.txt", result.name, result.alg);
    let path = Path::new(&filename);
    let display = path.display();
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };
    file.write("======================================================\n".as_bytes()).ok();
    file.write("                   PLANNING RESULT                    \n".as_bytes()).ok();
    file.write("======================================================\n".as_bytes()).ok();
    file.write(&format!("name: {:?}\n", result.name).as_bytes()).ok();
    file.write(&format!("algo: {:?}\n", result.alg).as_bytes()).ok();
    file.write(&format!("found: {:?}\n", result.plan_found).as_bytes()).ok();
    file.write(&format!("lenght: {:?}\n", result.plan_length).as_bytes()).ok();
    file.write(&format!("time: {:?}\n", result.time_to_solve).as_bytes()).ok();
    file.write(&format!("size: {:?} MB", result.model_size / 1000000).as_bytes()).ok();
    file.write("======================================================\n".as_bytes()).ok();
    for t in 0..result.trace.len() {
        // file.write(&format!("frame: {:?}\n", t).as_bytes()).ok();
        file.write(&format!("{:?}: {:?}\n", t, result.trace[t].trans).as_bytes()).ok();
        file.write("------------------------------------------------------\n".as_bytes()).ok();
    }
    file.write("                    END OF RESULT                     \n".as_bytes()).ok();
    file.write("======================================================\n".as_bytes()).ok();
    
}
