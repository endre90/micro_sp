use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

// When a goal appears on the goal to be queued variable, this task takes it
// and puts in the queue of goals to be executed. It also clears the variable so that
// new goals can arrive.
pub async fn goal_queuer(
    name: &str,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(3000));

    log::info!(target: &&format!("{}_goal_queuer", name), "Online.");
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
                    log::info!(target: &&format!("{}_goal_queuer", name), "New goal arrived: {}.", goal.to_string());
                    (
                        goal.clone(),
                        GoalPriority::from_str(&priority.to_string()).to_int(),
                    )
                })
                .collect::<Vec<(SPValue, i64)>>(),
            _ => {
                log::error!(target: &&format!("{}_goal_queuer", name), "Type of incoming_goals has to be a Map(goal, prio).");
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
                    .map(|(goal, priority)| {
                        (
                            goal.clone(),
                            GoalPriority::from_str(&priority.to_string()).to_int(),
                        )
                    })
                    .collect::<Vec<(SPValue, i64)>>(),
                _ => {
                    log::error!(target: &&format!("{}_goal_queuer", name), "Type of scheduled_goals has to be a Map(goal, prio).");
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
