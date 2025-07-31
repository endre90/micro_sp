use std::error::Error;

use redis::{AsyncCommands, aio::MultiplexedConnection};

use crate::{SPTransformStamped, ToSPValue, tf_key};

pub(super) async fn insert_transform(
    con: &mut MultiplexedConnection,
    transform: &SPTransformStamped,
) -> Result<(), Box<dyn Error>> {
    let key = tf_key(&transform.child_frame_id);
    let value_str = serde_json::to_string(&transform.to_spvalue())?;
    con.set::<_, _, ()>(&key, value_str).await?;

    Ok(())

    // let value_str = match serde_json::to_string(&transform.to_spvalue()) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         // e.to_string()
    //         log::error!("Failed to serialize transform for key '{key}': {e}");
    //         return Error(e.to_string());
    //     }
    // };

    // match con.set::<_, _, ()>(&key, value_str).await {
    //     Ok(_) => {}
    //     Err(e) => {
    //         log::error!("Redis SET command for key '{key}' failed: {e}");
    //     }
    // }
}

#[cfg(test)]
mod tests_for_insert_transform {
    use super::insert_transform;
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
    async fn test_insert_transform_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let transform_to_insert = create_dummy_transform("new_child");

        insert_transform(&mut con, &transform_to_insert).await.expect("Failed to insert transform!");

        let key = tf_key("new_child");

        let result_str: String = con.get(&key).await.unwrap();

        let expected_str =
            serde_json::to_string(&transform_to_insert.clone().to_spvalue()).unwrap();
        assert_eq!(result_str, expected_str);
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_transform_overwrite() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        let mut initial_transform = create_dummy_transform("child_to_overwrite");
        initial_transform.transform.translation.x = ordered_float::OrderedFloat(1.0);

        let mut new_transform = create_dummy_transform("child_to_overwrite");
        new_transform.transform.translation.x = ordered_float::OrderedFloat(99.0);

        insert_transform(&mut con, &initial_transform).await.expect("Failed to insert transform!");
        insert_transform(&mut con, &new_transform).await.expect("Failed to insert transform!");

        let key = tf_key("child_to_overwrite");
        let result_str: String = con.get(key).await.unwrap();
        let result_val: SPValue = serde_json::from_str(&result_str).unwrap();

        let expected_val = new_transform.to_spvalue();
        assert_eq!(result_val, expected_val);
    }
}
