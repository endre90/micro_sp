use crate::*;
use std::sync::Arc;
use tokio::time::{Duration, interval};

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
        format!("{}_plan_id", sp_id),
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
        let old_info = state.get_string_or_default_to_unknown(
            &format!("{}_planner_information", sp_id),
            &log_target,
        );

        let new_state = process_planner_tick(sp_id, &model, &state, &log_target);

        let new_info = new_state.get_string_or_default_to_unknown(
            &format!("{}_planner_information", sp_id),
            &log_target,
        );
        if old_info != new_info && !new_info.is_empty() {
            log::info!(target: log_target, "{}", new_info);
        }

        let modified_state = state.get_diff_partial_state(&new_state);
        if !modified_state.state.is_empty() {
            StateManager::set_state(&mut con, &modified_state).await;
        }
        // let modified_state = state.get_diff_partial_state(&new_state);
        // StateManager::set_state(con, &modified_state).await;
        // StateManager::set_state(&mut con, &new_state).await;
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
    plan_id: String,
    planner_information: String,
}

fn process_planner_tick(sp_id: &str, model: &Model, state: &State, log_target: &str) -> State {
    let mut ctx = PlannerContext {
        replan_trigger: state
            .get_bool_or_default_to_false(&format!("{}_replan_trigger", sp_id), &log_target),
        replanned: state.get_bool_or_default_to_false(&format!("{}_replanned", sp_id), &log_target),
        plan_counter: state
            .get_int_or_default_to_zero(&format!("{}_plan_counter", sp_id), &log_target),
        replan_counter: state
            .get_int_or_default_to_zero(&format!("{}_replan_counter", sp_id), &log_target),
        replan_counter_total: state
            .get_int_or_default_to_zero(&format!("{}_replan_counter_total", sp_id), &log_target),
        planner_state: state
            .get_string_or_default_to_unknown(&format!("{}_planner_state", sp_id), &log_target),
        plan_id: state
            .get_string_or_default_to_unknown(&format!("{}_plan_id", sp_id), &log_target),
        plan: state
            .get_array_or_default_to_empty(&format!("{}_plan", sp_id), &log_target)
            .iter()
            .filter(|val| val.is_string())
            .map(|y| y.to_string())
            .collect(),
        planner_information: state.get_string_or_default_to_unknown(
            &format!("{}_planner_information", sp_id),
            &log_target,
        ),
    };

    let mut new_state = state.clone();

    if !ctx.replan_trigger {
        // ctx.planner_information = "Planner is not triggered".to_string();
        ctx.replanned = false;
    } else if ctx.replanned {
        ctx.replan_trigger = false;
        ctx.replanned = false;
    } else {
        handle_replan_request(&sp_id, &mut ctx, &mut new_state, model, state, &log_target);
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
        .update(
            &format!("{}_plan_id", sp_id),
            ctx.plan_id.to_spvalue(),
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
    log_target: &str
) {
    *new_state = reset_all_operations(&new_state, &model);
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
    let plan_result = bfs_operation_planner(
        state.clone(),
        goal,
        model.operations.clone(),
        20,
        &log_target,
    );

    if !plan_result.found {
        ctx.plan_id = "".to_string();
        ctx.planner_information = format!(
            "Planner triggered (try {}/{}): No plan was found.",
            ctx.replan_counter, MAX_REPLAN_RETRIES
        );
        ctx.planner_state = PlannerState::NotFound.to_string();
    } else {
        ctx.planner_information = "Planning completed.".to_string();
        ctx.planner_state = PlannerState::Found.to_string();
        ctx.plan_id = nanoid::nanoid!(10, &NANOID_ALPHABET);
        ctx.replan_counter = 0;

        if plan_result.length > 0 {
            ctx.replanned = true;
            ctx.plan_counter += 1;
            ctx.plan = plan_result.plan;
            ctx.planner_information = format!(
                "Got a new plan {}:\n{}",
                ctx.plan_id,
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
