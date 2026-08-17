use crate::{
    ToSPValue, TransformsManager, check_would_produce_cycle, lookup_transform_with_root, tf_key,
};
use crate::SPConnection;
use redis::AsyncCommands;
use std::error::Error;

pub(super) async fn reparent_transform(
    con: &mut SPConnection,
    new_parent_frame_id: &str,
    child_frame_id: &str,
) -> Result<(), Box<dyn Error>> {
    let buffer = TransformsManager::get_all_transforms(con).await?;
    let redis_key = tf_key(child_frame_id);
    let Some(original_transform) = buffer.get(child_frame_id) else {
        return Err("Can't reparent non-existent transform '{child_frame_id}'.".into());
    };

    let mut updated_transform = original_transform.clone();
    updated_transform.parent_frame_id = new_parent_frame_id.to_string();

    if check_would_produce_cycle(&updated_transform, &buffer) {
        return Err("Reparenting '{child_frame_id}' to '{new_parent_frame_id}' would create a cycle. Aborting.".into());
    }

    let Some(lookup_tf) =
        lookup_transform_with_root(new_parent_frame_id, child_frame_id, "world", &buffer)
    else {
        return Err("Failed to calculate the new transform from '{new_parent_frame_id}' to '{child_frame_id}'.".into());
    };

    updated_transform.transform = lookup_tf.transform;
    let updated_value_json = serde_json::to_string(&updated_transform.to_spvalue())?;

    con.set::<_, _, ()>(redis_key, updated_value_json).await?;

    Ok(())
}

#[cfg(test)]
mod tests_for_reparent_transform {
    use super::reparent_transform;
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
    async fn test_reparent_transform_success() {
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

        let result = reparent_transform(&mut con, "parent2", "child").await;
        assert!(result.is_ok());

        let result_str: String = con.get(tf_key("child")).await.unwrap();
        let result_val: SPValue = serde_json::from_str(&result_str).unwrap();

        if let SPValue::Transform(TransformOrUnknown::Transform(result_tf)) = result_val {
            assert_eq!(result_tf.parent_frame_id, "parent2");
            assert_eq!(result_tf.transform.translation.x, OrderedFloat(10.0));
        } else {
            panic!("Result was not a valid transform");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_reparent_transform_child_not_found() {
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

        let result = reparent_transform(&mut con, "parent1", "non_existent_child").await;
        assert!(result.is_err());
    }

    /// Reparenting keeps the frame where it is in space, which means the pose
    /// relative to the new parent has to be resolvable. If the new parent sits
    /// in a segment that does not reach the `world` root, that maths cannot be
    /// done - and rather than writing a bogus pose the call must fail and leave
    /// the frame exactly as it was.
    #[tokio::test]
    #[serial]
    async fn an_unresolvable_new_pose_aborts_the_reparent() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        // `child` hangs off the world root; `orphan` hangs off a parent that is
        // not in the buffer at all, so nothing under it can be resolved.
        let child = create_dummy_transform("world", "child");
        let orphan = create_dummy_transform("detached_root", "orphan");
        for tf in [&child, &orphan] {
            let _: () = con
                .set(
                    tf_key(&tf.child_frame_id),
                    serde_json::to_string(&tf.to_spvalue()).unwrap(),
                )
                .await
                .unwrap();
        }

        let result = reparent_transform(&mut con, "orphan", "child").await;
        assert!(
            result.is_err(),
            "reparenting under an unreachable parent must fail"
        );

        let stored = TransformsManager::get_transform(&mut con, "child")
            .await
            .unwrap();
        assert_eq!(
            stored.parent_frame_id, "world",
            "the frame must keep its original parent when the reparent aborts"
        );
        assert_eq!(stored.transform, child.transform);
    }
}
