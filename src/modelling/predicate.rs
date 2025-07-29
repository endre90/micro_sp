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
                    if let Some(value) = state.get_value(&vx.name) {
                        value == vy
                    } else {
                        false
                    }
                }
                (SPWrapped::SPValue(vx), SPWrapped::SPVariable(vy)) => {
                    if let Some(value) = state.get_value(&vy.name) {
                        vx == value
                    } else {
                        false
                    }
                }
                (SPWrapped::SPValue(vx), SPWrapped::SPValue(vy)) => vx == vy,
            },
            Predicate::NEQ(x, y) => match (x, y) {
                (SPWrapped::SPVariable(vx), SPWrapped::SPVariable(vy)) => {
                    state.get_value(&vx.name) != state.get_value(&vy.name)
                }
                (SPWrapped::SPVariable(vx), SPWrapped::SPValue(vy)) => {
                    if let Some(value) = state.get_value(&vx.name) {
                        value != vy
                    } else {
                        false
                    }
                }
                (SPWrapped::SPValue(vx), SPWrapped::SPVariable(vy)) => {
                    if let Some(value) = state.get_value(&vy.name) {
                        vx != value
                    } else {
                        false
                    }
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

    pub fn get_predicate_vars(&self) -> Vec<SPVariable> {
        let mut vars = match self {
            Predicate::AND(preds) | Predicate::OR(preds) => {
                preds.iter().flat_map(|p| p.get_predicate_vars()).collect()
            }
            Predicate::NOT(p) => p.get_predicate_vars(),
            Predicate::EQ(lhs, rhs) | Predicate::NEQ(lhs, rhs) => {
                let mut found = Vec::new();
                if let SPWrapped::SPVariable(v) = lhs {
                    found.push(v.clone());
                }
                if let SPWrapped::SPVariable(v) = rhs {
                    found.push(v.clone());
                }
                found
            }
            Predicate::TRUE | Predicate::FALSE => vec![],
        };

        vars.sort();
        vars.dedup();
        vars
    }

    pub fn get_predicate_var_keys(&self) -> Vec<String> {
        self.get_predicate_vars().iter().map(|var| var.name.to_owned()).collect()
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
        assert!(eq1.eval(&state));
        assert_ne!(true, eq2.eval(&state));
    }

    #[test]
    fn test_predicate_neq() {
        let state = State::from_vec(&john_doe());
        let neq1 = Predicate::NEQ(v!("name").wrap(), "John".wrap());
        let neq2 = Predicate::NEQ(v!("name").wrap(), "Jack".wrap());
        assert_ne!(true, neq1.eval(&state));
        assert!(neq2.eval(&state));
    }

    #[test]
    // Let's see...
    // #[should_panic]
    fn test_predicate_eq_panic_not_in_state() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::EQ(v!("v1").wrap(), "John".wrap());
        assert_eq!(eq1.eval(&state), false);
    }

    #[test]
    #[should_panic]
    fn test_predicate_eq_wrong_var() {
        let state = State::from_vec(&john_doe());
        let eq1 = Predicate::EQ(v!("name").wrap(), v!("surname").wrap());
        assert!(eq1.eval(&state));
    }

    #[test]
    fn test_predicate_not() {
        let s1 = State::from_vec(&john_doe());
        let not = Predicate::NOT(Box::new(Predicate::EQ(bv!("smart").wrap(), false.wrap())));
        let notf = Predicate::NOT(Box::new(Predicate::EQ(bv!("smart").wrap(), true.wrap())));
        assert!(not.eval(&s1));
        assert!(!notf.eval(&s1));
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
        assert!(and.eval(&s1));
        assert!(!andf.eval(&s1));
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
        assert!(or.eval(&s1));
        assert!(or2.eval(&s1));
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
        assert!(cmplx.eval(&s1));
    }

    #[test]
    fn test_predicate_eq_macro() {
        let state = State::from_vec(&john_doe());
        let eq1 = eq!(v!("name").wrap(), "John".wrap());
        let eq2 = eq!(v!("name").wrap(), "Jack".wrap());
        assert!(eq1.eval(&state));
        assert_ne!(true, eq2.eval(&state));
    }

    #[test]
    fn test_predicate_not_macro() {
        let s1 = State::from_vec(&john_doe());
        let not = not!(eq!(bv!("smart").wrap(), false.wrap()));
        let notf = not!(eq!(bv!("smart").wrap(), true.wrap()));
        assert!(not.eval(&s1));
        assert!(!notf.eval(&s1));
    }

    #[test]
    fn test_predicate_neq_macro() {
        let state = State::from_vec(&john_doe());
        let neq1 = neq!(v!("name").wrap(), "John".wrap());
        let neq2 = neq!(v!("name").wrap(), "Jack".wrap());
        assert_ne!(true, neq1.eval(&state));
        assert!(neq2.eval(&state));
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
        assert!(and.eval(&s1));
        assert!(!andf.eval(&s1));
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
        assert!(or.eval(&s1));
        assert!(or2.eval(&s1));
    }

    fn make_robot_initial_state() -> State {
        let state = State::new();
        let state = state.add(SPAssignment::new(
            v!("runner_goal"),
            "var:ur_current_pose == c".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            av!("runner_plan"),
            Vec::<String>::new().to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(bv!("runner_replan"), true.to_spvalue()));
        let state = state.add(SPAssignment::new(
            bv!("runner_replanned"),
            false.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            bv!("ur_action_trigger"),
            false.to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(
            v!("ur_action_state"),
            "initial".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(v!("ur_current_pose"), "a".to_spvalue()));
        let state = state.add(SPAssignment::new(v!("ur_command"), "movej".to_spvalue()));
        let state = state.add(SPAssignment::new(fv!("ur_velocity"), 0.2.to_spvalue()));
        let state = state.add(SPAssignment::new(fv!("ur_acceleration"), 0.4.to_spvalue()));
        let state = state.add(SPAssignment::new(
            v!("ur_goal_feature_id"),
            "a".to_spvalue(),
        ));
        let state = state.add(SPAssignment::new(v!("ur_tcp_id"), "svt_tcp".to_spvalue()));
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
