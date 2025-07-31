use crate::{SPTransformStamped, TransformsManager, get_tree_root, lookup_transform_with_root};
use redis::aio::MultiplexedConnection;
use std::error::Error;

pub(super) async fn lookup_transform(
    con: &mut MultiplexedConnection,
    parent_frame_id: &str,
    child_frame_id: &str,
) -> Result<SPTransformStamped, Box<dyn Error>> {
    let buffer = TransformsManager::get_all_transforms(con).await?;
    let root = get_tree_root(&buffer).unwrap_or_else(|| "world".to_string());
    let result = lookup_transform_with_root(parent_frame_id, child_frame_id, &root, &buffer);

    if result.is_none() {
        return Err("Couldn't lookup transform from parent '{parent_frame_id}' to child '{child_frame_id}'.".into());
    } else {
        Ok(result.unwrap())
    }
}

#[cfg(test)]
mod tests_for_insert_transform {
    use super::lookup_transform;
    use crate::*;
    use redis::AsyncCommands;
    use serial_test::serial;
    use std::time::SystemTime;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    fn create_dummy_transform(parent_id: &str, child_id: &str) -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: parent_id.to_string(),
            child_frame_id: child_id.to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::UNKNOWN,
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_lookup_transform_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let tf = create_dummy_transform("world", "robot");
        let _: () = con
            .set(
                tf_key("robot"),
                serde_json::to_string(&tf.to_spvalue()).unwrap(),
            )
            .await
            .unwrap();

        let result = lookup_transform(&mut con, "world", "robot").await;
        assert!(result.is_ok());
        let found_tf = result.unwrap();
        assert_eq!(found_tf.parent_frame_id, "world");
        assert_eq!(found_tf.child_frame_id, "robot");
    }

    #[tokio::test]
    #[serial]
    async fn test_lookup_transform_not_found() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let tf = create_dummy_transform("world", "robot");
        let _: () = con
            .set(
                tf_key("robot"),
                serde_json::to_string(&tf.to_spvalue()).unwrap(),
            )
            .await
            .unwrap();

        let result = lookup_transform(&mut con, "world", "non_existent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_lookup_transform_on_empty_db() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let result = lookup_transform(&mut con, "world", "robot").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_lookup_transform_finds_correct_root() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let tf1 = create_dummy_transform("custom_root", "robot");
        let tf2 = create_dummy_transform("robot", "gripper");

        let _: () = con
            .mset(&[
                (
                    tf_key("robot"),
                    serde_json::to_string(&tf1.to_spvalue()).unwrap(),
                ),
                (
                    tf_key("gripper"),
                    serde_json::to_string(&tf2.to_spvalue()).unwrap(),
                ),
            ])
            .await
            .unwrap();

        let result = lookup_transform(&mut con, "robot", "gripper").await;
        assert!(result.is_ok());
        let found_tf = result.unwrap();
        assert_eq!(found_tf.parent_frame_id, "robot");
        assert_eq!(found_tf.child_frame_id, "gripper");
    }

    #[tokio::test]
    #[serial]
    async fn test_lookup_transform_calculates_correct_transform() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let tf1 = create_dummy_transform("world", "robot");
        let mut tf2 = create_dummy_transform("robot", "gripper");
        tf2.transform.translation.x = ordered_float::OrderedFloat(123.45);

        let _: () = con
            .mset(&[
                (
                    tf_key("robot"),
                    serde_json::to_string(&tf1.to_spvalue()).unwrap(),
                ),
                (
                    tf_key("gripper"),
                    serde_json::to_string(&tf2.to_spvalue()).unwrap(),
                ),
            ])
            .await
            .unwrap();

        let result = lookup_transform(&mut con, "world", "gripper").await;
        assert!(result.is_ok());
        let found_tf = result.unwrap();
        assert_eq!(found_tf.parent_frame_id, "world");
        assert_eq!(found_tf.child_frame_id, "gripper");
        assert_eq!(
            found_tf.transform.translation.x,
            ordered_float::OrderedFloat(123.45)
        );
    }
}
