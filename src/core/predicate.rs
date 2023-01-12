use crate::{SPWrapped, SPVariable};
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
}

// TODO: clean from unwraps...
impl Predicate {
    pub fn eval(self, state: &State) -> bool {
        match self {
            Predicate::TRUE => true,
            Predicate::FALSE => false,
            Predicate::NOT(p) => !p.eval(&state.clone()),
            Predicate::AND(p) => p.iter().all(|pp| pp.clone().eval(&state)),
            Predicate::OR(p) => p.iter().any(|pp| pp.clone().eval(&state)),
            Predicate::EQ(x, y) => match x {
                SPCommon::SPVariable(vx) => match y {
                    SPCommon::SPVariable(vy) => {
                        state.state.clone().get(&vx.name).unwrap().val
                            == state.state.clone().get(&vy.name).unwrap().val
                    }
                    SPCommon::SPValue(vy) => state.state.clone().get(&vx.name).unwrap().val == vy,
                },
                SPCommon::SPValue(vx) => match y {
                    SPCommon::SPVariable(vy) => {
                        vx == state.state.clone().get(&vy.name).unwrap().val
                    }
                    SPCommon::SPValue(vy) => vx == vy,
                },
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
                SPCommon::SPVariable(vx) => s.push(vx.to_owned()),
                _ => (),
            }
            match y {
                SPCommon::SPVariable(vy) => s.push(vy.to_owned()),
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
        };

        write!(fmtr, "{}", &s)
    }
}

// pub fn predicate_to_state_space(pred: &Predicate) -> Vec<State> {
//     let mut temp = Vec::new();
//     let mut unresolved = Vec::new();
//     let mut resolved = Vec::new();
//     match pred {
//         Predicate::TRUE => {}
//         Predicate::FALSE => {}
//         Predicate::AND(x) => temp.extend(x.iter().flat_map(|p| predicate_to_state_space(p))),
//         Predicate::OR(x) => temp.extend(x.iter().flat_map(|p| predicate_to_state_space(p))),
//         Predicate::NOT(x) => temp.extend(predicate_to_state_space(x)),
//         Predicate::EQ(x, y) => match x {
//             SPCommon::SPVariable(vx) => match y {
//                 SPCommon::SPVariable(vy) => unresolved.push((vx.to_owned(), vy.to_owned())),
//                 SPCommon::SPValue(vy) => resolved.push((vx.to_owned(), vy.to_owned()))
//             },
//             SPCommon::SPValue(vx) => match y {
//                 SPCommon::SPVariable(vy) => resolved.push((vy.to_owned(), vx.to_owned())),
//                 SPCommon::SPValue(_) => ()
//             },
//         }
//     }

//     resolved.sort();
//     resolved.dedup();
//     unresolved.sort();
//     unresolved.dedup();

//     for u in unresolved {
//         let mut resolved_extension = vec!();
//         for r in resolved {
//             if u.0 == r.0 {
//                 resolved_extension.push((u.1, r.0))
//             }
//         }
//     }

//     // cant be hashmap because we have multiple resolutions
//     let mut res = resolved.iter().map(|(var, val)| (var.to_owned(), val.to_owned())).collect::<HashMap<SPVariable, SPValue>>();
//     let mut unres = unresolved.iter().map(|(var1, var2)| (var1.to_owned(), var2.to_owned())).collect::<HashMap<SPVariable, SPVariable>>();

//     for u in unres {
//         if u.0 in res {
//             res.get(u.0)
//         }
//         for r in res
//     }

//     vec!(State{state:res})

//     // make mutable hashmaps first...
//     // and then compare and see stuff

//     // for u in unresolved {
//     //     match u.0 ==
//     // }
//     // s
// }
