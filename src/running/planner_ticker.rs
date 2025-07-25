use crate::*;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::
    // sync::RwLock,
    time::{Duration, interval}
;

pub async fn planner_ticker(
    sp_id: &str,
    model: &Model,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(500));
    let log_target = &format!("{}_planner", sp_id);

    log::info!(target: log_target, "Online.");

    // Get only the relevant keys from the state
    log::info!(target: &format!("{}_operation_runner", sp_id), "Online.");
    let mut keys: Vec<String> = model
        .operations
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    // We also need some of the planner vars
    keys.extend(vec![
        format!("{}_planner_information", sp_id),
        format!("{}_planner_state", sp_id),
        format!("{}_plan_state", sp_id),
        format!("{}_plan_current_step", sp_id),
        format!("{}_plan", sp_id),
        format!("{}_replan_trigger", sp_id),
        format!("{}_replanned", sp_id),
        format!("{}_plan_counter", sp_id),
        format!("{}_replan_counter", sp_id),
        format!("{}_replan_counter_total", sp_id),
        format!("{}_current_goal_state", sp_id),
        format!("{}_current_goal_predicate", sp_id),
    ]);

    // And the operation names
    keys.extend(
        model
            .operations
            .iter()
            .flat_map(|op| vec![format!("{}", op.name)])
            .collect::<Vec<String>>(),
    );

    // let last_known_state: Arc<RwLock<Option<State>>> = Arc::new(RwLock::new(None));

    // loop {
    //     interval.tick().await;
    //     let mut con = connection_manager.get_connection().await;

    //     if let Err(e) = con.set::<_, _, ()>("heartbeat", "alive").await {
    //         handle_redis_error(&e, &log_target, connection_manager).await;
    //         continue;
    //     }
    //     match redis_get_state_for_keys(&mut con, &keys).await {
    //         Some(current_state) => {
    //             *last_known_state.write().await = Some(current_state.clone());
    //             let old_info = current_state.get_string_or_default_to_unknown(
    //                 &log_target,
    //                 &format!("{}_planner_information", sp_id),
    //             );

    //             let new_state = process_planner_tick(sp_id, &model, &current_state, &log_target);

    //             let new_info = new_state.get_string_or_default_to_unknown(
    //                 &log_target,
    //                 &format!("{}_planner_information", sp_id),
    //             );
    //             if old_info != new_info && !new_info.is_empty() {
    //                 log::info!(target: log_target, "{}", new_info);
    //             }

    //             let modified_state = current_state.get_diff_partial_state(&new_state);
    //             if !modified_state.state.is_empty() {
    //                 redis_set_state(&mut con, modified_state).await;
    //             }
    //         }
    //         None => restore_state_from_snapshot(&mut con, &last_known_state, &log_target).await,
    //     }
    // }

    loop {
        interval.tick().await;
        let mut con = connection_manager.get_connection().await;

        if let Err(e) = con.set::<_, _, ()>("heartbeat", "alive").await {
            handle_redis_error(&e, &log_target, connection_manager).await;
            continue;
        }
        let state = match redis_get_state_for_keys(&mut con, &keys).await {
            Some(s) => s,
            None => continue,
        };
        let old_info = state.get_string_or_default_to_unknown(
            &log_target,
            &format!("{}_planner_information", sp_id),
        );

        let new_state = process_planner_tick(sp_id, &model, &state, &log_target);

        let new_info = new_state.get_string_or_default_to_unknown(
            &log_target,
            &format!("{}_planner_information", sp_id),
        );
        if old_info != new_info && !new_info.is_empty() {
            log::info!(target: log_target, "{}", new_info);
        }

        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            redis_set_state(&mut con, modified_state).await;
        }
    }
}

struct PlannerContext {
    replan_trigger: bool,
    replanned: bool,
    plan_counter: i64,
    replan_counter: i64,
    replan_counter_total: i64,
    planner_state: String,
    plan: Vec<String>,
    planner_information: String,
}

fn process_planner_tick(sp_id: &str, model: &Model, state: &State, log_target: &str) -> State {
    let mut ctx = PlannerContext {
        replan_trigger: state.get_bool_or_default_to_false(
            &log_target,
            &format!("{}_replan_trigger", sp_id),
        ),
        replanned: state.get_bool_or_default_to_false(
            &log_target,
            &format!("{}_replanned", sp_id),
        ),
        plan_counter: state.get_int_or_default_to_zero(
            &log_target,
            &format!("{}_plan_counter", sp_id),
        ),
        replan_counter: state.get_int_or_default_to_zero(
            &log_target,
            &format!("{}_replan_counter", sp_id),
        ),
        replan_counter_total: state.get_int_or_default_to_zero(
            &log_target,
            &format!("{}_replan_counter_total", sp_id),
        ),
        planner_state: state.get_string_or_default_to_unknown(
            &log_target,
            &format!("{}_planner_state", sp_id),
        ),
        plan: state
            .get_array_or_default_to_empty(
                &log_target,
                &format!("{}_plan", sp_id),
            )
            .iter()
            .filter(|val| val.is_string())
            .map(|y| y.to_string())
            .collect(),
        planner_information: state.get_string_or_default_to_unknown(
            &log_target,
            &format!("{}_planner_information", sp_id),
        ),
    };

    let mut new_state = state.clone();

    if !ctx.replan_trigger {
        ctx.planner_information = "Planner is not triggered".to_string();
        ctx.replanned = false;
    } else if ctx.replanned {
        ctx.replan_trigger = false;
        ctx.replanned = false;
    } else {
        handle_replan_request(&sp_id, &mut ctx, &mut new_state, model, state);
    }

    new_state
        .update(
            &format!("{}_replan_trigger", sp_id),
            ctx.replan_trigger.to_spvalue(),
        )
        .update(&format!("{}_replanned", sp_id), ctx.replanned.to_spvalue())
        .update(
            &format!("{}_plan_counter", sp_id),
            ctx.plan_counter.to_spvalue(),
        )
        .update(
            &format!("{}_replan_counter", sp_id),
            ctx.replan_counter.to_spvalue(),
        )
        .update(
            &format!("{}_replan_counter_total", sp_id),
            ctx.replan_counter_total.to_spvalue(),
        )
        .update(
            &format!("{}_planner_state", sp_id),
            ctx.planner_state.to_spvalue(),
        )
        .update(&format!("{}_plan", sp_id), ctx.plan.to_spvalue())
        .update(
            &format!("{}_planner_information", sp_id),
            ctx.planner_information.to_spvalue(),
        )
}

fn handle_replan_request(
    sp_id: &str,
    ctx: &mut PlannerContext,
    new_state: &mut State,
    model: &Model,
    state: &State,
) {
    *new_state = reset_all_operations(new_state);
    ctx.plan = vec![];

    let planner_state = PlannerState::from_str(&ctx.planner_state);
    if planner_state != PlannerState::Ready {
        return;
    }

    if ctx.replan_counter >= MAX_REPLAN_RETRIES {
        ctx.planner_information = "Max allowed replan retries reached.".to_string();
        ctx.replan_trigger = false;
        return;
    }

    ctx.replan_counter += 1;
    ctx.replan_counter_total += 1;

    let goal = state.extract_goal(&sp_id);
    let plan_result = bfs_operation_planner(state.clone(), goal, model.operations.clone(), 20);

    if !plan_result.found {
        ctx.planner_information = format!(
            "Planner triggered (try {}/{}): No plan was found.",
            ctx.replan_counter, MAX_REPLAN_RETRIES
        );
        ctx.planner_state = PlannerState::NotFound.to_string();
    } else {
        ctx.planner_information = "Planning completed.".to_string();
        ctx.planner_state = PlannerState::Found.to_string();
        ctx.replan_counter = 0;

        if plan_result.length > 0 {
            ctx.replanned = true;
            ctx.plan_counter += 1;
            ctx.plan = plan_result.plan;
            ctx.planner_information = format!(
                "Got a new plan:\n{}",
                ctx.plan
                    .iter()
                    .enumerate()
                    .map(|(index, step)| format!("       {} -> {}", index + 1, step))
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        } else {
            ctx.planner_information = "We are already in the goal. No action needed.".to_string();
        }
    }
}

// OLD< WORKING
// /// An operation planner is an algorithm which given a planning problem Ψ,
// /// returns a sequence of operations that takes the system from its current
// /// state to a state where the goal predicate is satisfied. While planning,
// /// the operation planner is avoiding the running guards gr and the running
// /// actions Ar, treating operation preconditions and postconditions as
// /// planning transitions. This function triggers the planner based on the
// /// current state of the system.
// pub async fn planner_ticker(
//     sp_id: &str,
//     model: &Model,
//     command_sender: mpsc::Sender<StateManagement>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut interval = interval(Duration::from_millis(100));
//     let model = model.clone();

//     // For nicer logging
//     // let mut plan_current_step_old = 0;
//     let mut planner_information_old = "".to_string();
//     let mut plan_old: Vec<String> = vec![];

//     log::info!(target: &&log_target, "Online.");

//     loop {
//         let (response_tx, response_rx) = oneshot::channel();
//         command_sender
//             .send(StateManagement::GetState(response_tx))
//             .await?;
//         let mut state = response_rx.await?;
//         let mut replan_trigger = state.get_bool_or_default_to_false(
//             &log_target,
//             &format!("{}_replan_trigger", sp_id),
//         );
//         let mut replanned = state.get_bool_or_default_to_false(
//             &log_target,
//             &format!("{}_replanned", sp_id),
//         );
//         let mut plan_counter = state.get_int_or_default_to_zero(
//             &log_target,
//             &format!("{}_plan_counter", sp_id),
//         );
//         let mut replan_counter = state.get_int_or_default_to_zero(
//             &log_target,
//             &format!("{}_replan_counter", sp_id),
//         );
//         let mut replan_counter_total = state.get_int_or_default_to_zero(
//             &log_target,
//             &format!("{}_replan_counter_total", sp_id),
//         );
//         let mut plan_state = state.get_string_or_default_to_unknown(
//             &log_target,
//             &format!("{}_plan_state", sp_id),
//         );
//         let mut planner_state = state.get_string_or_default_to_unknown(
//             &log_target,
//             &format!("{}_planner_state", sp_id),
//         );
//         let mut plan_current_step = state.get_int_or_default_to_zero(
//             &log_target,
//             &format!("{}_plan_current_step", sp_id),
//         );
//         let plan_of_sp_values = state.get_array_or_default_to_empty(
//             &log_target,
//             &format!("{}_plan", sp_id),
//         );

//         let mut plan: Vec<String> = plan_of_sp_values
//             .iter()
//             .filter(|val| val.is_string())
//             .map(|y| y.to_string())
//             .collect();

//         let mut planner_information = state.get_string_or_default_to_unknown(
//             &log_target,
//             &format!("{}_planner_information", sp_id),
//         );

//         // Log only when something changes and not every tick
//         // if plan_current_step_old != plan_current_step {
//         //     log::info!(target: &log_target, "Plan current step: {plan_current_step}.");
//         //     plan_current_step_old = plan_current_step
//         // }

//         if planner_information_old != planner_information {
//             log::info!(target: &log_target, "{planner_information}");
//             planner_information_old = planner_information.clone()
//         }

//         match (replan_trigger, replanned) {
//             (true, true) => {
//                 replan_trigger = false;
//                 replanned = false;
//             }
//             (true, false) => {
//             plan_current_step = 0;
//             plan = vec!();
//             plan_state = PlanState::Initial.to_string();
//             state = reset_all_operations(&state);
//                 match PlannerState::from_str(&planner_state) {
//                     PlannerState::Found => {
//                         // plan_state = PlanState::Initial.to_string();
//                         // Waiting for the operation runner to reset state back to ready
//                     }
//                     PlannerState::NotFound => {
//                         // plan_state = PlanState::Initial.to_string();
//                         // Waiting for the operation runner to reset state back to ready
//                     }
//                     PlannerState::Ready => {
//                         if replan_counter < MAX_REPLAN_RETRIES {
//                             let goal = state.extract_goal(sp_id);
//                             replan_counter = replan_counter + 1;
//                             replan_counter_total = replan_counter_total + 1;
//                             let state_clone = state.clone();
//                             log::info!(target: &log_target, "Starting to plan.");
//                             let new_plan = bfs_operation_planner(
//                                 state_clone,
//                                 goal,
//                                 model.operations.clone(),
//                                 20,
//                             );
//                             if !new_plan.found {
//                                 log::info!(target: &log_target, "Planner triggered (try {replan_counter}/{MAX_REPLAN_RETRIES}): No plan was found.");
//                                 planner_state = PlannerState::NotFound.to_string();
//                             } else {
//                                 log::info!(target: &log_target, "Planning completed.");
//                                 planner_state = PlannerState::Found.to_string();
//                                 replan_counter = 0;
//                                 if new_plan.length == 0 {
//                                     log::info!(target: &log_target, "Planner triggered (try {replan_counter}/{MAX_REPLAN_RETRIES}): We are already in the goal, no action will be taken.");
//                                 } else {
//                                     log::info!(target: &log_target, "Planning completed (try {replan_counter}/{MAX_REPLAN_RETRIES}). A new plan was found.");

//                                     plan = new_plan.plan;
//                                     replanned = true;
//                                     plan_counter = plan_counter + 1;
//                                     if plan_old != plan {
//                                         planner_information = format!("Got a plan:\n{}",
//                                             plan.iter()
//                                                 .enumerate()
//                                                 .map(|(index, step)| format!("       {} -> {}", index + 1, step))
//                                                 .collect::<Vec<String>>()
//                                                 .join("\n")
//                                         );
//                                         plan_old = plan.clone()
//                                     }
//                                 }
//                             }
//                         } else {
//                             // planner_state = PlannerState::NotFound.to_string();
//                             planner_information = "Max allowed replan retries reached.".to_string();
//                             replan_trigger = false;
//                             replanned = false;
//                         }
//                     }

//                     PlannerState::UNKNOWN => {
//                         // plan = vec!();
//                         planner_state = PlannerState::Ready.to_string();
//                     }
//                 }
//             }

//             (false, _) => {
//                 planner_information = "Planner is not triggered".to_string();
//                 replanned = false;
//             }
//         };

//         // Instead of doing this, maybe just directly change the state with individual messages?
//         let new_state = state
//             .update(
//                 &format!("{}_replan_trigger", sp_id),
//                 replan_trigger.to_spvalue(),
//             )
//             .update(&format!("{}_replanned", sp_id), replanned.to_spvalue())
//             .update(
//                 &format!("{}_plan_counter", sp_id),
//                 plan_counter.to_spvalue(),
//             )
//             .update(
//                 &format!("{}_replan_counter", sp_id),
//                 replan_counter.to_spvalue(),
//             )
//             .update(
//                 &format!("{}_planner_state", sp_id),
//                 planner_state.to_spvalue(),
//             )
//             .update(
//                 &format!("{}_plan_state", sp_id),
//                 plan_state.to_spvalue(),
//             )
//             .update(&format!("{}_plan", sp_id), plan.to_spvalue())
//             .update(
//                 &format!("{}_planner_information", sp_id),
//                 planner_information.to_spvalue(),
//             )
//             .update(
//                 &format!("{}_replan_counter_total", sp_id),
//                 replan_counter_total.to_spvalue(),
//             )
//             .update(
//                 &format!("{}_plan_current_step", sp_id),
//                 plan_current_step.to_spvalue(),
//             );

//         let modified_state = state.get_diff_partial_state(&new_state);
//         command_sender
//             .send(StateManagement::SetPartialState(modified_state))
//             .await?;

//         interval.tick().await;
//     }
// }
