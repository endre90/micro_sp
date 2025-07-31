use crate::{SPTransformStamped, ToSPValue, tf_key};
use redis::{AsyncCommands, Value, aio::MultiplexedConnection};
use std::error::Error;

pub(super) async fn insert_transforms(
    con: &mut MultiplexedConnection,
    transforms: &Vec<SPTransformStamped>,
) -> Result<(), Box<dyn Error>> {
    if transforms.is_empty() {
        return Err("There are no transforms to insert, vector is empty.".into());
    }

    // let key_value_pairs: Vec<(String, String)> = transforms
    //     .into_iter()
    //     .filter_map(|transform| {
    //         let key = tf_key(&transform.child_frame_id);
    //         match serde_json::to_string(&transform.to_spvalue()) {
    //             Ok(value_str) => Some((key, value_str)),
    //             Err(e) => {
    //                 log::error!(
    //                     "Failed to serialize transform for child '{}': {}",
    //                     transform.child_frame_id,
    //                     e
    //                 );
    //                 None
    //             }
    //         }
    //     })
    //     .collect();

    let key_value_pairs: Vec<(String, String)> = transforms
        .iter()
        .map(|transform| {
            let key = tf_key(&transform.child_frame_id);
            serde_json::to_string(&transform.clone().to_spvalue()).map(|value_str| (key, value_str))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if key_value_pairs.is_empty() {
        return Err("No valid transforms to set after serialization.".into());
    }

    // match con.mset::<_, String, Value>(&key_value_pairs).await {
    //     Ok(_) => {}
    //     Err(e) => {
    //         log::error!("Redis MSET command for multiple transforms failed: {}", e);
    //     }
    // }
    con.mset::<_, String, Value>(&key_value_pairs).await?;
    Ok(())
}

#[cfg(test)]
mod tests_for_insert_transform {
    use super::insert_transforms;
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
    async fn test_insert_transforms_success() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let mut con = ConnectionManager::new().await.get_connection().await;
        let tf1 = create_dummy_transform("child1");
        let tf2 = create_dummy_transform("child2");
        let transforms_to_insert = vec![tf1.clone(), tf2.clone()];

        insert_transforms(&mut con, &transforms_to_insert)
            .await
            .expect("Failed to insert transforms");

        let keys = vec![tf_key("child1"), tf_key("child2")];
        let result: Vec<String> = con.mget(keys).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], serde_json::to_string(&tf1.to_spvalue()).unwrap());
        assert_eq!(result[1], serde_json::to_string(&tf2.to_spvalue()).unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_transforms_empty_vector() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let mut con = ConnectionManager::new().await.get_connection().await;
        let transforms_to_insert = vec![];
        let result = insert_transforms(&mut con, &transforms_to_insert).await;

        assert!(result.is_err());

        let all_keys: Vec<String> = con.keys("*").await.unwrap();
        assert!(all_keys.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_transforms_overwrite_multiple() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let mut con = ConnectionManager::new().await.get_connection().await;

        let initial_tf1 = create_dummy_transform("child1");
        let initial_tf2 = create_dummy_transform("child2");
        let initial_transforms = vec![initial_tf1, initial_tf2];
        insert_transforms(&mut con, &initial_transforms)
            .await
            .expect("Failed to insert transforms");

        let mut new_tf1 = create_dummy_transform("child1");
        new_tf1.transform.translation.x = ordered_float::OrderedFloat(100.0);
        let mut new_tf2 = create_dummy_transform("child2");
        new_tf2.transform.translation.y = ordered_float::OrderedFloat(200.0);
        let new_transforms = vec![new_tf1.clone(), new_tf2.clone()];

        insert_transforms(&mut con, &new_transforms)
            .await
            .expect("Failed to insert transforms");

        let keys = vec![tf_key("child1"), tf_key("child2")];
        let result: Vec<String> = con.mget(keys).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            serde_json::to_string(&new_tf1.to_spvalue()).unwrap()
        );
        assert_eq!(
            result[1],
            serde_json::to_string(&new_tf2.to_spvalue()).unwrap()
        );
    }
}
