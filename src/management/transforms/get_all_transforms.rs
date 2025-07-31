use futures::StreamExt;
use rayon::prelude::*;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use std::collections::HashMap;
use std::error::Error;

use crate::{SPTransformStamped, SPValue, TF_PREFIX, TransformOrUnknown};

pub(super) async fn get_all_transforms(
    con: &mut MultiplexedConnection,
) -> Result<HashMap<String, SPTransformStamped>, Box<dyn Error>> {
    let mut con_clone = con.clone();

    let iter: redis::AsyncIter<String> = con_clone.scan_match(&format!("{}*", TF_PREFIX)).await?;

    let keys: Vec<String> = iter.collect().await;

    if keys.is_empty() {
        return Err("No transforms retreived.".into());
    }

    let values: Vec<Option<String>> = con.mget(&keys).await?;
    let transform_map = keys
        .into_par_iter()
        .zip(values.into_par_iter())
        .filter_map(|(key, value_opt)| {
            let value_str = value_opt?;

            match serde_json::from_str::<SPValue>(&value_str) {
                Ok(SPValue::Transform(TransformOrUnknown::Transform(transform))) => {
                    let child_id = key.strip_prefix(TF_PREFIX).unwrap_or(&key).to_string();
                    Some((child_id, transform))
                }
                Ok(_) => {
                    log::warn!("Key '{}' held an SPValue that was not a Transform.", key);
                    None
                }
                Err(e) => {
                    log::error!("Failed to deserialize SPValue for key '{}': {}", key, e);
                    None
                }
            }
        })
        .collect();

    Ok(transform_map)
}

#[cfg(test)]
mod tests {
    use super::{TF_PREFIX, get_all_transforms};
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use std::time::SystemTime;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

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

        let mut tf1 = create_dummy_transform("tf1");
        tf1.transform.translation.x = ordered_float::OrderedFloat(1.0);
        tf1.transform.translation.y = ordered_float::OrderedFloat(2.0);
        let mut tf2 = create_dummy_transform("tf2");
        tf2.transform.translation.x = ordered_float::OrderedFloat(1.0);
        tf2.transform.translation.y = ordered_float::OrderedFloat(2.0);

        let _: () = con
            .mset(&[
                (
                    format!("{}{}", TF_PREFIX, "tf1"),
                    serde_json::to_string(&tf1.to_spvalue()).unwrap(),
                ),
                (
                    format!("{}{}", TF_PREFIX, "tf2"),
                    serde_json::to_string(&tf2.to_spvalue()).unwrap(),
                ),
                ("some_other_key".to_string(), "some_other_value".to_string()),
            ])
            .await
            .unwrap();

        let transforms = get_all_transforms(&mut con).await.unwrap();

        assert_eq!(transforms.len(), 2);
        assert_eq!(transforms.get("tf1"), Some(&tf1));
        assert_eq!(transforms.get("tf2"), Some(&tf2));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms_when_none_exist() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let _: () = con.set("some_other_key", "some_other_value").await.unwrap();

        let transforms = get_all_transforms(&mut con).await;

        assert!(transforms.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms_from_empty_db() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let transforms = get_all_transforms(&mut con).await;

        assert!(transforms.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_transforms_with_invalid_data() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let mut tf1 = create_dummy_transform("tf1");
        tf1.transform.translation.x = ordered_float::OrderedFloat(1.0);
        tf1.transform.translation.y = ordered_float::OrderedFloat(2.0);

        let _: () = con
            .mset(&[
                (
                    format!("{}{}", TF_PREFIX, "child1"),
                    serde_json::to_string(&tf1.to_spvalue()).unwrap(),
                ),
                (
                    format!("{}{}", TF_PREFIX, "child2_bad"),
                    "this is not json".to_string(),
                ),
            ])
            .await
            .unwrap();

        let transforms = get_all_transforms(&mut con).await.unwrap();

        assert_eq!(transforms.len(), 1);
        assert_eq!(transforms.get("child1"), Some(&tf1));
        assert!(transforms.get("child2_bad").is_none());
    }
}
