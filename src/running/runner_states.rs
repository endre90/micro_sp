//! The lifecycle state enums the runners share.
//!
//! Each enum here is a wire format: a runner writes it into Redis as the string
//! its [`Display`](std::fmt::Display) / `to_spvalue` produces, and another runner - usually in
//! another process - reads it back with `from_str`. Anything unrecognised parses
//! to `UNKNOWN`, which is also every enum's [`Default`], so a key that was never
//! initialised is never mistaken for a real state.
//!
//! ```
//! use micro_sp::*;
//!
//! let state = PlanState::Executing;
//! assert_eq!(state.to_string(), "executing");
//! assert_eq!(PlanState::from_str("executing"), PlanState::Executing);
//!
//! // A value nothing recognises is UNKNOWN rather than a guess.
//! assert_eq!(PlanState::from_str("nonsense"), PlanState::UNKNOWN);
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::*;

/// A plan: the operation sequence the planner found for one goal.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// Name of the plan, normally the model's name.
    pub name: String,
    /// The goal predicate this plan was found for.
    pub goal: Predicate,
    /// The operations to execute, in order.
    pub plan: Vec<Operation>,
    /// Index of the operation currently being executed.
    pub time_step: u32,
    /// How far the plan has got.
    pub state: PlanState,
    /// How long the planner took to find it.
    pub time: std::time::Duration,
}

/// How far the currently executing plan has got.
///
/// Written to `{sp_id}_plan_state` by `plan_runner` and read by `goal_runner`.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanState {
    /// A plan exists but no step has been taken yet; serialises to `"initial"`.
    Initial,
    /// The plan runner is driving the plan's operations; `"executing"`.
    Executing,
    /// An operation became unrecoverable, so the plan cannot finish; `"failed"`.
    Failed,
    /// Every operation in the plan reached a terminal success; `"completed"`.
    Completed,
    /// An operation was cancelled, normally by the dashboard stop command.
    ///
    /// Serialises to `"cancelled"`, but `from_str` has no arm for it, so it
    /// reads back as [`PlanState::UNKNOWN`] and `goal_runner`'s `Cancelled` arm
    /// is never reached. A cancelled goal is still released, but it is reported
    /// as unknown rather than as cancelled.
    Cancelled,
    /// Nothing recognisable was read; `"UNKNOWN"`. Also the [`Default`].
    UNKNOWN,
}

/// How far the currently enabled SOP has got.
///
/// Written to `{sp_id}_sop_state` by `sop_runner`.
#[derive(Debug, Clone, PartialEq)]
pub enum SOPState {
    /// The SOP is enabled but has not started; serialises to `"initial"`.
    Initial,
    /// At least one of the SOP's operations is running; `"executing"`.
    Executing,
    /// An operation in the SOP is unrecoverable; `"fatal"`.
    Fatal,
    /// The whole SOP tree reached a terminal success; `"completed"`.
    Completed,
    /// A branch of the SOP was cancelled.
    ///
    /// Serialises to `"cancelled"`, but - as for [`PlanState::Cancelled`] -
    /// `from_str` has no arm for it, so readers see `UNKNOWN`.
    Cancelled,
    /// Nothing recognisable was read; `"UNKNOWN"`. Also the [`Default`].
    UNKNOWN,
}

/// The planner handshake, written to `{sp_id}_planner_state`.
#[derive(Debug, Clone, PartialEq)]
pub enum PlannerState {
    /// A plan was found for the current goal; serialises to `"found"`.
    Found,
    /// No plan reaches the current goal; `"not_found"`.
    NotFound,
    /// The planner may be triggered for the current goal; `"ready"`.
    Ready,
    /// Nothing recognisable was read; `"UNKNOWN"`. Also the [`Default`].
    UNKNOWN,
}

impl Default for PlanState {
    fn default() -> Self {
        PlanState::UNKNOWN
    }
}

impl Default for SOPState {
    fn default() -> Self {
        SOPState::UNKNOWN
    }
}

impl Default for PlannerState {
    fn default() -> Self {
        PlannerState::UNKNOWN
    }
}

impl PlanState {
    /// Parses the string form written to `{sp_id}_plan_state`.
    ///
    /// Anything unrecognised becomes [`PlanState::UNKNOWN`] - including
    /// `"cancelled"`, which has no arm here even though `Display` produces it.
    pub fn from_str(x: &str) -> PlanState {
        match x {
            "initial" => PlanState::Initial,
            "executing" => PlanState::Executing,
            "failed" => PlanState::Failed,
            "completed" => PlanState::Completed,
            _ => PlanState::UNKNOWN,
        }
    }

    /// The state as the [`SPValue`] stored in the shared state, matching
    /// [`Display`](std::fmt::Display).
    pub fn to_spvalue(self) -> SPValue {
        match self {
            PlanState::Initial => "initial".to_spvalue(),
            PlanState::Executing => "executing".to_spvalue(),
            PlanState::Failed => "failed".to_spvalue(),
            PlanState::Completed => "completed".to_spvalue(),
            PlanState::Cancelled => "cancelled".to_spvalue(),
            PlanState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl SOPState {
    /// Parses the string form written to `{sp_id}_sop_state`.
    ///
    /// Anything unrecognised becomes [`SOPState::UNKNOWN`] - including
    /// `"cancelled"`, which has no arm here even though `Display` produces it.
    pub fn from_str(x: &str) -> SOPState {
        match x {
            "initial" => SOPState::Initial,
            "executing" => SOPState::Executing,
            "fatal" => SOPState::Fatal,
            "completed" => SOPState::Completed,
            _ => SOPState::UNKNOWN,
        }
    }

    /// The state as the [`SPValue`] stored in the shared state, matching
    /// [`Display`](std::fmt::Display).
    pub fn to_spvalue(self) -> SPValue {
        match self {
            SOPState::Initial => "initial".to_spvalue(),
            SOPState::Executing => "executing".to_spvalue(),
            SOPState::Fatal => "fatal".to_spvalue(),
            SOPState::Completed => "completed".to_spvalue(),
            SOPState::Cancelled => "cancelled".to_spvalue(),
            SOPState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl PlannerState {
    /// Parses the string form written to `{sp_id}_planner_state`; anything
    /// unrecognised becomes [`PlannerState::UNKNOWN`].
    pub fn from_str(x: &str) -> PlannerState {
        match x {
            "found" => PlannerState::Found,
            "not_found" => PlannerState::NotFound,
            "ready" => PlannerState::Ready,
            _ => PlannerState::UNKNOWN,
        }
    }

    /// The state as the [`SPValue`] stored in the shared state, matching
    /// [`Display`](std::fmt::Display).
    pub fn to_spvalue(self) -> SPValue {
        match self {
            PlannerState::Found => "found".to_spvalue(),
            PlannerState::NotFound => "not_found".to_spvalue(),
            PlannerState::Ready => "ready".to_spvalue(),
            PlannerState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl fmt::Display for PlanState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlanState::UNKNOWN => write!(f, "UNKNOWN"),
            PlanState::Initial => write!(f, "initial"),
            PlanState::Executing => write!(f, "executing"),
            PlanState::Failed => write!(f, "failed"),
            PlanState::Completed => write!(f, "completed"),
            PlanState::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl fmt::Display for SOPState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SOPState::UNKNOWN => write!(f, "UNKNOWN"),
            SOPState::Initial => write!(f, "initial"),
            SOPState::Executing => write!(f, "executing"),
            SOPState::Fatal => write!(f, "fatal"),
            SOPState::Completed => write!(f, "completed"),
            SOPState::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl fmt::Display for PlannerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlannerState::UNKNOWN => write!(f, "UNKNOWN"),
            PlannerState::Found => write!(f, "found"),
            PlannerState::NotFound => write!(f, "not_found"),
            PlannerState::Ready => write!(f, "ready"),
        }
    }
}

/// The lifecycle of a one-shot request to a service interface, such as the
/// transform interface's `{sp_id}_tf_request_state`.
pub enum ServiceRequestState {
    /// Nothing recognisable was read; serialises to `"UNKNOWN"`. The [`Default`].
    UNKNOWN,
    /// Idle and ready to accept a request; `"initial"`.
    Initial,
    /// The request was served; `"succeeded"`.
    Succeeded,
    /// The request could not be served; `"failed"`.
    Failed,
}

impl Default for ServiceRequestState {
    fn default() -> Self {
        ServiceRequestState::UNKNOWN
    }
}

impl ServiceRequestState {
    /// Parses the string form; anything unrecognised becomes
    /// [`ServiceRequestState::UNKNOWN`].
    pub fn from_str(x: &str) -> ServiceRequestState {
        match x {
            "initial" => ServiceRequestState::Initial,
            "succeeded" => ServiceRequestState::Succeeded,
            "failed" => ServiceRequestState::Failed,
            _ => ServiceRequestState::UNKNOWN,
        }
    }
}

impl fmt::Display for ServiceRequestState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServiceRequestState::Initial => write!(f, "initial"),
            ServiceRequestState::Succeeded => write!(f, "succeeded"),
            ServiceRequestState::Failed => write!(f, "failed"),
            ServiceRequestState::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}
/// The lifecycle of a long-running request to an interface, such as a timer's
/// `{sp_id}_timer_{n}_request_state`.
///
/// Unlike [`ServiceRequestState`] it has an in-progress state, so a caller can
/// tell "started" from "finished".
pub enum ActionRequestState {
    /// Nothing recognisable was read; serialises to `"UNKNOWN"`. The [`Default`].
    UNKNOWN,
    /// Idle and ready to accept a request; `"initial"`.
    Initial,
    /// The request was accepted and is still running; `"executing"`.
    Executing,
    /// The request ran to completion; `"succeeded"`.
    Succeeded,
    /// The request could not be served, or was aborted; `"failed"`.
    Failed,
}

impl Default for ActionRequestState {
    fn default() -> Self {
        ActionRequestState::UNKNOWN
    }
}

impl ActionRequestState {
    /// Parses the string form; anything unrecognised becomes
    /// [`ActionRequestState::UNKNOWN`].
    pub fn from_str(x: &str) -> ActionRequestState {
        match x {
            "initial" => ActionRequestState::Initial,
            "executing" => ActionRequestState::Executing,
            "succeeded" => ActionRequestState::Succeeded,
            "failed" => ActionRequestState::Failed,
            _ => ActionRequestState::UNKNOWN,
        }
    }
}

impl fmt::Display for ActionRequestState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActionRequestState::Initial => write!(f, "initial"),
            ActionRequestState::Executing => write!(f, "executing"),
            ActionRequestState::Succeeded => write!(f, "succeeded"),
            ActionRequestState::Failed => write!(f, "failed"),
            ActionRequestState::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}
/// The overall mode of a runner, held in `{sp_id}_runner_state`.
///
/// The only enum here that also goes through serde.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum RunnerState {
    /// Online with nothing to do; serialises to `"idle"`.
    Idle,
    /// Executing normally; `"running"`.
    Running,
    /// Halted and not accepting work; `"stopped"`.
    Stopped,
    /// Temporarily suspended, expected to resume; `"paused"`.
    Paused,
    /// Nothing recognisable was read; `"UNKNOWN"`. Also the [`Default`].
    UNKNOWN,
}

impl Default for RunnerState {
    fn default() -> Self {
        RunnerState::UNKNOWN
    }
}

impl RunnerState {
    /// Parses the string form; anything unrecognised becomes
    /// [`RunnerState::UNKNOWN`].
    pub fn from_str(x: &str) -> RunnerState {
        match x {
            "idle" => RunnerState::Idle,
            "running" => RunnerState::Running,
            "paused" => RunnerState::Paused,
            "stopped" => RunnerState::Stopped,
            _ => RunnerState::UNKNOWN,
        }
    }

    /// The state as the [`SPValue`] stored in the shared state, matching
    /// [`Display`](std::fmt::Display).
    pub fn to_spvalue(self) -> SPValue {
        match self {
            RunnerState::Running => "running".to_spvalue(),
            RunnerState::Paused => "paused".to_spvalue(),
            RunnerState::Stopped => "stopped".to_spvalue(),
            RunnerState::Idle => "idle".to_spvalue(),
            RunnerState::UNKNOWN => "UNKNOWN".to_spvalue(),
        }
    }
}

impl fmt::Display for RunnerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunnerState::UNKNOWN => write!(f, "UNKNOWN"),
            RunnerState::Running => write!(f, "running"),
            RunnerState::Paused => write!(f, "paused"),
            RunnerState::Stopped => write!(f, "stopped"),
            RunnerState::Idle => write!(f, "idle"),
        }
    }
}

/// These enums are the crate's wire format. Every one of them is written into
/// Redis as the string `Display`/`to_spvalue` produces and read back out of
/// Redis through `from_str`, by a *different* runner in a different process -
/// `plan_runner` writes `{sp_id}_plan_state`, `goal_runner` reads it; the SOP
/// runner writes `{sp_id}_sop_state`, the operations read it.
///
/// So the property that actually matters here is not "does `Display` produce a
/// nice string" but `from_str(x.to_string()) == x` for every variant. A variant
/// that fails it is silently unreachable on the reading side, and the branch
/// that handles it is dead code - which is exactly what the two tests at the
/// bottom of this module pin down.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_state_maps_strings_both_ways() {
        let pairs = [
            (PlanState::Initial, "initial"),
            (PlanState::Executing, "executing"),
            (PlanState::Failed, "failed"),
            (PlanState::Completed, "completed"),
            (PlanState::UNKNOWN, "UNKNOWN"),
        ];

        for (variant, text) in pairs {
            assert_eq!(variant.to_string(), text, "Display for {variant:?}");
            assert_eq!(
                variant.clone().to_spvalue(),
                text.to_spvalue(),
                "to_spvalue for {variant:?}"
            );
            assert_eq!(
                PlanState::from_str(text),
                variant,
                "{text} must parse back to the variant it was written from"
            );
        }
    }

    /// BUG (dead branch): `PlanState::Cancelled` renders as "cancelled" but
    /// `from_str` has no arm for it - the `"cancelled" => PlanState::Cancelled`
    /// line is commented out - so it reads back as `UNKNOWN`.
    ///
    /// Consequence, and the reason this is pinned rather than ignored:
    /// `process_operation` sets `{sp_id}_plan_state` to `PlanState::Cancelled`
    /// when a planned operation is cancelled, and `goal_runner` matches on
    /// `PlanState::from_str(&plan_state)` with a `PlanState::Cancelled` arm that
    /// moves the goal to `GoalState::Cancelled`. That arm can never be taken:
    /// the cancelled plan arrives as `UNKNOWN` and the goal stays `Executing`.
    ///
    /// Uncommenting the `from_str` arm fixes it and makes this test fail, which
    /// is the intended signal - update it then.
    #[test]
    fn plan_state_cancelled_does_not_survive_the_round_trip() {
        assert_eq!(PlanState::Cancelled.to_string(), "cancelled");
        assert_eq!(
            PlanState::Cancelled.to_spvalue(),
            "cancelled".to_spvalue(),
            "to_spvalue must still produce the same wire string as Display"
        );
        assert_eq!(
            PlanState::from_str("cancelled"),
            PlanState::UNKNOWN,
            "if this now returns Cancelled the bug is fixed - see the doc comment"
        );
    }

    #[test]
    fn sop_state_maps_strings_both_ways() {
        let pairs = [
            (SOPState::Initial, "initial"),
            (SOPState::Executing, "executing"),
            (SOPState::Fatal, "fatal"),
            (SOPState::Completed, "completed"),
            (SOPState::UNKNOWN, "UNKNOWN"),
        ];

        for (variant, text) in pairs {
            assert_eq!(variant.to_string(), text, "Display for {variant:?}");
            assert_eq!(variant.clone().to_spvalue(), text.to_spvalue());
            assert_eq!(SOPState::from_str(text), variant);
        }
    }

    /// The same hole as `PlanState::Cancelled`, in the enum the SOP runner uses.
    /// `SOP::get_state` can return `SOPState::Cancelled` for a cancelled branch
    /// and `sop_runner` writes it to `{sp_id}_sop_state`; anything reading that
    /// back with `from_str` sees `UNKNOWN`.
    #[test]
    fn sop_state_cancelled_does_not_survive_the_round_trip() {
        assert_eq!(SOPState::Cancelled.to_string(), "cancelled");
        assert_eq!(
            SOPState::Cancelled.to_spvalue(),
            "cancelled".to_spvalue(),
            "to_spvalue must still produce the same wire string as Display"
        );
        assert_eq!(SOPState::from_str("cancelled"), SOPState::UNKNOWN);
    }

    #[test]
    fn planner_state_maps_strings_both_ways() {
        let pairs = [
            (PlannerState::Found, "found"),
            (PlannerState::NotFound, "not_found"),
            (PlannerState::Ready, "ready"),
            (PlannerState::UNKNOWN, "UNKNOWN"),
        ];

        for (variant, text) in pairs {
            assert_eq!(variant.to_string(), text, "Display for {variant:?}");
            assert_eq!(variant.clone().to_spvalue(), text.to_spvalue());
            assert_eq!(PlannerState::from_str(text), variant);
        }
    }

    #[test]
    fn service_request_state_maps_strings_both_ways() {
        let pairs = [
            (ServiceRequestState::Initial, "initial"),
            (ServiceRequestState::Succeeded, "succeeded"),
            (ServiceRequestState::Failed, "failed"),
            (ServiceRequestState::UNKNOWN, "UNKNOWN"),
        ];

        for (variant, text) in pairs {
            assert_eq!(variant.to_string(), text);
            assert_eq!(ServiceRequestState::from_str(text).to_string(), text);
        }
    }

    #[test]
    fn action_request_state_maps_strings_both_ways() {
        let pairs = [
            (ActionRequestState::Initial, "initial"),
            (ActionRequestState::Executing, "executing"),
            (ActionRequestState::Succeeded, "succeeded"),
            (ActionRequestState::Failed, "failed"),
            (ActionRequestState::UNKNOWN, "UNKNOWN"),
        ];

        for (variant, text) in pairs {
            assert_eq!(variant.to_string(), text);
            assert_eq!(ActionRequestState::from_str(text).to_string(), text);
        }
    }

    #[test]
    fn runner_state_maps_strings_both_ways() {
        let pairs = [
            (RunnerState::Idle, "idle"),
            (RunnerState::Running, "running"),
            (RunnerState::Paused, "paused"),
            (RunnerState::Stopped, "stopped"),
            (RunnerState::UNKNOWN, "UNKNOWN"),
        ];

        for (variant, text) in pairs {
            assert_eq!(variant.to_string(), text, "Display for {variant:?}");
            assert_eq!(variant.clone().to_spvalue(), text.to_spvalue());
            assert_eq!(RunnerState::from_str(text), variant);
        }
    }

    /// A value that is not one of the known strings - a typo, a key that was
    /// never initialised, a state written by an older build - has to land on
    /// `UNKNOWN` rather than on whichever variant happens to be first.
    #[test]
    fn anything_unrecognised_parses_as_unknown() {
        for junk in ["", "Initial", "INITIAL", " initial", "nonsense", "42"] {
            assert_eq!(PlanState::from_str(junk), PlanState::UNKNOWN, "{junk:?}");
            assert_eq!(SOPState::from_str(junk), SOPState::UNKNOWN, "{junk:?}");
            assert_eq!(
                PlannerState::from_str(junk),
                PlannerState::UNKNOWN,
                "{junk:?}"
            );
            assert_eq!(RunnerState::from_str(junk), RunnerState::UNKNOWN, "{junk:?}");
            assert_eq!(ServiceRequestState::from_str(junk).to_string(), "UNKNOWN");
            assert_eq!(ActionRequestState::from_str(junk).to_string(), "UNKNOWN");
        }
    }

    /// Everything defaults to `UNKNOWN`, so a freshly constructed runner never
    /// claims to be in a real state before it has read one.
    #[test]
    fn every_default_is_unknown() {
        assert_eq!(PlanState::default(), PlanState::UNKNOWN);
        assert_eq!(SOPState::default(), SOPState::UNKNOWN);
        assert_eq!(PlannerState::default(), PlannerState::UNKNOWN);
        assert_eq!(RunnerState::default(), RunnerState::UNKNOWN);
        assert_eq!(ServiceRequestState::default().to_string(), "UNKNOWN");
        assert_eq!(ActionRequestState::default().to_string(), "UNKNOWN");
    }

    /// `RunnerState` is the one that also goes through serde (it is
    /// `Serialize`/`Deserialize`), so pin that path too.
    #[test]
    fn runner_state_survives_serde() {
        for variant in [
            RunnerState::Idle,
            RunnerState::Running,
            RunnerState::Stopped,
            RunnerState::Paused,
            RunnerState::UNKNOWN,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: RunnerState = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }
}