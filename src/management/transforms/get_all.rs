use rayon::prelude::*;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use std::collections::HashMap;

use crate::SPTransformStamped;

const TF_PREFIX: &str = "tf:";



pub async fn get_all_transforms(
    con: &mut MultiplexedConnection,
) -> HashMap<String, SPTransformStamped> {
    let mut con_clone = con.clone();
    let mut iter: redis::AsyncIter<String> =
        match con_clone.scan_match(&format!("{}*", TF_PREFIX)).await {
            Ok(it) => it,
            Err(e) => {
                log::error!("Redis SCAN command failed for transforms: {}", e);
                return HashMap::new();
            }
        };

    let mut keys = Vec::new();
    while let Some(key) = iter.next_item().await {
        keys.push(key);
    }

    if keys.is_empty() {
        return HashMap::new();
    }

    let values: Vec<Option<String>> = match con.mget(&keys).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("Redis MGET command failed for transforms: {}", e);
            return HashMap::new();
        }
    };

    keys.into_par_iter()
        .zip(values.into_par_iter())
        .filter_map(|(key, value_opt)| {
            let value_str = value_opt?;

            match serde_json::from_str::<SPTransformStamped>(&value_str) {
                Ok(transform) => {
                    let child_id = key.strip_prefix(TF_PREFIX).unwrap_or(&key).to_string();
                    Some((child_id, transform))
                }
                Err(e) => {
                    log::error!("Failed to deserialize transform for key '{}': {}", key, e);
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConnectionManager, MapOrUnknown, SPTransform, SPTransformStamped};
    use redis::AsyncCommands;
    use serial_test::serial;
    use std::time::SystemTime;
    use testcontainers::{core::ContainerPort, runners::AsyncRunner, ImageExt};
    use testcontainers_modules::redis::Redis;

    fn tf_key(child: &str) -> String {
        format!("{}:{}", TF_PREFIX, child)
    }

    fn create_dummy_transform(child_id: &str) -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "world".to_string(),
            child_frame_id: child_id.to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::UNKNOWN,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let transform1 = create_dummy_transform("robot1");
        let transform2 = create_dummy_transform("robot2");

        let key1 = tf_key(&transform1.child_frame_id);
        let key2 = tf_key(&transform2.child_frame_id);

        let _: () = con
            .mset(&[
                (&key1, &serde_json::to_string(&transform1).unwrap()),
                (&key2, &serde_json::to_string(&transform2).unwrap()),
            ])
            .await
            .unwrap();

        let result_map = get_all_transforms(&mut con).await;

        assert_eq!(result_map.len(), 2);
        assert_eq!(result_map.get("robot1").unwrap(), &transform1);
        assert_eq!(result_map.get("robot2").unwrap(), &transform2);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms_empty_db() {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let host_port = container.get_host_port_ipv4(6379).await.unwrap();
        let client = redis::Client::open(format!("redis://127.0.0.1:{}/", host_port)).unwrap();
        let mut con = client.get_multiplexed_async_connection().await.unwrap();

        let result_map = get_all_transforms(&mut con).await;

        assert!(result_map.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms_with_malformed_data() {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let host_port = container.get_host_port_ipv4(6379).await.unwrap();
        let client = redis::Client::open(format!("redis://127.0.0.1:{}/", host_port)).unwrap();
        let mut con = client.get_multiplexed_async_connection().await.unwrap();

        let valid_transform = create_dummy_transform("valid_robot");
        let valid_key = tf_key(&valid_transform.child_frame_id);
        let malformed_key = tf_key("malformed_robot");

        let _: () = con
            .mset(&[
                (&valid_key, &serde_json::to_string(&valid_transform).unwrap()),
                (&malformed_key, &"this is not valid json".to_string()),
            ])
            .await
            .unwrap();

        let result_map = get_all_transforms(&mut con).await;

        assert_eq!(result_map.len(), 1);
        assert!(result_map.contains_key("valid_robot"));
        assert!(!result_map.contains_key("malformed_robot"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms_with_other_data_present() {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let host_port = container.get_host_port_ipv4(6379).await.unwrap();
        let client = redis::Client::open(format!("redis://127.0.0.1:{}/", host_port)).unwrap();
        let mut con = client.get_multiplexed_async_connection().await.unwrap();

        let transform = create_dummy_transform("robot1");
        let key = tf_key(&transform.child_frame_id);

        let _: () = con.set(&key, &serde_json::to_string(&transform).unwrap()).await.unwrap();
        let _: () = con.set("some_other_key", "some_other_value").await.unwrap();

        let result_map = get_all_transforms(&mut con).await;

        assert_eq!(result_map.len(), 1);
        assert!(result_map.contains_key("robot1"));
    }
}
