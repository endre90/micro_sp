use crate::{TransformsManager, list_frames_in_dir, load_new_scenario};
use crate::SPConnection;
use std::error::Error;

pub(super) async fn load_transforms_from_path(
    con: &mut SPConnection,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let list = list_frames_in_dir(path)?;

    let frames = load_new_scenario(&list);
    let frames_to_insert: Vec<_> = frames.values().cloned().collect();

    TransformsManager::insert_transforms(con, &frames_to_insert).await?;

    Ok(())
}

/// Loading a scenario directory straight into Redis.
///
/// This is the one-call bootstrap a consuming package uses at startup: point it
/// at a directory of frame JSON and the transform buffer is populated. It
/// composes `list_frames_in_dir` + `load_new_scenario` +
/// `TransformsManager::insert_transforms`, so the thing worth testing is the
/// composition - specifically that a bad path is an `Err` the caller can react
/// to rather than a silently empty buffer.
#[cfg(test)]
mod tests {
    use crate::*;
    use serial_test::serial;
    use std::sync::Arc;
    use testcontainers::{ContainerAsync, ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    async fn redis() -> (ContainerAsync<Redis>, Arc<ConnectionManager>) {
        let container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();
        let manager = Arc::new(ConnectionManager::new().await);
        let mut con = manager.get_connection().await;
        StateManager::flush_state(&mut con).await;
        (container, manager)
    }

    #[tokio::test]
    #[serial]
    async fn the_example_scenario_loads_into_redis() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let path = format!("{}/src/transforms/examples/data", manifest_dir);

        TransformsManager::load_transforms_from_path(&mut con, &path)
            .await
            .expect("the bundled example scenario should load");

        let loaded = TransformsManager::get_all_transforms(&mut con).await.unwrap();
        assert!(!loaded.is_empty(), "nothing was inserted");
        assert!(
            loaded.contains_key("chair"),
            "expected the example frames, got {:?}",
            loaded.keys()
        );

        // And the frames are usable straight away - a lookup resolves through
        // the chain that was just loaded.
        assert!(
            TransformsManager::get_transform(&mut con, "chair").await.is_ok(),
            "a loaded frame should be readable"
        );
    }

    /// A path that does not exist is an error the caller sees, not a silent
    /// no-op - starting a robot against an empty scene because of a typo'd path
    /// is exactly what this has to prevent.
    #[tokio::test]
    #[serial]
    async fn a_path_that_does_not_exist_is_an_error() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let result =
            TransformsManager::load_transforms_from_path(&mut con, "/no/such/scenario").await;

        assert!(result.is_err());
        // Note `get_all_transforms` reports an *error* rather than an empty map
        // when the keyspace holds no transforms, so "nothing was written" is
        // checked that way round.
        assert!(
            TransformsManager::get_all_transforms(&mut con).await.is_err(),
            "nothing should have been written"
        );
    }

    /// An existing but empty directory is not an error - it is a scenario with
    /// no frames in it.
    #[tokio::test]
    #[serial]
    async fn an_empty_directory_loads_nothing_without_failing() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let dir = std::env::temp_dir().join(format!(
            "micro_sp_empty_scenario_{}",
            nanoid::nanoid!(10, &NANOID_ALPHABET)
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let result =
            TransformsManager::load_transforms_from_path(&mut con, dir.to_str().unwrap()).await;
        let _ = std::fs::remove_dir_all(&dir);

        assert!(result.is_ok(), "an empty scenario is not a failure");
        assert!(TransformsManager::get_all_transforms(&mut con).await.is_err());
    }

    /// The wart the two tests above have to work around, pinned on its own:
    /// `get_all_transforms` treats "there are no transforms" as an error rather
    /// than as an empty result, so a caller cannot tell an empty scene from a
    /// failed read without matching on the message. Worth knowing before
    /// writing `?` in front of it.
    #[tokio::test]
    #[serial]
    async fn get_all_transforms_errors_on_an_empty_keyspace() {
        let (_container, manager) = redis().await;
        let mut con = manager.get_connection().await;

        let empty = TransformsManager::get_all_transforms(&mut con).await;
        assert!(empty.is_err(), "expected the empty-keyspace error");

        // With one transform present it succeeds.
        TransformsManager::insert_transform(
            &mut con,
            &SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: std::time::SystemTime::now(),
                parent_frame_id: "world".to_string(),
                child_frame_id: "a".to_string(),
                transform: SPTransform::default(),
                metadata: MapOrUnknown::UNKNOWN,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            TransformsManager::get_all_transforms(&mut con).await.unwrap().len(),
            1
        );
    }
}
