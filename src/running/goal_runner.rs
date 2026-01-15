use crate::*;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use tokio::time::{Duration, interval};

static TICK_INTERVAL: u64 = 500; // millis

#[derive(Debug, PartialEq, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GoalPriority {
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

pub fn goal_predicate_to_sp_value(goal: &String, priority: GoalPriority) -> SPValue {
    let unique_id = nanoid::nanoid!(10, &NANOID_ALPHABET); // 64^10 unique ids
    let id_val = SPValue::String(StringOrUnknown::String(unique_id));
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
            GoalPriority::High => 1,
            GoalPriority::Normal => 2,
            GoalPriority::Low => 3,
        }
    }

    pub fn from_str(x: &str) -> GoalPriority {
        match x {
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

pub async fn goal_runner(
    sp_id: &str,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    initialize_env_logger();
    let mut interval = interval(Duration::from_millis(TICK_INTERVAL));
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
        format!("{}_replan_trigger", sp_id),
        format!("{}_replanned", sp_id),
        format!("{}_plan_current_step", sp_id),
    ];

    loop {
        interval.tick().await;
        if let Err(_) = connection_manager.check_redis_health(&log_target).await {
            continue;
        }
        let mut con = connection_manager.get_connection().await;
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

        let plan_state =
            state.get_string_or_default_to_unknown(&format!("{}_plan_state", sp_id), &log_target);

        // let mut planner_state =
            // state.get_string_or_default_to_unknown(&format!("{}_planner_state", sp_id), &log_target);

        // let plan_of_sp_values =
            // state.get_array_or_default_to_empty(&format!("{}_plan", sp_id), &log_target);

        // let mut plan: Vec<String> = plan_of_sp_values
        //     .iter()
        //     .filter(|val| val.is_string())
        //     .map(|y| y.to_string())
        //     .collect();

        // Should be array of arrays Array(Goal1(array(id, prio, pred), Goal2(Array(id, prio, pred))))))
        let scheduled_goals_sp_val =
            state.get_array_or_default_to_empty(&format!("{}_scheduled_goals", sp_id), &log_target);

        let mut scheduled_goals = vec![];
        for goal_sp_val in scheduled_goals_sp_val {
            match sp_value_to_goal(&goal_sp_val) {
                Ok(goal) => scheduled_goals.push(goal),
                Err(_) => (),
            }
        }

        if goal_info_old != goal_runner_information {
            log::info!(target: &format!("{}_goal_runner", sp_id), "{goal_runner_information}");
            goal_info_old = goal_runner_information.clone()
        }

        let mut new_state = state.clone();

        // Can't check if scheduled goals is empty because it can be and we still have one goal executing
        match GoalState::from_str(&current_goal_state.to_string()) {
            GoalState::Initial => {
                if !scheduled_goals.is_empty() {
                    match scheduled_goals.split_first() {
                        Some((current, rest)) => {
                            let rest_of_the_goals: Vec<SPValue> =
                                rest.iter().map(|x| goal_to_sp_value(x)).collect();
                            goal_runner_information = format!(
                                "Got new goal '{}' with id: '{}'.",
                                current.predicate, current.id
                            );
                            new_state = new_state
                                .update(
                                    &format!("{}_scheduled_goals", sp_id),
                                    rest_of_the_goals.to_spvalue(),
                                )
                                .update(
                                    &format!("{}_current_goal_id", sp_id),
                                    current.id.to_string().to_spvalue(),
                                )
                                .update(
                                    &format!("{}_current_goal_state", sp_id),
                                    GoalState::Executing.to_string().to_spvalue(),
                                )
                                .update(
                                    &format!("{}_current_goal_predicate", sp_id),
                                    current.predicate.to_string().to_spvalue(),
                                )
                                .update(&format!("{}_replan_trigger", sp_id), true.to_spvalue())
                                .update(&format!("{}_replanned", sp_id), false.to_spvalue())
                                .update(&format!("{}_plan_current_step", sp_id), 0.to_spvalue())
                                .update(
                                    &format!("{}_plan", sp_id),
                                    Vec::<String>::new().to_spvalue(),
                                )
                                .update(&format!("{}_plan_state", sp_id), "initial".to_spvalue())
                                .update(&format!("{}_planner_state", sp_id), "ready".to_spvalue())
                        }
                        None => log::error!(target: log_target, "This shouldn't happen."),
                    }
                } else {
                    goal_runner_information = "No goals scheduled, list is empty.".to_string();
                }
            }

            GoalState::Executing => {
                goal_runner_information = "Goal is executing.".to_string();
            match PlanState::from_str(&plan_state) {
                PlanState::Initial => (),
                PlanState::Executing => (),
                PlanState::Failed => {
                    new_state = new_state
                    .update(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::Failed.to_string().to_spvalue(),
                    )
                },
                PlanState::Completed => {
                    new_state = new_state
                    .update(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::Completed.to_string().to_spvalue(),
                    )
                },
                PlanState::Cancelled => {
                    new_state = new_state
                    .update(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::Cancelled.to_string().to_spvalue(),
                    )
                },
                PlanState::UNKNOWN => {
                    new_state = new_state
                    .update(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::UNKNOWN.to_string().to_spvalue(),
                    )
                },
            }
            }
            GoalState::Failed
            | GoalState::Completed
            | GoalState::Cancelled
            | GoalState::UNKNOWN => {
                goal_runner_information = "Goal is terminated.".to_string();
                new_state = new_state
                    // .update(
                    //     &format!("{}_current_goal_id", sp_id),
                    //     "".to_string().to_spvalue(),
                    // )
                    .update(
                        &format!("{}_current_goal_state", sp_id),
                        GoalState::Initial.to_string().to_spvalue(),
                    )
                    // .update(
                        // &format!("{}_current_goal_predicate", sp_id),
                        // "".to_string().to_spvalue(),
                    // )
            }
        }
        new_state = new_state.update(
            &format!("{}_goal_runner_information", sp_id),
            goal_runner_information.to_string().to_spvalue(),
        );
        let modified_state = state.get_diff_partial_state(&new_state);
        StateManager::set_state(&mut con, &modified_state).await;
    }
}
