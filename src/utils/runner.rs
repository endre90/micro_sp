use super::*;
use itertools::sorted;

/// For a given source state in a plan, return a corresponding sink state.
pub fn get_sink(
    result: &PlanningResult,
    measured_source: &State,
    command_source: &State,
) -> CompleteState {
    match measured_source.kind == Kind::Measured && command_source.kind == Kind::Command {
        true => match result.trace.iter().find(|x| {
            sorted(x.source.measured.vec.clone()).collect::<Vec<EnumValue>>()
                == measured_source.vec.clone()
                && sorted(x.source.command.vec.clone()).collect::<Vec<EnumValue>>()
                    == command_source.vec.clone()
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
                Predicate::SET(EnumValue::new(
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
        // state_to_predicate(&state.handshake),
        state_to_predicate(&state.command),
        state_to_predicate(&state.estimated),
    ])
}

/// When called, generate a new planning problem where the initial state is the current measured state.
/// When Paradigm::Raar, the reference variables should take values from their actual counterparts when
/// problem is refreshing (actually, maybe always, not only when Paradigm::Raar?. test).
pub fn refresh_problem(
    prob: &PlanningProblem,
    current_measured: &State,
    current_command: &State,
) -> PlanningProblem {
    match prob.paradigm {
        Paradigm::Raar => PlanningProblem {
            name: prob.name.to_owned(),
            init: Predicate::AND(vec![
                state_to_predicate(&current_measured),
                state_to_predicate(&current_command),
                // state_to_predicate(&measured_to_command(&current_measured, &prob)),
            ]),
            goal: prob.goal.to_owned(),
            trans: prob.trans.to_owned(),
            invars: prob.invars.to_owned(),
            max_steps: prob.max_steps,
            paradigm: prob.paradigm.to_owned(),
        },
        Paradigm::Invar => PlanningProblem {
            name: prob.name.to_owned(),
            init: Predicate::AND(vec![
                state_to_predicate(&current_measured),
                state_to_predicate(&current_command),
            ]),
            goal: prob.goal.to_owned(),
            trans: prob.trans.to_owned(),
            invars: prob.invars.to_owned(),
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
            "       | measured:  {:?}",
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
            "       | measured:  {:?}",
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
