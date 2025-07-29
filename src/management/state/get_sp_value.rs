use crate::SPValue;
use redis::{AsyncCommands, aio::MultiplexedConnection};

pub(super) async fn get_sp_value(con: &mut MultiplexedConnection, var: &str) -> Option<SPValue> {
    let redis_result: Option<String> = match con.get(var).await {
        Ok(value) => value,
        Err(e) => {
            log::error!("Failed to get '{var}' from Redis: {e}");
            return None;
        }
    };

    let Some(value_str) = redis_result else {
        return None;
    };

    match serde_json::from_str(&value_str) {
        Ok(deserialized_value) => Some(deserialized_value),
        Err(e) => {
            log::error!("Deserializing value for '{var}' failed: {e}");
            None
        }
    }
}
