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

pub fn frame_to_measured_state(vars: &Vec<EnumVariable>, vals: &Vec<&str>) -> State {
    State {
        vec: vars
            .iter()
            .filter(|c| c.kind == Kind::Measured)
            .map(|x| {
                EnumValue::new(
                    &x,
                    vals.iter()
                        .filter(|y| y.split(" -> ").collect::<Vec<&str>>()[0] == x.name)
                        .map(|z| z.split(" -> ").collect::<Vec<&str>>()[1])
                        .collect::<Vec<&str>>()[0],
                        None
                )
            })
            .collect(),
        kind: Kind::Measured,
    }
}

pub fn frame_to_command_state(vars: &Vec<EnumVariable>, vals: &Vec<&str>) -> State {
    State {
        vec: vars
            .iter()
            .filter(|c| c.kind == Kind::Command)
            .map(|x| {
                EnumValue::new(
                    &x,
                    vals.iter()
                        .filter(|y| y.split(" -> ").collect::<Vec<&str>>()[0] == x.name)
                        .map(|z| z.split(" -> ").collect::<Vec<&str>>()[1])
                        .collect::<Vec<&str>>()[0],
                        None
                )
            })
            .collect(),
        kind: Kind::Command,
    }
}

pub fn frame_to_estimated_state(vars: &Vec<EnumVariable>, vals: &Vec<&str>) -> State {
    State {
        vec: vars
            .iter()
            .filter(|c| c.kind == Kind::Estimated)
            .map(|x| {
                EnumValue::new(
                    &x,
                    vals.iter()
                        .filter(|y| y.split(" -> ").collect::<Vec<&str>>()[0] == x.name)
                        .map(|z| z.split(" -> ").collect::<Vec<&str>>()[1])
                        .collect::<Vec<&str>>()[0],
                        None
                )
            })
            .collect(),
        kind: Kind::Estimated,
    }
}

pub fn frame_to_complete_state(vars: &Vec<EnumVariable>, vals: &Vec<&str>) -> CompleteState {
    CompleteState {
        measured: frame_to_measured_state(&vars, &vals),
        command: frame_to_command_state(&vars, &vals),
        estimated: frame_to_estimated_state(&vars, &vals),
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
                source: frame_to_complete_state(
                    &vars,
                    &x.source.iter().map(|x| x.as_str()).collect(),
                ),
                trans: x.trans.clone(),
                sink: frame_to_complete_state(&vars, &x.sink.iter().map(|x| x.as_str()).collect()),
            })
            .collect(),
        time_to_solve: res.time_to_solve,
    }
}

pub fn get_sink(table: &PlanningResultStates, source: &State) -> CompleteState {
    // let untimed_source: Vec<(String, String)> = source.vec.iter().map(|x| (x.var.name, x.val)).collect();
    // let untimed_table = table.trace.iter().map(|x| PlanningFrameStates { source:  } x.source.measured.)
    match source.kind == Kind::Measured {
        true => match table.trace.iter().find(|x| x.source.measured.vec == source.vec.clone()) {
            Some(x) => x.sink.to_owned(),
            None => CompleteState::empty()
        },
        false => panic!("asdf"),
    }
}

pub fn measured_state_to_predicate(state: &State) -> Predicate {
    match state.kind {
        Kind::Measured => Predicate::AND(
            state
                .vec
                .iter()
                .map(|x| {
                    Predicate::EQ(
                        EnumValue::new(
                            &EnumVariable::new(
                                &x.var.name,
                                &x.var.domain.iter().map(|x| x.as_str()).collect(),
                                Some(&x.var.param),
                                &x.var.kind,
                            ),
                            &x.val,
                            Some(&x.lifetime)
                        )
                    )
                })
                .collect::<Vec<Predicate>>(),
        ),
        Kind::Command => panic!("not measured type"),
        Kind::Estimated => panic!("not measured type")
    }
}

pub fn refresh_problem(prob: &PlanningProblem, current: &State) -> PlanningProblem {
    PlanningProblem {
        name: prob.name.to_owned(),
        init: measured_state_to_predicate(&current),
        goal: prob.goal.to_owned(),
        trans: prob.trans.to_owned(),
        ltl_specs: prob.ltl_specs.to_owned(),
        max_steps: prob.max_steps,
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
