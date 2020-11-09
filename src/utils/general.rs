use super::*;
use itertools::sorted;
use structopt::StructOpt;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct ArgsCLI {
    /// Online planning and acting
    #[structopt(long, short = "r", parse(try_from_str), default_value = "false")]
    pub run: bool,
    /// Compositional planning algorithm
    #[structopt(long, short = "c", parse(try_from_str), default_value = "false")]
    pub comp: bool,
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
}

pub struct Args {
    pub run: bool,
    pub print: bool,
    pub filesave: bool,
    pub comp: bool,
    pub dummy: bool,
    pub model: ParamPlanningProblem,
}

pub fn handle_args() -> Args {
    let args = ArgsCLI::from_args();
    Args {
        run: args.run,
        print: args.print,
        filesave: args.filesave,
        comp: args.comp,
        dummy: args.dummy,
        model: match args.model.as_str() {
            "dummy_robot" => dummy_robot::model::model(args.instance.as_str()),
            "blocksworld" => match args.variant.as_str() {
                "enum_bool_explicit" => {
                    blocksworld::models::enum_bool_explicit::model(args.instance.as_str())
                }
                "enum_bool_invariant" => {
                    blocksworld::models::enum_bool_invariants::model(args.instance.as_str())
                }
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
            //"gripper" => gripper::parser::parser(args.instance.as_str()),
            _ => panic!("unknown problem"),
        },
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
    println!("plan_found: {:?}", result.plan_found);
    println!("plan_lenght: {:?}", result.plan_length);
    println!("time_to_solve: {:?}", result.time_to_solve);
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
}

/// Pretty print a planning result.
pub fn pprint_result_trans_only(result: &PlanningResult) -> () {
    println!("======================================================");
    println!("                   PLANNING RESULT                    ");
    println!("======================================================");
    println!("name: {:?}", result.name);
    println!("found: {:?}", result.plan_found);
    println!("lenght: {:?}", result.plan_length);
    // println!("time: {:?}", result.time_to_solve);
    println!("======================================================");
    for t in 0..result.trace.len() {
        // println!("frame: {:?}", t);
        println!("{:?}: {:?}", t, result.trace[t].trans);
        println!("------------------------------------------------------");
    }
    println!("                    END OF RESULT                     ");
    println!("======================================================");
}

// /// Pretty print a planning result to a file.
pub fn pprint_result_to_file(result: &PlanningResult) -> () {
    let filename = format!("{}.txt", result.name);
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
    file.write(&format!("found: {:?}\n", result.plan_found).as_bytes()).ok();
    file.write(&format!("lenght: {:?}\n", result.plan_length).as_bytes()).ok();
    // file.write(&format!("time: {:?}\n", result.time_to_solve).as_bytes()).ok();
    file.write("======================================================\n".as_bytes()).ok();
    for t in 0..result.trace.len() {
        // file.write(&format!("frame: {:?}\n", t).as_bytes()).ok();
        file.write(&format!("{:?}: {:?}\n", t, result.trace[t].trans).as_bytes()).ok();
        file.write("------------------------------------------------------\n".as_bytes()).ok();
    }
    file.write("                    END OF RESULT                     \n".as_bytes()).ok();
    file.write("======================================================\n".as_bytes()).ok();
    
}
