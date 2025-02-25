use tokio::sync::{mpsc, oneshot};

use crate::*;

/// Available commands that the async tasks can ask from the state manager.
pub enum StateManagement {
    GetState(oneshot::Sender<State>),
    Get((String, oneshot::Sender<SPValue>)),
    SetPartialState(State),
    Set((String, SPValue)),
}

/// Instead of sharing the state with Arc<Mutex<State>>, use a buffer of state read/write requests.
pub async fn state_manager(mut receiver: mpsc::Receiver<StateManagement>, mut state: State) {
    while let Some(command) = receiver.recv().await {
        match command {
            StateManagement::GetState(response_sender) => {
                let _ = response_sender.send(state.clone());
            }
            StateManagement::Get((var, response_sender)) => {
                let _ = response_sender.send(state.get_value(&var));
            }
            StateManagement::SetPartialState(partial_state) => {
                for (var, assignment) in partial_state.state {
                    state = state.update(&var, assignment.val)
                }
            }
            StateManagement::Set((var, new_val)) => {
                state = state.update(&var, new_val);
            }
        }
    }
}
