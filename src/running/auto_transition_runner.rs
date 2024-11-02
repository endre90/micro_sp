use crate::*;
use tokio::{
    sync::{mpsc, oneshot},
    time::{interval, Duration},
};

pub async fn auto_transition_runner(
    name: &str,
    model: &Model,
    command_sender: mpsc::Sender<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_millis(100));
    let model = model.clone();
    loop {
        // current try:
        // read the whole state, take the transition to produce a new state
        // then take the diff from the new state compared to the old state and send a request to change only those values

        let (response_tx, response_rx) = oneshot::channel();
        command_sender.send(Command::GetState(response_tx)).await?; // TODO: maybe we can just ask for values from the guard
        let state = response_rx.await?;

        // Auto transitions should be taken as soon as guard becomas true
        for t in &model.auto_transitions {
            if t.clone().eval_running(&state) {
                let new_state = t.clone().take_running(&state);
                log::info!(target: &&format!("{}_auto_runner", name), "Executed auto transition: '{}'.", t.name);

                let modified_state = state.get_diff_partial_state(&new_state);
                command_sender
                    .send(Command::SetPartialState(modified_state))
                    .await?;
            }
        }
        interval.tick().await;
    }
}
