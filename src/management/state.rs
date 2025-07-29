use crate::*;
use redis::{aio::MultiplexedConnection};

mod build_state;
mod get_full_state;
mod get_sp_value;
mod get_state_for_keys;
mod set_sp_value;
mod set_state;

pub struct StateManager {}

impl StateManager {
    pub async fn get_full_state(con: &mut MultiplexedConnection) -> Option<State> {
        get_full_state::get_full_state(con).await
    }

    pub async fn get_state_for_keys(
        con: &mut MultiplexedConnection,
        keys: &Vec<String>,
    ) -> Option<State> {
        get_state_for_keys::get_state_for_keys(con, keys).await
    }

    pub async fn get_sp_value(con: &mut MultiplexedConnection, var: &str) -> Option<SPValue> {
        get_sp_value::get_sp_value(con, var).await
    }

    pub async fn set_state(con: &mut MultiplexedConnection, state: &State) {
        set_state::set_state(con, state).await
    }

    pub async fn set_sp_value(con: &mut MultiplexedConnection, key: &str, value: &SPValue) {
        set_sp_value::set_sp_value(con, key, value).await
    }

    pub fn build_state(keys: Vec<String>, values: Vec<Option<String>>) -> State {
        build_state::build_state(keys, values)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::time::SystemTime;

//     use super::*;
//     use serial_test::serial;
//     use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
//     use testcontainers_modules::redis::Redis;

//     fn dummy_state() -> State {
//         let mut state = State::new();
//         state
//             .state
//             .insert("x".to_string(), assign!(iv!("x"), 1.to_spvalue()));
//         state
//             .state
//             .insert("y".to_string(), assign!(iv!("y"), 2.to_spvalue()));
//         state
//             .state
//             .insert("z".to_string(), assign!(iv!("z"), 3.to_spvalue()));
//         state
//     }

//     fn create_dummy_transform(parent: &str, child: &str) -> SPTransformStamped {
//         SPTransformStamped {
//             active_transform: true,
//             enable_transform: true,
//             time_stamp: SystemTime::now(),
//             parent_frame_id: parent.to_string(),
//             child_frame_id: child.to_string(),
//             transform: SPTransform::default(),
//             metadata: MapOrUnknown::UNKNOWN,
//         }
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_set_and_get_single_value() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;
//         let key = "x";
//         let value = 123.to_spvalue();

//         StateManager::set_sp_value(&mut con, key, &value).await;

//         let retrieved = StateManager::get_sp_value(&mut con, key)
//             .await
//             .expect("Value should exist");

//         assert_eq!(value, retrieved);
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_get_non_existent_value() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;

//         let retrieved = StateManager::get_sp_value(&mut con, "key-does-not-exist").await;
//         assert!(
//             retrieved.is_none(),
//             "Getting a non-existent key should return None"
//         );
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_get_state_on_empty_db() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;

//         let state = StateManager::get_full_state(&mut con)
//             .await
//             .expect("redis_get_state should not fail on an empty DB");

//         println!("{:?}", state);

//         assert!(
//             state.state.iter().len() == 1,
//             "State map should only have heartbeat"
//         );
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_set_partial_and_get_full_state() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;
//         let mut initial_state = dummy_state();

//         StateManager::set_state(&mut con, &initial_state).await;

//         let retrieved_state = StateManager::get_full_state(&mut con)
//             .await
//             .expect("Failed to get state");

//         initial_state.state.remove("heartbeat").unwrap();
//         assert_eq!(initial_state, retrieved_state);
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_overwrite_and_add_values() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;

//         StateManager::set_state(&mut con, &dummy_state()).await;

//         let mut partial_update = State::new();
//         partial_update
//             .state
//             .insert("x".to_string(), assign!(iv!("x"), 99.to_spvalue()));
//         partial_update
//             .state
//             .insert("j".to_string(), assign!(iv!("j"), 100.to_spvalue()));

//         StateManager::set_state(&mut con, &partial_update).await;

//         let final_state = StateManager::get_full_state(&mut con).await.unwrap();

//         let get_val = |s: &State, k: &str| s.state.get(k).unwrap().val.clone();

//         assert_eq!(get_val(&final_state, "x"), 99.to_spvalue()); // Overwritten
//         assert_eq!(get_val(&final_state, "y"), 2.to_spvalue()); // Unchanged
//         assert_eq!(get_val(&final_state, "z"), 3.to_spvalue()); // Unchanged
//         assert_eq!(get_val(&final_state, "j"), 100.to_spvalue()); // Added
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_add_and_get_all_transforms() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;
//         let mut transforms_to_add = HashMap::new();
//         transforms_to_add.insert(
//             "floor".to_string(),
//             create_dummy_transform("world", "floor"),
//         );
//         transforms_to_add.insert(
//             "table".to_string(),
//             create_dummy_transform("world", "table"),
//         );

//         StateManager::insert_transforms(&mut con, transforms_to_add).await;
//         let fetched_transforms = StateManager::get_all_transforms(&mut con).await;

//         assert_eq!(2, fetched_transforms.len());
//         assert!(fetched_transforms.contains_key("floor"));
//         assert!(fetched_transforms.contains_key("table"));
//         assert_eq!(
//             fetched_transforms.get("table").unwrap().parent_frame_id,
//             "world"
//         );
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_move_transform() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;
//         let mut initial_data = HashMap::new();
//         initial_data.insert(
//             "floor".to_string(),
//             create_dummy_transform("world", "floor"),
//         );
//         StateManager::insert_transforms(&mut con, initial_data).await;

//         let new_transform_data = SPTransform::default();

//         StateManager::move_transform(&mut con, "floor", new_transform_data.clone()).await;

//         let final_state = StateManager::get_all_transforms(&mut con).await;
//         let moved_transform = final_state.get("floor").unwrap();
//         assert_eq!(moved_transform.transform, new_transform_data);
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_lookup_transform() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;
//         let mut initial_data = HashMap::new();
//         initial_data.insert(
//             "floor".to_string(),
//             create_dummy_transform("world", "floor"),
//         );
//         initial_data.insert(
//             "table".to_string(),
//             create_dummy_transform("floor", "table"),
//         );
//         initial_data.insert("cup".to_string(), create_dummy_transform("table", "cup"));
//         StateManager::insert_transforms(&mut con, initial_data).await;

//         let result = StateManager::lookup_transform(&mut con, "world", "cup").await;

//         assert!(result.is_some());
//         assert_eq!(result.unwrap().parent_frame_id, "world");
//     }

//     #[tokio::test]
//     #[serial]
//     async fn test_reparent_transform() {
//         let _container = Redis::default()
//             .with_mapped_port(6379, ContainerPort::Tcp(6379))
//             .start()
//             .await
//             .unwrap();

//         let mut con = ConnectionManager::new().await.get_connection().await;
//         let mut initial_data = HashMap::new();

//         initial_data.insert(
//             "floor".to_string(),
//             create_dummy_transform("world", "floor"),
//         );
//         initial_data.insert(
//             "robot".to_string(),
//             create_dummy_transform("world", "robot"),
//         );
//         StateManager::insert_transforms(&mut con, initial_data).await;
//         let success = StateManager::reparent_transform(&mut con, "robot", "floor").await;

//         assert_eq!(true, success);

//         let final_state = StateManager::get_all_transforms(&mut con).await;
//         let reparented_transform = final_state.get("floor").unwrap();
//         assert_eq!(reparented_transform.parent_frame_id, "robot");
//     }
// }
