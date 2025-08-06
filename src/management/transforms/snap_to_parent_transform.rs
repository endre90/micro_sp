use crate::{
    check_would_produce_cycle, tf_key, SPTransform, ToSPValue, TransformsManager
};
use redis::{AsyncCommands, aio::MultiplexedConnection};
use std::error::Error;

pub(super) async fn snap_to_parent_transform(
    con: &mut MultiplexedConnection,
    new_parent_frame_id: &str,
    child_frame_id: &str,
) -> Result<(), Box<dyn Error>> {
    let buffer = TransformsManager::get_all_transforms(con).await?;
    let redis_key = tf_key(child_frame_id);
    let Some(original_transform) = buffer.get(child_frame_id) else {
        return Err("Can't snap to parent non-existent transform '{child_frame_id}'.".into());
    };

    let mut updated_transform = original_transform.clone();
    updated_transform.parent_frame_id = new_parent_frame_id.to_string();

    if check_would_produce_cycle(&updated_transform, &buffer) {
        return Err("Snapping '{child_frame_id}' to '{new_parent_frame_id}' would create a cycle. Aborting.".into());
    }

    let snap_tf = SPTransform::default();

    updated_transform.transform = snap_tf;
    let updated_value_json = serde_json::to_string(&updated_transform.to_spvalue())?;

    con.set::<_, _, ()>(redis_key, updated_value_json).await?;

    Ok(())
}

#[cfg(test)]
mod tests_for_reparent_transform {
    use super::snap_to_parent_transform;
    use crate::*;
    use ordered_float::OrderedFloat;
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
    async fn test_snap_to_parent_transform_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let tf1 = create_dummy_transform("world", "parent1");
        let mut tf2 = create_dummy_transform("parent1", "child");
        let tf3 = create_dummy_transform("world", "parent2");
        tf2.transform.translation.x = OrderedFloat(10.0);

        let _: () = con
            .mset(&[
                (
                    tf_key("parent1"),
                    serde_json::to_string(&tf1.clone().to_spvalue()).unwrap(),
                ),
                (
                    tf_key("child"),
                    serde_json::to_string(&tf2.clone().to_spvalue()).unwrap(),
                ),
                (
                    tf_key("parent2"),
                    serde_json::to_string(&tf3.clone().to_spvalue()).unwrap(),
                ),
            ])
            .await
            .unwrap();

        let result = snap_to_parent_transform(&mut con, "parent2", "child").await;
        assert!(result.is_ok());

        let result_str: String = con.get(tf_key("child")).await.unwrap();
        let result_val: SPValue = serde_json::from_str(&result_str).unwrap();

        if let SPValue::Transform(TransformOrUnknown::Transform(result_tf)) = result_val {
            assert_eq!(result_tf.parent_frame_id, "parent2");
            assert_eq!(result_tf.transform.translation.x, OrderedFloat(0.0));
        } else {
            panic!("Result was not a valid transform");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_snap_to_parent_transform_child_not_found() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let tf1 = create_dummy_transform("world", "parent1");
        let _: () = con
            .set(
                tf_key("parent1"),
                serde_json::to_string(&tf1.to_spvalue()).unwrap(),
            )
            .await
            .unwrap();

        let result = snap_to_parent_transform(&mut con, "parent1", "non_existent_child").await;
        assert!(result.is_err());
    }
}
