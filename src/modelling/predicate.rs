use serde::{Deserialize, Serialize};

use crate::{SPVariable, SPVariableType, SPWrapped, State};
use std::fmt;

/// A predicate is an equality logical formula that can evaluate to either true or false.
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum Predicate {
    TRUE,
    FALSE,
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    EQ(SPWrapped, SPWrapped),
    NEQ(SPWrapped, SPWrapped),
}

impl Predicate {
    /// Evaluate a predicate based on the given state.
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

    /// Keep only the variables in the predicate from the `only` list.
    pub fn keep_only(&self, only: &Vec<String>) -> Option<Predicate> {
        match self {
            Predicate::TRUE => Some(Predicate::TRUE),
            Predicate::FALSE => Some(Predicate::FALSE),
            Predicate::NOT(x) => match x.keep_only(only) {
                Some(x) => Some(Predicate::NOT(Box::new(x))),
                None => None,
            },
            Predicate::AND(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().keep_only(only)).collect();
                new.dedup();
                if new.len() == 0 {
                    None
                } else if new.len() == 1 {
                    Some(new[0].clone())
                } else {
                    Some(Predicate::AND(new))
                }
            }
            Predicate::OR(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().keep_only(only)).collect();
                new.dedup();
                if new.len() == 0 {
                    None
                } else if new.len() == 1 {
                    Some(new[0].clone())
                } else {
                    Some(Predicate::OR(new))
                }
            }
            Predicate::EQ(x, y) | Predicate::NEQ(x, y) => {
                let remove_x = match x {
                    SPWrapped::SPValue(_) => false,
                    SPWrapped::SPVariable(vx) => !only.contains(&vx.name),
                };
                let remove_y = match y {
                    SPWrapped::SPValue(_) => false,
                    SPWrapped::SPVariable(vy) => !only.contains(&vy.name),
                };

                if remove_x || remove_y {
                    None
                } else {
                    Some(self.clone())
                }
            }
        }
    }

    /// Remove the variables in the predicate from the `remove` list.
    pub fn remove(&self, remove: &Vec<String>) -> Option<Predicate> {
        match self {
            Predicate::TRUE => Some(Predicate::TRUE),
            Predicate::FALSE => Some(Predicate::FALSE),
            Predicate::NOT(x) => match x.remove(remove) {
                Some(x) => Some(Predicate::NOT(Box::new(x))),
                None => None,
            },
            Predicate::AND(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().remove(remove)).collect();
                new.dedup();
                if new.len() == 0 {
                    None
                } else if new.len() == 1 {
                    Some(new[0].clone())
                } else {
                    Some(Predicate::AND(new))
                }
            }
            Predicate::OR(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().remove(remove)).collect();
                new.dedup();
                if new.len() == 0 {
                    None
                } else if new.len() == 1 {
                    Some(new[0].clone())
                } else {
                    Some(Predicate::OR(new))
                }
            }
            Predicate::EQ(x, y) | Predicate::NEQ(x, y) => {
                let remove_x = match x {
                    SPWrapped::SPValue(_) => false,
                    SPWrapped::SPVariable(vx) => remove.contains(&vx.name),
                };
                let remove_y = match y {
                    SPWrapped::SPValue(_) => false,
                    SPWrapped::SPVariable(vy) => remove.contains(&vy.name),
                };

                if remove_x || remove_y {
                    None
                } else {
                    Some(self.clone())
                }
            }
        }
    }
}

// TODO: test...
pub fn get_predicate_vars_all(pred: &Predicate) -> Vec<SPVariable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars_all(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars_all(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars_all(x)),
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

// TODO: test...
pub fn get_predicate_vars_planner(pred: &Predicate) -> Vec<SPVariable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars_planner(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars_planner(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars_planner(x)),
        Predicate::EQ(x, y) | Predicate::NEQ(x, y) => {
            match x {
                SPWrapped::SPVariable(vx) => match vx.variable_type {
                    SPVariableType::Runner => (),
                    _ => s.push(vx.to_owned()),
                },
                _ => (),
            }
            match y {
                SPWrapped::SPVariable(vy) => match vy.variable_type {
                    SPVariableType::Runner => (),
                    _ => s.push(vy.to_owned()),
                },
                _ => (),
            }
        }
    }
    s.sort();
    s.dedup();
    s
}

// TODO: test...
pub fn get_predicate_vars_runner(pred: &Predicate) -> Vec<SPVariable> {
    let mut s = Vec::new();
    match pred {
        Predicate::TRUE => {}
        Predicate::FALSE => {}
        Predicate::AND(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars_runner(p))),
        Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars_runner(p))),
        Predicate::NOT(x) => s.extend(get_predicate_vars_runner(x)),
        Predicate::EQ(x, y) => {
            match x {
                SPWrapped::SPVariable(vx) => match vx.variable_type {
                    SPVariableType::Runner => s.push(vx.to_owned()),
                    _ => (),
                },
                _ => (),
            }
            match y {
                SPWrapped::SPVariable(vy) => match vy.variable_type {
                    SPVariableType::Runner => s.push(vy.to_owned()),
                    _ => (),
                },
                _ => (),
            }
        }
        Predicate::NEQ(x, y) => {
            match x {
                SPWrapped::SPVariable(vx) => match vx.variable_type {
                    SPVariableType::Runner => s.push(vx.to_owned()),
                    _ => (),
                },
                _ => (),
            }
            match y {
                SPWrapped::SPVariable(vy) => match vy.variable_type {
                    SPVariableType::Runner => s.push(vy.to_owned()),
                    _ => (),
                },
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
