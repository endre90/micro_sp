use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

/// An operation planner is an algorithm which given a planning problem Ψ,
/// returns a sequence of operations that takes the system from its current
/// state to a state where the goal predicate is satisfied. While planning,
/// the operation planner is avoiding the running guards gr and the running
/// actions Ar, treating operation preconditions and postconditions as
/// planning transitions. This function triggers the planner based on the
/// current state of the system.
pub async fn planner_ticker(
    model: &Model,
    command_sender: mpsc::Sender<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = &model.name;
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    // For nicer logging
    let mut plan_current_step_old = 0;

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender.send(Command::GetState(response_tx)).await?;
        let state = response_rx.await?;

        let mut replan_trigger = state.get_or_default_bool(
            &format!("{}_planner_ticker", name),
            &format!("{}_replan_trigger", name),
        );
        let mut replanned = state.get_or_default_bool(
            &format!("{}_planner_ticker", name),
            &format!("{}_replanned", name),
        );
        let mut plan_counter = state.get_or_default_i64(
            &format!("{}_planner_ticker", name),
            &format!("{}_plan_counter", name),
        );
        let mut replan_counter = state.get_or_default_i64(
            &format!("{}_planner_ticker", name),
            &format!("{}_replan_counter", name),
        );
        let mut replan_counter_total = state.get_or_default_i64(
            &format!("{}_planner_ticker", name),
            &format!("{}_replan_counter_total", name),
        );
        let mut plan_state = state.get_or_default_string(
            &format!("{}_planner_ticker", name),
            &format!("{}_plan_state", name),
        );
        let mut plan_current_step = state.get_or_default_i64(
            &format!("{}_planner_ticker", name),
            &format!("{}_plan_current_step", name),
        );
        let mut plan = state.get_or_default_array_of_strings(
            &format!("{}_planner_ticker", name),
            &format!("{}_plan", name),
        );

        if plan_current_step_old != plan_current_step {
            log::info!(target: &format!("{}_planner_ticker", name), "Plan current step: {plan_current_step}.");
            plan_current_step_old = plan_current_step
        }
        
        match (replan_trigger, replanned) {
            (true, true) => {
                log::info!(target: &format!("{}_planner_ticker", name), "Planner triggered and (re)planned.");
                replan_trigger = false;
                replanned = false;
            }
            (true, false) => {
                plan_current_step = 0;
                if replan_counter < MAX_REPLAN_RETRIES {
                    let goal = state.extract_goal(name);
                    replan_counter = replan_counter + 1;
                    replan_counter_total = replan_counter_total + 1;
                    log::info!(target: &format!("{name}_planner_ticker"), 
                        "Planner triggered, initiating (re)planning, try {replan_counter} out of {MAX_REPLAN_RETRIES}.");
                    let state_clone = state.clone();
                    let new_plan =
                        bfs_operation_planner(state_clone, goal, model.operations.clone(), 30);
                    if !new_plan.found {
                        log::error!(target: &format!("{}_planner_ticker", name), "No plan was found");
                        plan_state = PlanState::NotFound.to_string();
                        replan_counter = replan_counter + 1;
                    } else {
                        replan_counter = 0;
                        if new_plan.length == 0 {
                            log::info!(target: &format!("{}_planner_ticker", name), "We are already in the goal.");
                            plan_state = PlanState::Completed.to_string();
                        } else {
                            log::info!(target: &format!("{}_planner_ticker", name), "A new plan was found:");
                            log::info!(target: &format!("{}_planner_ticker", name), "Plan: {:?}", new_plan.plan);
                            plan = new_plan.plan;
                            plan_state = PlanState::Initial.to_string();
                            replanned = true;
                            plan_counter = plan_counter + 1;
                        }
                    }
                } else {
                    log::error!(target: &format!("{}_planner_ticker", name), "Max allowed replan retries reached.");
                    replan_trigger = false;
                    replanned = false;
                }
            }

            (false, _) => {
                log::info!(target: &format!("{}_planner_ticker", name), 
            "Planner is not triggered.");
                replanned = false;
            }
        };

        // Instead of doing this, maybe just directly change the state with individual messages?
        let new_state = state
            .update(
                &format!("{}_replan_trigger", name),
                replan_trigger.to_spvalue(),
            )
            .update(&format!("{}_replanned", name), replanned.to_spvalue())
            .update(&format!("{}_plan_counter", name), plan_counter.to_spvalue())
            .update(
                &format!("{}_replan_counter", name),
                replan_counter.to_spvalue(),
            )
            .update(&format!("{}_plan_state", name), plan_state.to_spvalue())
            .update(
                &format!("{}_plan_current_step", name),
                plan_current_step.to_spvalue(),
            )
            .update(&format!("{}_plan", name), plan.to_spvalue())
            .update(
                &format!("{}_replan_counter_total", name),
                replan_counter_total.to_spvalue(),
            );

        let modified_state = state.get_diff_partial_state(&new_state);
        command_sender
            .send(Command::SetPartialState(modified_state))
            .await?;

        interval.tick().await;
    }
}
