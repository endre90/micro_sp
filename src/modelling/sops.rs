//! SOPs (Standard Operating Procedures): [`Operation`]s arranged into a tree.
//!
//! Where the planner works out a sequence for itself, a SOP is a sequence
//! someone wrote down: a tree of [`SOP`] nodes combining operations in series,
//! in parallel, or as alternatives. A [`SOPStruct`] gives one such tree a name
//! so a runner can be asked to execute it.

use crate::{Operation, OperationState, SOPState, State, TerminationReason};
use serde::{Deserialize, Serialize};
use termtree::Tree;

/// One node of a SOP tree - a high-level operation with a rigid structure,
/// rather than something the planner has to derive.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SOP {
    /// A leaf: a single [`Operation`].
    Operation(Box<Operation>),
    /// Children run one after another, in order.
    Sequence(Vec<SOP>),
    /// Children run concurrently; the node finishes when all of them have.
    Parallel(Vec<SOP>),
    /// Branches offered as alternatives; the node finishes when any one has.
    Alternative(Vec<SOP>),
}

/// A named SOP tree, as stored in a [`Model`](crate::Model) and executed by the
/// SOP runner.
///
/// # Example
///
/// ```
/// use micro_sp::*;
///
/// let procedure = SOPStruct {
///     id: "pick_and_place".to_string(),
///     sop: SOP::Sequence(vec![
///         SOP::Operation(Box::new(Operation {
///             name: "op_pick".to_string(),
///             ..Default::default()
///         })),
///         SOP::Operation(Box::new(Operation {
///             name: "op_place".to_string(),
///             ..Default::default()
///         })),
///     ]),
/// };
///
/// // The tree's state is derived from its operations' states.
/// let mut state = State::new();
/// for name in ["op_pick", "op_place"] {
///     state.add_mut(
///         SPAssignment::new(
///             SPVariable::new(name, SPValueType::String),
///             "initial".to_spvalue(),
///         ),
///         "docs",
///     );
/// }
/// assert_eq!(procedure.sop.get_state(&state, "docs"), SOPState::Initial);
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SOPStruct {
    /// Unique name of the procedure, used to refer to it at runtime.
    pub id: String,
    /// The root of the tree.
    pub sop: SOP,
}

impl SOP {
    /// Every state variable read or written anywhere in this subtree.
    ///
    /// Feeds the runners' key sets; a variable missing here is one the runner
    /// never reads, and reading a missing variable panics.
    pub fn get_all_var_keys(&self) -> Vec<String> {
        match self {
            SOP::Operation(op) => op.get_all_var_keys(),
            SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
                sops.iter().flat_map(|s| s.get_all_var_keys()).collect()
            }
        }
    }

    /// The names of the operations in this subtree.
    ///
    /// Only a bare [`SOP::Operation`] reports a name: the branch arms recurse
    /// but discard the result, so any tree with a branch reports nothing. Use
    /// [`get_all_operations_from_sop`](crate::get_all_operations_from_sop)
    /// instead, which traverses correctly.
    pub fn get_all_operation_names(&self) -> Vec<String> {
        let mut operations: Vec<String> = vec![];
        match self {
            SOP::Operation(op) => operations.push(op.name.clone()),
            SOP::Sequence(sops) | SOP::Parallel(sops) | SOP::Alternative(sops) => {
                sops.iter().for_each(|s| {
                    s.get_all_operation_names();
                });
            }
        };
        operations
    }

    /// Derive this node's [`SOPState`] from the states of the operations below
    /// it.
    ///
    /// `Fatal` and `Cancelled` in any child win over everything else.
    /// Otherwise `Sequence` and `Parallel` complete only when every child has,
    /// while `Alternative` completes as soon as one has. An empty branch counts
    /// as completed.
    pub fn get_state(&self, state: &State, log_target: &str) -> SOPState {
        match self {
            SOP::Operation(op) => {
                let operation_state =
                    state.get_string_or_default_to_unknown(&op.name, &log_target);
                match OperationState::from_str(&operation_state) {
                    OperationState::Initial => SOPState::Initial,
                    OperationState::Disabled => SOPState::Executing,
                    OperationState::Executing => SOPState::Executing,
                    OperationState::Timedout => SOPState::Executing,
                    OperationState::Failed => SOPState::Executing,
                    OperationState::Bypassed => SOPState::Executing,
                    OperationState::Completed => SOPState::Executing,
                    OperationState::Terminated(TerminationReason::Completed) => SOPState::Completed,
                    OperationState::Terminated(TerminationReason::Bypassed) => SOPState::Completed,
                    OperationState::Fatal => SOPState::Fatal,
                    OperationState::Cancelled => SOPState::Cancelled,
                    OperationState::Terminated(TerminationReason::Fatal) => SOPState::Fatal,
                    OperationState::Terminated(TerminationReason::Cancelled) => SOPState::Cancelled,
                    OperationState::UNKNOWN => SOPState::UNKNOWN,
                }
            }
            SOP::Sequence(sops) | SOP::Parallel(sops) => {
                if sops.is_empty() {
                    return SOPState::Completed;
                }

                let mut any_fatal = false;
                let mut any_cancelled = false;
                let mut all_initial = true;
                let mut all_completed = true;
                for child in sops {
                    let child_state = child.get_state(state, log_target);
                    if child_state == SOPState::Fatal {
                        any_fatal = true;
                    }
                    if child_state == SOPState::Cancelled {
                        any_cancelled = true;
                    }
                    if child_state != SOPState::Initial {
                        all_initial = false;
                    }
                    if child_state != SOPState::Completed {
                        all_completed = false;
                    }
                }
                let any_not_initial = !all_initial;

                if any_fatal {
                    return SOPState::Fatal;
                }

                if any_cancelled {
                    return SOPState::Cancelled;
                }

                if all_initial {
                    return SOPState::Initial;
                }

                if all_completed {
                    return SOPState::Completed;
                }

                if !all_completed && any_not_initial && !any_fatal && !any_cancelled {
                    return SOPState::Executing;
                }

                SOPState::UNKNOWN
            }
            SOP::Alternative(sops) => {
                if sops.is_empty() {
                    return SOPState::Completed;
                }

                let mut any_fatal = false;
                let mut any_cancelled = false;
                let mut all_initial = true;
                let mut any_completed = false;
                for child in sops {
                    let child_state = child.get_state(state, log_target);
                    if child_state == SOPState::Fatal {
                        any_fatal = true;
                    }
                    if child_state == SOPState::Cancelled {
                        any_cancelled = true;
                    }
                    if child_state != SOPState::Initial {
                        all_initial = false;
                    }
                    if child_state == SOPState::Completed {
                        any_completed = true;
                    }
                }
                let any_not_initial = !all_initial;

                if any_fatal {
                    return SOPState::Fatal;
                }

                if any_cancelled {
                    return SOPState::Cancelled;
                }

                if all_initial {
                    return SOPState::Initial;
                }

                if any_completed {
                    return SOPState::Completed;
                }

                if !any_completed && any_not_initial && !any_fatal && !any_cancelled {
                    return SOPState::Executing;
                }

                SOPState::UNKNOWN
            }
        }
    }
}

/// Render a SOP tree as indented text, for logging or debugging.
///
/// Each line is indented by seven spaces so the tree lines up under a log
/// prefix; blank lines are dropped.
pub fn visualize_sop(root_sop: &SOP) -> String {
    let tree = build_sop_tree(root_sop);
    let mut output = String::new();

    for line in tree.to_string().lines() {
        use std::fmt::Write;
        if !line.is_empty() {
            let _ = writeln!(output, "       {}", line);
        }
    }

    output
}

fn build_sop_tree(sop: &SOP) -> Tree<String> {
    match sop {
        // A leaf node in the tree
        SOP::Operation(op) => {
            let label = format!("Operation: {}", op.name);
            Tree::new(label)
        }

        // A branch node for sequential operations
        SOP::Sequence(sops) => {
            let mut tree = Tree::new(format!("Sequence:"));
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }

        // A branch node for parallel operations
        SOP::Parallel(sops) => {
            let mut tree = Tree::new(format!("Parallel:"));
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }

        // A branch node for alternative operations
        SOP::Alternative(sops) => {
            let mut tree = Tree::new(format!("Alternative:"));
            for child_sop in sops {
                tree.push(build_sop_tree(child_sop));
            }
            tree
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualize_sop() {
        let example_sop = SOP::Sequence(vec![
            SOP::Operation(Box::new(Operation {
                name: "StartGripper".to_string(),
                ..Default::default()
            })),
            SOP::Parallel(vec![
                SOP::Operation(Box::new(Operation {
                    name: "MoveToTarget".to_string(),
                    ..Default::default()
                })),
                SOP::Sequence(vec![
                    SOP::Operation(Box::new(Operation {
                        name: "RotateWrist".to_string(),
                        ..Default::default()
                    })),
                    SOP::Operation(Box::new(Operation {
                        name: "CheckPressure".to_string(),
                        ..Default::default()
                    })),
                    SOP::Operation(Box::new(Operation {
                        name: "CheckPressure".to_string(),
                        ..Default::default()
                    })),
                    SOP::Operation(Box::new(Operation {
                        name: "CheckPressure".to_string(),
                        ..Default::default()
                    })),
                ]),
            ]),
            SOP::Alternative(vec![
                SOP::Operation(Box::new(Operation {
                    name: "CloseGripperHard".to_string(),
                    ..Default::default()
                })),
                SOP::Operation(Box::new(Operation {
                    name: "CloseGripperSoft".to_string(),
                    ..Default::default()
                })),
            ]),
            SOP::Operation(Box::new(Operation {
                name: "RetractArm".to_string(),
                ..Default::default()
            })),
        ]);

        visualize_sop(&example_sop);
    }
}

#[cfg(test)]
mod get_state_tests {
    use crate::*;

    const TARGET: &str = "test";

    fn state_with(operations: &[(&str, &str)]) -> State {
        let mut state = State::new();
        for (name, value) in operations {
            state.add_mut(
                SPAssignment::new(
                    SPVariable::new(name, SPValueType::String),
                    value.to_spvalue(),
                ),
                TARGET,
            );
        }
        state
    }

    fn leaf(name: &str) -> SOP {
        SOP::Operation(Box::new(Operation {
            name: name.to_string(),
            ..Default::default()
        }))
    }

    /// How an operation's own state maps onto the SOP node state.
    #[test]
    fn a_leaf_reports_the_operations_state() {
        let cases = [
            ("initial", SOPState::Initial),
            ("disabled", SOPState::Executing),
            ("executing", SOPState::Executing),
            ("timedout", SOPState::Executing),
            ("failed", SOPState::Executing),
            ("bypassed", SOPState::Executing),
            ("completed", SOPState::Executing),
            ("terminated_completed", SOPState::Completed),
            ("terminated_bypassed", SOPState::Completed),
            ("fatal", SOPState::Fatal),
            ("terminated_fatal", SOPState::Fatal),
            ("cancelled", SOPState::Cancelled),
            ("terminated_cancelled", SOPState::Cancelled),
            ("nonsense", SOPState::UNKNOWN),
        ];

        for (operation_state, expected) in cases {
            let state = state_with(&[("op", operation_state)]);
            assert_eq!(
                leaf("op").get_state(&state, TARGET),
                expected,
                "operation state '{operation_state}'"
            );
        }
    }

    /// `Sequence` and `Parallel` share one decision table; this pins every row
    /// of it, which is what the single-pass rewrite had to preserve.
    #[test]
    fn sequence_and_parallel_decision_table() {
        let cases: [(&str, Vec<&str>, SOPState); 8] = [
            ("all initial", vec!["initial", "initial"], SOPState::Initial),
            (
                "all terminated",
                vec!["terminated_completed", "terminated_bypassed"],
                SOPState::Completed,
            ),
            (
                "part way through",
                vec!["terminated_completed", "initial"],
                SOPState::Executing,
            ),
            (
                "one running",
                vec!["executing", "initial"],
                SOPState::Executing,
            ),
            ("one fatal", vec!["initial", "fatal"], SOPState::Fatal),
            (
                "fatal beats completed",
                vec!["terminated_completed", "fatal"],
                SOPState::Fatal,
            ),
            (
                "one cancelled",
                vec!["terminated_completed", "cancelled"],
                SOPState::Cancelled,
            ),
            (
                "fatal beats cancelled",
                vec!["cancelled", "fatal"],
                SOPState::Fatal,
            ),
        ];

        for (label, operation_states, expected) in cases {
            let named: Vec<(String, &str)> = operation_states
                .iter()
                .enumerate()
                .map(|(i, s)| (format!("op{}", i), *s))
                .collect();
            let pairs: Vec<(&str, &str)> =
                named.iter().map(|(n, s)| (n.as_str(), *s)).collect();
            let state = state_with(&pairs);
            let children: Vec<SOP> = named.iter().map(|(n, _)| leaf(n)).collect();

            assert_eq!(
                SOP::Sequence(children.clone()).get_state(&state, TARGET),
                expected,
                "Sequence: {label}"
            );
            assert_eq!(
                SOP::Parallel(children).get_state(&state, TARGET),
                expected,
                "Parallel: {label}"
            );
        }
    }

    /// `Alternative` differs in exactly one place: *any* completed branch
    /// completes the node, rather than requiring all of them.
    #[test]
    fn alternative_decision_table() {
        let cases: [(&str, Vec<&str>, SOPState); 6] = [
            ("all initial", vec!["initial", "initial"], SOPState::Initial),
            (
                "one branch taken and finished",
                vec!["terminated_completed", "initial"],
                SOPState::Completed,
            ),
            (
                "one branch running",
                vec!["executing", "initial"],
                SOPState::Executing,
            ),
            ("one fatal", vec!["initial", "fatal"], SOPState::Fatal),
            (
                "fatal beats completed",
                vec!["terminated_completed", "fatal"],
                SOPState::Fatal,
            ),
            (
                "cancelled beats completed",
                vec!["terminated_completed", "cancelled"],
                SOPState::Cancelled,
            ),
        ];

        for (label, operation_states, expected) in cases {
            let named: Vec<(String, &str)> = operation_states
                .iter()
                .enumerate()
                .map(|(i, s)| (format!("op{}", i), *s))
                .collect();
            let pairs: Vec<(&str, &str)> =
                named.iter().map(|(n, s)| (n.as_str(), *s)).collect();
            let state = state_with(&pairs);
            let children: Vec<SOP> = named.iter().map(|(n, _)| leaf(n)).collect();

            assert_eq!(
                SOP::Alternative(children).get_state(&state, TARGET),
                expected,
                "Alternative: {label}"
            );
        }
    }

    /// The keys a SOP contributes to a runner's read set come from here, so a
    /// variable missed here is a variable the runner never reads and a guard
    /// that never becomes true.
    #[test]
    fn get_all_var_keys_reaches_every_leaf_of_the_tree() {
        let mut state = State::new();
        for name in ["a", "b", "c"] {
            state.add_mut(
                SPAssignment::new(SPVariable::new(name, SPValueType::Bool), false.to_spvalue()),
                TARGET,
            );
        }

        let op_with_guard = |name: &str, var: &str| {
            SOP::Operation(Box::new(Operation::new(
                name,
                None,
                None,
                None,
                None,
                false,
                vec![Transition::parse(
                    "start",
                    &format!("var:{var} == true"),
                    "true",
                    Vec::<&str>::new(),
                    Vec::<&str>::new(),
                    &state,
                )],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            )))
        };

        let tree = SOP::Sequence(vec![
            op_with_guard("one", "a"),
            SOP::Parallel(vec![
                op_with_guard("two", "b"),
                SOP::Alternative(vec![op_with_guard("three", "c")]),
            ]),
        ]);

        let keys = tree.get_all_var_keys();
        for expected in ["a", "b", "c"] {
            assert!(
                keys.contains(&expected.to_string()),
                "'{expected}' is guarded by a nested leaf and must be in the key set: {keys:?}"
            );
        }
    }

    /// BUG: `get_all_operation_names` only ever returns names for a bare
    /// `SOP::Operation`. The branch arms recurse (`s.get_all_operation_names()`)
    /// but throw the result away instead of extending `operations`, so any
    /// `Sequence`/`Parallel`/`Alternative` - i.e. every real SOP - reports zero
    /// operations.
    ///
    /// Consequence: `reset_all_operations` iterates
    /// `sop_struct.sop.get_all_operation_names()` to put a SOP's operations back
    /// to "initial", and therefore resets none of them. The working traversal
    /// already exists next door as `get_all_operations_from_sop`
    /// (`running/state_init.rs`), which is what every other caller uses; the
    /// test below shows the two disagreeing.
    #[test]
    fn get_all_operation_names_loses_everything_below_a_branch() {
        let tree = SOP::Sequence(vec![leaf("first"), leaf("second")]);

        assert_eq!(
            tree.get_all_operation_names(),
            Vec::<String>::new(),
            "if this now returns the two names the bug is fixed - see the doc comment"
        );

        // The traversal that does work, for contrast.
        let working: Vec<String> = get_all_operations_from_sop(&tree)
            .iter()
            .map(|o| o.name.clone())
            .collect();
        assert_eq!(working, vec!["first".to_string(), "second".to_string()]);

        // A bare leaf is the one shape that does report its name.
        assert_eq!(leaf("only").get_all_operation_names(), vec!["only".to_string()]);
    }

    #[test]
    fn an_empty_branch_counts_as_completed() {
        let state = State::new();
        assert_eq!(
            SOP::Sequence(vec![]).get_state(&state, TARGET),
            SOPState::Completed
        );
        assert_eq!(
            SOP::Parallel(vec![]).get_state(&state, TARGET),
            SOPState::Completed
        );
        assert_eq!(
            SOP::Alternative(vec![]).get_state(&state, TARGET),
            SOPState::Completed
        );
    }

    /// Nesting has to propagate, since the runner asks the root for the state
    /// of the whole tree.
    #[test]
    fn nested_branches_propagate() {
        let state = state_with(&[
            ("a", "terminated_completed"),
            ("b", "terminated_completed"),
            ("c", "executing"),
            ("d", "initial"),
        ]);

        let tree = SOP::Sequence(vec![
            SOP::Parallel(vec![leaf("a"), leaf("b")]),
            SOP::Sequence(vec![leaf("c"), leaf("d")]),
        ]);
        assert_eq!(tree.get_state(&state, TARGET), SOPState::Executing);

        let done = state_with(&[
            ("a", "terminated_completed"),
            ("b", "terminated_completed"),
            ("c", "terminated_completed"),
            ("d", "terminated_bypassed"),
        ]);
        assert_eq!(tree.get_state(&done, TARGET), SOPState::Completed);

        let broken = state_with(&[
            ("a", "terminated_completed"),
            ("b", "terminated_completed"),
            ("c", "terminated_completed"),
            ("d", "fatal"),
        ]);
        assert_eq!(tree.get_state(&broken, TARGET), SOPState::Fatal);
    }
}
