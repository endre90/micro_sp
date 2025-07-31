use crate::{SPTransform, SPTransformStamped, SPValue, ToSPValue, TransformOrUnknown, tf_key};
use redis::{AsyncCommands, aio::MultiplexedConnection};
use std::error::Error;

pub(super) async fn move_transform(
    con: &mut MultiplexedConnection,
    name: &str,
    new_transform: SPTransform,
) -> Result<(), Box<dyn Error>> {
    let redis_key = tf_key(name);
    let redis_value: String = match con.get(&redis_key).await {
        Ok(Some(val)) => val,
        Ok(None) => {
            return Err("Transform '{name}' not found in Redis, cannot move.".into());
        }
        Err(e) => e.to_string()
    };

    let mut sp_tf_stamped: SPTransformStamped = match serde_json::from_str::<SPValue>(&redis_value)
    {
        Ok(SPValue::Transform(TransformOrUnknown::Transform(val))) => val,
        _ => {
            return Err("Value for '{name}' is not a valid transform, cannot move.".into());
        }
    };

    sp_tf_stamped.transform = new_transform;
    let updated_value_json = serde_json::to_string(&sp_tf_stamped.to_spvalue())?;
    // let updated_value_json = match serde_json::to_string(&sp_tf_stamped.to_spvalue()) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         log::error!("Failed to serialize updated transform '{name}': {e}");
    //         return;
    //     }
    // };
    con.set::<_, _, ()>(&redis_key, updated_value_json).await?;
    Ok(())

    // if let Err(e) = con.set::<_, _, ()>(&redis_key, updated_value_json).await {
    //     log::error!("Failed to SET updated transform '{name}': {e}");
    // }
}

#[cfg(test)]
mod tests_for_move_transform {
    use super::move_transform;
    use crate::*;
    use ordered_float::OrderedFloat;
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
    async fn test_move_transform_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let child_id = "robot1";
        let initial_tf = create_dummy_transform(child_id);
        let redis_key = tf_key(child_id);

        let _: () = con
            .set(
                &redis_key,
                serde_json::to_string(&initial_tf.clone().to_spvalue()).unwrap(),
            )
            .await
            .unwrap();

        let new_transform = SPTransform {
            translation: SPTranslation {
                x: OrderedFloat(10.0),
                y: OrderedFloat(20.0),
                z: OrderedFloat(30.0),
            },
            rotation: SPRotation {
                x: OrderedFloat(0.0),
                y: OrderedFloat(0.0),
                z: OrderedFloat(1.0),
                w: OrderedFloat(0.0),
            },
        };

        let result = move_transform(&mut con, child_id, new_transform.clone()).await;
        assert!(result.is_ok());

        let result_str: String = con.get(&redis_key).await.unwrap();
        let result_val: SPValue = serde_json::from_str(&result_str).unwrap();

        if let SPValue::Transform(TransformOrUnknown::Transform(result_tf)) = result_val {
            assert_eq!(result_tf.transform, new_transform);
            assert_eq!(result_tf.child_frame_id, initial_tf.child_frame_id);
        } else {
            panic!("Result was not a valid transform");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_move_transform_not_found() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let child_id = "non_existent_robot";

        let new_transform = SPTransform::default();

        let result = move_transform(&mut con, child_id, new_transform).await;
        assert!(result.is_err());

        let exists: bool = con.exists(tf_key(child_id)).await.unwrap();
        assert!(
            !exists,
            "Function should not create a key that doesn't exist."
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_move_transform_invalid_data() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let child_id = "robot_with_bad_data";
        let redis_key = tf_key(child_id);
        let initial_bad_data = "this is not a transform";

        let _: () = con.set(&redis_key, initial_bad_data).await.unwrap();

        let new_transform = SPTransform::default();
        let result = move_transform(&mut con, child_id, new_transform).await;
        assert!(result.is_err());

        let result_str: String = con.get(&redis_key).await.unwrap();
        assert_eq!(
            result_str, initial_bad_data,
            "Function should not modify invalid data."
        );
    }
}
