use super::*;

pub struct GetPredicateVars {
    pub pred: Predicate,
    pub vars: Vec<EnumVariable>
}

pub struct GetProblemVars {
    pub pred: PlanningProblem,
    pub vars: Vec<EnumVariable>
}

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

impl GetPredicateVars {
    pub fn new(pred: &Predicate) -> Vec<EnumVariable> {
        let mut s = Vec::new();
        match pred {
            Predicate::TRUE => {},
            Predicate::FALSE => {},
            Predicate::AND(x) => s.extend(x.iter().flat_map(|p| GetPredicateVars::new(p))),
            Predicate::OR(x) => s.extend(x.iter().flat_map(|p| GetPredicateVars::new(p))),
            Predicate::NOT(x) => s.extend(GetPredicateVars::new(x)),
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
}

impl GetProblemVars {
    pub fn new(prob: &PlanningProblem) -> Vec<EnumVariable> {
        let mut s = Vec::new();
        for t in &prob.trans {
            s.extend(GetPredicateVars::new(&t.guard));
            s.extend(GetPredicateVars::new(&t.update));
        }
        s.extend(GetPredicateVars::new(&prob.init));
        s.extend(GetPredicateVars::new(&prob.goal));
        s.sort();
        s.dedup();
        s
    }
}

// maybe write some more tests for this fn
#[test]
fn test_get_predicate_vars(){

    let x = EnumVariable::new("x", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let y = EnumVariable::new("y", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);
    let z = EnumVariable::new("z", "letters", &vec!("a", "b", "c", "d"), None, &ControlKind::None);

    let n = Predicate::AND(vec!(Predicate::EQRR(x, y.clone()), Predicate::EQRR(y, z)));

    println!("predicate: {:?}", n);
    let vars = GetPredicateVars::new(&n);
    for var in vars {
        println!("var: {:?}", var);
    }
}