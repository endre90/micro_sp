use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

const TF_PREFIX: &str = "tf";

fn tf_key(child_id: &str) -> String {
    format!("{}:{}", TF_PREFIX, child_id)
}

pub(super) async fn remove_transform(con: &mut MultiplexedConnection, key: &str) {
    let redis_key = tf_key(key);
    match con.del::<_, ()>(&redis_key).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Redis DEL command for key '{}' failed: {}", redis_key, e);
        }
    }
}