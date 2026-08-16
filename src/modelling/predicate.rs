use serde::{Deserialize, Serialize};

// use crate::{SPVariable, SPWrapped, State};
use crate::*;
use std::fmt;

/// A predicate is an equality logical formula that can evaluate to either true or false.
/// An equality logic formula F is defined with the following grammar:
///     F : F ∧ F | F ∨ F | ¬F | atom
///     atom : term == term | true | false
///     term : variable | value
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum Predicate {
    TRUE,
    FALSE,
    NOT(Box<Predicate>),
    AND(Vec<Predicate>),
    OR(Vec<Predicate>),
    EQ(SPWrapped, SPWrapped),
    NEQ(SPWrapped, SPWrapped),
    LTEQ(SPWrapped, SPWrapped),
    GTEQ(SPWrapped, SPWrapped),
    LT(SPWrapped, SPWrapped),
    GT(SPWrapped, SPWrapped),
}

impl Predicate {
    // DONE: PERF: taking `self` by value was the root cause of a large fraction
    // of the allocation traffic in this crate. Because `eval` consumed the
    // predicate, every caller had to clone the whole tree first -
    // `precondition.clone().eval(..)` in a loop in eight `Operation` methods,
    // `transition.to_owned().eval(..)` in `process_transition`,
    // `goal.clone().eval(..)` once per node in the BFS planner. Each clone
    // deep-copied every `Predicate` node, every `SPWrapped` and every `SPValue`
    // inside it, then dropped it microseconds later.
    // `eval` now takes `&self` and uses `iter()` in the AND/OR arms, *and* all
    // of those callers have had their `.clone()` / `.to_owned()` removed, so
    // guard evaluation no longer allocates on any tick or planner node.


    // PERF: `AND`/`OR` already short-circuit via `all`/`any`, which is good.
    // Ordering conjuncts cheapest-first (e.g. literal comparisons before
    // variable lookups) would help further, and is easy to do once at model
    // build time rather than per evaluation.

    // PERF: the comparison arms call `x.evaluate(..)` and `y.evaluate(..)`,
    // each of which clones an `SPValue` out of the state (and, today, clones
    // the whole state map first - see `State::get_value`). For the common
    // "variable vs literal" case a borrowing comparison would allocate nothing.
    pub fn eval(&self, state: &State, log_target: &str) -> bool {
        match self {
            Predicate::TRUE => true,
            Predicate::FALSE => false,
            Predicate::NOT(p) => !p.eval(state, log_target),
            Predicate::AND(p) => p.iter().all(|pp| pp.eval(state, log_target)),
            Predicate::OR(p) => p.iter().any(|pp| pp.eval(state, log_target)),

            Predicate::EQ(x, y) => x.evaluate(&state, log_target) == y.evaluate(&state, log_target),
            Predicate::NEQ(x, y) => x.evaluate(&state, log_target) != y.evaluate(&state, log_target),
            Predicate::LTEQ(x, y) => x.evaluate(&state, log_target) <= y.evaluate(&state, log_target),
            Predicate::GTEQ(x, y) => x.evaluate(&state, log_target) >= y.evaluate(&state, log_target),
            Predicate::LT(x, y) => x.evaluate(&state, log_target) < y.evaluate(&state, log_target),
            Predicate::GT(x, y) => x.evaluate(&state, log_target) > y.evaluate(&state, log_target),
        }
    }

    // experimental
    pub fn keep_only(&self, only: &Vec<String>) -> Option<Predicate> {
        match self {
            Predicate::TRUE => Some(Predicate::TRUE),
            Predicate::FALSE => Some(Predicate::FALSE),
            Predicate::NOT(x) => x.keep_only(only).map(|x| Predicate::NOT(Box::new(x))),
            Predicate::AND(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.keep_only(only)).collect();
                new.dedup();
                match new.len() {
                    0 => None,
                    1 => Some(new[0].clone()),
                    _ => Some(Predicate::AND(new)),
                }
            }
            Predicate::OR(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.keep_only(only)).collect();
                new.dedup();
                match new.len() {
                    0 => None,
                    1 => Some(new[0].clone()),
                    _ => Some(Predicate::OR(new)),
                }
            }
            Predicate::EQ(x, y)
            | Predicate::NEQ(x, y)
            | Predicate::LTEQ(x, y)
            | Predicate::GTEQ(x, y)
            | Predicate::LT(x, y)
            | Predicate::GT(x, y) => {
                let remove_x = x.get_variables().iter().any(|v| !only.contains(&v.name));
                let remove_y = y.get_variables().iter().any(|v| !only.contains(&v.name));

                if remove_x || remove_y {
                    None
                } else {
                    Some(self.clone())
                }
            }
        }
    }
    // experimental
    pub fn remove(&self, remove: &Vec<String>) -> Option<Predicate> {
        match self {
            Predicate::TRUE => Some(Predicate::TRUE),
            Predicate::FALSE => Some(Predicate::FALSE),
            Predicate::NOT(x) => x.remove(remove).map(|x| Predicate::NOT(Box::new(x))),
            Predicate::AND(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.remove(remove)).collect();
                new.dedup();
                match new.len() {
                    0 => None,
                    1 => Some(new[0].clone()),
                    _ => Some(Predicate::AND(new)),
                }
            }
            Predicate::OR(x) => {
                let mut new: Vec<_> = x.iter().flat_map(|p| p.remove(remove)).collect();
                new.dedup();
                match new.len() {
                    0 => None,
                    1 => Some(new[0].clone()),
                    _ => Some(Predicate::OR(new)),
                }
            }
            Predicate::EQ(x, y)
            | Predicate::NEQ(x, y)
            | Predicate::LTEQ(x, y)
            | Predicate::GTEQ(x, y)
            | Predicate::LT(x, y)
            | Predicate::GT(x, y) => {
                let remove_x = x.get_variables().iter().any(|v| remove.contains(&v.name));
                let remove_y = y.get_variables().iter().any(|v| remove.contains(&v.name));

                if remove_x || remove_y {
                    None
                } else {
                    Some(self.clone())
                }
            }
        }
    }

    // PERF: allocates a `Vec` per node during the recursion, then sorts and
    // dedups. Only called when building key sets at startup, so it is not on the
    // hot path - but `Transition::get_all_var_keys` and
    // `Operation::get_all_var_keys` are called per operation when key sets are
    // rebuilt, and they in turn call this twice per transition. If key sets ever
    // move to being recomputed per tick (e.g. when the active operation set
    // changes), pass a `&mut Vec`/`&mut HashSet` accumulator down the recursion
    // instead of returning a fresh `Vec` at every level.
    pub fn get_predicate_vars(&self) -> Vec<SPVariable> {
        let mut vars = match self {
            Predicate::AND(preds) | Predicate::OR(preds) => {
                preds.iter().flat_map(|p| p.get_predicate_vars()).collect()
            }
            Predicate::NOT(p) => p.get_predicate_vars(),
            Predicate::EQ(lhs, rhs)
            | Predicate::NEQ(lhs, rhs)
            | Predicate::LTEQ(lhs, rhs)
            | Predicate::GTEQ(lhs, rhs)
            | Predicate::LT(lhs, rhs)
            | Predicate::GT(lhs, rhs) => {
                let mut found = lhs.get_variables();
                found.extend(rhs.get_variables());
                found
            }
            Predicate::TRUE | Predicate::FALSE => vec![],
        };

        vars.sort();
        vars.dedup();
        vars
    }

    // }

    // /// Keep only the variables in the predicate from the `only` list.
    // pub fn keep_only(&self, only: &Vec<String>) -> Option<Predicate> {
    //     match self {
    //         Predicate::TRUE => Some(Predicate::TRUE),
    //         Predicate::FALSE => Some(Predicate::FALSE),
    //         Predicate::NOT(x) => match x.keep_only(only) {
    //             Some(x) => Some(Predicate::NOT(Box::new(x))),
    //             None => None,
    //         },
    //         Predicate::AND(x) => {
    //             let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().keep_only(only)).collect();
    //             new.dedup();
    //             if new.len() == 0 {
    //                 None
    //             } else if new.len() == 1 {
    //                 Some(new[0].clone())
    //             } else {
    //                 Some(Predicate::AND(new))
    //             }
    //         }
    //         Predicate::OR(x) => {
    //             let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().keep_only(only)).collect();
    //             new.dedup();
    //             if new.len() == 0 {
    //                 None
    //             } else if new.len() == 1 {
    //                 Some(new[0].clone())
    //             } else {
    //                 Some(Predicate::OR(new))
    //             }
    //         }
    //         Predicate::EQ(x, y)
    //         | Predicate::NEQ(x, y)
    //         | Predicate::LTEQ(x, y)
    //         | Predicate::GTEQ(x, y)
    //         | Predicate::LT(x, y)
    //         | Predicate::GT(x, y) => {
    //             let remove_x = match x {
    //                 SPWrapped::SPValue(_) => false,
    //                 SPWrapped::SPVariable(vx) => !only.contains(&vx.name),
    //             };
    //             let remove_y = match y {
    //                 SPWrapped::SPValue(_) => false,
    //                 SPWrapped::SPVariable(vy) => !only.contains(&vy.name),
    //             };

    //             if remove_x || remove_y {
    //                 None
    //             } else {
    //                 Some(self.clone())
    //             }
    //         }
    //     }
    // }

    // /// Remove the variables in the predicate from the `remove` list.
    // pub fn remove(&self, remove: &Vec<String>) -> Option<Predicate> {
    //     match self {
    //         Predicate::TRUE => Some(Predicate::TRUE),
    //         Predicate::FALSE => Some(Predicate::FALSE),
    //         Predicate::NOT(x) => match x.remove(remove) {
    //             Some(x) => Some(Predicate::NOT(Box::new(x))),
    //             None => None,
    //         },
    //         Predicate::AND(x) => {
    //             let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().remove(remove)).collect();
    //             new.dedup();
    //             if new.len() == 0 {
    //                 None
    //             } else if new.len() == 1 {
    //                 Some(new[0].clone())
    //             } else {
    //                 Some(Predicate::AND(new))
    //             }
    //         }
    //         Predicate::OR(x) => {
    //             let mut new: Vec<_> = x.iter().flat_map(|p| p.clone().remove(remove)).collect();
    //             new.dedup();
    //             if new.len() == 0 {
    //                 None
    //             } else if new.len() == 1 {
    //                 Some(new[0].clone())
    //             } else {
    //                 Some(Predicate::OR(new))
    //             }
    //         }
    //         Predicate::EQ(x, y)
    //         | Predicate::NEQ(x, y)
    //         | Predicate::LTEQ(x, y)
    //         | Predicate::GTEQ(x, y)
    //         | Predicate::LT(x, y)
    //         | Predicate::GT(x, y) => {
    //             let remove_x = match x {
    //                 SPWrapped::SPValue(_) => false,
    //                 SPWrapped::SPVariable(vx) => remove.contains(&vx.name),
    //             };
    //             let remove_y = match y {
    //                 SPWrapped::SPValue(_) => false,
    //                 SPWrapped::SPVariable(vy) => remove.contains(&vy.name),
    //             };

    //             if remove_x || remove_y {
    //                 None
    //             } else {
    //                 Some(self.clone())
    //             }
    //         }
    //     }
    // }

    // pub fn get_predicate_vars(&self) -> Vec<SPVariable> {
    //     let mut vars = match self {
    //         Predicate::AND(preds) | Predicate::OR(preds) => {
    //             preds.iter().flat_map(|p| p.get_predicate_vars()).collect()
    //         }
    //         Predicate::NOT(p) => p.get_predicate_vars(),
    //         Predicate::EQ(lhs, rhs)
    //         | Predicate::NEQ(lhs, rhs)
    //         | Predicate::LTEQ(lhs, rhs)
    //         | Predicate::GTEQ(lhs, rhs)
    //         | Predicate::LT(lhs, rhs)
    //         | Predicate::GT(lhs, rhs) => {
    //             let mut found = Vec::new();
    //             if let SPWrapped::SPVariable(v) = lhs {
    //                 found.push(v.clone());
    //             }
    //             if let SPWrapped::SPVariable(v) = rhs {
    //                 found.push(v.clone());
    //             }
    //             found
    //         }
    //         Predicate::TRUE | Predicate::FALSE => vec![],
    //     };

    //     vars.sort();
    //     vars.dedup();
    //     vars
    // }

    pub fn get_predicate_var_keys(&self) -> Vec<String> {
        self.get_predicate_vars()
            .iter()
            .map(|var| var.name.to_owned())
            .collect()
    }

    // let mut s = Vec::new();
    // match self {
    //     Predicate::TRUE => {}
    //     Predicate::FALSE => {}
    //     Predicate::AND(x) => s.extend(x.iter().flat_map(|p| self.get_predicate_vars(p))),
    //     Predicate::OR(x) => s.extend(x.iter().flat_map(|p| get_predicate_vars(p))),
    //     Predicate::NOT(x) => s.extend(get_predicate_vars(x)),
    //     Predicate::EQ(x, y) => {
    //         match x {
    //             SPWrapped::SPVariable(vx) => s.push(vx.to_owned()),
    //             _ => (),
    //         }
    //         match y {
    //             SPWrapped::SPVariable(vy) => s.push(vy.to_owned()),
    //             _ => (),
    //         }
    //     }
    //     Predicate::NEQ(x, y) => {
    //         match x {
    //             SPWrapped::SPVariable(vx) => s.push(vx.to_owned()),
    //             _ => (),
    //         }
    //         match y {
    //             SPWrapped::SPVariable(vy) => s.push(vy.to_owned()),
    //             _ => (),
    //         }
    //     }
    // }
    // s.sort();
    // s.dedup();
    // s
}
// }

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
            Predicate::LTEQ(x, y) => format!("{} <= {}", x, y),
            Predicate::GTEQ(x, y) => format!("{} >= {}", x, y),
            Predicate::LT(x, y) => format!("{} < {}", x, y),
            Predicate::GT(x, y) => format!("{} > {}", x, y),
        };

        write!(fmtr, "{}", &s)
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    fn john_doe() -> Vec<(SPVariable, SPValue)> {
        let name = v!("name");
        let surname = v!("surname");
        let height = iv!("height");
        let weight = fv!("weight");
        let smart = bv!("smart");

        vec![
            (name, "John".to_spvalue()),
            (surname, "Doe".to_spvalue()),
            (height, 185.to_spvalue()),
            (weight, 80.0.to_spvalue()),
            (smart, true.to_spvalue()),
        ]
    }

    #[test]
    fn test_predicate_eq() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::EQ(v!("name").wrap(), "John".wrap());
        let eq2 = Predicate::EQ(v!("name").wrap(), "Jack".wrap());
        assert!(eq1.eval(&state, "t"));
        assert_ne!(true, eq2.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_lteq() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::LTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            185.wrap(),
        );
        let eq2 = Predicate::LTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            186.wrap(),
        );
        let eq3 = Predicate::LTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            184.wrap(),
        );
        let eq4 = Predicate::LTEQ(
            state.get_float_or_default_to_zero("weight", "t").wrap(),
            state.get_int_or_default_to_zero("height", "t").wrap(),
        );
        let eq5 = Predicate::LTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            state.get_float_or_default_to_zero("weight", "t").wrap(),
        );
        assert!(eq1.eval(&state, "t"));
        assert!(eq2.eval(&state, "t"));
        assert!(!eq3.eval(&state, "t"));
        assert!(eq4.eval(&state, "t"));
        assert!(!eq5.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_lt() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::LT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            185.wrap(),
        );
        let eq2 = Predicate::LT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            186.wrap(),
        );
        let eq3 = Predicate::LT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            184.wrap(),
        );
        let eq4 = Predicate::LT(
            state.get_float_or_default_to_zero("weight", "t").wrap(),
            state.get_int_or_default_to_zero("height", "t").wrap(),
        );
        let eq5 = Predicate::LT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            state.get_float_or_default_to_zero("weight", "t").wrap(),
        );
        assert!(!eq1.eval(&state, "t"));
        assert!(eq2.eval(&state, "t"));
        assert!(!eq3.eval(&state, "t"));
        assert!(eq4.eval(&state, "t"));
        assert!(!eq5.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_gteq() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::GTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            185.wrap(),
        );
        let eq2 = Predicate::GTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            184.wrap(),
        );
        let eq3 = Predicate::GTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            186.wrap(),
        );
        let eq4 = Predicate::GTEQ(
            state.get_float_or_default_to_zero("weight", "t").wrap(),
            state.get_int_or_default_to_zero("height", "t").wrap(),
        );
        let eq5 = Predicate::GTEQ(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            state.get_float_or_default_to_zero("weight", "t").wrap(),
        );
        assert!(eq1.eval(&state, "t"));
        assert!(eq2.eval(&state, "t"));
        assert!(!eq3.eval(&state, "t"));
        assert!(!eq4.eval(&state, "t"));
        assert!(eq5.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_gt() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::GT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            185.wrap(),
        );
        let eq2 = Predicate::GT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            184.wrap(),
        );
        let eq3 = Predicate::GT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            186.wrap(),
        );
        let eq4 = Predicate::GT(
            state.get_float_or_default_to_zero("weight", "t").wrap(),
            state.get_int_or_default_to_zero("height", "t").wrap(),
        );
        let eq5 = Predicate::GT(
            state.get_int_or_default_to_zero("height", "t").wrap(),
            state.get_float_or_default_to_zero("weight", "t").wrap(),
        );
        assert!(!eq1.eval(&state, "t"));
        assert!(eq2.eval(&state, "t"));
        assert!(!eq3.eval(&state, "t"));
        assert!(!eq4.eval(&state, "t"));
        assert!(eq5.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_neq() {
        let state = State::from_vec(&john_doe());
        let neq1 = Predicate::NEQ(v!("name").wrap(), "John".wrap());
        let neq2 = Predicate::NEQ(v!("name").wrap(), "Jack".wrap());
        assert_ne!(true, neq1.eval(&state, "t"));
        assert!(neq2.eval(&state, "t"));
    }

    #[test]
    #[should_panic]
    fn test_predicate_eq_panic_not_in_state() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::EQ(v!("v1").wrap(), "John".wrap());
        assert!(eq1.eval(&state, "t"));
    }

    #[test]
    #[should_panic]
    fn test_predicate_eq_wrong_var() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::EQ(v!("name").wrap(), v!("surname").wrap());
        assert!(eq1.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_not() {
        let s1 = State::from_vec(&john_doe());
        let not = Predicate::NOT(Box::new(Predicate::EQ(bv!("smart").wrap(), false.wrap())));
        let notf = Predicate::NOT(Box::new(Predicate::EQ(bv!("smart").wrap(), true.wrap())));
        assert!(not.eval(&s1, "t"));
        assert!(!notf.eval(&s1, "t"));
    }

    #[test]
    fn test_predicate_and() {
        let john_doe = john_doe();
        let s1 = State::from_vec(&john_doe);
        let eq = Predicate::EQ(bv!("smart").wrap(), true.wrap());
        let eq2 = Predicate::EQ(fv!("weight").wrap(), 80.0.wrap());
        let eqf = Predicate::EQ(iv!("height").wrap(), 175.wrap());
        let and = Predicate::AND(vec![eq.clone(), eq2.clone()]);
        let andf = Predicate::AND(vec![eq, eq2, eqf]);
        assert!(and.eval(&s1, "t"));
        assert!(!andf.eval(&s1, "t"));
    }

    #[test]
    fn test_predicate_or() {
        let john_doe = john_doe();
        let s1 = State::from_vec(&john_doe);
        let eq = Predicate::EQ(bv!("smart").wrap(), true.wrap());
        let eq2 = Predicate::EQ(fv!("weight").wrap(), 80.0.wrap());
        let eqf = Predicate::EQ(iv!("height").wrap(), 175.wrap());
        let or = Predicate::OR(vec![eq.clone(), eq2.clone()]);
        let or2 = Predicate::OR(vec![eq, eq2, eqf]);
        assert!(or.eval(&s1, "t"));
        assert!(or2.eval(&s1, "t"));
    }

    #[test]
    fn test_predicate_complex() {
        let john_doe = john_doe();
        let s1 = State::from_vec(&john_doe);
        let eq = Predicate::EQ(bv!("smart").wrap(), true.wrap());
        let eq2 = Predicate::EQ(fv!("weight").wrap(), 80.0.wrap());
        let eqf = Predicate::EQ(iv!("height").wrap(), 175.wrap());
        let and = Predicate::AND(vec![eq.clone(), eq2.clone()]);
        let andf = Predicate::AND(vec![eq.clone(), eq2.clone(), eqf.clone()]);
        let or = Predicate::OR(vec![eq.clone(), eq2.clone()]);
        let or2 = Predicate::OR(vec![eq, eq2, eqf]);
        let not = Predicate::NOT(Box::new(or.clone()));
        let cmplx = Predicate::AND(vec![
            Predicate::NOT(Box::new(not.clone())),
            or,
            or2,
            and,
            Predicate::NOT(Box::new(andf)),
        ]);
        assert!(cmplx.eval(&s1, "t"));
    }

    #[test]
    fn test_predicate_eq_macro() {
        let state = State::from_vec(&john_doe());
        let eq1 = eq!(v!("name").wrap(), "John".wrap());
        let eq2 = eq!(v!("name").wrap(), "Jack".wrap());
        assert!(eq1.eval(&state, "t"));
        assert_ne!(true, eq2.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_not_macro() {
        let s1 = State::from_vec(&john_doe());
        let not = not!(eq!(bv!("smart").wrap(), false.wrap()));
        let notf = not!(eq!(bv!("smart").wrap(), true.wrap()));
        assert!(not.eval(&s1, "t"));
        assert!(!notf.eval(&s1, "t"));
    }

    #[test]
    fn test_predicate_neq_macro() {
        let state = State::from_vec(&john_doe());
        let neq1 = neq!(v!("name").wrap(), "John".wrap());
        let neq2 = neq!(v!("name").wrap(), "Jack".wrap());
        assert_ne!(true, neq1.eval(&state, "t"));
        assert!(neq2.eval(&state, "t"));
    }

    #[test]
    fn test_predicate_and_macro() {
        let john_doe = john_doe();
        let s1 = State::from_vec(&john_doe);
        let eq = eq!(bv!("smart").wrap(), true.wrap());
        let eq2 = eq!(fv!("weight").wrap(), 80.0.wrap());
        let eqf = eq!(iv!("height").wrap(), 175.wrap());
        let and = and!(vec![eq.clone(), eq2.clone()]);
        let andf = and!(vec![eq, eq2, eqf]);
        assert!(and.eval(&s1, "t"));
        assert!(!andf.eval(&s1, "t"));
    }

    #[test]
    fn test_predicate_or_macro() {
        let john_doe = john_doe();
        let s1 = State::from_vec(&john_doe);
        let eq = eq!(bv!("smart").wrap(), true.wrap());
        let eq2 = eq!(fv!("weight").wrap(), 80.0.wrap());
        let eqf = eq!(iv!("height").wrap(), 175.wrap());
        let or = or!(vec![eq.clone(), eq2.clone()]);
        let or2 = or!(vec![eq, eq2, eqf]);
        assert!(or.eval(&s1, "t"));
        assert!(or2.eval(&s1, "t"));
    }

    fn make_robot_initial_state() -> State {
        let state = State::new();
        let state = state.add(
            SPAssignment::new(v!("runner_goal"), "var:ur_current_pose == c".to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(av!("runner_plan"), Vec::<String>::new().to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(bv!("runner_replan"), true.to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(bv!("runner_replanned"), false.to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(bv!("ur_action_trigger"), false.to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(v!("ur_action_state"), "initial".to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(v!("ur_current_pose"), "a".to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(v!("ur_command"), "movej".to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(fv!("ur_velocity"), 0.2.to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(fv!("ur_acceleration"), 0.4.to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(v!("ur_goal_feature_id"), "a".to_spvalue()),
            "test",
        );
        let state = state.add(
            SPAssignment::new(v!("ur_tcp_id"), "svt_tcp".to_spvalue()),
            "test",
        );
        state
    }

    #[test]
    fn test_predicate_get_all_variables() {
        let state = make_robot_initial_state();
        let pred = pred_parser::pred(
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != a",
            &state,
        ).unwrap();
        let vars = pred.get_predicate_vars();
        let vars_init = vec![
            v!("ur_action_state"),
            bv!("ur_action_trigger"),
            v!("ur_current_pose"),
        ];
        assert_eq!(vars, vars_init)
    }

    #[test]
    fn test_predicate_get_variables() {
        let state = make_robot_initial_state();
        let pred = pred_parser::pred(
            "var:ur_action_trigger == false && var:ur_action_state == initial && var:ur_current_pose != a",
            &state,
        ).unwrap();
        let vars = pred.get_predicate_vars();
        let vars_init = vec![
            v!("ur_action_state"),
            bv!("ur_action_trigger"),
            v!("ur_current_pose"),
        ];
        assert_eq!(vars, vars_init)
    }

    #[test]
    fn test_predicate_keep_only() {
        let state = make_robot_initial_state();
        let pred = pred_parser::pred(
            "var:ur_action_trigger == false && var:ur_action_state == initial || (var:ur_current_pose != a && var:ur_action_state == executing)",
            &state,
        ).unwrap();
        let new_pred = pred.keep_only(&vec!["ur_action_state".to_string()]);
        println!("{:?}", new_pred)
    }

    #[test]
    fn test_predicate_remove() {
        let state = make_robot_initial_state();
        let pred = pred_parser::pred(
            "var:ur_action_trigger == false && var:ur_action_state == initial || (var:ur_current_pose != a && var:ur_action_state == executing)",
            &state,
        ).unwrap();
        let new_pred = pred.remove(&vec![
            "ur_action_state".to_string(),
            "ur_action_trigger".to_string(),
            "ur_current_pose".to_string(),
        ]);
        println!("{:?}", new_pred)
    }
}

/// Predicate projection and rendering.
///
/// `keep_only` and `remove` are the two projection operators - they cut a guard
/// down to the variables a caller cares about, and are marked experimental in
/// the source. They are also the two functions in this file with a genuinely
/// subtle contract: what happens to an `AND`/`OR` when every one of its
/// children is projected away, and what happens to a comparison when only *one*
/// side mentions a removed variable. Both answers matter, because a projection
/// that silently turns a restrictive guard into a permissive one is how a
/// planner ends up producing a plan that cannot execute.
#[cfg(test)]
mod projection_tests {
    use crate::*;

    const TARGET: &str = "test";

    fn state() -> State {
        State::from_vec(&vec![
            (SPVariable::new("a", SPValueType::Bool), true.to_spvalue()),
            (SPVariable::new("b", SPValueType::Bool), false.to_spvalue()),
            (SPVariable::new("n", SPValueType::Int64), 5.to_spvalue()),
        ])
    }

    fn var(name: &str, state: &State) -> SPVariable {
        state.get_assignment(name, TARGET).var
    }

    fn eq(name: &str, value: SPValue, state: &State) -> Predicate {
        Predicate::EQ(var(name, state).wrap(), value.wrap())
    }

    fn only(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    /// A comparison survives only if *every* variable it mentions is kept.
    #[test]
    fn keep_only_drops_a_comparison_that_mentions_anything_else() {
        let state = state();
        let about_a = eq("a", true.to_spvalue(), &state);

        assert_eq!(about_a.keep_only(&only(&["a"])), Some(about_a.clone()));
        assert_eq!(about_a.keep_only(&only(&["b"])), None);
        assert_eq!(about_a.keep_only(&only(&[])), None);

        // Both sides count, not just the left one.
        let a_vs_b = Predicate::EQ(var("a", &state).wrap(), var("b", &state).wrap());
        assert_eq!(a_vs_b.keep_only(&only(&["a"])), None);
        assert_eq!(a_vs_b.keep_only(&only(&["a", "b"])), Some(a_vs_b.clone()));
    }

    /// The constants survive any projection - they mention nothing.
    #[test]
    fn the_constants_always_survive() {
        assert_eq!(Predicate::TRUE.keep_only(&only(&[])), Some(Predicate::TRUE));
        assert_eq!(Predicate::FALSE.keep_only(&only(&[])), Some(Predicate::FALSE));
        assert_eq!(Predicate::TRUE.remove(&only(&["a"])), Some(Predicate::TRUE));
        assert_eq!(Predicate::FALSE.remove(&only(&["a"])), Some(Predicate::FALSE));
    }

    /// The interesting case: a conjunction whose children are partly projected
    /// away. What is left is the conjunction of the survivors - and a single
    /// survivor is unwrapped rather than left as a one-element `AND`.
    #[test]
    fn a_conjunction_keeps_the_children_that_survive() {
        let state = state();
        let about_a = eq("a", true.to_spvalue(), &state);
        let about_b = eq("b", false.to_spvalue(), &state);
        let about_n = eq("n", 5.to_spvalue(), &state);
        let all = Predicate::AND(vec![about_a.clone(), about_b.clone(), about_n.clone()]);

        assert_eq!(all.keep_only(&only(&["a", "b", "n"])), Some(all.clone()));
        assert_eq!(
            all.keep_only(&only(&["a", "b"])),
            Some(Predicate::AND(vec![about_a.clone(), about_b]))
        );
        assert_eq!(
            all.keep_only(&only(&["a"])),
            Some(about_a),
            "one survivor is unwrapped rather than left in a one-element AND"
        );
    }

    /// And the case worth being loud about: projecting away *everything* in a
    /// conjunction yields `None`, not `TRUE`. A caller that treats `None` as
    /// "no constraint" is the one that turns a guard into a tautology - the
    /// function itself refuses to make that decision.
    #[test]
    fn projecting_away_every_child_yields_none_rather_than_true() {
        let state = state();
        let all = Predicate::AND(vec![
            eq("a", true.to_spvalue(), &state),
            eq("b", false.to_spvalue(), &state),
        ]);

        assert_eq!(all.keep_only(&only(&["n"])), None);
        assert_eq!(all.remove(&only(&["a", "b"])), None);

        let any = Predicate::OR(vec![
            eq("a", true.to_spvalue(), &state),
            eq("b", false.to_spvalue(), &state),
        ]);
        assert_eq!(any.keep_only(&only(&["n"])), None);
        assert_eq!(any.remove(&only(&["a", "b"])), None);
    }

    /// `remove` is the complement of `keep_only`: it drops the comparisons that
    /// mention the listed variables and keeps everything else.
    #[test]
    fn remove_is_the_complement_of_keep_only() {
        let state = state();
        let about_a = eq("a", true.to_spvalue(), &state);
        let about_b = eq("b", false.to_spvalue(), &state);
        let both = Predicate::AND(vec![about_a.clone(), about_b.clone()]);

        assert_eq!(both.remove(&only(&["b"])), Some(about_a.clone()));
        assert_eq!(both.remove(&only(&["a"])), Some(about_b));
        assert_eq!(both.remove(&only(&[])), Some(both.clone()));
    }

    /// `NOT` follows its child: if the child projects away, so does the
    /// negation, rather than becoming `NOT(TRUE)`.
    #[test]
    fn a_negation_follows_its_child() {
        let state = state();
        let about_a = eq("a", true.to_spvalue(), &state);
        let negated = Predicate::NOT(Box::new(about_a.clone()));

        assert_eq!(negated.keep_only(&only(&["a"])), Some(negated.clone()));
        assert_eq!(negated.keep_only(&only(&["b"])), None);
        assert_eq!(negated.remove(&only(&["a"])), None);
    }

    #[test]
    fn projection_recurses_through_nesting() {
        let state = state();
        let about_a = eq("a", true.to_spvalue(), &state);
        let about_b = eq("b", false.to_spvalue(), &state);
        let about_n = eq("n", 5.to_spvalue(), &state);

        let nested = Predicate::AND(vec![
            about_a.clone(),
            Predicate::OR(vec![about_b.clone(), about_n.clone()]),
        ]);

        assert_eq!(
            nested.keep_only(&only(&["a", "n"])),
            Some(Predicate::AND(vec![about_a.clone(), about_n.clone()])),
            "the inner OR loses one child and is unwrapped"
        );
        assert_eq!(
            nested.keep_only(&only(&["a"])),
            Some(about_a),
            "with the whole inner OR gone, the AND unwraps too"
        );
    }

    /// Every comparison operator is treated the same way by the projection, so
    /// the shared match arm has to actually cover all six.
    #[test]
    fn every_comparison_operator_projects_the_same_way() {
        let state = state();
        let lhs = var("n", &state).wrap();
        let rhs = 5.to_spvalue().wrap();

        let operators = [
            Predicate::EQ(lhs.clone(), rhs.clone()),
            Predicate::NEQ(lhs.clone(), rhs.clone()),
            Predicate::LTEQ(lhs.clone(), rhs.clone()),
            Predicate::GTEQ(lhs.clone(), rhs.clone()),
            Predicate::LT(lhs.clone(), rhs.clone()),
            Predicate::GT(lhs, rhs),
        ];

        for predicate in operators {
            assert_eq!(
                predicate.keep_only(&only(&["n"])),
                Some(predicate.clone()),
                "{predicate} should have been kept"
            );
            assert_eq!(
                predicate.keep_only(&only(&["a"])),
                None,
                "{predicate} should have been projected away"
            );
            assert_eq!(predicate.remove(&only(&["n"])), None);
        }
    }

    /// The ordering comparisons themselves, which the projection tests build on
    /// but never evaluate.
    #[test]
    fn the_ordering_operators_compare_values() {
        let state = state();
        let n = var("n", &state).wrap();

        assert!(Predicate::LT(n.clone(), 6.to_spvalue().wrap()).eval(&state, TARGET));
        assert!(!Predicate::LT(n.clone(), 5.to_spvalue().wrap()).eval(&state, TARGET));
        assert!(Predicate::LTEQ(n.clone(), 5.to_spvalue().wrap()).eval(&state, TARGET));
        assert!(Predicate::GT(n.clone(), 4.to_spvalue().wrap()).eval(&state, TARGET));
        assert!(!Predicate::GT(n.clone(), 5.to_spvalue().wrap()).eval(&state, TARGET));
        assert!(Predicate::GTEQ(n, 5.to_spvalue().wrap()).eval(&state, TARGET));
    }

    /// `Display` is what a disabled operation's message renders, so every
    /// variant has to have a form.
    #[test]
    fn display_renders_every_variant() {
        let state = state();
        let a = var("a", &state).wrap();
        let value = true.to_spvalue().wrap();

        assert_eq!(Predicate::TRUE.to_string(), "TRUE");
        assert_eq!(Predicate::FALSE.to_string(), "FALSE");
        assert_eq!(Predicate::EQ(a.clone(), value.clone()).to_string(), "a = true");
        assert_eq!(Predicate::NEQ(a.clone(), value.clone()).to_string(), "a != true");
        assert_eq!(Predicate::LTEQ(a.clone(), value.clone()).to_string(), "a <= true");
        assert_eq!(Predicate::GTEQ(a.clone(), value.clone()).to_string(), "a >= true");
        assert_eq!(Predicate::LT(a.clone(), value.clone()).to_string(), "a < true");
        assert_eq!(Predicate::GT(a.clone(), value.clone()).to_string(), "a > true");
        assert_eq!(
            Predicate::NOT(Box::new(Predicate::TRUE)).to_string(),
            "!(TRUE)"
        );
        assert_eq!(
            Predicate::AND(vec![Predicate::TRUE, Predicate::FALSE]).to_string(),
            "(TRUE && FALSE)"
        );
        assert_eq!(
            Predicate::OR(vec![Predicate::TRUE, Predicate::FALSE]).to_string(),
            "(TRUE || FALSE)"
        );
    }
}
