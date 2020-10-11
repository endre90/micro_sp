use super::*;
use z3_v2::*;
use z3_sys::*;
use itertools::sorted;

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

pub fn get_predicate_vars(pred: &Predicate) -> Vec<EnumVariable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars(x)),
        Predicate::EQ(x) => s.push(x.var.clone()),
    }
    s.sort();
    s.dedup();
    s
}

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
    PlanningResult {
        plan_found: plan_found,
        plan_length: nr_steps - 1,
        trace: trace,
        time_to_solve: planning_time,
    }
}

// pub fn get_sink(table: &PlanningResultStates, source: &State) -> CompleteState {
//     // let untimed_source: Vec<(String, String)> = source.vec.iter().map(|x| (x.var.name, x.val)).collect();
//     // let untimed_table = table.trace.iter().map(|x| PlanningFrameStates { source:  } x.source.measured.)
//     match source.kind == Kind::Measured {
//         true => match table.trace.iter().find(|x| x.source.measured.vec == source.vec.clone()) {
//             Some(x) => x.sink.to_owned(),
//             None => CompleteState::empty()
//         },
//         false => panic!("asdf"),
//     }
// }

// revisit this...
pub fn measured_state_to_predicate(state: &State) -> Predicate {
    match state.kind {
        Kind::Measured => Predicate::AND(
            state
                .vec
                .iter()
                .map(|x| {
                    Predicate::EQ(EnumValue::new(
                        &EnumVariable::new(
                            &x.var.name,
                            &x.var.domain.iter().map(|x| x.as_str()).collect(),
                            Some(&x.var.param),
                            &x.var.kind,
                        ),
                        &x.val,
                        Some(&x.lifetime),
                    ))
                })
                .collect::<Vec<Predicate>>(),
        ),
        Kind::Command => panic!("not measured type"),
        Kind::Estimated => panic!("not measured type"),
    }
}

pub fn refresh_problem(prob: &PlanningProblem, current: &State) -> PlanningProblem {
    PlanningProblem {
        name: prob.name.to_owned(),
        init: measured_state_to_predicate(&current),
        goal: prob.goal.to_owned(),
        trans: prob.trans.to_owned(),
        max_steps: prob.max_steps,
    }
}

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
