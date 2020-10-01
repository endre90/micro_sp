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
        Predicate::EQRL(x, _) => s.push(x.clone())
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

pub fn to_state(vars: &Vec<EnumVariable>, vals: &Vec<&str>) -> State {
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

pub fn result_to_states(
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
                source: to_state(&vars, &x.source.iter().map(|x| x.as_str()).collect()),
                trans: x.trans.clone(),
                sink: to_state(&vars, &x.sink.iter().map(|x| x.as_str()).collect()),
            })
            .collect(),
        time_to_solve: res.time_to_solve,
    }
}
