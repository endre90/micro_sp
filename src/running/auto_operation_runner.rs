use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

/// Automatic operations should be taken as soon as their guard becomes true.
pub async fn auto_operation_runner(
    name: &str,
    model: &Model,
    command_sender: mpsc::Sender<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();
    loop {
        let (response_tx, response_rx) = oneshot::channel();
        command_sender.send(Command::GetState(response_tx)).await?;
        let state = response_rx.await?;

        for o in &model.auto_operations {
            if o.eval_running(&state) {
                let new_state = o.start_running(&state);
                log::info!(target: &&format!("{}_auto_runner", name), "Started auto operation: '{}'.", o.name);

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(Command::SetPartialState(modified_state))
                    .await?;
            } else if o.can_be_completed(&state) {
                let new_state = o.complete_running(&state);
                log::info!(target: &&format!("{}_auto_runner", name), "Completed auto operation: '{}'.", o.name);
                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(Command::SetPartialState(modified_state))
                    .await?;
            }
        }
        interval.tick().await;
    }
}
