use super::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct ArgsCLI {
    /// Online planning and acting
    #[structopt(long, short = "r", parse(try_from_str), default_value = "false")]
    pub run: bool,
    // Compositional planning algorithm
    #[structopt(long, short = "c", parse(try_from_str), default_value = "false")]
    pub comp: bool,
    /// Generate dummy driver (inverse micro_sp)
    #[structopt(long, short = "d", parse(try_from_str), default_value = "false")]
    pub dummy: bool,
    /// Print the states in the frames
    #[structopt(long, short = "p", parse(try_from_str), default_value = "false")]
    pub print: bool,
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
    pub comp: bool,
    pub dummy: bool,
    pub model: ParamPlanningProblem
}

pub fn handle_args() -> Args {
    let args = ArgsCLI::from_args();
    Args {
        run: args.run,
        print: args.print,
        comp: args.comp,
        dummy: args.dummy,
        model: match args.model.as_str() {
            "dummy_robot" => dummy_robot::model::model(args.instance.as_str()),
            "blocksworld" => match args.variant.as_str() {
                "enum_bool_invariants" => blocksworld::models::enum_bool_invariants::model(args.instance.as_str()),
                "bool_invariants" => blocksworld::models::bool_invariants::model(args.instance.as_str()),
                _ => panic!("unknown problem")
            }
            //"gripper" => gripper::parser::parser(args.instance.as_str()),
            _ => panic!("unknown problem")
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