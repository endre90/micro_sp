use crate::{TransformsManager, list_frames_in_dir, load_new_scenario};
use redis::aio::MultiplexedConnection;
use std::error::Error;

pub(super) async fn load_transforms_from_path(
    con: &mut MultiplexedConnection,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let list = list_frames_in_dir(path)?;

    let frames = load_new_scenario(&list);
    let frames_to_insert: Vec<_> = frames.values().cloned().collect();

    TransformsManager::insert_transforms(con, &frames_to_insert).await?;

    Ok(())
}
