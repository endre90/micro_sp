use crate::{SPTransformStamped, SPValue, TransformOrUnknown, tf_key};
use crate::SPConnection;
use redis::AsyncCommands;
use std::error::Error;

pub(super) async fn get_transform(
    con: &mut SPConnection,
    frame_id: &str,
) -> Result<SPTransformStamped, Box<dyn Error>> {
    let redis_key = tf_key(frame_id);
    let redis_value: String = match con.get(&redis_key).await {
        Ok(Some(val)) => val,
        Ok(None) => {
            return Err(format!("Transform '{}' not found in Redis.", frame_id).into());
        }
        Err(e) => e.to_string(),
    };

    match serde_json::from_str::<SPValue>(&redis_value) {
        Ok(SPValue::Transform(TransformOrUnknown::Transform(val))) => Ok(val),
        _ => {
            return Err(format!("Value for '{}' is not a valid transform.", frame_id).into());
        }
    }
}

#[cfg(test)]
mod tests_for_get_transform {
    use super::get_transform;
    use crate::*; // Pulls in SPTransformStamped, SPValue, tf_key, ConnectionManager, etc.
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
    async fn test_get_transform_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let frame_id = "robot";
        let tf = create_dummy_transform("world", frame_id);

        let _: () = con
            .set(
                tf_key(frame_id),
                serde_json::to_string(&tf.to_spvalue()).unwrap(),
            )
            .await
            .unwrap();

        let result = get_transform(&mut con, frame_id).await;

        assert!(result.is_ok());
        let found_tf = result.unwrap();
        assert_eq!(found_tf.parent_frame_id, "world");
        assert_eq!(found_tf.child_frame_id, frame_id);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_transform_not_found() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let frame_id = "non_existent";

        let result = get_transform(&mut con, frame_id).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("Transform '{frame_id}' not found in Redis.")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_transform_invalid_value() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        let frame_id = "corrupt_frame";
        let invalid_json = "{}".to_string();

        let _: () = con.set(tf_key(frame_id), invalid_json).await.unwrap();

        let result = get_transform(&mut con, frame_id).await;

        assert!(
            result.is_err(),
            "Expected get_transform to return an Err for invalid data, but got Ok"
        );

        let err_msg = result.unwrap_err().to_string();

        assert!(
            err_msg.contains("is not a valid transform") || err_msg.contains("expected"),
            "Failed with an unexpected error message format: {}",
            err_msg
        );
    }
}
