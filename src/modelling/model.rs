//! The [`Model`]: everything a system is capable of doing.
//!
//! A model is the top-level artefact a user writes and hands to the runners. It
//! collects the planned [`Operation`]s, the automatic behaviour that runs
//! outside any plan, and the [`SOPStruct`]s available as ready-made procedures.

use crate::*;
use serde::{Deserialize, Serialize};

/// A model contains behavior that defines what a system is capable of doing.
///
/// # Example
///
/// ```
/// use micro_sp::*;
///
/// let mut state = State::new();
/// state.add_mut(
///     SPAssignment::new(SPVariable::new("pos", SPValueType::String), "a".to_spvalue()),
///     "docs",
/// );
///
/// let blink = Transition::parse(
///     "blink",
///     "var:pos == a",
///     "true",
///     vec!["var:pos <- b"],
///     Vec::<&str>::new(),
///     &state,
/// );
///
/// let model = Model::new("demo", vec![blink], vec![], vec![], vec![], vec![]);
/// assert_eq!(model.name, "demo");
/// assert_eq!(model.auto_transitions.len(), 1);
/// ```
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Name of the model, used for logging and as a human-readable handle.
    pub name: String,
    /// Transitions the automatic transition runner takes whenever their guards
    /// hold, independently of any plan.
    pub auto_transitions: Vec<Transition>,
    /// Operations the automatic operation runner drives on its own, without the
    /// planner scheduling them.
    pub auto_operations: Vec<Operation>,
    /// Automatic operations of which only one may execute at a time.
    pub mutexed_auto_operations: Vec<Operation>,
    /// Ready-made procedures the SOP runner can be asked to execute.
    pub sops: Vec<SOPStruct>,
    /// The operations the planner may sequence into a plan.
    pub operations: Vec<Operation>,
}

impl Model {
    /// Build a model from its parts.
    ///
    /// Every operation - automatic, mutexed and planned alike - gets its name
    /// prefixed with `op_`, which is the name the runners and the state use for
    /// its lifecycle variable. `auto_transitions` and `sops` are stored as
    /// given.
    ///
    /// # Example
    ///
    /// ```
    /// use micro_sp::*;
    ///
    /// let operation = Operation {
    ///     name: "move".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// let model = Model::new("demo", vec![], vec![], vec![], vec![], vec![operation]);
    /// assert_eq!(model.operations[0].name, "op_move");
    /// ```
    pub fn new(
        name: &str,
        auto_transitions: Vec<Transition>,
        auto_operations: Vec<Operation>,
        mutexed_auto_operations: Vec<Operation>,
        sops: Vec<SOPStruct>,
        operations: Vec<Operation>,
    ) -> Model {
        Model {
            name: name.to_string(),
            auto_transitions,
            auto_operations: auto_operations
                .iter()
                .map(|o| Operation {
                    name: format!("op_{}", o.name),
                    timeout_executing_ms: o.timeout_executing_ms,
                    timeout_disabled_ms: o.timeout_disabled_ms,
                    failure_retries: o.failure_retries,
                    timeout_retries: o.timeout_retries,
                    can_be_bypassed: o.can_be_bypassed,
                    preconditions: o.preconditions.clone(),
                    postconditions: o.postconditions.clone(),
                    failure_transitions: o.failure_transitions.clone(),
                    timeout_transitions: o.timeout_transitions.clone(),
                    bypass_transitions: o.bypass_transitions.clone(),
                    cancel_transitions: o.cancel_transitions.clone(),
                    state: o.state.clone(),
                })
                .collect(),
            mutexed_auto_operations: mutexed_auto_operations
                .iter()
                .map(|o| Operation {
                    name: format!("op_{}", o.name),
                    timeout_executing_ms: o.timeout_executing_ms,
                    timeout_disabled_ms: o.timeout_disabled_ms,
                    failure_retries: o.failure_retries,
                    timeout_retries: o.timeout_retries,
                    can_be_bypassed: o.can_be_bypassed,
                    preconditions: o.preconditions.clone(),
                    postconditions: o.postconditions.clone(),
                    failure_transitions: o.failure_transitions.clone(),
                    timeout_transitions: o.timeout_transitions.clone(),
                    bypass_transitions: o.bypass_transitions.clone(),
                    cancel_transitions: o.cancel_transitions.clone(),
                    state: o.state.clone(),
                })
                .collect(),
            sops,
            operations: operations
                .iter()
                .map(|o| Operation {
                    name: format!("op_{}", o.name),
                    timeout_executing_ms: o.timeout_executing_ms,
                    timeout_disabled_ms: o.timeout_disabled_ms,
                    failure_retries: o.failure_retries,
                    timeout_retries: o.timeout_retries,
                    can_be_bypassed: o.can_be_bypassed,
                    preconditions: o.preconditions.clone(),
                    postconditions: o.postconditions.clone(),
                    failure_transitions: o.failure_transitions.clone(),
                    timeout_transitions: o.timeout_transitions.clone(),
                    bypass_transitions: o.bypass_transitions.clone(),
                    cancel_transitions: o.cancel_transitions.clone(),
                    state: o.state.clone(),
                })
                .collect(),
        }
    }

}
