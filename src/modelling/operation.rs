//! [`Operation`]s: a [`Transition`] wrapped in a lifecycle.
//!
//! An operation is the unit and the runners execute. It can be scheduled by the
//! planner or taken automatically. 
//! 
//! It goes from `initial` through `executing` to `completed`, with branches for
//! failure, timeout, bypass and cancellation, and each branch carries its own
//! transitions. The current state lives in the [`State`] under the operation's
//! own name, which is why every method here takes a state and returns a new one.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::*;

/// Where an [`Operation`] is in its lifecycle.
///
/// Stored in the [`State`] as a lowercase string under the operation's name;
/// see [`OperationState::as_str`] and [`OperationState::from_str`].
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum OperationState {
    /// Planned and ready to start, waiting for a precondition to hold.
    Initial,
    /// Cannot start yet: no precondition guard is true.
    Disabled,
    /// A precondition fired and its actions were taken; the operation is running.
    Executing,
    /// A postcondition fired and its actions were taken; the operation succeeded.
    Completed,
    /// A failed or timed-out operation was waved through instead of retried.
    Bypassed,
    /// The operation stayed in `Executing` (or `Disabled`) past its deadline.
    Timedout,
    /// A failure transition fired.
    Failed,
    /// Failed or timed out with no retries left; unrecoverable without a replan.
    Fatal,
    /// Stopped on request, e.g. a `stop` dashboard command.
    Cancelled,
    /// Finished for good, with the reason it finished. This is the state a SOP
    /// or plan runner waits for before moving on.
    // Paused on request, e.g. a `pause` dashboard command. Can be continued.
    // Paused,
    Terminated(TerminationReason),
    /// Not yet initialized, or a value that does not parse as any of the above.
    UNKNOWN,
}

/// Why an [`Operation`] reached [`OperationState::Terminated`].
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum TerminationReason {
    /// Terminated after completing successfully.
    Completed,
    /// Terminated after being bypassed.
    Bypassed,
    /// Terminated after becoming fatal.
    Fatal,
    /// Terminated after being cancelled.
    Cancelled,
}

impl Default for OperationState {
    fn default() -> Self {
        OperationState::UNKNOWN
    }
}

impl OperationState {
    /// Parse the lowercase name produced by [`OperationState::as_str`].
    ///
    /// Anything unrecognised becomes [`OperationState::UNKNOWN`] rather than an
    /// error, so a garbled state value degrades instead of panicking.
    pub fn from_str(x: &str) -> OperationState {
        match x {
            "initial" => OperationState::Initial,
            "disabled" => OperationState::Disabled,
            "executing" => OperationState::Executing,
            "timedout" => OperationState::Timedout,
            "failed" => OperationState::Failed,
            "fatal" => OperationState::Fatal,
            "completed" => OperationState::Completed,
            "bypassed" => OperationState::Bypassed,
            "cancelled" => OperationState::Cancelled,
            // "paused" => OperationState::Paused,
            "terminated_completed" => OperationState::Terminated(TerminationReason::Completed),
            "terminated_bypassed" => OperationState::Terminated(TerminationReason::Bypassed),
            "terminated_fatal" => OperationState::Terminated(TerminationReason::Fatal),
            "terminated_cancelled" => OperationState::Terminated(TerminationReason::Cancelled),
            _ => OperationState::UNKNOWN,
        }
    }
    /// The value as written into the [`State`].
    ///
    /// Note [`OperationState::UNKNOWN`] becomes `SPValue::String(UNKNOWN)`, the
    /// unknown *variant*, not the literal string `"UNKNOWN"`.
    pub fn to_spvalue(self) -> SPValue {
        self.to_string().to_spvalue()
    }

    /// The same text `Display` produces, without allocating.
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationState::Initial => "initial",
            OperationState::Disabled => "disabled",
            OperationState::Executing => "executing",
            OperationState::Timedout => "timedout",
            OperationState::Failed => "failed",
            OperationState::Fatal => "fatal",
            OperationState::Completed => "completed",
            OperationState::Bypassed => "bypassed",
            OperationState::Cancelled => "cancelled",
            // OperationState::Paused => "paused",
            OperationState::Terminated(TerminationReason::Completed) => "terminated_completed",
            OperationState::Terminated(TerminationReason::Bypassed) => "terminated_bypassed",
            OperationState::Terminated(TerminationReason::Fatal) => "terminated_fatal",
            OperationState::Terminated(TerminationReason::Cancelled) => "terminated_cancelled",
            OperationState::UNKNOWN => "UNKNOWN",
        }
    }
}

/// Allocation-free equivalent of `value == expected.to_spvalue()`.
fn value_is(value: &SPValue, expected: OperationState) -> bool {
    let expected_is_unknown = matches!(expected, OperationState::UNKNOWN);
    match value {
        SPValue::String(StringOrUnknown::UNKNOWN) => expected_is_unknown,
        SPValue::String(StringOrUnknown::String(s)) => {
            !expected_is_unknown && s == expected.as_str()
        }
        _ => false,
    }
}

impl fmt::Display for OperationState {
    /// Delegates to [`OperationState::as_str`] so the two cannot drift apart -
    /// the allocation-free comparisons depend on them agreeing exactly.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A [`Transition`] wrapped in a lifecycle: something the system can be asked
/// to do, with preconditions, postconditions and failure handling.
///
/// The operation's live state is *not* the `state` field but the value stored
/// in the [`State`] under `name`; the methods here read and write that. Build
/// one with [`Operation::new`].
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    /// Unique name; also the state variable holding this operation's state in Redis.
    pub name: String,
    /// Initial state stamped into the model. The state at runtime
    /// is the value stored under [`Operation::name`] in the [`State`].
    pub state: OperationState,
    /// Deadline in `executing`, in milliseconds. `None` disables the timeout.
    pub timeout_executing_ms: Option<i64>,
    /// Deadline in `disabled`, in milliseconds. `None` disables the timeout.
    pub timeout_disabled_ms: Option<i64>,
    /// How many times a failed operation may be retried before going fatal.
    pub failure_retries: i64,
    /// How many times a timed-out operation may be retried before going fatal.
    pub timeout_retries: i64,
    /// Whether a failed or timed-out operation may be bypassed and the plan
    /// (or SOP) carried on regardless.
    pub can_be_bypassed: bool,
    /// Guards that start the operation; the first one that holds is taken.
    pub preconditions: Vec<Transition>,
    /// Guards that complete the operation; the first one that holds is taken.
    pub postconditions: Vec<Transition>,
    /// Guards that fail the operation while it is executing.
    pub failure_transitions: Vec<Transition>,
    /// Guards checked before bypassing. If any are declared, one must hold or
    /// the operation cannot be bypassed at all; if none are, bypass is
    /// unconditional.
    pub bypass_transitions: Vec<Transition>,
    /// Guards checked before timing out, with the same all-or-nothing rule as
    /// [`Operation::bypass_transitions`].
    pub timeout_transitions: Vec<Transition>,
    /// Extra assignments to make when the operation is cancelled.
    pub cancel_transitions: Vec<Transition>,
}

impl Default for Operation {
    fn default() -> Self {
        Operation {
            name: "unknown".to_string(),
            state: OperationState::UNKNOWN,
            timeout_executing_ms: None,
            timeout_disabled_ms: None,
            failure_retries: 0,
            timeout_retries: 0,
            can_be_bypassed: false,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            failure_transitions: Vec::new(),
            timeout_transitions: Vec::new(),
            bypass_transitions: Vec::new(),
            cancel_transitions: Vec::new(),
        }
    }
}

impl Operation {
    /// Define an operation.
    ///
    /// The state is set to [`OperationState::UNKNOWN`]; the runners call
    /// [`Operation::initialize`] to put it in `initial`.
    ///
    /// # Arguments
    ///
    /// * `name` - unique, and also the name of the state variable that will
    ///   hold this operation's state.
    /// * `timeout_executing_ms` - deadline while executing. `None` means
    ///   [`MAX_ALLOWED_OPERATION_DURATION_MS`] (10 minutes), *not* "no timeout".
    /// * `timeout_disabled_ms` - deadline while disabled, same `None` default.
    /// * `fail_retries` - retries allowed after a failure; `None` means 0.
    /// * `timeout_retries` - retries allowed after a timeout; `None` means 0.
    /// * `can_be_bypassed` - whether a failed or timed-out operation may be
    ///   waved through instead of turning fatal.
    /// * `preconditions` / `postconditions` - guards that start and complete
    ///   the operation; the first one that holds is taken.
    /// * `failure_transitions`, `timeout_transitions`, `bypass_transitions`,
    ///   `cancel_transitions` - the off-nominal branches. An empty
    ///   `timeout_transitions` or `bypass_transitions` makes that branch
    ///   unconditional; a non-empty one can *prevent* it.
    ///
    /// # Example
    ///
    /// ```
    /// use micro_sp::*;
    ///
    /// let mut state = State::new();
    /// state.add_mut(
    ///     SPAssignment::new(SPVariable::new("gripper", SPValueType::String), "open".to_spvalue()),
    ///     "docs",
    /// );
    /// // Every operation needs a variable of its own to hold its state.
    /// state.add_mut(
    ///     SPAssignment::new(SPVariable::new("op_close", SPValueType::String), "initial".to_spvalue()),
    ///     "docs",
    /// );
    ///
    /// let close = Operation::new(
    ///     "op_close",
    ///     Some(5000), // give up after 5s of executing
    ///     None,       // disabled timeout defaults to MAX_ALLOWED_OPERATION_DURATION_MS
    ///     Some(2),    // two retries after a failure
    ///     None,       // no retries after a timeout
    ///     false,      // may not be bypassed
    ///     vec![Transition::parse(
    ///         "start", "var:gripper == open", "true",
    ///         vec!["var:gripper <- closing"], Vec::<&str>::new(), &state,
    ///     )],
    ///     vec![Transition::parse(
    ///         "finish", "var:gripper == closing", "true",
    ///         vec!["var:gripper <- closed"], Vec::<&str>::new(), &state,
    ///     )],
    ///     vec![], // failure
    ///     vec![], // timeout
    ///     vec![], // bypass
    ///     vec![], // cancel
    /// );
    ///
    /// assert!(close.eval(&state, "docs"));
    /// let state = close.start(&state, "docs");
    /// assert_eq!(state.get_value("op_close", "docs"), Some("executing".to_spvalue()));
    ///
    /// assert!(close.can_be_completed(&state, "docs"));
    /// let state = close.complete(&state, "docs");
    /// assert_eq!(state.get_value("gripper", "docs"), Some("closed".to_spvalue()));
    /// ```
    pub fn new(
        name: &str,
        timeout_executing_ms: Option<i64>,
        timeout_disabled_ms: Option<i64>,
        fail_retries: Option<i64>,
        timeout_retries: Option<i64>,
        can_be_bypassed: bool,
        preconditions: Vec<Transition>,
        postconditions: Vec<Transition>,
        failure_transitions: Vec<Transition>,
        timeout_transitions: Vec<Transition>,
        bypass_transitions: Vec<Transition>,
        cancel_transitions: Vec<Transition>,
    ) -> Operation {
        Operation {
            name: name.to_string(),
            state: OperationState::UNKNOWN,
            timeout_executing_ms: match timeout_executing_ms {
                None => Some(MAX_ALLOWED_OPERATION_DURATION_MS),
                Some(x) => Some(x),
            },
            timeout_disabled_ms: match timeout_disabled_ms {
                None => Some(MAX_ALLOWED_OPERATION_DURATION_MS),
                Some(x) => Some(x),
            },
            timeout_transitions,
            failure_retries: match fail_retries {
                Some(x) => x,
                None => 0,
            },
            timeout_retries: match timeout_retries {
                Some(x) => x,
                None => 0,
            },
            can_be_bypassed,
            preconditions,
            postconditions,
            failure_transitions,
            bypass_transitions,
            cancel_transitions,
        }
    }

    /// Check the guard of the planning precondidion transition.
    pub fn eval_planning(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Initial) {
                for precondition in &self.preconditions {
                    if precondition.eval_planning(state, &log_target) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Execute the planing actions of both the pre and post conditions.
    /// Index 0 taken as to indicate that the firstly defined transition should be taken when planning.
    pub fn take_planning(&self, state: &State, log_target: &str) -> State {
        let mut new_state = state.clone();
        self.preconditions[0].take_planning_mut(&mut new_state, &log_target);
        self.postconditions[0].take_planning_mut(&mut new_state, &log_target);
        new_state
    }

    /// Whether the operation can start right now.
    ///
    /// True when it is `initial` or `disabled` and some precondition's full
    /// running guard holds. `log_target` is only the `log` crate target used
    /// for warnings.
    pub fn eval(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Initial)
                || value_is(&value, OperationState::Disabled)
            {
                for precondition in &self.preconditions {
                    if precondition.eval(state, &log_target) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check the guard and return a tuple: (is_enabled, index_of_enabled_transition)
    pub fn evaluate_with_transition_index(&self, state: &State, log_target: &str) -> (bool, usize) {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Initial) {
                for (index, precondition) in self.preconditions.iter().enumerate() {
                    if precondition.eval(state, &log_target) {
                        return (true, index);
                    }
                }
            }
        }
        (false, 0)
    }

    /// Check the running postondition guard.
    pub fn can_be_completed_with_transition_index(
        &self,
        state: &State,
        log_target: &str,
    ) -> (bool, usize) {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Executing) {
                for (index, postcondition) in self.postconditions.iter().enumerate() {
                    if postcondition.eval(state, &log_target) {
                        return (true, index);
                    }
                }
            }
        }
        (false, 0)
    }

    /// Check the running postondition guard.
    pub fn can_be_completed(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Executing) {
                for postcondition in &self.postconditions {
                    if postcondition.eval(&state, &log_target) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check the running fail_transition guard.
    pub fn can_be_failed(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Executing) {
                for fail_transition in &self.failure_transitions {
                    if fail_transition.eval(&state, &log_target) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Whether the operation has outstayed its deadline.
    ///
    /// Compares the `{name}_elapsed_executing_ms` / `{name}_elapsed_disabled_ms`
    /// counters, maintained by the time runner, against
    /// [`Operation::timeout_executing_ms`] and
    /// [`Operation::timeout_disabled_ms`].
    pub fn can_be_timedout(&self, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Executing) {
                if let Some(timeout_executing_ms) = self.timeout_executing_ms {
                    let elapased_ms = state.get_int_or_default_to_zero(
                        &format!("{}_elapsed_executing_ms", &self.name),
                        &log_target,
                    );
                    if elapased_ms > timeout_executing_ms {
                        return true;
                    }
                }
            }
            if value_is(&value, OperationState::Disabled) {
                if let Some(timeout_disabled_ms) = self.timeout_disabled_ms {
                    let elapased_ms = state.get_int_or_default_to_zero(
                        &format!("{}_elapsed_disabled_ms", &self.name),
                        &log_target,
                    );
                    if elapased_ms > timeout_disabled_ms {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if we can stop the execution and cancel the operations
    pub fn can_be_cancelled(&self, sp_id: &str, state: &State, log_target: &str) -> bool {
        if let Some(value) = state.get_value(&self.name, &log_target) {
            if value_is(&value, OperationState::Initial)
                || value_is(&value, OperationState::Executing)
                || value_is(&value, OperationState::Disabled)
                || value_is(&value, OperationState::Failed)
                || value_is(&value, OperationState::Timedout)
            {
                if let Some(dashboard_command) =
                    state.get_value(&format!("{}_dashboard_command", sp_id), &log_target)
                {
                    if let SPValue::String(StringOrUnknown::String(db)) = dashboard_command {
                        match db.as_str() {
                            "stop" => return true,
                            _ => (),
                        }
                    }
                }
            }
        }
        false
    }

    /// Start executing the operation. Check for eval_running() first.
    pub fn disable(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Initial) {
            let action = Action::new(assignment.var, OperationState::Disabled.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            log::error!(target: &log_target, "Can't block an operation which is not in its initial state.");
            state.clone()
        }
    }

    /// Move the operation to `cancelled`.
    ///
    /// Unguarded: it cancels from any state, including terminal ones. Check
    /// [`Operation::can_be_cancelled`] first.
    pub fn cancel(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        let action = Action::new(
            assignment.var,
            OperationState::Cancelled.to_spvalue().wrap(),
        );
        action.assign(&state, &log_target)
    }

    /// Start executing the operation. Check for eval_running() first.
    pub fn start(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Initial)
            || value_is(&assignment.val, OperationState::Disabled)
        {
            for precondition in &self.preconditions {
                if precondition.eval(state, &log_target) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Executing.to_spvalue().wrap(),
                    );
                    let mut new_state = state.clone();
                    precondition.take_mut(&mut new_state, &log_target);
                    action.assign_mut(&mut new_state, &log_target);
                    return new_state;
                }
            }
        }
        state.clone()
    }

    /// Complete executing the operation. Check for can_be_completed() first.
    pub fn complete(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Executing) {
            for postcondition in &self.postconditions {
                if postcondition.eval(&state, &log_target) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Completed.to_spvalue().wrap(),
                    );
                    let mut new_state = state.clone();
                    action.assign_mut(&mut new_state, &log_target);
                    postcondition.take_mut(&mut new_state, &log_target);
                    return new_state;
                }
            }
        }
        state.clone()
    }

    /// Fail the executing operation. Check for can_be_failed() first.
    pub fn fail(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Executing) {
            for fail_transition in &self.failure_transitions {
                if fail_transition.eval(&state, &log_target) {
                    let action =
                        Action::new(assignment.var, OperationState::Failed.to_spvalue().wrap());
                    let mut new_state = state.clone();
                    action.assign_mut(&mut new_state, &log_target);
                    fail_transition.take_mut(&mut new_state, &log_target);
                    return new_state;
                }
            }
        }
        state.clone()
    }

    /// Move a failed or timed-out operation to `fatal`, out of retries.
    ///
    /// From any other state this logs an error and returns the state unchanged.
    pub fn fatal(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Failed)
            || value_is(&assignment.val, OperationState::Timedout)
        {
            let action = Action::new(assignment.var, OperationState::Fatal.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            log::error!(target: &log_target, "Can't fatal an operation which hasn't failed or timedout.");
            state.clone()
        }
    }

    /// Move a finished operation to `terminated_*`, the state runners wait for
    /// before advancing a plan or SOP.
    ///
    /// Only [`TerminationReason::Completed`] is implemented, and only from
    /// `completed`; every other reason currently returns the state unchanged.
    pub fn terminate(
        &self,
        state: &State,
        termination_reason: TerminationReason,
        log_target: &str,
    ) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        match termination_reason {
            TerminationReason::Completed => {
                if value_is(&assignment.val, OperationState::Completed) {
                    let action = Action::new(
                        assignment.var,
                        OperationState::Terminated(TerminationReason::Completed)
                            .to_spvalue()
                            .wrap(),
                    );
                    action.assign(&state, &log_target)
                } else {
                    log::error!(target: &log_target, "Can't terminate_complete an operation which is not completed.");
                    state.clone()
                }
            }
            _ => state.clone(),
        }
    }

    /// Move a failed or timed-out operation to `bypassed`, carrying on despite
    /// the problem.
    ///
    /// With no [`Operation::bypass_transitions`] declared this is
    /// unconditional; with them, one must have a guard that holds, so a bypass
    /// transition can also *forbid* the bypass.
    pub fn bypass(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Failed)
            || value_is(&assignment.val, OperationState::Timedout)
        {
            if self.bypass_transitions.len() > 0 {
                for bypass_transition in &self.bypass_transitions {
                    if bypass_transition.eval(&state, &log_target) {
                        // Carefull: this can forbid the operation to bypass!
                        // Useful when you want to have different options to bypass and add some alternative conditions here
                        let action = Action::new(
                            assignment.var,
                            OperationState::Bypassed.to_spvalue().wrap(),
                        );
                        let mut new_state = state.clone();
                        action.assign_mut(&mut new_state, &log_target);
                        bypass_transition.take_mut(&mut new_state, &log_target);
                        return new_state;
                    }
                }
            } else {
                let action =
                    Action::new(assignment.var, OperationState::Bypassed.to_spvalue().wrap());
                return action.assign(&state, &log_target);
            }
        }
        state.clone()
    }

    /// Timeout an executing the operation.
    pub fn timeout(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Executing)
            || value_is(&assignment.val, OperationState::Disabled)
        {
            if self.timeout_transitions.len() > 0 {
                for timeout_transition in &self.timeout_transitions {
                    if timeout_transition.eval(&state, &log_target) {
                        // Carefull: this can forbid the operation to timeout!
                        // Useful when you want to have different options to timeout and add some alternative conditions here
                        let action = Action::new(
                            assignment.var,
                            OperationState::Timedout.to_spvalue().wrap(),
                        );
                        let mut new_state = state.clone();
                        action.assign_mut(&mut new_state, &log_target);
                        timeout_transition.take_mut(&mut new_state, &log_target);
                        return new_state;
                    }
                }
            } else {
                let action =
                    Action::new(assignment.var, OperationState::Timedout.to_spvalue().wrap());
                return action.assign(&state, &log_target);
            }
        }
        state.clone()
    }

    /// Retry the execution of the operation, allows for retries without immediate replanning.
    /// However, do we have to reset the variables before we can go back the initial state?
    /// Otherwise we might end up in disabled? Let's try withthe emulation.
    pub fn retry(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Failed)
            || value_is(&assignment.val, OperationState::Timedout)
        {
            let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            state.clone()
        }
    }

    /// Force the operation to `initial` from any state.
    ///
    /// Unguarded on purpose: this is the recovery path for an operation found
    /// in an unrecognised state.
    pub fn initialize(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
        action.assign(&state, &log_target)
    }

    /// Put a `completed` or `fatal` operation back to `initial` so it can run
    /// again. From any other state the state is returned unchanged.
    pub fn reinitialize(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        if value_is(&assignment.val, OperationState::Completed)
            || value_is(&assignment.val, OperationState::Fatal)
        {
            let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
            action.assign(&state, &log_target)
        } else {
            state.clone()
        }
    }

    /// Every state variable read or written by any of this operation's
    /// transitions.
    ///
    /// This is what the runners use to build their `get_state_for_keys` key
    /// sets, so a variable missing here is a variable missing from the state
    /// the runner reads - and reading a missing variable panics. It does *not*
    /// include the operation's own bookkeeping variables (`{name}`,
    /// `{name}_information`, ...); see `running::runner_keys`.
    pub fn get_all_var_keys(&self) -> Vec<String> {
        let mut all_keys: Vec<String> = self
            .preconditions
            .iter()
            .flat_map(|t| t.get_all_var_keys())
            .chain(
                self.postconditions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .chain(
                self.failure_transitions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .chain(
                self.timeout_transitions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            // `bypass_transitions` was missing here, so the variables that only
            // a bypass guard/action touches were absent from every key set
            // built from this function.
            .chain(
                self.bypass_transitions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .chain(
                self.cancel_transitions
                    .iter()
                    .flat_map(|t| t.get_all_var_keys()),
            )
            .collect();

        all_keys.sort_unstable();
        all_keys.dedup();

        all_keys
    }
}
#[cfg(test)]
mod operation_state_tests {
    use super::*;

    fn all_states() -> Vec<OperationState> {
        vec![
            OperationState::Initial,
            OperationState::Disabled,
            OperationState::Executing,
            OperationState::Completed,
            OperationState::Bypassed,
            OperationState::Timedout,
            OperationState::Failed,
            OperationState::Fatal,
            OperationState::Cancelled,
            OperationState::Terminated(TerminationReason::Completed),
            OperationState::Terminated(TerminationReason::Bypassed),
            OperationState::Terminated(TerminationReason::Fatal),
            OperationState::Terminated(TerminationReason::Cancelled),
            OperationState::UNKNOWN,
        ]
    }

    /// The allocation-free comparisons rely on `as_str` producing exactly what
    /// `Display`/`to_spvalue` produce, and on `from_str` round-tripping it.
    #[test]
    fn as_str_agrees_with_display_and_round_trips() {
        for state in all_states() {
            assert_eq!(state.as_str(), state.to_string(), "Display mismatch");
            assert_eq!(
                OperationState::from_str(state.as_str()),
                state,
                "from_str did not round-trip '{}'",
                state.as_str()
            );
        }
    }

    /// `value_is` has to answer exactly what `value == expected.to_spvalue()`
    /// used to, for every pairing - including the wrong-type case, which was
    /// `false` before and must stay `false`.
    #[test]
    fn value_is_matches_the_old_spvalue_comparison() {
        for expected in all_states() {
            for actual in all_states() {
                let old = actual.clone().to_spvalue() == expected.clone().to_spvalue();
                let new = value_is(&actual.clone().to_spvalue(), expected.clone());
                assert_eq!(new, old, "{:?} vs {:?}", actual, expected);
            }

            // Values that do not come from `to_spvalue()`: other types, the
            // UNKNOWN variant, and the literal string "UNKNOWN" - which is a
            // *different* value from the UNKNOWN variant and must stay so.
            for wrong_type in [
                SPValue::Bool(BoolOrUnknown::Bool(true)),
                SPValue::Int64(IntOrUnknown::Int64(1)),
                SPValue::String(StringOrUnknown::UNKNOWN),
                SPValue::String(StringOrUnknown::String("UNKNOWN".to_string())),
                SPValue::String(StringOrUnknown::String("".to_string())),
            ] {
                assert_eq!(
                    value_is(&wrong_type, expected.clone()),
                    wrong_type == expected.clone().to_spvalue(),
                    "wrong-typed value {:?} vs {:?}",
                    wrong_type,
                    expected
                );
            }
        }
    }
}

#[cfg(test)]
mod can_be_cancelled_tests {
    use crate::*;

    const SP_ID: &str = "sp";
    const TARGET: &str = "test";

    fn state_with(operation_state: &str, dashboard_command: &str) -> State {
        let mut state = State::new();
        state.add_mut(
            SPAssignment::new(
                SPVariable::new("op_x", SPValueType::String),
                operation_state.to_spvalue(),
            ),
            TARGET,
        );
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(&format!("{}_dashboard_command", SP_ID), SPValueType::String),
                dashboard_command.to_spvalue(),
            ),
            TARGET,
        );
        state
    }

    fn operation() -> Operation {
        Operation {
            name: "op_x".to_string(),
            ..Default::default()
        }
    }

    /// The states where cancelling means something: planned, running, or stuck
    /// somewhere it can still be recovered from.
    #[test]
    fn stop_cancels_an_operation_that_has_not_finished() {
        for operation_state in ["initial", "executing", "disabled", "failed", "timedout"] {
            assert!(
                operation().can_be_cancelled(SP_ID, &state_with(operation_state, "stop"), TARGET),
                "'{operation_state}' should be cancellable"
            );
        }
    }

    /// The bug: the guard was a tautology, so `stop` drove *finished*
    /// operations to `Cancelled` too - `Operation::cancel` does not check the
    /// current state before assigning.
    #[test]
    fn stop_leaves_a_finished_operation_alone() {
        for operation_state in [
            "completed",
            "bypassed",
            "fatal",
            "cancelled",
            "terminated_completed",
            "terminated_bypassed",
            "terminated_fatal",
            "terminated_cancelled",
        ] {
            assert!(
                !operation().can_be_cancelled(SP_ID, &state_with(operation_state, "stop"), TARGET),
                "'{operation_state}' is terminal and should not be cancellable"
            );
        }
    }

    #[test]
    fn without_a_stop_command_nothing_is_cancellable() {
        for operation_state in ["initial", "executing", "disabled", "failed", "timedout"] {
            for command in ["none", "start", ""] {
                assert!(
                    !operation().can_be_cancelled(
                        SP_ID,
                        &state_with(operation_state, command),
                        TARGET
                    ),
                    "'{operation_state}' with command '{command}' should not be cancellable"
                );
            }
        }
    }
}

/// The state-transition methods, guarded.
///
/// Every one of these is written as "if the operation is in the state I expect,
/// do the thing; otherwise log and return the state unchanged". Those guards
/// are the last line of defence against a runner driving an operation out of
/// order, and returning the state *unchanged* is what makes a wrong call a
/// no-op rather than a corruption. `process_operation` covers the in-order
/// paths; this covers the refusals, plus the two `_with_transition_index`
/// lookups that nothing currently calls.
#[cfg(test)]
mod guard_tests {
    use crate::*;

    const TARGET: &str = "test";
    const OP: &str = "op_test";

    fn world() -> State {
        let mut state = State::new();
        for name in ["go", "alt", "done", "broken", "late"] {
            state.add_mut(
                SPAssignment::new(SPVariable::new(name, SPValueType::Bool), false.to_spvalue()),
                TARGET,
            );
        }
        state
    }

    fn transition(name: &str, guard: &str, state: &State) -> Transition {
        Transition::parse(
            name,
            guard,
            "true",
            Vec::<&str>::new(),
            Vec::<&str>::new(),
            state,
        )
    }

    /// An operation with two preconditions, two postconditions, a failure
    /// transition and a timeout transition - enough to reach every branch.
    fn operation(state: &State) -> Operation {
        Operation::new(
            OP,
            Some(1000),
            Some(1000),
            None,
            None,
            true,
            vec![
                transition("start_a", "var:go == true", state),
                transition("start_b", "var:alt == true", state),
            ],
            vec![
                transition("complete_a", "var:done == true", state),
                transition("complete_b", "var:alt == true", state),
            ],
            vec![transition("fail", "var:broken == true", state)],
            vec![transition("timeout", "var:late == true", state)],
            vec![],
            vec![],
        )
    }

    fn in_state(state: &State, operation_state: &str) -> State {
        let mut state = state.clone();
        state.add_mut(
            SPAssignment::new(
                SPVariable::new(OP, SPValueType::String),
                operation_state.to_spvalue(),
            ),
            TARGET,
        );
        state
    }

    fn op_state(state: &State) -> String {
        state.get_string_or_default_to_unknown(OP, TARGET)
    }

    /// `evaluate_with_transition_index` reports *which* precondition enabled the
    /// operation - the index the `start` path is supposed to reuse rather than
    /// re-walking the guards.
    #[test]
    fn evaluate_with_transition_index_reports_the_matching_precondition() {
        let world = world();
        let operation = operation(&world);

        let first = in_state(&world.update("go", true.to_spvalue()), "initial");
        assert_eq!(
            operation.evaluate_with_transition_index(&first, TARGET),
            (true, 0)
        );

        let second = in_state(&world.update("alt", true.to_spvalue()), "initial");
        assert_eq!(
            operation.evaluate_with_transition_index(&second, TARGET),
            (true, 1)
        );

        let neither = in_state(&world, "initial");
        assert_eq!(
            operation.evaluate_with_transition_index(&neither, TARGET),
            (false, 0)
        );
    }

    /// Unlike `eval`, the indexed form only accepts `initial` - it does not
    /// also accept `disabled`. That asymmetry is why swapping one for the other
    /// would be a behaviour change, and is worth having pinned.
    #[test]
    fn evaluate_with_transition_index_does_not_accept_a_disabled_operation() {
        let world = world().update("go", true.to_spvalue());
        let operation = operation(&world);
        let disabled = in_state(&world, "disabled");

        assert!(operation.eval(&disabled, TARGET), "eval accepts disabled");
        assert_eq!(
            operation.evaluate_with_transition_index(&disabled, TARGET),
            (false, 0),
            "the indexed form does not"
        );
    }

    #[test]
    fn can_be_completed_with_transition_index_reports_the_matching_postcondition() {
        let world = world();
        let operation = operation(&world);

        let first = in_state(&world.update("done", true.to_spvalue()), "executing");
        assert_eq!(
            operation.can_be_completed_with_transition_index(&first, TARGET),
            (true, 0)
        );

        let second = in_state(&world.update("alt", true.to_spvalue()), "executing");
        assert_eq!(
            operation.can_be_completed_with_transition_index(&second, TARGET),
            (true, 1)
        );

        // Only while executing.
        let not_executing = in_state(&world.update("done", true.to_spvalue()), "initial");
        assert_eq!(
            operation.can_be_completed_with_transition_index(&not_executing, TARGET),
            (false, 0)
        );
    }

    /// Every method refuses to act from a state it does not expect, and returns
    /// the state untouched when it does. This is the table - each row is a
    /// method, the state it refuses from, and the state that must be unchanged
    /// afterwards.
    #[test]
    fn every_transition_method_refuses_from_the_wrong_state() {
        let world = world()
            .update("go", true.to_spvalue())
            .update("done", true.to_spvalue())
            .update("broken", true.to_spvalue())
            .update("late", true.to_spvalue());
        let operation = operation(&world);

        type Method = fn(&Operation, &State, &str) -> State;
        let cases: Vec<(&str, Method, &str)> = vec![
            ("disable", Operation::disable, "executing"),
            ("start", Operation::start, "completed"),
            ("complete", Operation::complete, "initial"),
            ("fail", Operation::fail, "initial"),
            ("timeout", Operation::timeout, "initial"),
            ("fatal", Operation::fatal, "executing"),
            ("retry", Operation::retry, "executing"),
            ("bypass", Operation::bypass, "executing"),
        ];

        for (label, method, wrong_state) in cases {
            let state = in_state(&world, wrong_state);
            let after = method(&operation, &state, TARGET);
            assert_eq!(
                after, state,
                "{label} from '{wrong_state}' must leave the state untouched"
            );
        }
    }

    /// And each one does act from the state it does expect.
    #[test]
    fn every_transition_method_acts_from_the_right_state() {
        let world = world()
            .update("go", true.to_spvalue())
            .update("done", true.to_spvalue())
            .update("broken", true.to_spvalue())
            .update("late", true.to_spvalue());
        let operation = operation(&world);

        type Method = fn(&Operation, &State, &str) -> State;
        let cases: Vec<(&str, Method, &str, &str)> = vec![
            ("disable", Operation::disable, "initial", "disabled"),
            ("start", Operation::start, "initial", "executing"),
            (
                "start from disabled",
                Operation::start,
                "disabled",
                "executing",
            ),
            ("complete", Operation::complete, "executing", "completed"),
            ("fail", Operation::fail, "executing", "failed"),
            ("timeout", Operation::timeout, "executing", "timedout"),
            (
                "timeout from disabled",
                Operation::timeout,
                "disabled",
                "timedout",
            ),
            ("fatal from failed", Operation::fatal, "failed", "fatal"),
            ("fatal from timedout", Operation::fatal, "timedout", "fatal"),
            ("retry from failed", Operation::retry, "failed", "initial"),
            (
                "retry from timedout",
                Operation::retry,
                "timedout",
                "initial",
            ),
            // `bypass` is reached from a *failed* or *timedout* operation - it
            // is the "this step did not work, carry on anyway" path, not
            // something an executing operation does.
            (
                "bypass from failed",
                Operation::bypass,
                "failed",
                "bypassed",
            ),
            (
                "bypass from timedout",
                Operation::bypass,
                "timedout",
                "bypassed",
            ),
        ];

        for (label, method, from, expected) in cases {
            let state = in_state(&world, from);
            let after = method(&operation, &state, TARGET);
            assert_eq!(op_state(&after), expected, "{label}");
        }
    }

    /// `cancel` is the exception: it has no guard at all, so it moves an
    /// operation to `cancelled` from anywhere. That is deliberate - the caller
    /// (`process_operation`) checks `can_be_cancelled` first - but it means
    /// calling it directly bypasses that check entirely.
    #[test]
    fn cancel_has_no_guard_of_its_own() {
        let world = world();
        let operation = operation(&world);

        for from in [
            "initial",
            "executing",
            "completed",
            "terminated_completed",
            "fatal",
        ] {
            let state = in_state(&world, from);
            assert_eq!(
                op_state(&operation.cancel(&state, TARGET)),
                "cancelled",
                "cancel from '{from}'"
            );
        }
    }

    /// `initialize` is the other unguarded one - it is the recovery path for an
    /// operation in an unrecognised state, so it has to work from anywhere.
    #[test]
    fn initialize_works_from_any_state() {
        let world = world();
        let operation = operation(&world);

        for from in ["nonsense", "executing", "terminated_fatal"] {
            let state = in_state(&world, from);
            assert_eq!(op_state(&operation.initialize(&state, TARGET)), "initial");
        }
    }

    /// `reinitialize` is guarded, and only from the two states where starting
    /// over makes sense.
    #[test]
    fn reinitialize_only_from_completed_or_fatal() {
        let world = world();
        let operation = operation(&world);

        for from in ["completed", "fatal"] {
            let state = in_state(&world, from);
            assert_eq!(
                op_state(&operation.reinitialize(&state, TARGET)),
                "initial",
                "{from}"
            );
        }
        for from in ["executing", "initial", "failed"] {
            let state = in_state(&world, from);
            assert_eq!(operation.reinitialize(&state, TARGET), state, "{from}");
        }
    }

    /// BUG: `terminate` only implements `TerminationReason::Completed`; its
    /// `_ => state.clone()` arm makes the other three silent no-ops. See
    /// `running::process_operation::state_machine_tests` and
    /// `running::sop_runner::tests::a_bypassed_operation_never_lets_its_sop_finish`
    /// for what that costs at runtime.
    #[test]
    fn terminate_only_handles_the_completed_reason() {
        let world = world();
        let operation = operation(&world);

        let completed = in_state(&world, "completed");
        assert_eq!(
            op_state(&operation.terminate(&completed, TerminationReason::Completed, TARGET)),
            "terminated_completed"
        );

        for (from, reason) in [
            ("bypassed", TerminationReason::Bypassed),
            ("fatal", TerminationReason::Fatal),
            ("cancelled", TerminationReason::Cancelled),
        ] {
            let state = in_state(&world, from);
            assert_eq!(
                operation.terminate(&state, reason, TARGET),
                state,
                "terminate({from}) is currently a no-op - if this now changes the \
                 state the bug is fixed"
            );
        }

        // And the Completed reason is itself guarded on the operation actually
        // being completed.
        let executing = in_state(&world, "executing");
        assert_eq!(
            operation.terminate(&executing, TerminationReason::Completed, TARGET),
            executing
        );
    }

    /// With no timeout/bypass transitions declared, `timeout` and `bypass` are
    /// unconditional; with them declared, the transition's guard can *prevent*
    /// the operation from timing out at all - the "careful" note in the source.
    #[test]
    fn a_timeout_transition_guard_can_prevent_the_timeout() {
        let world = world();
        let guarded = operation(&world); // its timeout transition needs `late`

        let not_late = in_state(&world, "executing");
        assert_eq!(
            guarded.timeout(&not_late, TARGET),
            not_late,
            "the timeout transition's guard is false, so it does not time out"
        );

        let late = in_state(&world.update("late", true.to_spvalue()), "executing");
        assert_eq!(op_state(&guarded.timeout(&late, TARGET)), "timedout");

        // With no timeout transitions at all it is unconditional.
        let plain = Operation::new(
            OP,
            Some(1000),
            Some(1000),
            None,
            None,
            true,
            vec![transition("start", "true", &world)],
            vec![transition("complete", "true", &world)],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(op_state(&plain.timeout(&not_late, TARGET)), "timedout");

        let timedout = in_state(&world, "timedout");
        assert_eq!(op_state(&plain.bypass(&timedout, TARGET)), "bypassed");
    }
}
