use std::{collections::HashSet, time::Instant};

use crate::*;

/// Minimal Breadth First Search algorithm for sequencing operations.
pub fn bfs_operation_planner(
    state: State,
    goal: Predicate,
    model: Vec<Operation>,
    max_depth: usize,
    log_target: &str
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
                let (s, path) = match stack.pop() {
                    Some(popped) => popped,
                    None => {
                        log::error!(target: &&format!("operation_planner"), 
                            "Failed to pop value from stack? This shouldn't happen.");
                        log::error!(target: &&format!("operation_planner"), 
                            "Breaking the search with empty planning result.");
                        break PlanningResult {
                            found: false,
                            ..Default::default()
                        };
                    }
                };
                match goal.clone().eval(&s, &log_target) {
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
                                    .for_each(|o| match o.clone().eval_planning(&s, &log_target) {
                                        false => (),
                                        true => {
                                            let next_s = o.clone().take_planning(&s, &log_target);
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
