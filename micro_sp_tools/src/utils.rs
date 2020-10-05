use super::*;

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
        Predicate::EQRL(x, _) => s.push(x.clone()),
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

pub fn frame_to_state(vars: &Vec<EnumVariable>, vals: &Vec<&str>) -> State {
    State {
        measured: vars
            .iter()
            .filter(|c| c.kind == ControlKind::Measured)
            .map(|x| {
                EnumVariableValue::new(
                    &x,
                    vals.iter()
                        .filter(|y| y.split(" -> ").collect::<Vec<&str>>()[0] == x.name)
                        .map(|z| z.split(" -> ").collect::<Vec<&str>>()[1])
                        .collect::<Vec<&str>>()[0],
                )
            })
            .collect(),
        command: vars
            .iter()
            .filter(|c| c.kind == ControlKind::Command)
            .map(|x| {
                EnumVariableValue::new(
                    &x,
                    vals.iter()
                        .filter(|y| y.split(" -> ").collect::<Vec<&str>>()[0] == x.name)
                        .map(|z| z.split(" -> ").collect::<Vec<&str>>()[1])
                        .collect::<Vec<&str>>()[0],
                )
            })
            .collect(),
        estimated: vars
            .iter()
            .filter(|c| c.kind == ControlKind::Estimated)
            .map(|x| {
                EnumVariableValue::new(
                    &x,
                    vals.iter()
                        .filter(|y| y.split(" -> ").collect::<Vec<&str>>()[0] == x.name)
                        .map(|z| z.split(" -> ").collect::<Vec<&str>>()[1])
                        .collect::<Vec<&str>>()[0],
                )
            })
            .collect(),
    }
}

pub fn result_to_table(
    prob: &PlanningProblem,
    res: &PlanningResultStrings,
) -> PlanningResultStates {
    let vars = get_problem_vars(&prob);
    PlanningResultStates {
        plan_found: res.plan_found,
        plan_length: res.plan_length,
        trace: res
            .trace
            .iter()
            .map(|x| PlanningFrameStates {
                source: frame_to_state(&vars, &x.source.iter().map(|x| x.as_str()).collect()),
                trans: x.trans.clone(),
                sink: frame_to_state(&vars, &x.sink.iter().map(|x| x.as_str()).collect()),
            })
            .collect(),
        time_to_solve: res.time_to_solve,
    }
}

pub fn get_sink(table: &PlanningResultStates, source: &State) -> State {
    match table
        .trace
        .iter()
        .find(|x| x.source.measured == source.measured)
    {
        Some(x) => x.sink.to_owned(),
        None => State::new(),
    }
}

pub fn state_to_predicate(state: &State, kind: &ControlKind) -> Predicate {
    match kind {
        ControlKind::Measured => Predicate::AND(
            state
                .measured
                .iter()
                .map(|x| {
                    Predicate::EQRL(
                        EnumVariable::new(
                            &x.var.name,
                            &x.var.domain.iter().map(|x| x.as_str()).collect(),
                            Some(&x.var.param),
                            Some(&x.var.kind),
                        ),
                        x.val.to_owned(),
                    )
                })
                .collect::<Vec<Predicate>>(),
        ),
        ControlKind::Estimated => Predicate::AND(
            state
                .estimated
                .iter()
                .map(|x| {
                    Predicate::EQRL(
                        EnumVariable::new(
                            &x.var.name,
                            &x.var.domain.iter().map(|x| x.as_str()).collect(),
                            Some(&x.var.param),
                            Some(&x.var.kind),
                        ),
                        x.val.to_owned(),
                    )
                })
                .collect::<Vec<Predicate>>(),
        ),
        ControlKind::Command => Predicate::AND(
            state
                .command
                .iter()
                .map(|x| {
                    Predicate::EQRL(
                        EnumVariable::new(
                            &x.var.name,
                            &x.var.domain.iter().map(|x| x.as_str()).collect(),
                            Some(&x.var.param),
                            Some(&x.var.kind),
                        ),
                        x.val.to_owned(),
                    )
                })
                .collect::<Vec<Predicate>>(),
        ),
        ControlKind::None => Predicate::AND(
            vec!(
                state_to_predicate(&state, &ControlKind::Measured),
                state_to_predicate(&state, &ControlKind::Estimated),
                state_to_predicate(&state, &ControlKind::Command)
            )
        )
    }
}

pub fn refresh_problem(prob: &PlanningProblem, current: &State) -> PlanningProblem {
    PlanningProblem {
        name: prob.name.to_owned(),
        init: state_to_predicate(&current, &ControlKind::Measured),
        goal: prob.goal.to_owned(),
        trans: prob.trans.to_owned(),
        ltl_specs: prob.ltl_specs.to_owned(),
        max_steps: prob.max_steps

    }
}

pub fn pprint_result(result: &PlanningResultStrings) -> () {
    println!("\n");
    println!("============================================");
    println!("              PLANNING RESULT               ");
    println!("============================================");
    println!("plan_found: {:?}", result.plan_found);
    println!("plan_lenght: {:?}", result.plan_length);
    println!("time_to_solve: {:?}", result.time_to_solve);
    println!("============================================");
    for t in 0..result.trace.len() {
        println!("frame: {:?}", t);
        println!("source: {:?}", result.trace[t].source);
        println!("trans: {:?}", result.trace[t].trans);
        println!("sink: {:?}", result.trace[t].sink);
        println!("============================================");
    }
    println!("               END OF RESULT                ");
    println!("============================================");
}
