use crate::{SPVariable, SPWrapped, State};
use std::fmt;

/// A predicate is an equality logical formula that can evaluate to either true or false.
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Predicate {
    TRUE,
    FALSE,
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    EQ(SPWrapped, SPWrapped),
    NEQ(SPWrapped, SPWrapped),
    // TON(SPWrapped, SPWrapped),
    // TOFF(SPWrapped, SPWrapped),
}

impl Predicate {
    pub fn eval(self, state: &State) -> bool {
        match self {
            Predicate::TRUE => true,
            Predicate::FALSE => false,
            Predicate::NOT(p) => !p.eval(&state.clone()),
            Predicate::AND(p) => p.iter().all(|pp| pp.clone().eval(&state)),
            Predicate::OR(p) => p.iter().any(|pp| pp.clone().eval(&state)),
            Predicate::EQ(x, y) => match (x, y) {
                (SPWrapped::SPVariable(vx), SPWrapped::SPVariable(vy)) => {
                    state.get_value(&vx.name) == state.get_value(&vy.name)
                }
                (SPWrapped::SPVariable(vx), SPWrapped::SPValue(vy)) => {
                    state.get_value(&vx.name) == vy
                }
                (SPWrapped::SPValue(vx), SPWrapped::SPVariable(vy)) => {
                    vx == state.get_value(&vy.name)
                }
                (SPWrapped::SPValue(vx), SPWrapped::SPValue(vy)) => vx == vy,
            },
            Predicate::NEQ(x, y) => match (x, y) {
                (SPWrapped::SPVariable(vx), SPWrapped::SPVariable(vy)) => {
                    state.get_value(&vx.name) != state.get_value(&vy.name)
                }
                (SPWrapped::SPVariable(vx), SPWrapped::SPValue(vy)) => {
                    state.get_value(&vx.name) != vy
                }
                (SPWrapped::SPValue(vx), SPWrapped::SPVariable(vy)) => {
                    vx != state.get_value(&vy.name)
                }
                (SPWrapped::SPValue(vx), SPWrapped::SPValue(vy)) => vx != vy,
            },
        }
    }
}

pub fn get_predicate_vars(pred: &Predicate) -> Vec<SPVariable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars(x)),
        Predicate::EQ(x, y) => {
            match x {
                SPWrapped::SPVariable(vx) => s.push(vx.to_owned()),
                _ => (),
            }
            match y {
                SPWrapped::SPVariable(vy) => s.push(vy.to_owned()),
                _ => (),
            }
        }
        Predicate::NEQ(x, y) => {
            match x {
                SPWrapped::SPVariable(vx) => s.push(vx.to_owned()),
                _ => (),
            }
            match y {
                SPWrapped::SPVariable(vy) => s.push(vy.to_owned()),
                _ => (),
            }
        }
    }
    s.sort();
    s.dedup();
    s
}

impl fmt::Display for Predicate {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = match &self {
            Predicate::AND(x) => {
                let children: Vec<_> = x.iter().map(|p| format!("{}", p)).collect();
                format!("({})", children.join(" && "))
            }
            Predicate::OR(x) => {
                let children: Vec<_> = x.iter().map(|p| format!("{}", p)).collect();
                format!("({})", children.join(" || "))
            }
            Predicate::NOT(p) => format!("!({})", p),
            Predicate::TRUE => "TRUE".into(),
            Predicate::FALSE => "FALSE".into(),
            Predicate::EQ(x, y) => format!("{} = {}", x, y),
            Predicate::NEQ(x, y) => format!("{} != {}", x, y),
        };

        write!(fmtr, "{}", &s)
    }
}