//! Storing the 3D transform tree in Redis.
//!
//! One key per frame, `tf:<child_frame_id>`, holding the frame's
//! [`SPTransformStamped`] as a JSON [`SPValue`]. [`TransformsManager`] is the
//! write path; the maths that walks the resulting tree lives in
//! [`crate::transforms`].

use crate::*;
use crate::SPConnection;
use std::collections::HashMap;
use std::error::Error;

mod get_all_transforms;
mod insert_transform;
mod insert_transforms;
mod load_transforms_from_path;
mod lookup_transform;
mod get_transform;
mod move_transform;
mod remove_transform;
mod reparent_transform;
mod snap_to_parent_transform;

/// Key prefix that separates transform frames from state variables in the
/// shared Redis keyspace.
pub const TF_PREFIX: &str = "tf:";

/// The Redis key a frame is stored under, i.e. [`TF_PREFIX`] + `child_id`.
///
/// ```
/// use micro_sp::tf_key;
///
/// assert_eq!(tf_key("gripper"), "tf:gripper");
/// ```
pub fn tf_key(child_id: &str) -> String {
    format!("{}{}", TF_PREFIX, child_id)
}

/// Reads and writes the transform tree through a Redis connection.
///
/// A stateless namespace of associated functions, each taking the connection
/// (`con`) to work on. Unlike [`StateManager`], these return `Err` rather than
/// logging and continuing - a frame that could not be read is not something a
/// lookup can paper over. The mutating calls that change a frame's parent
/// refuse to introduce a cycle.
pub struct TransformsManager {}

impl TransformsManager {
    /// Write one frame, keyed by its own `child_frame_id`. Overwrites any frame
    /// already stored under that id.
    pub async fn insert_transform(
        con: &mut SPConnection,
        transform: &SPTransformStamped,
    ) -> Result<(), Box<dyn Error>> {
        insert_transform::insert_transform(con, &transform).await
    }

    /// Write many frames in one `MSET`. An empty vector logs and succeeds
    /// without touching Redis.
    pub async fn insert_transforms(
        con: &mut SPConnection,
        transforms: &Vec<SPTransformStamped>,
    ) -> Result<(), Box<dyn Error>> {
        insert_transforms::insert_transforms(con, &transforms).await
    }

    /// Delete the frame whose child id is `key`. Deleting a frame that is not
    /// there is not an error; any children it had are left orphaned.
    pub async fn remove_transform(
        con: &mut SPConnection,
        key: &str,
    ) -> Result<(), Box<dyn Error>> {
        remove_transform::remove_transform(con, &key).await
    }

    /// Read the whole transform buffer, keyed by child frame id.
    ///
    /// `SCAN`s the [`TF_PREFIX`] keyspace and `MGET`s the hits. A key holding
    /// something that is not a transform is skipped with a warning; an empty
    /// result is reported as an `Err` rather than an empty map.
    pub async fn get_all_transforms(
        con: &mut SPConnection,
    ) -> Result<HashMap<String, SPTransformStamped>, Box<dyn Error>> {
        get_all_transforms::get_all_transforms(con).await
    }

    /// Replace the pose of frame `name`, keeping its parent.
    ///
    /// `Err` if the frame does not exist or does not hold a valid transform.
    pub async fn move_transform(
        con: &mut SPConnection,
        name: &str,
        new_transform: SPTransform,
    ) -> Result<(), Box<dyn Error>> {
        move_transform::move_transform(con, name, new_transform).await
    }

    /// Attach `child_frame_id` to a new parent without moving it in space.
    ///
    /// The stored pose is recomputed relative to the new parent, so the frame
    /// stays where it was. `Err` if the child does not exist, if the new
    /// parentage would create a cycle, or if the new pose cannot be resolved.
    pub async fn reparent_transform(
        con: &mut SPConnection,
        new_parent_frame_id: &str,
        child_frame_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        reparent_transform::reparent_transform(con, new_parent_frame_id, child_frame_id).await
    }

    /// Attach `child_frame_id` to a new parent *and* snap it onto that parent's
    /// origin (an identity pose).
    ///
    /// Unlike [`TransformsManager::reparent_transform`] this deliberately moves
    /// the frame. `Err` if the child does not exist or the move would create a
    /// cycle.
    pub async fn snap_to_parent_transform(
        con: &mut SPConnection,
        new_parent_frame_id: &str,
        child_frame_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        snap_to_parent_transform::snap_to_parent_transform(con, new_parent_frame_id, child_frame_id).await
    }

    /// Resolve the transform from `parent_frame_id` to `child_frame_id`.
    ///
    /// Reads the whole buffer, finds its root and walks up from the parent and
    /// down to the child. `Err` if either frame is unreachable, if the chain is
    /// longer than [`MAX_TRANSFORM_CHAIN`], or if the buffer contains a cycle.
    /// The two frames need not be directly related.
    ///
    /// ```no_run
    /// use micro_sp::*;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let connection_manager = ConnectionManager::new().await;
    /// let mut con = connection_manager.get_connection().await;
    ///
    /// let tf = TransformsManager::lookup_transform(&mut con, "world", "gripper").await?;
    /// println!("gripper is at {:?}", tf.transform.translation);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lookup_transform(
        con: &mut SPConnection,
        parent_frame_id: &str,
        child_frame_id: &str,
    ) -> Result<SPTransformStamped, Box<dyn Error>> {
        lookup_transform::lookup_transform(con, parent_frame_id, child_frame_id).await
    }

    /// Read one frame exactly as stored - its pose relative to its own parent,
    /// with no tree walking. `Err` if it is missing or not a valid transform.
    pub async fn get_transform(
        con: &mut SPConnection,
        frame_id: &str,
    ) -> Result<SPTransformStamped, Box<dyn Error>> {
        get_transform::get_transform(con, frame_id).await
    }

    /// Load a directory of frame JSON files into Redis - the startup bootstrap.
    ///
    /// `Err` if `path` cannot be listed; individual malformed files cost only
    /// their own frame (see [`crate::load_new_scenario`]).
    pub async fn load_transforms_from_path(
        con: &mut SPConnection,
        path: &str,
    ) -> Result<(), Box<dyn Error>> {
        load_transforms_from_path::load_transforms_from_path(con, path).await
    }
}

/// The public façade. Each associated function above only forwards to its
/// module, but the façade is what every caller outside this crate actually
/// uses, so the round trip through it is worth pinning: what goes in through
/// [`TransformsManager`] must come back out through it.
#[cfg(test)]
mod tests {
    use crate::*;
    use serial_test::serial;
    use std::time::SystemTime;
    use testcontainers::{ImageExt, core::ContainerPort, runners::AsyncRunner};
    use testcontainers_modules::redis::Redis;

    fn frame(child_id: &str) -> SPTransformStamped {
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

    /// `move_transform` replaces the pose but keeps the parent, and
    /// `remove_transform` takes the frame out of the buffer - both driven
    /// through the public manager rather than the private module functions.
    #[tokio::test]
    #[serial]
    async fn moving_and_removing_a_frame_through_the_manager() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;
        TransformsManager::insert_transform(&mut con, &frame("tool"))
            .await
            .unwrap();

        let mut moved = SPTransform::default();
        moved.translation.x = ordered_float::OrderedFloat(3.5);
        TransformsManager::move_transform(&mut con, "tool", moved.clone())
            .await
            .unwrap();

        let stored = TransformsManager::get_transform(&mut con, "tool")
            .await
            .unwrap();
        assert_eq!(stored.transform, moved, "the new pose must be stored");
        assert_eq!(
            stored.parent_frame_id, "world",
            "moving a frame must not change its parent"
        );

        TransformsManager::remove_transform(&mut con, "tool")
            .await
            .unwrap();
        assert!(
            TransformsManager::get_transform(&mut con, "tool")
                .await
                .is_err(),
            "the frame must be gone after removal"
        );
    }

    /// Moving a frame that was never inserted is an error rather than a silent
    /// insert, while removing one that is not there is deliberately fine.
    #[tokio::test]
    #[serial]
    async fn moving_a_missing_frame_errors_but_removing_one_does_not() {
        let _container = Redis::default()
            .with_mapped_port(6379, ContainerPort::Tcp(6379))
            .start()
            .await
            .unwrap();

        let mut con = ConnectionManager::new().await.get_connection().await;

        assert!(
            TransformsManager::move_transform(&mut con, "ghost", SPTransform::default())
                .await
                .is_err(),
            "moving a non-existent frame must fail"
        );
        assert!(
            TransformsManager::get_transform(&mut con, "ghost")
                .await
                .is_err(),
            "the failed move must not have created the frame"
        );
        assert!(
            TransformsManager::remove_transform(&mut con, "ghost")
                .await
                .is_ok(),
            "removing a frame that is not there is not an error"
        );
    }
}
