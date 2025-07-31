use redis::{AsyncCommands, Value, aio::MultiplexedConnection};

use crate::SPTransformStamped;

const TF_PREFIX: &str = "tf";

fn tf_key(child: &str) -> String {
    format!("{}:{}", TF_PREFIX, child)
}

pub(super) async fn insert_transform(
    con: &mut MultiplexedConnection,
    transform: &SPTransformStamped,
) {
    let key = tf_key(&transform.child_frame_id);
    let value_str = match serde_json::to_string(transform) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to serialize transform for key '{key}': {e}");
            return;
        }
    };

    match con.set::<_, _, ()>(&key, value_str).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Redis SET command for key '{key}' failed: {e}");
        }
    }
}

pub(super) async fn insert_transforms(
    con: &mut MultiplexedConnection,
    transforms: &Vec<SPTransformStamped>,
) {
    if transforms.is_empty() {
        return;
    }

    let key_value_pairs: Vec<(String, String)> = transforms
        .into_iter()
        .filter_map(|transform| {
            let key = tf_key(&transform.child_frame_id);
            match serde_json::to_string(&transform) {
                Ok(value_str) => Some((key, value_str)),
                Err(e) => {
                    log::error!(
                        "Failed to serialize transform for child '{}': {}",
                        transform.child_frame_id,
                        e
                    );
                    None
                }
            }
        })
        .collect();

    if key_value_pairs.is_empty() {
        log::warn!("No valid transforms to set after serialization.");
        return;
    }

    match con.mset::<_, String, Value>(&key_value_pairs).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Redis MSET command for multiple transforms failed: {}", e);
        }
    }
}