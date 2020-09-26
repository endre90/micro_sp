use super::*;

pub trait IterOps<T, I>: IntoIterator<Item = T>
    where I: IntoIterator<Item = T>,
          T: PartialEq {
    fn intersect(self, other: I) -> Vec<T>;
    fn difference(self, other: I) -> Vec<T>;
}

impl<T, I> IterOps<T, I> for I
    where I: IntoIterator<Item = T>,
          T: PartialEq
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
        Predicate::TRUE => {},
        Predicate::FALSE => {},
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars(x)),
        Predicate::EQRL(x, _) => s.push(x.clone()),
        Predicate::EQRR(x, y) => {
            s.push(x.clone());
            s.push(y.clone());
        }
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

pub fn result_to_states(result: &PlanningResult) -> Vec<(State, State)> {
    let mut table = vec!();
    for r in &result.trace {
        let mut source_kvps = vec!();
        let mut sink_kvps = vec!();
        for s in &r.source {
            let sep: Vec<&str> = s.split(" -> ").collect();
            source_kvps.push(KeyValuePair::new(sep[0], sep[1]));
        }
        for s in &r.sink {
            let sep: Vec<&str> = s.split(" -> ").collect();
            sink_kvps.push(KeyValuePair::new(sep[0], sep[1]));
        }
        table.push((State::new(&source_kvps), State::new(&sink_kvps)));
    }
    table
}

// maybe write some more tests for this fn
#[test]
fn test_get_predicate_vars(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let z = EnumVariable::new("z", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);

    let n = Predicate::AND(vec!(Predicate::EQRR(x, y.clone()), Predicate::EQRR(y, z)));

    println!("predicate: {:?}", n);
    let vars = get_predicate_vars(&n);
    for var in vars {
        println!("var: {:?}", var);
    }
}