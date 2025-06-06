use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

pub async fn goal_runner(
    sp_id: &str,
    _model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));

    // For nicer logging
    let mut goal_runner_information_old = "".to_string();
    // let mut current_goal_state_old = "".to_string();

    log::info!(target: &&format!("{}_goal_runner", sp_id), "Online.");

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

        let mut goal_runner_information = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_goal_runner_information", sp_id),
        );

        let mut current_goal_id = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_current_goal_id", sp_id),
        );

        let plan_state = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_plan_state", sp_id),
        );

        let current_goal_predicate = state.get_string_or_default_to_unknown(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_current_goal_predicate", sp_id),
        );

        let scheduled_goals = state.get_map_or_default_to_empty(
            &format!("{}_goal_runner", sp_id),
            &format!("{}_scheduled_goals", sp_id),
        );

        let mut rest_of_the_goals = scheduled_goals.clone();

        if goal_runner_information_old != goal_runner_information {
            log::info!(target: &format!("{}_goal_runner", sp_id), "{goal_runner_information}");
            goal_runner_information_old = goal_runner_information.clone()
        }

        match CurrentGoalState::from_str(&current_goal_state.to_string()) {
            CurrentGoalState::Empty => {
                goal_runner_information = "Current goal state is Empty.".to_string();
                // Load the first goal from the schedule to be executed
                // remove it from the schedule and move up the goals
                match scheduled_goals.split_first() {
                    Some((first, rest)) => {
                        current_goal_id = first.0.to_string();
                        rest_of_the_goals = rest.to_vec();
                    }
                    None => (),
                }

                // if let Some((first_goal_id, rest_of_the_goals)) = scheduled_goals.split_first() {
                //     let current_goal_id = first_goal_id.0.to_string();

                //     current_goal_state = CurrentGoalState::Initial.to_string();
                // }
            }
            CurrentGoalState::Initial => {
                goal_runner_information = format!("Initializing goal: {current_goal_id}.");
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
                goal_runner_information = format!("Executing goal: {current_goal_id}.");
                match PlanState::from_str(&plan_state) {
                    PlanState::Failed => {
                        // For now we fail, but maybe later we can don something about this
                        // Like change something so that the goal remains the same but we try to find
                        // another plan that doesn't fail

                        // TODO: store this goal in a failed goals archive and continue with the schedule

                        current_goal_state = CurrentGoalState::Failed.to_string();

                        // let new_state = state.update(
                        //     &format!("{}_current_goal_state", sp_id),
                        //     current_goal_state.to_spvalue(),
                        // );

                        // let modified_state = state.get_diff_partial_state(&new_state);
                        // command_sender
                        //     .send(StateManagement::SetPartialState(modified_state))
                        //     .await?;
                    }
                    // PlanState::NotFound => {
                    //     // For now we fail, but maybe later we can don something about this
                    //     // Like change something so that the goal remains the same but we try to find
                    //     // another plan that doesn't fail

                    //     // TODO: store this goal in a failed goals ARCHIVE and continue with the schedule
                    //     // Also mention the reason, log time, etc. Maybe operator can do this instead.

                    //     current_goal_state = CurrentGoalState::Failed.to_string();

                    //     // let new_state = state.update(
                    //     //     &format!("{}_current_goal_state", sp_id),
                    //     //     current_goal_state.to_spvalue(),
                    //     // );

                    //     // let modified_state = state.get_diff_partial_state(&new_state);
                    //     // command_sender
                    //     //     .send(StateManagement::SetPartialState(modified_state))
                    //     //     .await?;
                    // }
                    PlanState::Completed => {
                        current_goal_state = CurrentGoalState::Completed.to_string();

                        // let new_state = state.update(
                        //     &format!("{}_current_goal_state", sp_id),
                        //     current_goal_state.to_spvalue(),
                        // );

                        // let modified_state = state.get_diff_partial_state(&new_state);
                        // command_sender
                        //     .send(StateManagement::SetPartialState(modified_state))
                        //     .await?;
                    }
                    _ => (),
                }
            }
            CurrentGoalState::Paused => {
                goal_runner_information = "The goal runner is paused.".to_string();
            }
            CurrentGoalState::Failed => {
                goal_runner_information = format!("Goal failed: {current_goal_id}.");
                // Remove from the list and move on to the next one in the queue.
                // Maybe safe the goal in the pile of failed/cancelled/notfound goals
            }
            CurrentGoalState::Cancelled => {
                goal_runner_information = format!("Goal cancelled: {current_goal_id}.");
                // Remove from the list and move on to the next one in the queue.
                // Maybe safe the goal in the pile of failed/cancelled/notfound goals
            }
            CurrentGoalState::Completed => {
                goal_runner_information = format!("Goal completed: {current_goal_id}.");
                // Remove from the list and move on to the next one in the queue.
                // Log success
            }
        }

        let new_state = state
            .update(
                &format!("{}_goal_runner_information", sp_id),
                goal_runner_information.to_spvalue(),
            )
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

        interval.tick().await;
    }
}
