use crate::*;
use rayon::prelude::*;
use redis::AsyncCommands;
use redis::{aio::MultiplexedConnection, pipe};
use std::collections::HashMap;

const TRANSFORM_INDEX_KEY: &str = "transforms_index";

pub struct TransformsManager {}

impl TransformsManager {
        pub async fn insert_transform(
        con: &mut MultiplexedConnection,
        key: &str,
        transform: SPTransformStamped,
    ) {
        let sp_value = SPValue::Transform(TransformOrUnknown::Transform(transform));
        let value_str = match serde_json::to_string(&sp_value) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to serialize transform for key '{key}': {e}");
                return;
            }
        };

        let result: redis::RedisResult<()> = pipe()
            .atomic()
            .set(key, value_str)
            .sadd(TRANSFORM_INDEX_KEY, key)
            .query_async(con)
            .await;

        if let Err(e) = result {
            log::error!("Failed to add transform for key '{key}': {e}");
        }
    }

    pub async fn insert_transforms(
        con: &mut MultiplexedConnection,
        transforms: HashMap<String, SPTransformStamped>,
    ) {
        if transforms.is_empty() {
            return;
        }

        let keys: Vec<String> = transforms.keys().cloned().collect();

        let mset_values: Vec<(String, String)> = match transforms
            .into_iter()
            .map(|(key, transform)| {
                let sp_value = SPValue::Transform(TransformOrUnknown::Transform(transform));
                serde_json::to_string(&sp_value).map(|json_val| (key, json_val))
            })
            .collect()
        {
            Ok(vals) => vals,
            Err(e) => {
                log::error!("Failed to serialize one or more transforms: {e}");
                return;
            }
        };

        let result: redis::RedisResult<()> = pipe()
            .atomic()
            .mset(&mset_values)
            .sadd(TRANSFORM_INDEX_KEY, &keys)
            .query_async(con)
            .await;

        if let Err(e) = result {
            log::error!("Failed to add multiple transforms: {e}");
        }
    }

    pub async fn remove_transform(con: &mut MultiplexedConnection, key: &str) {
        let result: redis::RedisResult<()> = pipe()
            .atomic()
            .del(key)
            .srem(TRANSFORM_INDEX_KEY, key)
            .query_async(con)
            .await;

        if let Err(e) = result {
            log::error!("Failed to remove transform for key '{key}': {e}");
        }
    }

    pub async fn get_all_transforms(
        con: &mut MultiplexedConnection,
    ) -> HashMap<String, SPTransformStamped> {
        let keys: Vec<String> = match con.smembers(TRANSFORM_INDEX_KEY).await {
            Ok(k) => k,
            Err(e) => {
                log::error!("Failed to get transform keys: {e}");
                return HashMap::new();
            }
        };

        if keys.is_empty() {
            return HashMap::new();
        }

        let values: Vec<String> = match con.mget(keys.clone()).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to MGET transform values: {e}");
                return HashMap::new();
            }
        };

        let key_value_pairs: Vec<(String, String)> =
            keys.into_iter().zip(values.into_iter()).collect();

        // Use Rayon to process the data in parallel
        key_value_pairs
            .into_par_iter()
            .filter_map(
                |(key, value_str)| match serde_json::from_str::<SPValue>(&value_str) {
                    Ok(SPValue::Transform(TransformOrUnknown::Transform(transf))) => {
                        Some((key, transf))
                    }
                    Ok(_) => None,
                    Err(e) => {
                        log::error!("Deserialization failed for key '{key}': {e}");
                        None
                    }
                },
            )
            .collect()
    }

    pub async fn move_transform(
        con: &mut MultiplexedConnection,
        name: &str,
        new_transform: SPTransform,
    ) {
        let redis_value: String = match con.get(name).await {
            Ok(Some(val)) => val,
            Ok(None) => {
                log::warn!("Transform '{name}' not found in Redis, cannot move.");
                return;
            }
            Err(e) => {
                log::error!("Failed to GET transform '{name}': {e}");
                return;
            }
        };

        let mut sp_tf_stamped: SPTransformStamped =
            match serde_json::from_str::<SPValue>(&redis_value) {
                Ok(SPValue::Transform(TransformOrUnknown::Transform(val))) => val,
                _ => {
                    log::error!("Value for '{name}' is not a valid transform, cannot move.");
                    return;
                }
            };

        sp_tf_stamped.transform = new_transform;

        let updated_value_json = match serde_json::to_string(&sp_tf_stamped.to_spvalue()) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to serialize updated transform '{name}': {e}");
                return;
            }
        };

        if let Err(e) = con.set::<_, _, ()>(name, updated_value_json).await {
            log::error!("Failed to SET updated transform '{name}': {e}");
        }
    }

    pub async fn reparent_transform(
        con: &mut MultiplexedConnection,
        new_parent_frame_id: &str,
        child_frame_id: &str,
    ) -> bool {
        let buffer = TransformsManager::get_all_transforms(con).await;

        let Some(original_transform) = buffer.get(child_frame_id) else {
            log::error!(
                "Can't reparent non-existent transform '{}'.",
                child_frame_id
            );
            return false;
        };

        let mut updated_transform = original_transform.clone();
        updated_transform.parent_frame_id = new_parent_frame_id.to_string();

        if check_would_produce_cycle(&updated_transform, &buffer) {
            log::error!(
                "Reparenting '{}' to '{}' would create a cycle. Aborting.",
                child_frame_id,
                new_parent_frame_id
            );
            return false;
        }

        let Some(lookup_tf) =
            lookup_transform_with_root(new_parent_frame_id, child_frame_id, "world", &buffer)
        else {
            log::error!(
                "Failed to calculate the new transform from '{}' to '{}'.",
                new_parent_frame_id,
                child_frame_id
            );
            return false;
        };

        updated_transform.transform = lookup_tf.transform;
        let updated_value_json = match serde_json::to_string(&updated_transform.to_spvalue()) {
            Ok(s) => s,
            Err(e) => {
                log::error!(
                    "Failed to serialize reparented transform '{}': {e}",
                    child_frame_id
                );
                return false;
            }
        };

        if let Err(e) = con
            .set::<_, _, ()>(child_frame_id, updated_value_json)
            .await
        {
            log::error!(
                "Failed to SET reparented transform '{}': {e}",
                child_frame_id
            );
            return false;
        }

        log::info!(
            "Successfully reparented transform '{}' to new parent '{}'.",
            child_frame_id,
            new_parent_frame_id
        );
        true
    }

    pub async fn lookup_transform(
        con: &mut MultiplexedConnection,
        parent_frame_id: &str,
        child_frame_id: &str,
    ) -> Option<SPTransformStamped> {
        let buffer = TransformsManager::get_all_transforms(con).await;

        let root = get_tree_root(&buffer).unwrap_or_else(|| "world".to_string());

        let result = lookup_transform_with_root(parent_frame_id, child_frame_id, &root, &buffer);

        if result.is_none() {
            log::error!(
                "Couldn't lookup transform from parent '{}' to child '{}'.",
                parent_frame_id,
                child_frame_id
            );
        }

        result
    }

    pub async fn load_transform_scenario(con: &mut MultiplexedConnection, path: &str) {
        match list_frames_in_dir(&path) {
            Ok(list) => {
                let frames = load_new_scenario(&list);
                // if overlay { ??
                TransformsManager::insert_transforms(con, frames).await;
            }
            Err(_e) => (),
        }
    }
}