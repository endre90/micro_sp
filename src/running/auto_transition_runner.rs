use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

/// Automatic transitions should be taken as soon as their guard becomes true.
pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    command_sender: mpsc::Sender<StateManagement>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();

    'initialize: loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender
            .send(StateManagement::Get((
                "state_manager_online".to_string(),
                response_tx,
            )))
            .await?;
        let state_manager_online = response_rx.await?;
        match state_manager_online {
            SPValue::Bool(BoolOrUnknown::Bool(true)) => break 'initialize,
            _ => {},
        }
        interval.tick().await;
    }

    log::info!(target: &&format!("{}_auto_runner", name), "Online.");

    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender.send(StateManagement::GetState(response_tx)).await?;
        let state = response_rx.await?;

        for t in &model.auto_transitions {
            if t.clone().eval_running(&state) {
                let new_state = t.clone().take_running(&state);
                log::info!(target: &&format!("{}_auto_runner", name), "Executed auto transition: '{}'.", t.name);

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(StateManagement::SetPartialState(modified_state))
                    .await?;
            }
        }
        interval.tick().await;
    }
}
