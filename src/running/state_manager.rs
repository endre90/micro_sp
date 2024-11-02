use tokio::sync::{mpsc, oneshot};

use crate::*;

pub enum Command {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<SPValue>)),
    SetPartialState(State),
    Set((String, SPValue)),
}

pub async fn state_manager(mut receiver: mpsc::Receiver<Command>, mut state: State) {
    while let Some(command) = receiver.recv().await {
        match command {
            Command::GetState(response_sender) => {
                let _ = response_sender.send(state.clone());
            }
            Command::Get((var, response_sender)) => {
                let _ = response_sender.send(state.get_value(&var));
            }
            Command::SetPartialState(partial_state) => {
                for (var, assignment) in partial_state.state {
                    state = state.update(&var, assignment.val)
                }
            }
            Command::Set((var, new_val)) => {
                state = state.update(&var, new_val);
            }
        }
    }
}
