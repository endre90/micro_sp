//! Breadth-first planning over [`Operation`]s.
//!
//! This is the planner `planner_ticker` calls: it searches over operation
//! guards and effects and returns the operation names to execute. The search
//! keeps only parent links rather than a copy of the path per frontier node, and
//! compares states by the model's own variables so unrelated state (transforms,
//! interface variables) cannot make two equivalent states look different.

use crate::*;
use std::{
    collections::{HashSet, VecDeque},
    time::{Duration, Instant},
};

/// One expanded search node: the operation taken, and where it was taken from.
struct PlanNode {
    parent: Option<usize>,
    operation: usize,
}

fn reconstruct_plan(nodes: &[PlanNode], from: Option<usize>, model: &[Operation]) -> Vec<String> {
    let mut plan = Vec::new();
    let mut cursor = from;
    while let Some(index) = cursor {
        plan.push(model[nodes[index].operation].name.clone());
        cursor = nodes[index].parent;
    }
    plan.reverse();
    plan
}

/// The variables that make up a planning state's identity for the `visited` set.
fn planning_identity_keys(model: &[Operation]) -> Vec<String> {
    let mut keys: Vec<String> = model
        .iter()
        .flat_map(|op| op.get_all_var_keys())
        .chain(model.iter().map(|op| op.name.clone()))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// The values of `keys` in `state`, in the fixed order of `keys`, so two
/// identities are comparable without carrying the key names around.
fn state_identity(state: &State, keys: &[String]) -> Vec<Option<SPValue>> {
    keys.iter()
        .map(|key| state.state.get(key).map(|assignment| assignment.val.clone()))
        .collect()
}

/// Breadth-first search for a sequence of operations that reaches `goal`.
///
/// Explores from `state`, applying any operation in `model` whose planning guard
/// holds, and returns the shortest sequence of operation names to the first
/// state satisfying `goal`. The search stops at `max_depth` steps or after
/// `deadline_ms` milliseconds, reporting `found: false` either way; `log_target`
/// is the log target used for guard evaluation diagnostics.
///
/// Pure - no Redis, no async - so it can be called from `spawn_blocking`, which
/// is what `planner_ticker` does.
pub fn bfs_operation_planner(
    state: &State,
    goal: &Predicate,
    model: &[Operation],
    max_depth: usize,
    log_target: &str,
    deadline_ms: u64,
) -> PlanningResult {
    let now = Instant::now();
    let limit = Duration::from_millis(deadline_ms);

    let identity_keys = planning_identity_keys(model);

    let mut nodes: Vec<PlanNode> = Vec::new();
    let mut visited: HashSet<Vec<Option<SPValue>>> = HashSet::new();
    let mut frontier: VecDeque<(State, Option<usize>, usize)> = VecDeque::new();
    frontier.push_back((state.clone(), None, 0));

    loop {
        if now.elapsed() > limit {
            break PlanningResult {
                found: false,
                time: now.elapsed(),
                ..Default::default()
            };
        }

        let Some((s, parent, depth)) = frontier.pop_front() else {
            break PlanningResult {
                found: false,
                ..Default::default()
            };
        };

        if goal.eval(&s, &log_target) {
            let plan = reconstruct_plan(&nodes, parent, model);
            break PlanningResult {
                found: true,
                length: plan.len(),
                plan,
                time: now.elapsed(),
            };
        }

        // Nodes come off the frontier in non-decreasing depth order, so the
        // first one past the limit means everything left is too deep as well.
        if depth > max_depth {
            break PlanningResult {
                found: false,
                ..Default::default()
            };
        }

        if !visited.insert(state_identity(&s, &identity_keys)) {
            continue;
        }

        for (index, operation) in model.iter().enumerate() {
            if operation.eval_planning(&s, &log_target) {
                let next_state = operation.take_planning(&s, &log_target);
                nodes.push(PlanNode {
                    parent,
                    operation: index,
                });
                frontier.push_back((next_state, Some(nodes.len() - 1), depth + 1));
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::*;

    const TARGET: &str = "planner_test";

    /// `n` operations, each setting its own flag, plus `unrelated` variables
    /// the model never mentions - the interface variables, transforms and
    /// runner bookkeeping that make up most of a real state.
    fn problem(n: usize, unrelated: usize) -> (State, Predicate, Vec<Operation>) {
        let mut state = State::new();

        // No operation writes this, so any goal that needs it is unreachable.
        state.add_mut(
            SPAssignment::new(SPVariable::new("locked", SPValueType::Bool), false.to_spvalue()),
            TARGET,
        );

        for i in 0..unrelated {
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&format!("unrelated_{}", i), SPValueType::String),
                    format!("value_{}", i).to_spvalue(),
                ),
                TARGET,
            );
        }

        for i in 0..n {
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&format!("v{}", i), SPValueType::Bool),
                    false.to_spvalue(),
                ),
                TARGET,
            );
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&format!("op_set{}", i), SPValueType::String),
                    "initial".to_spvalue(),
                ),
                TARGET,
            );
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(&format!("op_set{}_elapsed_executing_ms", i), SPValueType::Int64),
                    0.to_spvalue(),
                ),
                TARGET,
            );
        }

        let mut operations = Vec::new();
        for i in 0..n {
            operations.push(Operation::new(
                &format!("set{}", i),
                None,
                None,
                None,
                None,
                false,
                vec![Transition::parse(
                    &format!("start_set{}", i),
                    &format!("var:v{} == false", i),
                    "true",
                    vec![format!("var:v{} <- true", i).as_str()],
                    Vec::<&str>::new(),
                    &state,
                )],
                vec![Transition::parse(
                    &format!("complete_set{}", i),
                    "true",
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &state,
                )],
                vec![],
                vec![],
                vec![],
                vec![],
            ));
        }

        let model = Model::new("t", vec![], vec![], vec![], vec![], operations);

        let goal_str = (0..n)
            .map(|i| format!("var:v{} == true", i))
            .collect::<Vec<String>>()
            .join(" && ");
        let goal = pred_parser::pred(&goal_str, &state).unwrap();

        (state, goal, model.operations)
    }

    #[test]
    fn finds_a_plan_containing_every_operation_exactly_once() {
        let (state, goal, operations) = problem(5, 0);
        let result = bfs_operation_planner(&state, &goal, &operations, 30, TARGET, 10_000);

        assert!(result.found);
        assert_eq!(result.length, 5);
        assert_eq!(result.plan.len(), 5);

        let mut sorted = result.plan.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 5, "each operation should appear once: {:?}", result.plan);
        for step in &result.plan {
            assert!(
                operations.iter().any(|op| &op.name == step),
                "plan step '{step}' is not an operation in the model"
            );
        }
    }

    /// The plan is reconstructed from parent links now rather than carried
    /// along in the frontier, so it has to come back in execution order.
    #[test]
    fn the_reconstructed_plan_is_in_execution_order() {
        let mut state = State::new();
        for var in ["a", "b", "c"] {
            state.add_mut(
                SPAssignment::new(SPVariable::new(var, SPValueType::Bool), false.to_spvalue()),
                TARGET,
            );
        }
        for op in ["op_first", "op_second", "op_third"] {
            state.add_mut(
                SPAssignment::new(SPVariable::new(op, SPValueType::String), "initial".to_spvalue()),
                TARGET,
            );
        }

        // A strict chain: b needs a, c needs b. Only one ordering is possible.
        let chain = |name: &str, needs: &str, sets: &str, state: &State| {
            Operation::new(
                name,
                None,
                None,
                None,
                None,
                false,
                vec![Transition::parse(
                    &format!("start_{}", name),
                    &format!("var:{} == true && var:{} == false", needs, sets),
                    "true",
                    vec![format!("var:{} <- true", sets).as_str()],
                    Vec::<&str>::new(),
                    state,
                )],
                vec![Transition::parse(
                    &format!("complete_{}", name),
                    "true",
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    state,
                )],
                vec![],
                vec![],
                vec![],
                vec![],
            )
        };

        let first = Operation::new(
            "first",
            None,
            None,
            None,
            None,
            false,
            vec![Transition::parse(
                "start_first",
                "var:a == false",
                "true",
                vec!["var:a <- true"],
                Vec::<&str>::new(),
                &state,
            )],
            vec![Transition::parse(
                "complete_first",
                "true",
                "true",
                Vec::<&str>::new(),
                Vec::<&str>::new(),
                &state,
            )],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        let model = Model::new(
            "t",
            vec![],
            vec![],
            vec![],
            vec![],
            vec![
                chain("third", "b", "c", &state),
                chain("second", "a", "b", &state),
                first,
            ],
        );

        let goal = pred_parser::pred("var:c == true", &state).unwrap();
        let result = bfs_operation_planner(&state, &goal, &model.operations, 30, TARGET, 10_000);

        assert!(result.found);
        assert_eq!(result.plan, vec!["op_first", "op_second", "op_third"]);
    }

    /// Variables the model does not mention cannot change the outcome - they
    /// are no longer part of a state's planning identity.
    #[test]
    fn unrelated_state_variables_do_not_change_the_result() {
        let (bare_state, bare_goal, bare_ops) = problem(5, 0);
        let (padded_state, padded_goal, padded_ops) = problem(5, 250);

        let bare = bfs_operation_planner(&bare_state, &bare_goal, &bare_ops, 30, TARGET, 10_000);
        let padded =
            bfs_operation_planner(&padded_state, &padded_goal, &padded_ops, 30, TARGET, 10_000);

        assert!(bare.found && padded.found);
        assert_eq!(bare.plan, padded.plan);
    }

    /// Exhausts the whole reachable state space and reports failure - which
    /// only terminates because the `visited` set recognises states it has
    /// already expanded.
    #[test]
    fn reports_not_found_for_an_unreachable_goal() {
        let (state, _, operations) = problem(3, 0);
        let goal = pred_parser::pred("var:locked == true", &state).unwrap();

        let result = bfs_operation_planner(&state, &goal, &operations, 30, TARGET, 10_000);
        assert!(!result.found);
        assert!(result.plan.is_empty());
    }

    #[test]
    fn respects_max_depth() {
        let (state, goal, operations) = problem(5, 0);

        // A plan needs all five operations; two levels is not enough.
        let shallow = bfs_operation_planner(&state, &goal, &operations, 2, TARGET, 10_000);
        assert!(!shallow.found);

        let deep = bfs_operation_planner(&state, &goal, &operations, 30, TARGET, 10_000);
        assert!(deep.found);
    }

    #[test]
    fn an_already_satisfied_goal_yields_an_empty_plan() {
        let (state, _, operations) = problem(3, 0);
        let goal = pred_parser::pred("var:v0 == false", &state).unwrap();

        let result = bfs_operation_planner(&state, &goal, &operations, 30, TARGET, 10_000);
        assert!(result.found);
        assert_eq!(result.length, 0);
        assert!(result.plan.is_empty());
    }

    /// 20 operations means a reachable space of 2^20 states, far more than a
    /// 50 ms budget allows. The deadline has to cut the search off rather than
    /// let it run to exhaustion.
    #[test]
    fn gives_up_at_the_deadline() {
        let (state, _, operations) = problem(20, 0);
        let goal = pred_parser::pred("var:locked == true", &state).unwrap();

        let started = std::time::Instant::now();
        let result = bfs_operation_planner(&state, &goal, &operations, 30, TARGET, 50);
        let elapsed = started.elapsed();

        assert!(!result.found);
        assert!(
            elapsed < std::time::Duration::from_secs(5),
            "the deadline should have stopped the search, took {:?}",
            elapsed
        );
    }
}
