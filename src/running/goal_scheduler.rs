use std::time::SystemTime;

use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

// When a goal appears on the goal to be queued variable, this task takes it
// and puts in the queue of goals to be executed. It also clears the variable so that
// new goals can arrive.
pub async fn goal_scheduler(
    name: &str, // micro_sp instance name
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(3000));

    log::info!(target: &&format!("{}_goal_scheduler", name), "Online.");
    // command_sender
    //     .send(StateManagement::Set((
    //         format!("{}_operation_runner_online", name),
    //         SPValue::Bool(BoolOrUnknown::Bool(true)),
    //     )))
    //     .await?;

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::Get((
                format!("{}_incoming_goals", name),
                response_tx,
            )))
            .await?;
        let incoming_goals = response_rx.await?;
        let incoming_goals_and_prios = match incoming_goals {
            SPValue::Map(MapOrUnknown::Map(map)) => map
                .iter()
                .map(|(goal, priority)| {
                    let goal_id = nanoid::nanoid!();
                    let goal_priority = GoalPriority::from_str(&priority.to_string());
                    log::info!(target: &&format!("{}_goal_scheduler", name), 
                        "New goal with id '{}' arrived: '{}'.", goal_id, goal.to_string());
                    let _ = add_goal_to_state(&name, &goal_id, &goal, &goal_priority, &command_sender); // Need also goal from state to remove stuff
                    (goal.clone(), goal_priority.to_int())
                })
                .collect::<Vec<(SPValue, i64)>>(),
            _ => {
                log::error!(target: &&format!("{}_goal_scheduler", name), "Type of incoming_goals has to be a Map.");
                vec![]
            }
        };
        if !incoming_goals_and_prios.is_empty() {
            let (response_tx, response_rx) = oneshot::channel();
            command_sender
                .send(StateManagement::Get((
                    format!("{}_scheduled_goals", name),
                    response_tx,
                )))
                .await?;
            let scheduled_goals = response_rx.await?;
            let mut scheduled_goals_and_prios = match scheduled_goals {
                SPValue::Map(MapOrUnknown::Map(map)) => map
                    .iter()
                    .map(|(goal_id, priority)| {
                        (
                            goal_id.clone(),
                            GoalPriority::from_str(&priority.to_string()).to_int(),
                        )
                    })
                    .collect::<Vec<(SPValue, i64)>>(),
                _ => {
                    log::error!(target: &&format!("{}_goal_scheduler", name), "Type of scheduled_goals has to be a Map.");
                    vec![]
                }
            };
            scheduled_goals_and_prios.extend(incoming_goals_and_prios);
            scheduled_goals_and_prios.sort_by_key(|(_, v)| *v); // Keeps the order of equal elements
            let new_goal_schedule = SPValue::Map(MapOrUnknown::Map(
                scheduled_goals_and_prios
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            GoalPriority::from_int(v).to_string().to_spvalue(),
                        )
                    })
                    .collect::<Vec<(SPValue, SPValue)>>(),
            ));
            command_sender
                .send(StateManagement::Set((
                    format!("{}_scheduled_goals", name),
                    new_goal_schedule,
                )))
                .await?;
            // Clear the incoming map
            command_sender
                .send(StateManagement::Set((
                    format!("{}_incoming_goals", name),
                    SPValue::Map(MapOrUnknown::Map(vec![])),
                )))
                .await?;
            continue;
        }
        interval.tick().await;
    }
}

async fn add_goal_to_state(
    name: &str, // micro_sp instance name
    id: &str, // goal_id
    predicate: &SPValue,
    priority: &GoalPriority,
    command_sender: &mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut goal_state = State::new();

    let goal_id: SPVariable = v!(&&format!("{}_goal_{}_id", name, id)); // An id to track the goal
    let goal_predicate = v!(&&format!("{}_goal_{}_predicate", name, id)); // The actual goal
    let goal_priority = v!(&&format!("{}_goal_{}_priority", name, id)); // High(1), Normal(2), or Low(3)
    let goal_time_arrived = tv!(&&format!("{}_goal_{}_time_arrived", name, id)); // When did the goal arrive
    let goal_time_started = tv!(&&format!("{}_goal_{}_time_started", name, id)); // Start of the execution time of he goal
    let goal_time_concluded = tv!(&&format!("{}_goal_{}_time_concluded", name, id)); // When was the goal concluded
    let goal_conclusion = v!(&&format!("{}_goal_{}_conclusion", name, id)); // Completed, Failed, Aborted, Timedout
    let goal_nr_of_replans = iv!(&&format!("{}_goal_{}_nr_of_replans", name, id));
    let goal_nr_of_failures = iv!(&&format!("{}_goal_{}_nr_of_failures", name, id));
    let goal_nr_of_timeouts = iv!(&&format!("{}_goal_{}_nr_of_timeouts", name, id));
    let goal_planned_paths = mv!(&&format!("{}_goal_{}_planned_paths", name, id)); // A map of (planned_path(Array), planning_duration(Time))
    let goal_log = mv!(&&format!("{}_goal_{}_log", name, id)); // A map of (goal_id(String), goal_log(Array(GoalLog)))
    let goal_execution_path = av!((&&format!("{}_goal_{}_execution_path", name, id))); // Which operations and autos did we take to reach the goal
    let goal_duration = iv!((&&format!("{}_goal_{}_duration", name, id))); // how many milliseconds did the goal take to conclude

    goal_state = goal_state.add(assign!(goal_id, SPValue::String(StringOrUnknown::UNKNOWN)));
    goal_state = goal_state.add(assign!(goal_predicate, predicate.to_owned()));
    goal_state = goal_state.add(assign!(goal_priority, priority.to_string().to_spvalue()));
    goal_state = goal_state.add(assign!(
        goal_time_arrived,
        SPValue::Time(TimeOrUnknown::Time(SystemTime::now()))
    ));
    goal_state = goal_state.add(assign!(
        goal_time_started,
        SPValue::Time(TimeOrUnknown::UNKNOWN)
    ));
    goal_state = goal_state.add(assign!(
        goal_time_concluded,
        SPValue::Time(TimeOrUnknown::UNKNOWN)
    ));
    goal_state = goal_state.add(assign!(
        goal_conclusion,
        SPValue::String(StringOrUnknown::UNKNOWN)
    ));
    goal_state = goal_state.add(assign!(
        goal_nr_of_replans,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));
    goal_state = goal_state.add(assign!(
        goal_nr_of_failures,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));
    goal_state = goal_state.add(assign!(
        goal_nr_of_timeouts,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));
    goal_state = goal_state.add(assign!(
        goal_planned_paths,
        SPValue::Map(MapOrUnknown::Map(vec!()))
    ));
    goal_state = goal_state.add(assign!(goal_log, SPValue::Map(MapOrUnknown::Map(vec!()))));
    goal_state = goal_state.add(assign!(
        goal_execution_path,
        SPValue::Array(ArrayOrUnknown::Array(vec!()))
    ));
    goal_state = goal_state.add(assign!(
        goal_duration,
        SPValue::Int64(IntOrUnknown::Int64(0))
    ));

    command_sender
        .send(StateManagement::SetPartialState(goal_state))
        .await?;

    Ok(())
}
