use std::{
    collections::HashSet,
    time::Instant,
};

use crate::{Operation, Predicate, State, PlanningResult};

pub fn bfs_operation_planner(
    state: State,
    goal: Predicate,
    model: Vec<Operation>,
    max_depth: usize,
) -> PlanningResult {
    let now = Instant::now();
    let mut visited: HashSet<State> = HashSet::new();
    let mut stack: Vec<(State, Vec<String>)> = vec![(state, vec![])];
    loop {
        match stack.len() {
            0 => {
                break PlanningResult {
                    found: false,
                    ..Default::default()
                }
            }
            _ => {
                let (s, path) = stack.pop().unwrap();
                match goal.clone().eval(&s) {
                    true => {
                        break PlanningResult {
                            found: true,
                            length: path.len(),
                            plan: path,
                            time: now.elapsed(),
                        }
                    }
                    false => match path.len() > max_depth {
                        true => {
                            break PlanningResult {
                                found: false,
                                ..Default::default()
                            }
                        }
                        false => match visited.contains(&s) {
                            true => continue,
                            false => {
                                visited.insert(s.clone());
                                model
                                    .iter()
                                    .for_each(|o| match o.clone().eval_planning(&s) {
                                        false => (),
                                        true => {
                                            let next_s = o.clone().take_planning(&s);
                                            let mut next_p = path.clone();
                                            next_p.push(o.name.clone());
                                            stack.insert(0, (next_s, next_p));
                                        }
                                    })
                            }
                        },
                    },
                }
            }
        }
    }
}
