//! Goal admission and scheduling.
//!
//! Whatever wants something done writes a [`Goal`](crate::running::goal_runner::Goal) into `{sp_id}_incoming_goals`.
//! [`goal_runner`] admits it to the priority-ordered queue in
//! `{sp_id}_scheduled_goals`, promotes one goal at a time to the current goal,
//! triggers the planner for it, and then watches `{sp_id}_plan_state` to decide
//! whether the goal completed, failed or was cancelled.

use crate::*;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

/// How urgent a goal is. The queue is sorted by this, `Top` first.
///
/// The declaration order *is* the ordering - reordering the variants inverts the
/// scheduler.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
    /// Runs before everything else. Useful for periodic housekeeping goals.
    Top,
    /// Runs before `Normal` and `Low`.
    High,
    /// The usual priority for ordinary work.
    Normal,
    /// Runs last. Also where an unrecognised priority ends up.
    Low,
}

/// One request for the system to reach a state.
///
/// Crosses the process boundary as a three-element `SPValue::Array`; see
/// [`goal_to_sp_value`] and [`sp_value_to_goal`].
///
/// ```
/// use micro_sp::running::goal_runner::{Goal, GoalPriority, goal_to_sp_value, sp_value_to_goal};
///
/// let goal = Goal {
///     id: "abc123".to_string(),
///     priority: GoalPriority::High,
///     predicate: "var:pos == c".to_string(),
/// };
///
/// // This is exactly what is written into `{sp_id}_incoming_goals`.
/// let encoded = goal_to_sp_value(&goal);
/// assert_eq!(sp_value_to_goal(&encoded), Ok(goal));
/// ```
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Goal {
    /// Unique id, a 10-character nanoid assigned when the goal is admitted.
    pub id: String,
    /// Where the goal sits in the queue.
    pub priority: GoalPriority,
    /// The goal itself, as a predicate string the planner parses.
    pub predicate: String,
}

/// Encodes a goal as the `[id, priority, predicate]` array stored in the state.
pub fn goal_to_sp_value(goal: &Goal) -> SPValue {
    let id_val = SPValue::String(StringOrUnknown::String(goal.id.clone()));
    let priority_val = SPValue::Int64(IntOrUnknown::Int64(goal.priority.to_int()));
    let predicate_val = SPValue::String(StringOrUnknown::String(goal.predicate.clone()));

    SPValue::Array(ArrayOrUnknown::Array(vec![
        id_val,
        priority_val,
        predicate_val,
    ]))
}

/// Encodes a bare predicate string as a goal array, using the given id and priority.
pub fn goal_string_to_sp_value(unique_id: &str, goal: &String, priority: GoalPriority) -> SPValue {
    let id_val = SPValue::String(StringOrUnknown::String(unique_id.to_string()));
    let priority_val = SPValue::Int64(IntOrUnknown::Int64(priority.to_int()));
    let predicate_val = SPValue::String(StringOrUnknown::String(goal.clone()));

    SPValue::Array(ArrayOrUnknown::Array(vec![
        id_val,
        priority_val,
        predicate_val,
    ]))
}

/// Decodes an `[id, priority, predicate]` array back into a [`Goal`].
///
/// Returns `Err` with a description if the value is not a three-element array of
/// the expected types.
pub fn sp_value_to_goal(sp_value: &SPValue) -> Result<Goal, String> {
    let arr = match sp_value {
        SPValue::Array(ArrayOrUnknown::Array(a)) => a,
        SPValue::Array(ArrayOrUnknown::UNKNOWN) => return Err("Goal Array is UNKNOWN".to_string()),
        _ => return Err(format!("Expected SPValue::Array, found {:?}", sp_value)),
    };

    if arr.len() != 3 {
        return Err(format!("Goal array expected length 3, found {}", arr.len()));
    }

    let id = match &arr[0] {
        SPValue::String(StringOrUnknown::String(s)) => s.clone(),
        _ => return Err(format!("ID expected String, found {:?}", arr[0])),
    };

    let priority = match &arr[1] {
        SPValue::Int64(IntOrUnknown::Int64(p)) => *p,
        _ => return Err(format!("Priority expected Int64, found {:?}", arr[1])),
    };

    let predicate = match &arr[2] {
        SPValue::String(StringOrUnknown::String(s)) => s.clone(),
        _ => return Err(format!("Predicate expected String, found {:?}", arr[2])),
    };

    Ok(Goal {
        id,
        priority: GoalPriority::from_int(&priority),
        predicate,
    })
}

/// Lifecycle of the current goal, mirrored in `{sp_id}_current_goal_state`.
#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalState {
    /// Promoted from the queue, not yet planned for.
    Initial,
    /// A plan exists and the plan runner is working through it.
    Executing,
    /// Planning or execution gave up on this goal.
    Failed,
    /// Aborted before it could complete.
    Cancelled,
    /// The goal predicate holds.
    Completed,
    /// No goal, or a state string that could not be parsed.
    UNKNOWN,
}

impl GoalPriority {
    /// Decodes the stored integer encoding. Out-of-range values become `Low`.
    pub fn from_int(x: &i64) -> GoalPriority {
        match x {
            0 => GoalPriority::Top,
            1 => GoalPriority::High,
            2 => GoalPriority::Normal,
            3 => GoalPriority::Low,
            _ => {
                log::error!(target: &&format!("goal_priority"), 
                    "Priority out of range [1, 2, 3], defaulting to low.");
                GoalPriority::Low
            }
        }
    }

    /// Encodes the priority as the integer stored in the state, `Top` being `0`.
    pub fn to_int(&self) -> i64 {
        match self {
            GoalPriority::Top => 0,
            GoalPriority::High => 1,
            GoalPriority::Normal => 2,
            GoalPriority::Low => 3,
        }
    }

    /// Parses `"top"`, `"high"`, `"normal"` or `"low"`. Anything else becomes `Low`.
    pub fn from_str(x: &str) -> GoalPriority {
        match x {
            "top" => GoalPriority::Top,
            "high" => GoalPriority::High,
            "normal" => GoalPriority::Normal,
            "low" => GoalPriority::Low,
            _ => {
                log::error!(target: &&format!("goal_priority"), 
                    "Unknown priority {}, defaulting to low.", x);
                GoalPriority::Low
            }
        }
    }
}

impl fmt::Display for GoalPriority {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GoalPriority::Top => write!(fmtr, "top"),
            GoalPriority::High => write!(fmtr, "high"),
            GoalPriority::Normal => write!(fmtr, "normal"),
            GoalPriority::Low => write!(fmtr, "low"),
        }
    }
}

impl GoalState {
    /// Parses the state string stored in Redis. Unknown strings become `UNKNOWN`.
    pub fn from_str(x: &str) -> GoalState {
        match x {
            "initial" => GoalState::Initial,
            "executing" => GoalState::Executing,
            "failed" => GoalState::Failed,
            "cancelled" => GoalState::Cancelled,
            "completed" => GoalState::Completed,
            "unknown" => GoalState::UNKNOWN,
            _ => GoalState::UNKNOWN,
        }
    }

    /// Encodes the state as the lowercase string [`SPValue`] stored in Redis.
    pub fn to_spvalue(self) -> SPValue {
        self.to_string().to_spvalue()
    }
}

impl fmt::Display for GoalState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GoalState::Initial => write!(f, "initial"),
            GoalState::Executing => write!(f, "executing"),
            GoalState::Cancelled => write!(f, "cancelled"),
            GoalState::Failed => write!(f, "failed"),
            GoalState::Completed => write!(f, "completed"),
            GoalState::UNKNOWN => write!(f, "unknown"),
        }
    }
}

/// Merge newly arrived goals into the queue, ordered by priority.
pub fn admit_goals(mut scheduled: Vec<Goal>, incoming: Vec<Goal>) -> Vec<Goal> {
    scheduled.extend(incoming.into_iter().map(|goal| Goal {
        id: nanoid::nanoid!(10, &NANOID_ALPHABET),
        ..goal
    }));

    // A goal written straight into `_scheduled_goals` bypasses the step above,
    // so anything still without an id gets one here - once; on the next tick it
    // has an id and is left alone.
    for goal in scheduled.iter_mut() {
        if goal.id.is_empty() || goal.id == "UNKNOWN" {
            goal.id = nanoid::nanoid!(10, &NANOID_ALPHABET);
        }
    }

    // Stable, so goals of equal priority keep their arrival order.
    scheduled.sort_by_key(|g| g.priority);
    scheduled
}

/// Runs the goal scheduler until the process ends.
///
/// On every tick it reads the goal keys for `sp_id` from Redis, moves goals from
/// `{sp_id}_incoming_goals` into the priority-sorted `{sp_id}_scheduled_goals`,
/// promotes the first one into `{sp_id}_current_goal_*`, triggers a replan, and
/// writes back the resulting goal state. `connection_manager` is the shared Redis
/// connection; log output goes to the `{sp_id}_goal_runner` target.
///
/// ```no_run
/// use micro_sp::*;
/// use micro_sp::running::goal_runner::goal_runner;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let connection_manager = Arc::new(ConnectionManager::new().await);
///
/// // Loops forever, so this is normally the whole body of its own task.
/// goal_runner("sp", &connection_manager).await?;
/// # Ok(())
/// # }
/// ```
pub async fn goal_runner(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    activity_log::init_from_env();
    let mut interval = runner_interval();
    let log_target = &format!("{}_goal_runner", sp_id);

    log::info!(target: log_target, "Online.");

    // For nicer logging
    let mut goal_info_old = String::new();

    let keys = goal_runner_keys(sp_id);

    let mut con = connection_manager.get_connection().await;

    loop {
        interval.tick().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

        let new_state = goal_tick(sp_id, &mut con, &state, &mut goal_info_old, &log_target).await;

        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            activity_log::log_state_diff(&log_target, &state, &modified_state);
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
}

/// The keys the goal runner reads and writes.
///
/// Static for the lifetime of the runner, so a caller sharing one snapshot
/// across several runners can fold this into its union once.
pub fn goal_runner_keys(sp_id: &str) -> Vec<String> {
    vec![
        format!("{}_current_goal_state", sp_id),
        format!("{}_current_goal_id", sp_id),
        format!("{}_current_goal_predicate", sp_id),
        format!("{}_goal_runner_information", sp_id),
        format!("{}_planner_state", sp_id),
        format!("{}_plan_state", sp_id),
        format!("{}_plan", sp_id),
        format!("{}_scheduled_goals", sp_id),
        format!("{}_incoming_goals", sp_id),
        format!("{}_replan_trigger", sp_id),
        format!("{}_replanned", sp_id),
        format!("{}_plan_current_step", sp_id),
        format!("{}_replan_for_same_goal", sp_id),
    ]
}

/// One tick of goal admission: drain whatever was posted, keep the queue in
/// priority order, promote one goal at a time, trigger the planner for it, and
/// advance the current goal on what the plan runner reported.
///
/// `goal_info_old` is the last line this runner logged, carried across ticks so
/// the same message is not repeated every 5 ms. `con` is needed for exactly one
/// thing - the atomic drain of `{sp_id}_incoming_goals` - and the caller still
/// owns the snapshot and the write. [`goal_runner`] calls it with a snapshot of
/// its own keys; the sequential runner calls it with the shared snapshot and
/// threads the result into the next body.
pub async fn goal_tick(
    sp_id: &str,
    con: &mut SPConnection,
    state: &State,
    goal_info_old: &mut String,
    log_target: &str,
) -> State {
    let current_goal_state = state.get_string_or_default_to_unknown(
        &format!("{}_current_goal_state", sp_id),
        &log_target,
    );

    let mut goal_runner_information = state.get_string_or_default_to_unknown(
        &format!("{}_goal_runner_information", sp_id),
        &log_target,
    );

    let current_goal_id = state
        .get_string_or_default_to_unknown(&format!("{}_current_goal_id", sp_id), &log_target);

    let current_goal_predicate = state.get_string_or_default_to_unknown(
        &format!("{}_current_goal_predicate", sp_id),
        &log_target,
    );

    let replan_for_same_goal = state
        .get_bool_or_default_to_false(&format!("{}_replan_for_same_goal", sp_id), &log_target);

    let plan_state =
        state.get_string_or_default_to_unknown(&format!("{}_plan_state", sp_id), &log_target);

    let scheduled_goals_sp_val =
        state.get_array_or_default_to_empty(&format!("{}_scheduled_goals", sp_id), &log_target);

    let mut scheduled_goals = vec![];
    for goal_sp_val in scheduled_goals_sp_val {
        match sp_value_to_goal(&goal_sp_val) {
            Ok(goal) => scheduled_goals.push(goal),
            Err(_) => (),
        }
    }

    // Drain the queue with an atomic take instead of reading it here and
    // blind-writing an empty array back at the end of the tick.
    // `_incoming_goals` is written by whatever is asking for work - a
    // dashboard, a bridge, another process - so a goal posted between that
    // read and that write would be erased, and because the poster's own
    // write succeeded nothing would ever retry it.
    //
    // The snapshot decides only *whether* to drain. An empty one means
    // there is nothing to take and the tick issues no write at all; a goal
    // landing right afterwards stays in the key and the next tick sees it.
    let incoming_goals_key = format!("{}_incoming_goals", sp_id);
    let incoming_goals_sp_val = if state
        .get_array_or_default_to_empty(&incoming_goals_key, &log_target)
        .is_empty()
    {
        vec![]
    } else {
        match StateManager::take_sp_value(
            con,
            &incoming_goals_key,
            &Vec::<SPValue>::new().to_spvalue(),
        )
        .await
        {
            Some(SPValue::Array(ArrayOrUnknown::Array(goals))) => goals,
            _ => vec![],
        }
    };

    let mut incoming_goals = vec![];
    for goal_sp_val in incoming_goals_sp_val {
        match sp_value_to_goal(&goal_sp_val) {
            Ok(goal) => incoming_goals.push(goal),
            Err(_) => (),
        }
    }

    if *goal_info_old != goal_runner_information {
        if goal_runner_information != "UNKNOWN".to_string() {
            log::info!(target: &format!("{}_goal_runner", sp_id), "{goal_runner_information}");
        }

        *goal_info_old = goal_runner_information.clone()
    }

    let mut new_state = state.clone();

    // Handle incoming goals first. A goal gets its unique id exactly once,
    // here, as it is admitted to the queue - goals already scheduled keep
    // theirs. Re-generating the whole queue's ids on every tick (which is
    // what this used to do) made the serialised value differ every time, so
    // an MSET went out 10x/s for as long as anything was queued, and a
    // goal's id changed while it waited.
    let scheduled_goals = admit_goals(scheduled_goals, incoming_goals);

    let scheduled_goals_sp_values: Vec<SPValue> = scheduled_goals
        .iter()
        .map(|x| goal_to_sp_value(x))
        .collect();
    new_state.update_mut(
        &format!("{}_scheduled_goals", sp_id),
        scheduled_goals_sp_values.to_spvalue(),
    );

    match GoalState::from_str(&current_goal_state) {
        GoalState::Initial => {
            if replan_for_same_goal {  // This should be Option(number of replans) for every goal
                goal_runner_information = format!(
                    "Replan for same goal {}: \n       {}",
                    current_goal_id, current_goal_predicate
                );
                new_state.update_mut(
                    &format!("{}_replan_for_same_goal", sp_id),
                    false.to_spvalue(),
                );
                new_state.update_mut(&format!("{}_replan_trigger", sp_id), true.to_spvalue());
                new_state.update_mut(&format!("{}_replanned", sp_id), false.to_spvalue());
                new_state.update_mut(&format!("{}_plan_current_step", sp_id), 0.to_spvalue());
                new_state
                    .update_mut(&format!("{}_plan", sp_id), Vec::<String>::new().to_spvalue());
                new_state.update_mut(&format!("{}_plan_state", sp_id), "initial".to_spvalue());
                new_state.update_mut(&format!("{}_planner_state", sp_id), "ready".to_spvalue());
            } else {
                if !scheduled_goals.is_empty() {
                    match scheduled_goals.split_first() {
                        Some((current, rest)) => {
                            let rest_of_the_goals: Vec<SPValue> =
                                rest.iter().map(|x| goal_to_sp_value(x)).collect();
                            goal_runner_information = format!(
                                "Initializing new goal {}: \n       {}",
                                current.id, current.predicate
                            );
                            new_state.update_mut(
                                &format!("{}_scheduled_goals", sp_id),
                                rest_of_the_goals.to_spvalue(),
                            );
                            new_state.update_mut(
                                &format!("{}_current_goal_id", sp_id),
                                current.id.to_string().to_spvalue(),
                            );
                            new_state.update_mut(
                                &format!("{}_current_goal_state", sp_id),
                                GoalState::Executing.to_string().to_spvalue(),
                            );
                            new_state.update_mut(
                                &format!("{}_current_goal_predicate", sp_id),
                                current.predicate.to_string().to_spvalue(),
                            );
                            new_state
                                .update_mut(&format!("{}_replan_trigger", sp_id), true.to_spvalue());
                            new_state
                                .update_mut(&format!("{}_replanned", sp_id), false.to_spvalue());
                            new_state
                                .update_mut(&format!("{}_plan_current_step", sp_id), 0.to_spvalue());
                            new_state.update_mut(
                                &format!("{}_plan", sp_id),
                                Vec::<String>::new().to_spvalue(),
                            );
                            new_state
                                .update_mut(&format!("{}_plan_state", sp_id), "initial".to_spvalue());
                            new_state
                                .update_mut(&format!("{}_planner_state", sp_id), "ready".to_spvalue());
                        }
                        None => {
                            log::error!(target: log_target, "This shouldn't happen, investigate.")
                        }
                    }
                } else {
                    goal_runner_information =
                        "No goals scheduled, goal list is empty.".to_string();
                }
            }
        }

        GoalState::Executing => {
            goal_runner_information = format!(
                "Executing goal {}: \n       {}",
                current_goal_id, current_goal_predicate
            );
            match PlanState::from_str(&plan_state) {
                PlanState::Initial => (),
                PlanState::Executing => (),
                PlanState::Failed => {
                    new_state.update_mut(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::Failed.to_string().to_spvalue(),
                    )
                }
                PlanState::Completed => {
                    new_state.update_mut(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::Completed.to_string().to_spvalue(),
                    )
                }
                PlanState::Cancelled => {
                    new_state.update_mut(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::Cancelled.to_string().to_spvalue(),
                    )
                }
                PlanState::UNKNOWN => {
                    new_state.update_mut(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::UNKNOWN.to_string().to_spvalue(),
                    )
                }
            }
        }

        // Plan fails only if operation is unrecoverable, so it is ok to go to initial here.
        GoalState::Failed => {
            goal_runner_information = format!(
                "Goal {} failed: \n       {}",
                current_goal_id, current_goal_predicate
            );
            new_state.update_mut(
                &format!("{}_current_goal_state", sp_id),
                GoalState::Initial.to_string().to_spvalue(),
            )
        }
        GoalState::Completed => {
            goal_runner_information = format!(
                "Goal {} completed: \n       {}",
                current_goal_id, current_goal_predicate
            );
            new_state.update_mut(
                &format!("{}_current_goal_state", sp_id),
                GoalState::Initial.to_string().to_spvalue(),
            )
        }
        GoalState::Cancelled => {
            goal_runner_information = format!(
                "Goal {} cancelled: \n       {}",
                current_goal_id, current_goal_predicate
            );
            new_state.update_mut(
                &format!("{}_current_goal_state", sp_id),
                GoalState::Initial.to_string().to_spvalue(),
            )
        }
        GoalState::UNKNOWN => {
            new_state.update_mut(
                &format!("{}_current_goal_state", sp_id),
                GoalState::Initial.to_string().to_spvalue(),
            )
        }
    }
    new_state.update_mut(
        &format!("{}_goal_runner_information", sp_id),
        goal_runner_information.to_spvalue(),
    );
    new_state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn goal(id: &str, priority: GoalPriority, predicate: &str) -> Goal {
        Goal {
            id: id.to_string(),
            priority,
            predicate: predicate.to_string(),
        }
    }

    /// The invariant that keeps the goal runner from writing on every tick: a
    /// queue that nothing has been added to must come back out unchanged, so
    /// the diff against the previous tick is empty.
    #[test]
    fn an_unchanged_queue_is_returned_unchanged() {
        let queue = vec![
            goal("aaa", GoalPriority::Normal, "var:x == true"),
            goal("bbb", GoalPriority::Low, "var:y == true"),
        ];

        let once = admit_goals(queue.clone(), vec![]);
        let twice = admit_goals(once.clone(), vec![]);
        let thrice = admit_goals(twice.clone(), vec![]);

        assert_eq!(once, queue);
        assert_eq!(twice, once);
        assert_eq!(thrice, twice, "repeated ticks must not perturb the queue");
    }

    #[test]
    fn admitted_goals_get_a_fresh_unique_id() {
        let incoming = vec![
            goal("", GoalPriority::Normal, "var:x == true"),
            goal("", GoalPriority::Normal, "var:y == true"),
        ];

        let queue = admit_goals(vec![], incoming);

        assert_eq!(queue.len(), 2);
        assert!(!queue[0].id.is_empty());
        assert!(!queue[1].id.is_empty());
        assert_ne!(queue[0].id, queue[1].id, "ids must be unique");
    }

    /// An id supplied by the caller on an *incoming* goal is replaced, matching
    /// the previous behaviour that every admitted goal gets a fresh unique id.
    #[test]
    fn incoming_ids_are_replaced_but_only_once() {
        let queue = admit_goals(
            vec![],
            vec![goal("caller_supplied", GoalPriority::Normal, "var:x == true")],
        );
        assert_eq!(queue.len(), 1);
        assert_ne!(queue[0].id, "caller_supplied");

        let assigned = queue[0].id.clone();
        let queue = admit_goals(queue, vec![]);
        assert_eq!(queue[0].id, assigned, "the id must survive the next tick");
    }

    /// A goal placed straight into the queue without going through the inbox
    /// still ends up with an id, and keeps it.
    #[test]
    fn a_queued_goal_without_an_id_gets_one_and_keeps_it() {
        let queue = admit_goals(vec![goal("", GoalPriority::High, "var:x == true")], vec![]);
        assert!(!queue[0].id.is_empty());

        let assigned = queue[0].id.clone();
        let queue = admit_goals(queue, vec![]);
        assert_eq!(queue[0].id, assigned);

        let queue = admit_goals(
            vec![goal("UNKNOWN", GoalPriority::High, "var:x == true")],
            vec![],
        );
        assert_ne!(queue[0].id, "UNKNOWN");
    }

    #[test]
    fn the_queue_is_ordered_by_priority_and_stable_within_it() {
        let queue = admit_goals(
            vec![
                goal("low", GoalPriority::Low, "var:a == true"),
                goal("normal_first", GoalPriority::Normal, "var:b == true"),
                goal("normal_second", GoalPriority::Normal, "var:c == true"),
                goal("top", GoalPriority::Top, "var:d == true"),
            ],
            vec![],
        );

        let ids: Vec<&str> = queue.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, vec!["top", "normal_first", "normal_second", "low"]);
    }

    #[test]
    fn admitted_goals_join_the_existing_queue() {
        let queue = admit_goals(
            vec![goal("queued", GoalPriority::Normal, "var:a == true")],
            vec![goal("", GoalPriority::Top, "var:b == true")],
        );

        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0].predicate, "var:b == true", "top priority goes first");
        assert_eq!(queue[1].id, "queued", "the queued goal keeps its id");
    }
}

/// The goal wire format.
///
/// A goal crosses the process boundary as a three-element `SPValue::Array`
/// written into `{sp_id}_incoming_goals` by whatever is asking for work - a
/// dashboard, a ROS bridge, another micro_sp process. `sp_value_to_goal` is the
/// only validation on that path, and a goal it rejects is dropped silently by
/// the runner (`Err(_) => ()`), so its rejection rules are part of the contract.
#[cfg(test)]
mod goal_encoding_tests {
    use super::*;

    fn goal(id: &str, priority: GoalPriority, predicate: &str) -> Goal {
        Goal {
            id: id.to_string(),
            priority,
            predicate: predicate.to_string(),
        }
    }

    #[test]
    fn a_goal_survives_the_round_trip() {
        for priority in [
            GoalPriority::Top,
            GoalPriority::High,
            GoalPriority::Normal,
            GoalPriority::Low,
        ] {
            let original = goal("abc123", priority, "var:pos == c");
            let encoded = goal_to_sp_value(&original);
            assert_eq!(sp_value_to_goal(&encoded), Ok(original));
        }
    }

    /// The convenience constructor used by callers that only have a predicate
    /// string has to produce the same encoding.
    #[test]
    fn goal_string_to_sp_value_matches_goal_to_sp_value() {
        let from_string =
            goal_string_to_sp_value("abc123", &"var:x == true".to_string(), GoalPriority::High);
        let from_goal = goal_to_sp_value(&goal("abc123", GoalPriority::High, "var:x == true"));
        assert_eq!(from_string, from_goal);
    }

    /// Every way a malformed goal can arrive. Each of these is dropped by the
    /// runner rather than crashing it, so the point is that they are *rejected*
    /// rather than decoded into something plausible-looking.
    #[test]
    fn a_malformed_goal_is_rejected() {
        let cases: Vec<(&str, SPValue)> = vec![
            ("not an array", "just a string".to_spvalue()),
            (
                "an UNKNOWN array",
                SPValue::Array(ArrayOrUnknown::UNKNOWN),
            ),
            ("too short", vec!["a".to_spvalue(), 1.to_spvalue()].to_spvalue()),
            (
                "too long",
                vec![
                    "a".to_spvalue(),
                    1.to_spvalue(),
                    "p".to_spvalue(),
                    "extra".to_spvalue(),
                ]
                .to_spvalue(),
            ),
            (
                "id of the wrong type",
                vec![1.to_spvalue(), 1.to_spvalue(), "p".to_spvalue()].to_spvalue(),
            ),
            (
                "priority of the wrong type",
                vec!["a".to_spvalue(), "high".to_spvalue(), "p".to_spvalue()].to_spvalue(),
            ),
            (
                "predicate of the wrong type",
                vec!["a".to_spvalue(), 1.to_spvalue(), 7.to_spvalue()].to_spvalue(),
            ),
        ];

        for (label, value) in cases {
            assert!(
                sp_value_to_goal(&value).is_err(),
                "{label} should have been rejected"
            );
        }
    }

    /// A priority integer that is out of range is clamped to `Low` rather than
    /// rejected - a goal with a nonsense priority still runs, last.
    #[test]
    fn an_out_of_range_priority_becomes_low() {
        for out_of_range in [-1, 4, 99, i64::MAX] {
            assert_eq!(GoalPriority::from_int(&out_of_range), GoalPriority::Low);
        }
    }

    #[test]
    fn priority_maps_between_int_string_and_variant() {
        let table = [
            (GoalPriority::Top, 0, "top"),
            (GoalPriority::High, 1, "high"),
            (GoalPriority::Normal, 2, "normal"),
            (GoalPriority::Low, 3, "low"),
        ];

        for (variant, int, text) in table {
            assert_eq!(variant.to_int(), int);
            assert_eq!(GoalPriority::from_int(&int), variant);
            assert_eq!(variant.to_string(), text);
            assert_eq!(GoalPriority::from_str(text), variant);
        }

        assert_eq!(GoalPriority::from_str("URGENT"), GoalPriority::Low);
        assert_eq!(GoalPriority::from_str(""), GoalPriority::Low);
    }

    /// The ordering the queue is sorted by. `Top` has to sort *first*, which
    /// depends on the declaration order of the enum rather than on anything
    /// written down - worth pinning, because reordering the variants silently
    /// inverts the scheduler.
    #[test]
    fn priorities_order_top_first() {
        let mut priorities = vec![
            GoalPriority::Low,
            GoalPriority::Top,
            GoalPriority::Normal,
            GoalPriority::High,
        ];
        priorities.sort();
        assert_eq!(
            priorities,
            vec![
                GoalPriority::Top,
                GoalPriority::High,
                GoalPriority::Normal,
                GoalPriority::Low
            ]
        );
    }

    /// Unlike `PlanState` and `SOPState`, `GoalState` round-trips completely -
    /// including `Cancelled`, and including the lowercase "unknown" that its
    /// `Display` produces.
    #[test]
    fn goal_state_round_trips_through_its_string_form() {
        for variant in [
            GoalState::Initial,
            GoalState::Executing,
            GoalState::Failed,
            GoalState::Cancelled,
            GoalState::Completed,
            GoalState::UNKNOWN,
        ] {
            let text = variant.to_string();
            assert_eq!(
                GoalState::from_str(&text),
                variant,
                "'{text}' must parse back to {variant:?}"
            );
            assert_eq!(variant.clone().to_spvalue(), text.to_spvalue());
        }
    }

    #[test]
    fn an_unrecognised_goal_state_is_unknown() {
        for junk in ["", "Initial", "UNKNOWN", "nonsense"] {
            assert_eq!(GoalState::from_str(junk), GoalState::UNKNOWN, "{junk:?}");
        }
    }
}

/// The goal runner, driven end to end against a real Redis.
///
/// The runner is one long loop with no extractable pure core, so this is the
/// only way to reach it. What it implements is the top of the control stack:
/// take the highest-priority queued goal, publish it as the current goal, ask
/// the planner for a plan, and then watch `{sp_id}_plan_state` to decide
/// whether the goal succeeded, failed or was cancelled.
///
/// Every one of those is a cross-runner handover through shared keys, so the
/// tests below are written as "set what the other runner would have set, then
/// check what this one does about it".
#[cfg(test)]
mod goal_runner_tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    const SP: &str = "sp";
    const TARGET: &str = "test";

    async fn redis() -> (ContainerAsync<Redis>, Arc<ConnectionManager>) {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let manager = Arc::new(ConnectionManager::new().await);
        let mut con = manager.get_connection().await;
        StateManager::flush_state(&mut con).await;
        let state = generate_runner_state_variables(SP, 0, TARGET);
        StateManager::set_state(&mut con, &state).await;
        (container, manager)
    }

    fn key(suffix: &str) -> String {
        format!("{SP}_{suffix}")
    }

    fn spawn_runner(manager: &Arc<ConnectionManager>) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(manager);
        tokio::spawn(async move {
            let _ = goal_runner(SP, &manager).await;
        })
    }

    async fn text(con: &mut SPConnection, suffix: &str) -> String {
        match StateManager::get_sp_value(con, &key(suffix)).await {
            Some(SPValue::String(StringOrUnknown::String(s))) => s,
            other => format!("{other:?}"),
        }
    }

    async fn wait_for(con: &mut SPConnection, suffix: &str, expected: &str, ms: u64) -> String {
        let deadline = std::time::Instant::now() + Duration::from_millis(ms);
        let mut last = String::new();
        while std::time::Instant::now() < deadline {
            last = text(con, suffix).await;
            if last == expected {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        last
    }

    async fn queue_incoming(con: &mut SPConnection, goals: Vec<(GoalPriority, &str)>) {
        let encoded: Vec<SPValue> = goals
            .into_iter()
            .map(|(priority, predicate)| {
                goal_string_to_sp_value("", &predicate.to_string(), priority)
            })
            .collect();
        StateManager::set_sp_value(con, &key("incoming_goals"), &encoded.to_spvalue()).await;
    }

    async fn scheduled(con: &mut SPConnection) -> Vec<Goal> {
        match StateManager::get_sp_value(con, &key("scheduled_goals")).await {
            Some(SPValue::Array(ArrayOrUnknown::Array(items))) => {
                items.iter().filter_map(|v| sp_value_to_goal(v).ok()).collect()
            }
            _ => vec![],
        }
    }

    /// The handover that starts everything: a goal arrives, gets admitted,
    /// becomes the current goal, and the planner is triggered.
    #[tokio::test]
    #[serial]
    async fn an_incoming_goal_becomes_the_current_goal_and_triggers_the_planner() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"initial".to_spvalue())
            .await;

        let runner = spawn_runner(&manager);
        queue_incoming(&mut con, vec![(GoalPriority::Normal, "var:pos == c")]).await;

        assert_eq!(
            wait_for(&mut con, "current_goal_state", "executing", 3000).await,
            "executing"
        );
        assert_eq!(text(&mut con, "current_goal_predicate").await, "var:pos == c");
        assert!(!text(&mut con, "current_goal_id").await.is_empty());
        runner.abort();

        // The planner handshake is armed, and the previous plan is cleared.
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("replan_trigger")).await,
            Some(true.to_spvalue())
        );
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("replanned")).await,
            Some(false.to_spvalue())
        );
        assert_eq!(text(&mut con, "planner_state").await, "ready");
        assert_eq!(text(&mut con, "plan_state").await, "initial");
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("plan_current_step")).await,
            Some(0.to_spvalue())
        );
        assert!(scheduled(&mut con).await.is_empty(), "the queue is drained");
    }

    /// The inbox is drained on admission, so a producer can write into it
    /// without first reading what is there.
    #[tokio::test]
    #[serial]
    async fn the_inbox_is_emptied_once_its_goals_are_admitted() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"executing".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("plan_state"), &"executing".to_spvalue()).await;

        let runner = spawn_runner(&manager);
        queue_incoming(
            &mut con,
            vec![
                (GoalPriority::Low, "var:a == true"),
                (GoalPriority::Top, "var:b == true"),
            ],
        )
        .await;

        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        while std::time::Instant::now() < deadline && scheduled(&mut con).await.len() < 2 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        runner.abort();

        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("incoming_goals")).await,
            Some(Vec::<SPValue>::new().to_spvalue())
        );

        // And the queue came out priority-ordered, with ids assigned.
        let queue = scheduled(&mut con).await;
        assert_eq!(queue.len(), 2);
        assert_eq!(queue[0].predicate, "var:b == true", "Top goes first");
        assert!(queue.iter().all(|g| !g.id.is_empty()));
    }

    /// Goals are taken one at a time: the second stays queued until the first
    /// has finished.
    #[tokio::test]
    #[serial]
    async fn only_one_goal_is_started_at_a_time() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"initial".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("plan_state"), &"executing".to_spvalue()).await;

        let runner = spawn_runner(&manager);
        queue_incoming(
            &mut con,
            vec![
                (GoalPriority::High, "var:first == true"),
                (GoalPriority::Normal, "var:second == true"),
            ],
        )
        .await;

        assert_eq!(
            wait_for(&mut con, "current_goal_state", "executing", 3000).await,
            "executing"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
        runner.abort();

        assert_eq!(text(&mut con, "current_goal_predicate").await, "var:first == true");
        let queue = scheduled(&mut con).await;
        assert_eq!(queue.len(), 1, "the second goal must still be queued");
        assert_eq!(queue[0].predicate, "var:second == true");
    }

    /// The outcomes the plan runner reports back, and what each does to the
    /// goal. The observable end state is the same for all of them - the goal is
    /// released and the next queued goal starts - so each test parks a second
    /// goal in the queue and checks that it takes over. Asserting on the
    /// intermediate `completed`/`failed` value would be a race: at a 5 ms tick
    /// the runner passes through it and back to `initial` faster than a poller
    /// can reliably observe.
    async fn the_plan_finishing_as(outcome: &str) -> String {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"executing".to_spvalue())
            .await;
        StateManager::set_sp_value(
            &mut con,
            &key("current_goal_predicate"),
            &"var:running == true".to_spvalue(),
        )
        .await;
        StateManager::set_sp_value(&mut con, &key("plan_state"), &"executing".to_spvalue()).await;

        let runner = spawn_runner(&manager);
        // Park the next goal in the queue so the handover is observable.
        queue_incoming(&mut con, vec![(GoalPriority::Normal, "var:next == true")]).await;
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        while std::time::Instant::now() < deadline && scheduled(&mut con).await.is_empty() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert_eq!(
            text(&mut con, "current_goal_predicate").await,
            "var:running == true",
            "the queued goal must not start while one is executing"
        );

        // The plan runner reports the outcome.
        StateManager::set_sp_value(&mut con, &key("plan_state"), &outcome.to_spvalue()).await;

        let taken_over = wait_for(&mut con, "current_goal_predicate", "var:next == true", 3000).await;
        runner.abort();
        taken_over
    }

    #[tokio::test]
    #[serial]
    async fn a_completed_plan_frees_the_runner_for_the_next_goal() {
        assert_eq!(
            the_plan_finishing_as("completed").await,
            "var:next == true",
            "a completed plan must release the goal and let the queue advance"
        );
    }

    #[tokio::test]
    #[serial]
    async fn a_failed_plan_frees_the_runner_for_the_next_goal() {
        assert_eq!(
            the_plan_finishing_as("failed").await,
            "var:next == true",
            "a failed plan must release the goal rather than wedge the runner"
        );
    }

    /// BUG, reachable from here: `process_operation` sets `{sp_id}_plan_state`
    /// to "cancelled" when a planned operation is cancelled, and the
    /// `PlanState::Cancelled` arm above is meant to move the goal to
    /// `GoalState::Cancelled`. It never runs, because `PlanState::from_str` has
    /// no "cancelled" arm and the value arrives as `UNKNOWN` - so the goal takes
    /// the `UNKNOWN` arm instead and is never reported as cancelled.
    ///
    /// The goal does still get released, which is why this has gone unnoticed;
    /// what is lost is the distinction between "the operator stopped it" and
    /// "something unrecognised happened". The information line is the stable
    /// witness: the `Cancelled` arm is the only thing that writes "cancelled"
    /// into it, and it never does. See
    /// `running::runner_states::tests::plan_state_cancelled_does_not_survive_the_round_trip`.
    #[tokio::test]
    #[serial]
    async fn a_cancelled_plan_is_never_reported_as_a_cancelled_goal() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"executing".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("plan_state"), &"executing".to_spvalue()).await;
        StateManager::set_sp_value(
            &mut con,
            &key("current_goal_id"),
            &"goal_one".to_spvalue(),
        )
        .await;

        let runner = spawn_runner(&manager);
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Watch every information line the runner writes from here on. Note
        // the goal id must not itself contain the word being searched for.
        let mut seen_information: Vec<String> = vec![];
        StateManager::set_sp_value(
            &mut con,
            &key("plan_state"),
            &PlanState::Cancelled.to_string().to_spvalue(),
        )
        .await;
        let deadline = std::time::Instant::now() + Duration::from_millis(1000);
        while std::time::Instant::now() < deadline {
            let information = text(&mut con, "goal_runner_information").await;
            if seen_information.last() != Some(&information) {
                seen_information.push(information);
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        runner.abort();

        assert!(
            !seen_information.iter().any(|i| i.contains("cancelled")),
            "if a 'cancelled' line now appears the PlanState::from_str hole is fixed: {seen_information:?}"
        );
        // It is released all the same, so the system does not wedge.
        assert_eq!(text(&mut con, "current_goal_state").await, "initial");
    }

    /// `_replan_for_same_goal` re-plans without taking a new goal off the
    /// queue - the escape hatch for "the world moved, work out a new route to
    /// the same place".
    #[tokio::test]
    #[serial]
    async fn replan_for_same_goal_re_triggers_the_planner_for_the_current_goal() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"initial".to_spvalue())
            .await;
        StateManager::set_sp_value(
            &mut con,
            &key("current_goal_predicate"),
            &"var:pos == c".to_spvalue(),
        )
        .await;
        StateManager::set_sp_value(&mut con, &key("current_goal_id"), &"goal_one".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("replan_for_same_goal"), &true.to_spvalue())
            .await;

        let runner = spawn_runner(&manager);
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        while std::time::Instant::now() < deadline
            && StateManager::get_sp_value(&mut con, &key("replan_for_same_goal")).await
                != Some(false.to_spvalue())
        {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        runner.abort();

        assert_eq!(
            text(&mut con, "current_goal_id").await,
            "goal_one",
            "the current goal must not have been replaced"
        );
        assert_eq!(text(&mut con, "current_goal_predicate").await, "var:pos == c");
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("replan_trigger")).await,
            Some(true.to_spvalue()),
            "the planner is asked again"
        );
        assert_eq!(text(&mut con, "planner_state").await, "ready");
        assert_eq!(text(&mut con, "plan_state").await, "initial");
        assert_eq!(
            StateManager::get_sp_value(&mut con, &key("replan_for_same_goal")).await,
            Some(false.to_spvalue()),
            "the request is consumed, so it re-plans once rather than forever"
        );
    }

    /// The limitation of that escape hatch, worth writing down because it is
    /// surprising: the replan branch leaves `_current_goal_state` at `initial`,
    /// so if anything is queued, the *very next* tick takes the `Initial` arm
    /// again, pops that goal, and replaces the goal the replan was for. A
    /// replan for the same goal is therefore only reliable while the queue is
    /// empty.
    #[tokio::test]
    #[serial]
    async fn a_queued_goal_overrides_a_replan_for_the_same_goal() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"initial".to_spvalue())
            .await;
        StateManager::set_sp_value(
            &mut con,
            &key("current_goal_predicate"),
            &"var:pos == c".to_spvalue(),
        )
        .await;
        StateManager::set_sp_value(&mut con, &key("current_goal_id"), &"goal_one".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("replan_for_same_goal"), &true.to_spvalue())
            .await;

        let runner = spawn_runner(&manager);
        queue_incoming(&mut con, vec![(GoalPriority::Normal, "var:other == true")]).await;

        let taken_over =
            wait_for(&mut con, "current_goal_predicate", "var:other == true", 3000).await;
        runner.abort();

        assert_eq!(
            taken_over, "var:other == true",
            "if this no longer happens, the replan branch now holds the goal"
        );
        assert_ne!(text(&mut con, "current_goal_id").await, "goal_one");
    }

    /// An unrecognised goal state - a key that was never initialised, or one a
    /// dashboard wrote by hand - is recovered from rather than being fatal.
    #[tokio::test]
    #[serial]
    async fn an_unknown_goal_state_recovers_to_initial() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"nonsense".to_spvalue())
            .await;

        let runner = spawn_runner(&manager);
        let seen = wait_for(&mut con, "current_goal_state", "initial", 3000).await;
        runner.abort();

        assert_eq!(seen, "initial");
    }

    /// With nothing queued and nothing running, the runner has to be silent -
    /// this is the state a deployment spends most of its time in. The
    /// "unchanged queue produces no write" property is the one the id-stability
    /// fix was about, and here it is end to end.
    #[tokio::test]
    #[serial]
    async fn an_idle_runner_with_a_queue_still_writes_nothing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"executing".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("plan_state"), &"executing".to_spvalue()).await;

        let runner = spawn_runner(&manager);
        // Leave a few goals sitting in the queue.
        queue_incoming(
            &mut con,
            vec![
                (GoalPriority::Normal, "var:a == true"),
                (GoalPriority::Low, "var:b == true"),
                (GoalPriority::High, "var:c == true"),
            ],
        )
        .await;
        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        while std::time::Instant::now() < deadline && scheduled(&mut con).await.len() < 3 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;

        let before = StateManager::get_full_state(&mut con).await.unwrap();
        tokio::time::sleep(Duration::from_millis(400)).await;
        let after = StateManager::get_full_state(&mut con).await.unwrap();

        assert!(!runner.is_finished());
        runner.abort();
        assert!(
            before.get_diff_partial_state(&after).state.is_empty(),
            "a queue that is not changing must not be rewritten on every tick: {:?}",
            before.get_diff_partial_state(&after)
        );
    }

    /// A malformed goal sitting directly in `_scheduled_goals` (not routed
    /// through the inbox - e.g. written by hand, or left over from a version
    /// that encoded goals differently) is dropped on the tick that reads it,
    /// rather than wedging the runner or corrupting the well-formed goals
    /// around it.
    #[tokio::test]
    #[serial]
    async fn a_malformed_scheduled_goal_is_dropped() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"executing".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("plan_state"), &"executing".to_spvalue()).await;
        StateManager::set_sp_value(
            &mut con,
            &key("scheduled_goals"),
            &vec![
                goal_string_to_sp_value("good_one", &"var:good == true".to_string(), GoalPriority::Normal),
                "not a goal".to_spvalue(),
            ]
            .to_spvalue(),
        )
        .await;

        let runner = spawn_runner(&manager);
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!runner.is_finished(), "the runner must survive a malformed scheduled goal");
        runner.abort();

        let queue = scheduled(&mut con).await;
        assert_eq!(queue.len(), 1, "only the well-formed goal should survive the tick");
        assert_eq!(queue[0].predicate, "var:good == true");
        assert_eq!(queue[0].id, "good_one", "its existing id must be preserved, not regenerated");
    }

    /// Setting `_current_goal_state` straight to "cancelled" (as opposed to
    /// going through `_plan_state`, which has no working path to this arm - see
    /// `a_cancelled_plan_is_never_reported_as_a_cancelled_goal`) does reach the
    /// `GoalState::Cancelled` arm: it is logged as cancelled and the goal is
    /// released back to `initial`.
    #[tokio::test]
    #[serial]
    async fn a_directly_cancelled_goal_is_reported_and_released() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"cancelled".to_spvalue())
            .await;
        StateManager::set_sp_value(
            &mut con,
            &key("current_goal_predicate"),
            &"var:pos == c".to_spvalue(),
        )
        .await;
        StateManager::set_sp_value(&mut con, &key("current_goal_id"), &"goal_one".to_spvalue())
            .await;

        let runner = spawn_runner(&manager);
        // The `Cancelled` arm releases the goal to `initial` on the very same
        // tick it reports it, so - as in
        // `a_cancelled_plan_is_never_reported_as_a_cancelled_goal` - the
        // information line has to be watched as it changes rather than read
        // once after the fact.
        let mut seen_information: Vec<String> = vec![];
        let deadline = std::time::Instant::now() + Duration::from_millis(1000);
        while std::time::Instant::now() < deadline {
            let information = text(&mut con, "goal_runner_information").await;
            if seen_information.last() != Some(&information) {
                seen_information.push(information);
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let seen = text(&mut con, "current_goal_state").await;
        runner.abort();

        assert_eq!(seen, "initial", "the goal must be released");
        assert!(
            seen_information.iter().any(|i| i.contains("cancelled") && i.contains("goal_one")),
            "unlike the plan_state route, this path must report the cancellation: {seen_information:?}"
        );
    }

    /// A malformed goal in the inbox is dropped without taking the runner down
    /// or poisoning the queue.
    #[tokio::test]
    #[serial]
    async fn a_malformed_incoming_goal_is_dropped() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;
        StateManager::set_sp_value(&mut con, &key("current_goal_state"), &"executing".to_spvalue())
            .await;
        StateManager::set_sp_value(&mut con, &key("plan_state"), &"executing".to_spvalue()).await;

        let runner = spawn_runner(&manager);
        StateManager::set_sp_value(
            &mut con,
            &key("incoming_goals"),
            &vec![
                "not a goal".to_spvalue(),
                goal_string_to_sp_value("", &"var:good == true".to_string(), GoalPriority::Normal),
            ]
            .to_spvalue(),
        )
        .await;

        let deadline = std::time::Instant::now() + Duration::from_millis(3000);
        while std::time::Instant::now() < deadline && scheduled(&mut con).await.is_empty() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!runner.is_finished(), "the runner must survive bad input");
        runner.abort();

        let queue = scheduled(&mut con).await;
        assert_eq!(queue.len(), 1, "only the well-formed goal is admitted");
        assert_eq!(queue[0].predicate, "var:good == true");
    }
}
