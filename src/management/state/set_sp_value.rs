use crate::SPValue;
use redis::{AsyncCommands, aio::MultiplexedConnection};

pub(super) async fn set_sp_value(con: &mut MultiplexedConnection, key: &str, value: &SPValue) {
    let value_str = match serde_json::to_string(value) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to serialize value for key '{key}': {e}");
            return;
        }
    };

    match con.set::<_, _, ()>(key, value_str).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Redis SET command for key '{key}' failed: {e}");
        }
    }
}
