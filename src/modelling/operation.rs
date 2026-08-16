use serde::{Deserialize, Serialize};
use std::fmt;

use crate::*;

/// Initial:   The operation planned and ready to be executed.
/// Blocked:   Can't move to executing stet because the precondition guard is false.
/// Executing: The precondition guard is enabled and the actions of the precondition are taken.
/// Completed: The postcondition guard is enabled and the actions of the postcondition are taken.
///            The operation is successfully completed.
/// Timedout:  The operation was in the executing state for more time than its deadline allows.
/// Failed:    The operations has failed due to an error.
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum OperationState {
    Initial,
    Disabled,
    Executing,
    Completed,
    Bypassed,
    Timedout,
    Failed,
    Fatal,
    Cancelled,
    Terminated(TerminationReason),
    // Paused,
    UNKNOWN,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum TerminationReason {
    Completed,
    Bypassed,
    Fatal,
    Cancelled,
}

impl Default for OperationState {
    fn default() -> Self {
        OperationState::UNKNOWN
    }
}

impl OperationState {
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
            "terminated_completed" => OperationState::Terminated(TerminationReason::Completed),
            "terminated_bypassed" => OperationState::Terminated(TerminationReason::Bypassed),
            "terminated_fatal" => OperationState::Terminated(TerminationReason::Fatal),
            "terminated_cancelled" => OperationState::Terminated(TerminationReason::Cancelled),
            _ => OperationState::UNKNOWN,
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        self.to_string().to_spvalue()
    }

    /// The same text `Display` produces, without allocating.
    ///
    /// DONE: PERF: the comparisons throughout `Operation` were written as
    /// `value == OperationState::Initial.to_spvalue()`, and `to_spvalue` goes
    /// through `to_string()` - so each one allocated a fresh `String`, wrapped
    /// it in an `SPValue`, compared, and dropped it. `can_be_cancelled` alone
    /// did that five times, for every operation, on every tick.
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
            OperationState::Terminated(TerminationReason::Completed) => "terminated_completed",
            OperationState::Terminated(TerminationReason::Bypassed) => "terminated_bypassed",
            OperationState::Terminated(TerminationReason::Fatal) => "terminated_fatal",
            OperationState::Terminated(TerminationReason::Cancelled) => "terminated_cancelled",
            OperationState::UNKNOWN => "UNKNOWN",
        }
    }
}

/// Allocation-free equivalent of `value == expected.to_spvalue()`.
///
/// The subtlety worth spelling out: `to_spvalue()` does *not* always produce
/// `SPValue::String(StringOrUnknown::String(..))`. `ToSPValue for String`
/// collapses "UNKNOWN"/"unknown"/"Unknown" to `StringOrUnknown::UNKNOWN`, so
/// `OperationState::UNKNOWN.to_spvalue()` is the UNKNOWN *variant*, not the
/// string "UNKNOWN". Both sides have to be compared in that same shape or this
/// silently disagrees with the comparison it replaced - which is exactly what
/// `value_is_matches_the_old_spvalue_comparison` pins down.
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

// PERF: an `Operation` owns six `Vec<Transition>`, each transition owning two
// predicate trees and two action vectors, so `Operation::clone()` is a deep
// copy of a fairly large graph. It is still cloned on some hot paths:
// `auto_operation_runner` clones a template per
// activation and again per tick when rebuilding `active_auto_ops`, the BFS
// planner clones one per node, and `Model::new` clones every transition vector
// three times over. (The `operation.clone().cancel(..)` calls in
// `process_operation` are gone - those methods take `&self`.)
// Suggested: hold the transition vectors as
// `Arc<[Transition]>` so cloning an `Operation` is a handful of refcount bumps,
// and store active operations as `Arc<Operation>` in the runners.
// PERF: the runners key everything off `operation.name` via `format!`. Caching
// the derived key strings (`{name}`, `{name}_information`,
// `{name}_elapsed_executing_ms`, ...) on the struct - or in a side table built
// when the operation is activated - removes about a dozen allocations per
// operation per tick.
#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub state: OperationState,
    pub timeout_executing_ms: Option<i64>,
    pub timeout_disabled_ms: Option<i64>,
    pub failure_retries: i64,
    pub timeout_retries: i64,
    pub can_be_bypassed: bool,
    pub preconditions: Vec<Transition>,
    pub postconditions: Vec<Transition>,
    pub failure_transitions: Vec<Transition>,
    pub bypass_transitions: Vec<Transition>,
    pub timeout_transitions: Vec<Transition>,
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
    /// Inex 0 taken as to indicate that the firstly defined transition should be taken when planning.
    ///
    /// DONE: this used to clone both transitions and build an intermediate
    /// `State` between them; it now applies both in place on a single copy.
    pub fn take_planning(&self, state: &State, log_target: &str) -> State {
        let mut new_state = state.clone();
        self.preconditions[0].take_planning_mut(&mut new_state, &log_target);
        self.postconditions[0].take_planning_mut(&mut new_state, &log_target);
        new_state
    }

    // DONE: PERF: this is evaluated for *every* auto operation on *every* tick
    // of `auto_operation_runner`, so it is the single most frequently executed
    // guard check in the system. Both of its old costs are gone:
    //   - `state.get_value(&self.name, ..)` no longer clones the entire state
    //     map before the cheap early-out runs (see `State::get_value`);
    //   - `precondition.clone().eval(state, ..)` used to deep-copy the
    //     transition for every precondition of every operation, including the
    //     ones whose guard is immediately false. `Transition::eval` and
    //     `Predicate::eval` take `&self`, so the guard walk is now a pure
    //     borrow. The same clone removal was applied to `eval_planning`,
    //     `evaluate_with_transition_index`, `can_be_completed(_with_index)`,
    //     `can_be_failed`, `start`, `complete`, `fail`, `bypass` and `timeout`.
    // PERF: `OperationState::Initial.to_spvalue()` allocates a fresh `String`
    // ("initial") and wraps it in an `SPValue` twice per call just to compare.
    // Comparing against `&'static str` (or matching on
    // `OperationState::from_str`) avoids two allocations per operation per tick.
    // The same pattern appears in `eval_planning`, `can_be_completed`,
    // `can_be_failed`, `can_be_timedout`, `can_be_cancelled` and every state
    // transition method below.
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

    // PERF: builds `format!("{}_elapsed_executing_ms", self.name)` (and the
    // disabled variant) on every call, i.e. per executing operation per tick.
    // Cache the key strings; see the note on the `Operation` struct.
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

    /// Check the running reset_transition guard.
    // pub fn can_be_reset(&self, state: &State, log_target: &str) -> bool {
    //     if let Some(value) = state.get_value(&self.name, &log_target) {
    //         if value == OperationState::Completed.to_spvalue() {
    //             for reset_transition in &self.reset_transitions {
    //                 if reset_transition.clone().eval(&state, &log_target) {
    //                     return true;
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }

    /// Check if we can stop the execution and cancel the operations
    // PERF: called first in almost every arm of `process_operation`, so it runs
    // for every active operation on every tick. It does two `state.get_value`
    // calls (each a full-map clone today), builds
    // `format!("{}_dashboard_command", sp_id)` every time, and constructs five
    // `OperationState::*.to_spvalue()` `String`s for the comparison. Since the
    // dashboard command is a single per-runner variable, read it *once* per
    // tick in the runner loop and pass the result in, rather than re-reading it
    // per operation.
    // DONE (correctness): the state guard read
    //     Initial || Executing || != Disabled || != Failed || != Timedout
    // Those last three are `!=`, almost certainly a typo for `==`, and they
    // make the whole expression a tautology: any value that is not `Disabled`
    // satisfies `!= Disabled`, and `Disabled` itself satisfies `!= Failed`. So
    // the guard was true for *every* operation state and the method reduced to
    // "is the dashboard command 'stop'".
    //
    // That is not harmless. `Operation::cancel` does not check the current
    // state - it assigns `Cancelled` unconditionally - so pressing stop drove
    // every operation to `Cancelled`, including ones that had already reached
    // `Completed`, `Bypassed`, `Fatal` or a `Terminated(..)` state. A finished
    // operation would be reported as cancelled.
    //
    // The guard now lists the states where cancelling an operation means
    // something: it has been planned or is running, or it is stuck in a state
    // it can still be recovered from. Terminal states are left alone.
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

    pub fn cancel(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        let action = Action::new(
            assignment.var,
            OperationState::Cancelled.to_spvalue().wrap(),
        );
        action.assign(&state, &log_target)
    }

    /// Start executing the operation. Check for eval_running() first.
    // PERF: re-evaluates every precondition guard that `Operation::eval` just
    // evaluated a moment earlier in `process_operation` - the whole guard set is
    // walked twice per start. `evaluate_with_transition_index` already exists
    // and returns the matching index; using it (and passing the index into
    // `start`) halves the guard work at the moment an operation starts.
    // DONE: PERF: the guard walk used to clone every precondition to evaluate it
    // and clone the matching one again to take it, and `take` + `assign` built
    // two intermediate `State`s. It now borrows the transition and applies both
    // the actions and the status write in place on a single copy.
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
    // PERF: same double evaluation as `start` - `can_be_completed` walks all
    // postcondition guards, then this walks them again to find the same one.
    // `can_be_completed_with_transition_index` already returns the index; wiring
    // it through `process_operation` removes the second walk. The same pattern
    // repeats in `fail`, `bypass` and `timeout`.
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

    // pub fn void(&self, state: &State, log_target: &str) -> State {
    //     let assignment = state.get_assignment(&self.name, &log_target);
    //     if assignment.val == OperationState::Terminated(TerminationReason::Completed).to_spvalue() {
    //         let action = Action::new(
    //             assignment.var,
    //             OperationState::Void
    //                 .to_spvalue()
    //                 .wrap(),
    //         );
    //         action.assign(&state, &log_target)
    //     } else {
    //         log::error!(target: &log_target, "Can't void an operation which is not terminated.");
    //         state.clone()
    //     }
    // }

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

    pub fn initialize(&self, state: &State, log_target: &str) -> State {
        let assignment = state.get_assignment(&self.name, &log_target);
        let action = Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
        action.assign(&state, &log_target)
    }

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

    /// Continue executing the next operation if this one has failed
    // pub fn continue_running_next(&self, state: &State, log_target: &str) -> State {
    //     let assignment = state.get_assignment(&self.name, &log_target);
    //     if assignment.val == OperationState::Bypassed.to_spvalue()
    //     {
    //         for postcondition in &self.bypass_transitions {
    //             if postcondition.clone().eval(&state, &log_target) {
    //                 let action = Action::new(
    //                     assignment.var,
    //                     OperationState::Completed.to_spvalue().wrap(),
    //                 );
    //                 return postcondition
    //                     .clone()
    //                     .take(&action.assign(&state, &log_target), &log_target);
    //             }
    //         }
    //     }
    //     state.clone()
    // }

    // pub fn terminate(&self, state: &State, log_target: &str) -> State {
    //     let assignment = state.get_assignment(&self.name, &log_target);
    //     if assignment.val == OperationState::Unrecoverable.to_spvalue()
    //         || assignment.val == OperationState::Bypassed.to_spvalue()
    //         || assignment.val == OperationState::Completed.to_spvalue()
    //     {
    //         let action = Action::new(
    //             assignment.var,
    //             OperationState::Terminated.to_spvalue().wrap(),
    //         );
    //         action.assign(&state, &log_target)
    //     } else {
    //         log::error!(target: &log_target, "Can't terminate an operation which is not unrecoverable, bypassed, or completed.");
    //         state.clone()
    //     }
    // }

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

    // Tricky, wait with this, maybe we want to resrt when it failed.
    // Reset the completed operation. Check for can_be_reset() first.
    // pub fn reset_running(&self, state: &State) -> State {
    //     let assignment = state.get_assignment(&self.name);
    //     if assignment.val == OperationState::Completed.to_spvalue() {
    //         for reset_transition in &self.reset_transitions {
    //             if reset_transition.clone().eval_running(&state) {
    //                 let action =
    //                     Action::new(assignment.var, OperationState::Initial.to_spvalue().wrap());
    //                 return reset_transition
    //                     .clone()
    //                     .take_running(&action.assign(&state));
    //             }
    //         }
    //     }
    //     state.clone()
    // }
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
                    !operation()
                        .can_be_cancelled(SP_ID, &state_with(operation_state, command), TARGET),
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
        assert_eq!(operation.evaluate_with_transition_index(&first, TARGET), (true, 0));

        let second = in_state(&world.update("alt", true.to_spvalue()), "initial");
        assert_eq!(operation.evaluate_with_transition_index(&second, TARGET), (true, 1));

        let neither = in_state(&world, "initial");
        assert_eq!(operation.evaluate_with_transition_index(&neither, TARGET), (false, 0));
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
            ("start from disabled", Operation::start, "disabled", "executing"),
            ("complete", Operation::complete, "executing", "completed"),
            ("fail", Operation::fail, "executing", "failed"),
            ("timeout", Operation::timeout, "executing", "timedout"),
            ("timeout from disabled", Operation::timeout, "disabled", "timedout"),
            ("fatal from failed", Operation::fatal, "failed", "fatal"),
            ("fatal from timedout", Operation::fatal, "timedout", "fatal"),
            ("retry from failed", Operation::retry, "failed", "initial"),
            ("retry from timedout", Operation::retry, "timedout", "initial"),
            // `bypass` is reached from a *failed* or *timedout* operation - it
            // is the "this step did not work, carry on anyway" path, not
            // something an executing operation does.
            ("bypass from failed", Operation::bypass, "failed", "bypassed"),
            ("bypass from timedout", Operation::bypass, "timedout", "bypassed"),
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

        for from in ["initial", "executing", "completed", "terminated_completed", "fatal"] {
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
            assert_eq!(op_state(&operation.reinitialize(&state, TARGET)), "initial", "{from}");
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
