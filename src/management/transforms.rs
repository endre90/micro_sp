use crate::*;
use redis::aio::MultiplexedConnection;
use std::collections::HashMap;
use std::error::Error;

mod get_all_transforms;
mod insert_transform;
mod insert_transforms;
mod load_transforms_from_path;
mod lookup_transform;
mod move_transform;
mod remove_transform;
mod reparent_transform;

pub const TF_PREFIX: &str = "tf:";

pub fn tf_key(child_id: &str) -> String {
    format!("{}{}", TF_PREFIX, child_id)
}

pub struct TransformsManager {}

impl TransformsManager {
    pub async fn insert_transform(
        con: &mut MultiplexedConnection,
        transform: &SPTransformStamped,
    ) -> Result<(), Box<dyn Error>> {
        insert_transform::insert_transform(con, &transform).await
    }

    pub async fn insert_transforms(
        con: &mut MultiplexedConnection,
        transforms: &Vec<SPTransformStamped>,
    ) -> Result<(), Box<dyn Error>> {
        insert_transforms::insert_transforms(con, &transforms).await
    }

    pub async fn remove_transform(
        con: &mut MultiplexedConnection,
        key: &str,
    ) -> Result<(), Box<dyn Error>> {
        remove_transform::remove_transform(con, &key).await
    }

    pub async fn get_all_transforms(
        con: &mut MultiplexedConnection,
    ) -> Result<HashMap<String, SPTransformStamped>, Box<dyn Error>> {
        get_all_transforms::get_all_transforms(con).await
    }

    // TODO: return something
    pub async fn move_transform(
        con: &mut MultiplexedConnection,
        name: &str,
        new_transform: SPTransform,
    ) -> Result<(), Box<dyn Error>> {
        move_transform::move_transform(con, name, new_transform).await
    }

    pub async fn reparent_transform(
        con: &mut MultiplexedConnection,
        new_parent_frame_id: &str,
        child_frame_id: &str,
    ) -> Result<(), Box<dyn Error>> {
        reparent_transform::reparent_transform(con, new_parent_frame_id, child_frame_id).await
    }

    pub async fn lookup_transform(
        con: &mut MultiplexedConnection,
        parent_frame_id: &str,
        child_frame_id: &str,
    ) -> Result<SPTransformStamped, Box<dyn Error>> {
        lookup_transform::lookup_transform(con, parent_frame_id, child_frame_id).await
    }

    pub async fn load_transforms_from_path(
        con: &mut MultiplexedConnection,
        path: &str,
    ) -> Result<(), Box<dyn Error>> {
        load_transforms_from_path::load_transforms_from_path(con, path).await
    }

    // TODO: CLONE TRANSFORM
}
