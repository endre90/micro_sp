use std::{collections::HashSet, time::{Duration, Instant}};
use crate::*;

/// Minimal Breadth First Search algorithm for sequencing operations.
/// use std::{collections::HashSet, time::{Duration, Instant}};

// use crate::*;

// PERF: the search itself has several avoidable multipliers, and since it runs
// synchronously inside an async task (see `handle_replan_request`) every one of
// them turns into blocked runner time:
//   1. `stack.insert(0, (next_s, next_p))` inserts at the *front* of a `Vec`,
//      which memmoves the entire frontier on every expansion - O(n) per node,
//      O(n^2) over the search. Use `VecDeque` with `push_back` + `pop_front`
//      (that also makes it an actual BFS; `Vec::pop` + front-insert is a
//      confusing hybrid).
//   2. `visited.insert(s.clone())` clones the full state per node, and hashing
//      it sorts every key (see `impl Hash for State`). Together these usually
//      dominate the runtime. Suggested: hash/store only the planning-relevant
//      variables - the union of `get_all_var_keys()` over `model` - as a small
//      canonical key, instead of the whole state. Runner bookkeeping like
//      `*_elapsed_executing_ms` currently makes equivalent states look
//      different, which both slows the search and can make it explore forever.
//   3. `goal.clone().eval(&s, ..)` clones the entire goal predicate tree once
//      per expanded node, purely because `Predicate::eval` takes `self` by
//      value. Changing `eval` to `&self` removes this outright.
//   4. `o.clone().eval_planning(..)` and `o.clone().take_planning(..)` clone the
//      whole `Operation` per operation per node - both already take `&self`, so
//      these `.clone()` calls are pure waste.
//   5. `path.clone()` per successor copies the whole plan prefix. A parent-link
//      / arena representation (store `(state, parent_idx, op_name)` and
//      reconstruct the path once at the end) makes this O(1).
//   6. `state` and `model` are taken by value, forcing the caller to clone the
//      whole state and the whole operation model per replan; `&State` and
//      `&[Operation]` would do, as nothing here mutates them.
// Fixing 1-3 alone typically turns a multi-second replan into a sub-100 ms one,
// which removes the visible stall when a goal is scheduled.
pub fn bfs_operation_planner(
    state: State,
    goal: Predicate,
    model: Vec<Operation>,
    max_depth: usize,
    log_target: &str,
    deadline_ms: u64,
) -> PlanningResult {
    let now = Instant::now();
    let limit = Duration::from_millis(deadline_ms);
    let mut visited: HashSet<State> = HashSet::new();
    let mut stack: Vec<(State, Vec<String>)> = vec![(state, vec![])];
    loop {
        if now.elapsed() > limit {
            break PlanningResult {
                found: false,
                time: now.elapsed(),
                ..Default::default()
            };
        }

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
// pub fn bfs_operation_planner(
//     state: State,
//     goal: Predicate,
//     model: Vec<Operation>,
//     max_depth: usize,
//     log_target: &str
// ) -> PlanningResult {
//     let now = Instant::now();
//     let mut visited: HashSet<State> = HashSet::new();
//     let mut stack: Vec<(State, Vec<String>)> = vec![(state, vec![])];
//     loop {
//         match stack.len() {
//             0 => {
//                 break PlanningResult {
//                     found: false,
//                     ..Default::default()
//                 }
//             }
//             _ => {
//                 let (s, path) = match stack.pop() {
//                     Some(popped) => popped,
//                     None => {
//                         log::error!(target: &&format!("operation_planner"), 
//                             "Failed to pop value from stack? This shouldn't happen.");
//                         log::error!(target: &&format!("operation_planner"), 
//                             "Breaking the search with empty planning result.");
//                         break PlanningResult {
//                             found: false,
//                             ..Default::default()
//                         };
//                     }
//                 };
//                 match goal.clone().eval(&s, &log_target) {
//                     true => {
//                         break PlanningResult {
//                             found: true,
//                             length: path.len(),
//                             plan: path,
//                             time: now.elapsed(),
//                         }
//                     }
//                     false => match path.len() > max_depth {
//                         true => {
//                             break PlanningResult {
//                                 found: false,
//                                 ..Default::default()
//                             }
//                         }
//                         false => match visited.contains(&s) {
//                             true => continue,
//                             false => {
//                                 visited.insert(s.clone());
//                                 model
//                                     .iter()
//                                     .for_each(|o| match o.clone().eval_planning(&s, &log_target) {
//                                         false => (),
//                                         true => {
//                                             let next_s = o.clone().take_planning(&s, &log_target);
//                                             let mut next_p = path.clone();
//                                             next_p.push(o.name.clone());
//                                             stack.insert(0, (next_s, next_p));
//                                         }
//                                     })
//                             }
//                         },
//                     },
//                 }
//             }
//         }
//     }
// }
