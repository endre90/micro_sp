use crate::{running::process_operation::OperationProcessingType, *};
use std::sync::Arc;
use tokio::{
    sync::mpsc,
    time::{Duration, interval},
};

pub static OPERAION_RUNNER_TICK_INTERVAL_MS: u64 = 200;

pub async fn planned_operation_runner(
    model: &Model,
    diagnostics_tx: mpsc::Sender<LogMsg>,
    connection_manager: &Arc<ConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sp_id = &model.name;
    let log_target = format!("{}_op_runner", sp_id);
    let mut interval = interval(Duration::from_millis(OPERAION_RUNNER_TICK_INTERVAL_MS));

    // Get only the relevant keys from the state
    log::info!(target: &log_target, "Online.");
    let mut keys: Vec<String> = model
        .operations
        .iter()
        .flat_map(|t| t.get_all_var_keys())
        .collect();

    // We also need some of the planner vars and dashboard
    keys.extend(vec![
        format!("{}_planner_state", sp_id),
        format!("{}_plan_state", sp_id),
        format!("{}_plan_current_step", sp_id),
        format!("{}_plan", sp_id),
        format!("{}_dashboard_command", sp_id),
    ]);

    // And the vars to keep trask of operation states
    keys.extend(
        model
            .operations
            .iter()
            .flat_map(|op| {
                vec![
                    format!("{}", op.name),
                    format!("{}_information", op.name),
                    format!("{}_failure_retry_counter", op.name),
                    format!("{}_timeout_retry_counter", op.name),
                    format!("{}_elapsed_executing_ms", op.name),
                    format!("{}_elapsed_disabled_ms", op.name),
                ]
            })
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
        // let con_clone = con.clone();
        let new_state =
            process_plan_tick(sp_id, &model, &state, diagnostics_tx.clone(), &log_target).await;
        let modified_state = state.get_diff_partial_state(&new_state);
        // StateManager::set_state(con, &modified_state).await;
        StateManager::set_state(&mut con, &modified_state).await;
    }
}

async fn process_plan_tick(
    sp_id: &str,
    // con: redis::aio::MultiplexedConnection,
    model: &Model,
    state: &State,
    diagnostics_tx: mpsc::Sender<LogMsg>,
    log_target: &str,
) -> State {
    let mut new_state = state.clone();
    let mut planner_state =
        state.get_string_or_default_to_unknown(&format!("{}_planner_state", sp_id), &log_target);

    let mut plan_state_str =
        state.get_string_or_default_to_unknown(&format!("{}_plan_state", sp_id), &log_target);
    let mut plan_current_step =
        state.get_int_or_default_to_zero(&format!("{}_plan_current_step", sp_id), &log_target);
    let plan_of_sp_values =
        state.get_array_or_default_to_empty(&format!("{}_plan", sp_id), &log_target);

    let plan: Vec<String> = plan_of_sp_values
        .iter()
        .filter(|val| val.is_string())
        .map(|y| y.to_string())
        .collect();

    match PlanState::from_str(&plan_state_str) {
        PlanState::Initial => {
            if planner_state == PlannerState::Found.to_string() {
                plan_state_str = PlanState::Executing.to_string();
                plan_current_step = 0;
            }
        }

        PlanState::Executing => {
            if let Some(dashboard_command) =
                    state.get_value(&format!("{}_dashboard_command", sp_id), &log_target)
                {
                    if let SPValue::String(StringOrUnknown::String(db)) = dashboard_command {
                        match db.as_str() {
                            "stop" => plan_state_str = PlanState::Cancelled.to_string(),
                            _ => (),
                        }
                    }
                }
            if let Some(op_name) = plan.get(plan_current_step as usize) {
                match model.operations.iter().find(|op| op.name == *op_name) {
                    Some(operation) => {
                        new_state = running::process_operation::process_operation(
                            new_state,
                            operation,
                            OperationProcessingType::Planned,
                            Some(&mut plan_current_step),
                            Some(&mut plan_state_str),
                            diagnostics_tx,
                            log_target,
                        )
                        .await;
                    }
                    None => {
                        log::error!("Operation '{}' not found in model!", op_name);
                        plan_state_str = PlanState::Failed.to_string();
                    }
                }
            } else {
                plan_state_str = PlanState::Completed.to_string();
            }
        }
        PlanState::Failed | PlanState::Completed | PlanState::Cancelled | PlanState::UNKNOWN => {
            plan_current_step = 0;
            new_state = reset_all_operations(&new_state, &model);
            plan_state_str = PlanState::Initial.to_string();
            planner_state = PlannerState::Ready.to_string();
        }
    }

    new_state = new_state
        .update(
            &format!("{}_plan_state", sp_id),
            plan_state_str.to_spvalue(),
        )
        .update(
            &format!("{}_planner_state", sp_id),
            planner_state.to_spvalue(),
        )
        .update(
            &format!("{}_plan_current_step", sp_id),
            plan_current_step.to_spvalue(),
        );

    new_state
}
