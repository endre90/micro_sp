use std::time::SystemTime;

use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

pub async fn goal_runner(
    sp_id: &str,
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));

    log::info!(target: &&format!("{}_goal_runner", sp_id), "Online.");

    // For nicer logging
    let mut plan_current_step_old = 0;
    let mut planner_information_old = "".to_string();
    let mut operation_state_old = "".to_string();
    let mut operation_information_old = "".to_string();
    let mut current_goal_state_old = "".to_string();
    let mut plan_old: Vec<String> = vec![];

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::GetState(response_tx))
            .await?;
        let state = response_rx.await?;

        let mut current_goal_state = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_current_goal_state", sp_id),
        );

        let current_goal_id = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_current_goal_id", sp_id),
        );

        let plan_state = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_plan_state", sp_id),
        );

        match CurrentGoalState::from_str(&current_goal_state.to_string()) {
            CurrentGoalState::Empty => {
                log::info!(target: &&format!("{}_goal_runner", sp_id), 
                        "Current goal state is Empty.");
                // Load the first goal from the schedule to be executed
                // remove it from the schedule and move up the goals
                let scheduled_goals = state.get_map_or_default_to_empty(
                    &format!("{}_goal_runner", sp_id),
                    &format!("{}_scheduled_goals", sp_id),
                );
                if let Some((first_goal_id, rest_of_the_goals)) = scheduled_goals.split_first() {
                    let current_goal_id = first_goal_id.0.to_string();
                    let current_goal_predicate = state.get_string_or_default_to_unknown(
                        &format!("{}_goal_runner", sp_id),
                        &format!("{}_current_goal_predicate", sp_id),
                    );
                    let current_goal_state = CurrentGoalState::Initial;

                    let new_state = state
                        .update(
                            &format!("{}_current_goal_id", sp_id),
                            current_goal_id.to_spvalue(),
                        )
                        .update(
                            &format!("{}_current_goal_predicate", sp_id),
                            current_goal_predicate.to_spvalue(),
                        )
                        .update(
                            &format!("{}_current_goal_state", sp_id),
                            current_goal_state.to_spvalue(),
                        )
                        .update(
                            &format!("{}_scheduled_goals", sp_id),
                            SPValue::Map(MapOrUnknown::Map(rest_of_the_goals.to_vec())),
                        );

                    let modified_state = state.get_diff_partial_state(&new_state);
                    command_sender
                        .send(StateManagement::SetPartialState(modified_state))
                        .await?;
                }
            }
            CurrentGoalState::Initial => {
                log::info!(target: &&format!("{}_goal_runner", sp_id), 
                    "Initializing goal: {}.", current_goal_id);

                let current_goal_state = CurrentGoalState::Executing;

                let new_state = state
                    .update(&format!("{}_replan_trigger", sp_id), true.to_spvalue())
                    .update(&format!("{}_replanned", sp_id), false.to_spvalue())
                    .update(&format!("{}_plan_current_step", sp_id), 0.to_spvalue())
                    .update(
                        &format!("{}_current_goal_state", sp_id),
                        current_goal_state.to_spvalue(),
                    );

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(StateManagement::SetPartialState(modified_state))
                    .await?;
            }
            CurrentGoalState::Executing => {
                match PlanState::from_str(&plan_state) {
                    PlanState::Failed => {
                        // For now we fail, but maybe later we can don something about this
                        // Like change something so that the goal remains the same but we try to find
                        // another plan that doesn't fail

                        // TODO: store this goal in a failed goals archive and continue with the schedule

                        let current_goal_state = CurrentGoalState::Failed;

                        let new_state = state.update(
                            &format!("{}_current_goal_state", sp_id),
                            current_goal_state.to_spvalue(),
                        );

                        let modified_state = state.get_diff_partial_state(&new_state);
                        command_sender
                            .send(StateManagement::SetPartialState(modified_state))
                            .await?;
                    }
                    PlanState::NotFound => {
                        // For now we fail, but maybe later we can don something about this
                        // Like change something so that the goal remains the same but we try to find
                        // another plan that doesn't fail

                        // TODO: store this goal in a failed goals ARCHIVE and continue with the schedule
                        // Also mention the reason, log time, etc. Maybe operator can do this instead.

                        let current_goal_state = CurrentGoalState::Failed;

                        let new_state = state.update(
                            &format!("{}_current_goal_state", sp_id),
                            current_goal_state.to_spvalue(),
                        );

                        let modified_state = state.get_diff_partial_state(&new_state);
                        command_sender
                            .send(StateManagement::SetPartialState(modified_state))
                            .await?;
                    }
                    PlanState::Completed => {
                        let current_goal_state = CurrentGoalState::Completed;

                        let new_state = state.update(
                            &format!("{}_current_goal_state", sp_id),
                            current_goal_state.to_spvalue(),
                        );

                        let modified_state = state.get_diff_partial_state(&new_state);
                        command_sender
                            .send(StateManagement::SetPartialState(modified_state))
                            .await?;
                    },
                    _ => (),
                }
            }
            CurrentGoalState::Paused => {
                log::warn!(target: &&format!("{}_goal_runner", sp_id), 
                    "The goal runner is paused.");
            }
            CurrentGoalState::Failed => {
                log::info!(target: &&format!("{}_goal_runner", sp_id), "Goal failed.");
                                // Remove from the list and move on to the next one in the queue.
                // Maybe safe the goal in the pile of failed/cancelled/notfound goals
            }
            CurrentGoalState::Cancelled => {
                log::info!(target: &&format!("{}_goal_runner", sp_id), "Goal cancelled.");
                // Remove from the list and move on to the next one in the queue.
                // Maybe safe the goal in the pile of failed/cancelled/notfound goals
            }
            CurrentGoalState::Completed => {
                log::info!(target: &&format!("{}_goal_runner", sp_id), "Goal completed.");
                // Remove from the list and move on to the next one in the queue.
                // Log success
            }
        }

        interval.tick().await;
    }
}
