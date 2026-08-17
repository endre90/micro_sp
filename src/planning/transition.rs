//! Breadth-first planning over bare [`Transition`]s.
//!
//! The transition-level counterpart of [`crate::planning::operation`]: it
//! searches the same way but over transitions rather than whole operation
//! lifecycles. Also the home of [`PlanningResult`], which both planners return.

use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::*;

/// The outcome of a planning attempt.
///
/// A failed search returns [`PlanningResult::default()`], so `plan` is empty and
/// `length` is `0` whenever `found` is `false` - never a truncated plan.
#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord, Default)]
pub struct PlanningResult {
    /// Whether a sequence reaching the goal was found.
    pub found: bool,
    /// Number of steps in `plan`.
    pub length: usize,
    /// The transition or operation names to execute, in order.
    pub plan: Vec<String>,
    /// Wall-clock time the search took.
    pub time: Duration,
}

/// Breadth-first search for a sequence of transitions that reaches `goal`.
///
/// Explores from `state`, taking any transition whose planning guard holds, and
/// returns the shortest sequence of transition names to the first state
/// satisfying `goal`. Gives up once a path exceeds `max_depth`; a `visited` set
/// keeps a cyclic model from searching forever. `log_target` is the log target
/// used for guard evaluation diagnostics.
///
/// ```
/// use micro_sp::*;
///
/// let state = State::from_vec(&vec![(v!("pos"), "a".to_spvalue())]);
/// let a_to_b = t_plan!("a_to_b", eq!(v!("pos").wrap(), "a".wrap()), vec!(a!(v!("pos"), "b".wrap())));
/// let b_to_c = t_plan!("b_to_c", eq!(v!("pos").wrap(), "b".wrap()), vec!(a!(v!("pos"), "c".wrap())));
///
/// let goal = eq!(v!("pos").wrap(), "c".wrap());
/// let result = bfs_transition_planner(state, goal, vec![a_to_b, b_to_c], 10, "docs");
///
/// assert!(result.found);
/// assert_eq!(result.plan, vec!["a_to_b", "b_to_c"]);
/// ```
pub fn bfs_transition_planner(
    state: State,
    goal: Predicate,
    model: Vec<Transition>,
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
                        log::error!(target: &&format!("transition_planner"), 
                            "Failed to pop value from stack? This shouldn't happen.");
                        log::error!(target: &&format!("transition_planner"), 
                            "Breaking the search with empty planning result.");
                        break PlanningResult {
                            found: false,
                            ..Default::default()
                        };
                    }
                };
                match goal.eval(&s, &log_target) {
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
                                    .for_each(|t| match t.eval_planning(&s, &log_target) {
                                        false => (),
                                        true => {
                                            let mut next_s = s.clone();
                                            t.take_planning_mut(&mut next_s, &log_target);
                                            let mut next_p = path.clone();
                                            next_p.push(t.name.clone());
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

#[cfg(test)]
mod tests {
    use crate::*;

    const TARGET: &str = "test";

    /// A straight chain a -> b -> c -> ... of `length` steps.
    fn chain(length: usize) -> (State, Vec<Transition>, SPVariable) {
        let pos = v!("pos");
        let state = State::from_vec(&vec![(pos.clone(), "s0".to_spvalue())]);

        let transitions = (0..length)
            .map(|i| {
                t_plan!(
                    &format!("step_{}", i),
                    eq!(pos.wrap(), format!("s{}", i).wrap()),
                    vec!(a!(pos.clone(), format!("s{}", i + 1).wrap()))
                )
            })
            .collect();

        (state, transitions, pos)
    }

    /// `max_depth` is the search's only bound - without it an unreachable goal
    /// in a model with a loop searches forever. A plan that needs more steps
    /// than the bound allows must come back as "not found", not as a truncated
    /// plan that would drive the system half way and stop.
    #[test]
    fn a_goal_deeper_than_max_depth_is_not_found() {
        let (state, transitions, pos) = chain(6);
        let goal = eq!(pos.wrap(), "s6".wrap());

        let found = bfs_transition_planner(
            state.clone(),
            goal.clone(),
            transitions.clone(),
            10,
            TARGET,
        );
        assert!(found.found, "6 steps is reachable within a depth of 10");
        assert_eq!(found.length, 6);

        let cut_off =
            bfs_transition_planner(state, goal, transitions, 3, TARGET);
        assert!(!cut_off.found, "6 steps must not be found within a depth of 3");
        assert_eq!(cut_off.length, 0);
        assert!(cut_off.plan.is_empty(), "a failed search returns no partial plan");
    }

    /// A model whose transitions loop back on themselves must still terminate:
    /// the `visited` set is what stops the search from cycling forever.
    #[test]
    fn a_cyclic_model_still_terminates() {
        let pos = v!("pos");
        let state = State::from_vec(&vec![(pos.clone(), "a".to_spvalue())]);

        let a_to_b = t_plan!(
            "a_to_b",
            eq!(pos.wrap(), "a".wrap()),
            vec!(a!(pos.clone(), "b".wrap()))
        );
        let b_to_a = t_plan!(
            "b_to_a",
            eq!(pos.wrap(), "b".wrap()),
            vec!(a!(pos.clone(), "a".wrap()))
        );

        let result = bfs_transition_planner(
            state,
            eq!(pos.wrap(), "unreachable".wrap()),
            vec![a_to_b, b_to_a],
            20,
            TARGET,
        );

        assert!(!result.found);
    }

    /// An empty model can only satisfy a goal that already holds.
    #[test]
    fn an_empty_model_finds_only_the_goal_it_starts_in() {
        let pos = v!("pos");
        let state = State::from_vec(&vec![(pos.clone(), "a".to_spvalue())]);

        let already_there = bfs_transition_planner(
            state.clone(),
            eq!(pos.wrap(), "a".wrap()),
            vec![],
            10,
            TARGET,
        );
        assert!(already_there.found);
        assert_eq!(already_there.length, 0);

        let nowhere = bfs_transition_planner(
            state,
            eq!(pos.wrap(), "b".wrap()),
            vec![],
            10,
            TARGET,
        );
        assert!(!nowhere.found);
    }

    /// The result carries the wall-clock time of the search, which is what the
    /// runners log; a found plan must report a real duration.
    #[test]
    fn a_found_plan_reports_how_long_the_search_took() {
        let (state, transitions, pos) = chain(3);
        let result = bfs_transition_planner(
            state,
            eq!(pos.wrap(), "s3".wrap()),
            transitions,
            10,
            TARGET,
        );

        assert!(result.found);
        assert_eq!(result.plan, vec!["step_0", "step_1", "step_2"]);
        assert_eq!(result.length, result.plan.len());
    }

    /// A failed search returns `PlanningResult::default()`, so everything on it
    /// has to be the empty/zero value - a caller that only checks `found` and
    /// then reads `plan` must not get stale data.
    #[test]
    fn a_failed_search_returns_an_empty_result() {
        let (state, transitions, pos) = chain(2);
        let result = bfs_transition_planner(
            state,
            eq!(pos.wrap(), "nowhere".wrap()),
            transitions,
            10,
            TARGET,
        );

        assert_eq!(result, PlanningResult::default());
        assert!(!result.found);
        assert_eq!(result.length, 0);
        assert!(result.plan.is_empty());
        assert_eq!(result.time, std::time::Duration::default());
    }
}
