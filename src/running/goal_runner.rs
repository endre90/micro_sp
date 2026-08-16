use crate::*;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

/// Override with `MICRO_SP_GOAL_TICK_MS`. See `running::tick`.
static TICK_INTERVAL: u64 = 1; // millis

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
    Top, // Useful to schedule housekeeping for example every 5 minutes
    High,
    Normal,
    Low,
}

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Goal {
    pub id: String, // use nanoid(10)
    pub priority: GoalPriority,
    pub predicate: String,
}

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

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalState {
    // Empty,
    Initial,
    Executing,
    // Paused,
    Failed,
    Cancelled,
    Completed,
    UNKNOWN,
}

impl GoalPriority {
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

    pub fn to_int(&self) -> i64 {
        match self {
            GoalPriority::Top => 0,
            GoalPriority::High => 1,
            GoalPriority::Normal => 2,
            GoalPriority::Low => 3,
        }
    }

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
    pub fn from_str(x: &str) -> GoalState {
        match x {
            // "empty" => CurrentGoalState::Empty,
            "initial" => GoalState::Initial,
            "executing" => GoalState::Executing,
            "failed" => GoalState::Failed,
            // "paused" => CurrentGoalState::Paused,
            "cancelled" => GoalState::Cancelled,
            "completed" => GoalState::Completed,
            "unknown" => GoalState::UNKNOWN,
            _ => {
                // log::error!(target: &&format!("goal_priority"),
                //     "Unknown goal state {}, defaulting to empty.", x);
                GoalState::UNKNOWN
            }
        }
    }
    pub fn to_spvalue(self) -> SPValue {
        self.to_string().to_spvalue()
    }
}

impl fmt::Display for GoalState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // GoalState::Empty => write!(f, "empty"),
            GoalState::Initial => write!(f, "initial"),
            GoalState::Executing => write!(f, "executing"),
            GoalState::Cancelled => write!(f, "cancelled"),
            // GoalState::Paused => write!(f, "paused"),
            GoalState::Failed => write!(f, "failed"),
            GoalState::Completed => write!(f, "completed"),
            GoalState::UNKNOWN => write!(f, "unknown"),
        }
    }
}

/// Merge newly arrived goals into the queue, ordered by priority.
///
/// An id is assigned exactly once - to a goal as it is admitted from
/// `_incoming_goals`, and to any already-queued goal that somehow has none -
/// and is never regenerated afterwards.
///
/// That stability is not cosmetic. `_scheduled_goals` is written back to Redis
/// through a diff against the previous tick, so if the ids change every tick
/// the serialised value differs every tick, the diff is never empty, and an
/// MSET goes out 10 times a second for as long as anything sits in the queue.
/// That is exactly what this used to do. Keeping ids stable means an unchanged
/// queue produces no write at all - and a goal keeps the id it was announced
/// with, instead of having it change under it while it waits.
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

// DONE: PERF: this runner used to re-generate every scheduled goal's id with
// `nanoid!` on *every* tick. The serialised `_scheduled_goals` value was
// therefore different every time, so `get_diff_partial_state` never came back
// empty and an MSET went out 10 times a second for as long as anything sat in
// the queue. Measured with three goals queued: 50 MSETs per 5 seconds, i.e. one
// on every single tick. It was also wrong - a goal's id changed under it while
// it waited, so the id logged when a goal was queued never matched the one it
// was eventually started with.
// Ids are now assigned once, at the moment a goal is admitted from
// `_incoming_goals`, which is the only point a goal actually needs one. A
// queue that is not changing now serialises to the same value and produces no
// write at all.
// DONE: PERF: `state.clone()` per tick plus a chain of up to nine `.update(..)`
// calls in the `Initial` arm, each cloning the whole state map. They are
// `update_mut` now, so the tick costs one clone instead of ten.
// DONE: PERF: `current_goal_state.to_string()` on a value that is already a
// `String` allocated a copy for nothing.
pub async fn goal_runner(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = runner_interval("MICRO_SP_GOAL_TICK_MS", TICK_INTERVAL);
    let log_target = &format!("{}_goal_runner", sp_id);

    log::info!(target: log_target, "Online.");

    // For nicer logging
    let mut goal_info_old = String::new();

    let keys: Vec<String> = vec![
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
    ];

    // PERF: one long-lived connection handle for the whole runner instead of
    // re-fetching one every tick, and no pre-flight PING before the real work.
    // `SPConnection` is cheap to clone, multiplexed and self-healing, so this
    // handle stays valid across reconnects; a dropped socket now surfaces as an
    // error on the command itself, which the callee already logs and skips.
    let mut con = connection_manager.get_connection().await;

    loop {
        interval.tick().await;
        let state = match StateManager::get_state_for_keys(&mut con, &keys, &log_target).await {
            Some(s) => s,
            None => continue,
        };

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

        let incoming_goals_sp_val =
            state.get_array_or_default_to_empty(&format!("{}_incoming_goals", sp_id), &log_target);

        let mut incoming_goals = vec![];
        for goal_sp_val in incoming_goals_sp_val {
            match sp_value_to_goal(&goal_sp_val) {
                Ok(goal) => incoming_goals.push(goal),
                Err(_) => (),
            }
        }

        if goal_info_old != goal_runner_information {
            if goal_runner_information != "UNKNOWN".to_string() {
                log::info!(target: &format!("{}_goal_runner", sp_id), "{goal_runner_information}");
            }

            goal_info_old = goal_runner_information.clone()
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
        new_state.update_mut(
            &format!("{}_incoming_goals", sp_id),
            Vec::<SPValue>::new().to_spvalue(),
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
        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
    }
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
