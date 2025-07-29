use crate::State;
use redis::{AsyncCommands, Value, aio::MultiplexedConnection};

pub(super) async fn set_state(con: &mut MultiplexedConnection, state: &State) {
    let items_to_set: Vec<(String, String)> = state
        .state
        .clone()
        .into_iter()
        .filter_map(
            |(key, assignment)| match serde_json::to_string(&assignment.val) {
                Ok(value_str) => Some((key, value_str)),
                Err(e) => {
                    log::error!("Failed to serialize value for key '{key}': {e}");
                    None
                }
            },
        )
        .collect();

    if !items_to_set.is_empty() {
        match con.mset::<_, String, Value>(&items_to_set).await {
            Ok(_) => {}
            Err(e) => log::error!("Redis MSET command failed: {e}"),
        }
    }
}
