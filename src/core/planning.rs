use std::{time::{Instant, Duration}, collections::HashSet};

use crate::{State, Transition, Predicate};

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Default)]
pub struct PlanningResult {
    pub found: bool,
    pub length: usize,
    pub plan: Vec<String>,
    pub time: Duration,
}

pub fn simple_transition_planner(
    state: State,
    goal: Predicate,
    model: Vec<Transition>,
    max_depth: usize,
) -> PlanningResult {
    let now = Instant::now();
    let mut visited: HashSet<State> = HashSet::new();
    let mut stack: Vec<(State, Vec<String>)> = vec!((state, vec!()));
    loop {
        match stack.len() {
            0 => break PlanningResult {
                found: false, 
                ..Default::default()
            },
            _ => {
                let (s, path) = stack.pop().unwrap();
                match goal.clone().eval(&s) {
                    true => break PlanningResult {
                        found: true,
                        length: path.len(),
                        plan: path,
                        time: now.elapsed()
                    },
                    false => match path.len() > max_depth {
                        true => break PlanningResult {
                            found: false, 
                            ..Default::default()
                        },
                        false => match visited.contains(&s) {
                            true => continue,
                            false => {
                                visited.insert(s.clone());
                                model.iter().for_each(|t| {
                                    match t.clone().eval(&s) {
                                        false => (),
                                        true => {
                                            let next_s = t.clone().take(&s);
                                            let mut next_p = path.clone();
                                            next_p.push(t.name.clone());
                                            stack.insert(0, (next_s, next_p));
                                        }
                                    }
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}