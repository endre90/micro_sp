use super::*;
use itertools::sorted;
use z3_sys::*;
use z3_v2::*;
// mod models;

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

    let problem = match problem_name.as_str() {
        "initial" => models::initial::raar_model(),
        "blocks_4_a" => models::blocksworld::instances::instance4::instance_4_a(),
        "blocks_4_b" => models::blocksworld::instances::instance4::instance_4_b(),
        _ => panic!("unknown model"),
    };

    Args {
        plan_only: plan_only,
        print_all: print_all,
        problem: problem,
    }
}

/// Given a predicate, return a vector of variables that play a role in it.
pub fn get_predicate_vars(pred: &Predicate) -> Vec<EnumVariable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars(x)),
        Predicate::EQ(x) => s.push(x.var.clone()),
        Predicate::HOLD(x) => s.push(x.clone()),
        Predicate::PBEQ(x, _) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::EQRR(x, y) => {
            s.push(x.clone());
            s.push(y.clone());
        }
    }
    s.sort();
    s.dedup();
    s
}

/// Given a planning problem, return a vector of all variables defined for that problem.
pub fn get_problem_vars(prob: &PlanningProblem) -> Vec<EnumVariable> {
    let mut s = Vec::new();
    for t in &prob.trans {
        s.extend(get_predicate_vars(&t.guard));
        s.extend(get_predicate_vars(&t.update));
    }
    s.extend(get_predicate_vars(&prob.init));
    s.extend(get_predicate_vars(&prob.goal));
    s.sort();
    s.dedup();
    s
}

/// After the incremental algorithm has found a model it is unrolled into a plan.
pub fn get_planning_result(
    ctx: &ContextZ3,
    prob: &PlanningProblem,
    model: Z3_model,
    nr_steps: u32,
    planning_time: std::time::Duration,
    plan_found: bool,
) -> PlanningResult {
    let model_str = ModelToStringZ3::new(&ctx, model);
    let model_vec: Vec<Vec<&str>> = model_str
        .lines()
        .map(|l| l.split(" -> ").collect())
        .collect();
    let vars = get_problem_vars(&prob);

    // println!("MODEL:");
    // for m in &model_vec {
    //     println!("{:?}", m);
    // }

    let mut trace: Vec<PlanningFrame> = vec![];
    for i in 0..nr_steps - 1 {
        let enum_vals_source: Vec<EnumValue> = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_s{}", i)))
            .map(|x| (x[0].trim_end_matches(&format!("_s{}", i)), x[1], i))
            .map(|x| (vars.iter().find(|y| y.name == x.0).unwrap(), x.1))
            .map(|x| EnumValue::new(&x.0, x.1, None))
            .collect();

        let enum_vals_sink: Vec<EnumValue> = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_s{}", i + 1)))
            .map(|x| (x[0].trim_end_matches(&format!("_s{}", i + 1)), x[1], i + 1))
            .map(|x| (vars.iter().find(|y| y.name == x.0).unwrap(), x.1))
            .map(|x| EnumValue::new(&x.0, x.1, None))
            .collect();

        let measured_source: Vec<EnumValue> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Measured)
            .map(|y| y.clone())
            .collect::<Vec<EnumValue>>();
        let command_source: Vec<EnumValue> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Command)
            .map(|y| y.clone())
            .collect();
        let estimated_source: Vec<EnumValue> = enum_vals_source
            .iter()
            .filter(|x| x.var.kind == Kind::Estimated)
            .map(|y| y.clone())
            .collect();

        let measured_sink: Vec<EnumValue> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Measured)
            .map(|y| y.clone())
            .collect();
        let command_sink: Vec<EnumValue> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Command)
            .map(|y| y.clone())
            .collect();
        let estimated_sink: Vec<EnumValue> = enum_vals_sink
            .iter()
            .filter(|x| x.var.kind == Kind::Estimated)
            .map(|y| y.clone())
            .collect();

        let trans = model_vec
            .iter()
            .filter(|x| x[0].ends_with(&format!("_t{}", i + 1)))
            .map(|x| (x[0].trim_end_matches(&format!("_t{}", i + 1)), x[1], i + 1))
            .find(|x| x.1 == "true")
            .map(|z| z.0)
            .unwrap_or_default();

        trace.push(PlanningFrame {
            source: CompleteState::from_states(
                &State::new(&measured_source, &Kind::Measured),
                &State::new(&command_source, &Kind::Command),
                &State::new(&estimated_source, &Kind::Estimated),
            ),
            trans: String::from(trans),
            sink: CompleteState::from_states(
                &State::new(&measured_sink, &Kind::Measured),
                &State::new(&command_sink, &Kind::Command),
                &State::new(&estimated_sink, &Kind::Estimated),
            ),
        });
    }
    match plan_found {
        true => PlanningResult {
            plan_found: plan_found,
            plan_length: nr_steps - 1,
            trace: trace,
            time_to_solve: planning_time,
        },
        false => PlanningResult {
            plan_found: plan_found,
            plan_length: 0,
            trace: vec![],
            time_to_solve: planning_time,
        },
    }
}

/// For a given source state in a plan, return a corresponding sink state.
pub fn get_sink(result: &PlanningResult, source: &State) -> CompleteState {
    match source.kind == Kind::Measured {
        true => match result.trace.iter().find(|x| {
            sorted(x.source.measured.vec.clone()).collect::<Vec<EnumValue>>() == source.vec.clone()
        }) {
            Some(x) => x.sink.to_owned(),
            None => CompleteState::empty(),
        },
        false => panic!("asdf"),
    }
}

/// Generate a predicate from a given state as a conjunction of values.
pub fn state_to_predicate(state: &State) -> Predicate {
    Predicate::AND(
        state
            .vec
            .iter()
            .map(|x| {
                Predicate::EQ(EnumValue::new(
                    &EnumVariable::new(
                        &x.var.name,
                        &x.var.domain.iter().map(|x| x.as_str()).collect(),
                        &x.var.r#type,
                        Some(&x.var.param),
                        &x.var.kind,
                    ),
                    &x.val,
                    Some(&x.lifetime),
                ))
            })
            .collect::<Vec<Predicate>>(),
    )
}

/// Refence variables should take actual values when problem is refreshed
pub fn measured_to_command(state: &State, prob: &PlanningProblem) -> State {
    let cmd_vars: Vec<EnumVariable> = get_problem_vars(&prob)
        .iter()
        .filter(|x| x.kind == Kind::Command)
        .map(|x| x.to_owned())
        .collect();
    let mut mapped = vec![];
    for mv in &state.vec {
        let _q = cmd_vars
            .iter()
            .filter(|x| x.r#type == mv.var.r#type)
            .map(|y| mapped.push(EnumValue::new(&y, &mv.val, None)));
    }
    State::new(&mapped, &Kind::Command)
}

/// Generate a predicate from a complete state as a conjunction of values.
pub fn complete_state_to_predicate(state: &CompleteState) -> Predicate {
    Predicate::AND(vec![
        state_to_predicate(&state.measured),
        state_to_predicate(&state.command),
        state_to_predicate(&state.estimated),
    ])
}

/// When called, generate a new planning problem where the initial state is the current measured state.
/// When Paradigm::Raar, the reference variables should take values from their actual counterparts when
/// problem is refreshing (actually, maybe always, not only when Paradigm::Raar?. test).
pub fn refresh_problem(prob: &PlanningProblem, current: &State) -> PlanningProblem {
    match prob.paradigm {
        Paradigm::Raar => PlanningProblem {
            name: prob.name.to_owned(),
            init: Predicate::AND(vec![
                state_to_predicate(&current),
                state_to_predicate(&measured_to_command(&current, &prob)),
            ]),
            goal: prob.goal.to_owned(),
            trans: prob.trans.to_owned(),
            invar: prob.invar.to_owned(),
            max_steps: prob.max_steps,
            paradigm: prob.paradigm.to_owned(),
        },
        Paradigm::Invar => PlanningProblem {
            name: prob.name.to_owned(),
            init: state_to_predicate(&current),
            goal: prob.goal.to_owned(),
            trans: prob.trans.to_owned(),
            invar: prob.invar.to_owned(),
            max_steps: prob.max_steps,
            paradigm: prob.paradigm.to_owned(),
        },
    }
}

/// Pretty print a planning result.
pub fn pprint_result(result: &PlanningResult) -> () {
    println!("======================================================");
    println!("                   PLANNING RESULT                    ");
    println!("======================================================");
    println!("plan_found: {:?}", result.plan_found);
    println!("plan_lenght: {:?}", result.plan_length);
    println!("time_to_solve: {:?}", result.time_to_solve);
    println!("======================================================");
    for t in 0..result.trace.len() {
        println!("frame: {:?}", t);
        println!("trans: {:?}", result.trace[t].trans);
        println!("------------------------------------------------------");
        println!(
            "       | measured: {:?}",
            sorted(
                result.trace[t]
                    .source
                    .measured
                    .vec
                    .iter()
                    .map(|x| format!("{} -> {}", x.var.name, x.val))
            )
            .collect::<Vec<String>>()
        );
        println!(
            "source | command: {:?}",
            sorted(
                result.trace[t]
                    .source
                    .command
                    .vec
                    .iter()
                    .map(|x| format!("{} -> {}", x.var.name, x.val))
            )
            .collect::<Vec<String>>()
        );
        println!(
            "       | estimated: {:?}",
            sorted(
                result.trace[t]
                    .source
                    .estimated
                    .vec
                    .iter()
                    .map(|x| format!("{} -> {}", x.var.name, x.val))
            )
            .collect::<Vec<String>>()
        );
        println!("------------------------------------------------------");
        println!(
            "       | measured: {:?}",
            sorted(
                result.trace[t]
                    .sink
                    .measured
                    .vec
                    .iter()
                    .map(|x| format!("{} -> {}", x.var.name, x.val))
            )
            .collect::<Vec<String>>()
        );
        println!(
            " sink  | command: {:?}",
            sorted(
                result.trace[t]
                    .sink
                    .command
                    .vec
                    .iter()
                    .map(|x| format!("{} -> {}", x.var.name, x.val))
            )
            .collect::<Vec<String>>()
        );
        println!(
            "       | estimated: {:?}",
            sorted(
                result.trace[t]
                    .sink
                    .estimated
                    .vec
                    .iter()
                    .map(|x| format!("{} -> {}", x.var.name, x.val))
            )
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
    println!("plan_found: {:?}", result.plan_found);
    println!("plan_lenght: {:?}", result.plan_length);
    println!("time_to_solve: {:?}", result.time_to_solve);
    println!("======================================================");
    for t in 0..result.trace.len() {
        println!("frame: {:?}", t);
        println!("trans: {:?}", result.trace[t].trans);
        println!("------------------------------------------------------");
    }
    println!("                    END OF RESULT                     ");
    println!("======================================================");
}
