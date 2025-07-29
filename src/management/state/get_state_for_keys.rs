use crate::{State, StateManager};
use redis::{AsyncCommands, aio::MultiplexedConnection};

pub(super) async fn get_state_for_keys(
    con: &mut MultiplexedConnection,
    keys: &Vec<String>,
) -> Option<State> {
    if keys.is_empty() {
        return Some(State::new());
    }

    let values: Vec<Option<String>> = match con.mget(keys).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to get values from Redis: {e}");
            return None;
        }
    };

    Some(StateManager::build_state(keys.clone(), values))
}
