//! Resolving the transform between two frames.
//!
//! A buffer of frames is a tree of parent-child poses. To get from any frame to
//! any other, walk up from the parent to the root and back down to the child,
//! then multiply the poses along the way -
//! [`lookup_transform_with_root`] is that walk. Poses are converted to
//! `nalgebra` isometries to do the maths and back again.

use crate::{
    is_cyclic_all, MapOrUnknown, SPRotation, SPTransform, SPTransformStamped, SPTranslation,
};
use nalgebra::{Isometry3, Quaternion, UnitQuaternion, Vector3};
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::time::SystemTime;

/// Hard ceiling on how many frames a lookup may traverse.
///
/// A bound, not a cycle detector: a well-formed chain longer than this is
/// refused too. It is what stops a malformed buffer from walking forever.
pub static MAX_TRANSFORM_CHAIN: u64 = 1000;

/// Compose a chain of isometries, in order, into a single one. An empty chain
/// gives the identity.
pub fn isometry_chain_product(vec: Vec<Isometry3<f64>>) -> Isometry3<f64> {
    vec.iter().fold(Isometry3::identity(), |a, &b| a * b)
}

/// Convert a stored [`SPTransform`] into the `nalgebra` isometry the maths runs on.
pub fn sp_transform_to_isometry(sp_transform: SPTransform) -> Isometry3<f64> {
    let translation = Vector3::new(
        sp_transform.translation.x.into_inner(),
        sp_transform.translation.y.into_inner(),
        sp_transform.translation.z.into_inner(),
    );
    let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
        sp_transform.rotation.w.into_inner(),
        sp_transform.rotation.x.into_inner(),
        sp_transform.rotation.y.into_inner(),
        sp_transform.rotation.z.into_inner(),
    ));

    Isometry3::from_parts(translation.into(), rotation)
}

/// Convert a `nalgebra` isometry back into the storable [`SPTransform`] form.
pub fn isometry_to_sp_transform(isometry: Isometry3<f64>) -> SPTransform {
    let translation_vector: &Vector3<f64> = &isometry.translation.vector;
    let rotation_quaternion: &Quaternion<f64> = isometry.rotation.quaternion();

    let sp_translation = SPTranslation {
        x: OrderedFloat(translation_vector.x),
        y: OrderedFloat(translation_vector.y),
        z: OrderedFloat(translation_vector.z),
    };

    let sp_rotation = SPRotation {
        w: OrderedFloat(rotation_quaternion.w),
        x: OrderedFloat(rotation_quaternion.i),
        y: OrderedFloat(rotation_quaternion.j),
        z: OrderedFloat(rotation_quaternion.k),
    };

    SPTransform {
        translation: sp_translation,
        rotation: sp_rotation,
    }
}

/// Resolve the pose of `child_frame_id` expressed in `parent_frame_id`.
///
/// Goes up from the parent to `root_frame_id` and down again to the child, so
/// the two frames need not be directly related. Returns `None` if `buffer`
/// contains a cycle, if either leg of the walk cannot reach the root, or if the
/// walk exceeds [`MAX_TRANSFORM_CHAIN`]. The result is stamped with the current
/// time and carries no metadata.
///
/// ```
/// use micro_sp::*;
/// use std::collections::HashMap;
/// use std::time::SystemTime;
///
/// fn frame(child: &str, parent: &str, x: f64) -> SPTransformStamped {
///     let mut transform = SPTransform::default();
///     transform.translation.x = ordered_float::OrderedFloat(x);
///     SPTransformStamped {
///         active_transform: true,
///         enable_transform: true,
///         time_stamp: SystemTime::now(),
///         parent_frame_id: parent.to_string(),
///         child_frame_id: child.to_string(),
///         transform,
///         metadata: MapOrUnknown::UNKNOWN,
///     }
/// }
///
/// // world -> base (1 m along x) -> tool (another 2 m along x)
/// let mut buffer = HashMap::new();
/// buffer.insert("base".to_string(), frame("base", "world", 1.0));
/// buffer.insert("tool".to_string(), frame("tool", "base", 2.0));
///
/// let tool_in_world =
///     lookup_transform_with_root("world", "tool", "world", &buffer).unwrap();
/// assert_eq!(tool_in_world.transform.translation.x.into_inner(), 3.0);
/// ```
pub fn lookup_transform_with_root(
    parent_frame_id: &str,
    child_frame_id: &str,
    root_frame_id: &str,
    buffer: &HashMap<String, SPTransformStamped>,
) -> Option<SPTransformStamped> {
    let buffer_local = buffer.clone();
    let mut chain = vec![];
    if !is_cyclic_all(&buffer_local) {
        match parent_to_root(parent_frame_id, root_frame_id, &buffer_local) {
            Some(up_chain) => match root_to_child(child_frame_id, root_frame_id, &buffer_local) {
                Some(down_chain) => {
                    chain.push(up_chain);
                    chain.push(down_chain);
                    let iso_3 = isometry_chain_product(chain);
                    Some(SPTransformStamped {
                        active_transform: buffer_local.get(child_frame_id).unwrap().active_transform,
                        enable_transform: buffer_local.get(child_frame_id).unwrap().enable_transform,
                        time_stamp: SystemTime::now(),
                        parent_frame_id: parent_frame_id.to_string(),
                        child_frame_id: child_frame_id.to_string(),
                        transform: isometry_to_sp_transform(iso_3),
                        metadata: MapOrUnknown::UNKNOWN,
                    })
                }
                None => None,
            },
            None => None,
        }
    } else {
        None
    }
}

/// Walk upstream from `parent_frame_id` to `root_frame_id`, composing the
/// *inverse* of each pose along the way.
///
/// The identity when the two ids are equal. `None` if a link in the chain is
/// missing from `buffer`, or if the walk exceeds [`MAX_TRANSFORM_CHAIN`]; both
/// are logged.
pub fn parent_to_root(
    parent_frame_id: &str,
    root_frame_id: &str,
    buffer: &HashMap<String, SPTransformStamped>,
) -> Option<Isometry3<f64>> {
    let mut current_parent = parent_frame_id.to_string();
    let mut path = vec![];
    let mut length = 0;

    if parent_frame_id == root_frame_id {
        return Some(Isometry3::identity());
    }

    let res = loop {
        if length >= MAX_TRANSFORM_CHAIN {
            log::error!(target: "transform_lookup", "Max transform chain exceeded.");
            break None;
        } else {
            length = length + 1;
            match buffer.get(&current_parent) {
                Some(parent) => {
                    path.push(sp_transform_to_isometry(parent.transform.clone()).inverse());
                    if parent.parent_frame_id == root_frame_id {
                        break Some(path);
                    } else {
                        current_parent = parent.parent_frame_id.to_string();
                    }
                }
                None => {
                    log::error!(target: "transform_lookup", "Failed to get parent for: {current_parent}.");
                    break None
                },
            }
        }
    };

    match res {
        Some(chain) => Some(isometry_chain_product(chain)),
        None => None,
    }
}

/// Breadth-first search downstream from `root_frame_id` to `child_frame_id`,
/// composing the poses along the path.
///
/// `None` if the child is not reachable from the root, or if the search exceeds
/// [`MAX_TRANSFORM_CHAIN`]; both are logged.
pub fn root_to_child(
    child_frame_id: &str,
    root_frame_id: &str,
    buffer: &HashMap<String, SPTransformStamped>,
) -> Option<Isometry3<f64>> {
    let mut length = 0;
    let mut stack = vec![];
    get_frame_children(root_frame_id, buffer)
        .iter()
        .for_each(|(k, v)| {
            stack.push((
                k.to_string(),
                vec![k.to_string()],
                vec![v.transform.clone()],
            ))
        });

    let res = loop {
        if length >= MAX_TRANSFORM_CHAIN {
            log::error!(target: "transform_lookup", "Max transform chain exceeded.");
            break None;
        } else {
            length = length + 1;
            match stack.pop() {
                Some((frame, path, chain)) => {
                    if frame == child_frame_id {
                        break Some(chain);
                    } else {
                        get_frame_children(&frame, buffer)
                            .iter()
                            .for_each(|(k, v)| {
                                let mut prev_path = path.clone();
                                let mut prev_chain = chain.clone();
                                prev_path.push(k.clone());
                                prev_chain.push(v.transform.clone());
                                stack.insert(
                                    0,
                                    (k.to_string(), prev_path.clone(), prev_chain.clone()),
                                )
                            })
                    }
                }
                None => {
                    log::error!(target: "transform_lookup", "No frames in the stack.");
                    log::error!(target: "transform_lookup", "Couldn't find transform to '{}'.", child_frame_id);
                    break None
                }
            }
        }
    };

    match res {
        Some(chain) => Some(isometry_chain_product(
            chain
                .iter()
                .map(|x| sp_transform_to_isometry(x.clone()))
                .collect(),
        )),
        None => None,
    }
}

/// Every `(child_id, frame)` in `buffer` whose parent is `frame`.
///
/// `frame` itself need not exist in the buffer - the root of a tree usually does
/// not, since nothing declares it as a child.
pub fn get_frame_children(
    frame: &str,
    buffer: &HashMap<String, SPTransformStamped>,
) -> Vec<(String, SPTransformStamped)> {
    buffer
        .iter()
        .filter(|(_, v)| v.parent_frame_id == frame)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

#[cfg(test)]
mod tests {

    use nalgebra::{Isometry3, Quaternion, Translation, UnitQuaternion, Vector3};
    use std::collections::HashMap;
    use std::time::SystemTime;

    use crate::*;

    #[test]
    fn test_simple_direct_child() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "child".to_string(),
            create_transform("root", "child", Isometry3::translation(1.0, 0.0, 0.0)),
        );

        let result = root_to_child("child", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        let expected_transform = Isometry3::translation(1.0, 0.0, 0.0);
        assert_eq!(transform.translation, expected_transform.translation);
    }

    // Test 2: Intermediate Frames
    #[test]
    fn test_intermediate_frames() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "intermediate".to_string(),
            create_transform(
                "root",
                "intermediate",
                Isometry3::translation(1.0, 1.0, 0.0),
            ),
        );
        buffer.insert(
            "child".to_string(),
            create_transform(
                "intermediate",
                "child",
                Isometry3::translation(1.0, 0.0, 1.0),
            ),
        );

        let result = root_to_child("child", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        let expected_transform = Isometry3::translation(2.0, 1.0, 1.0);
        assert_eq!(transform.translation, expected_transform.translation);
    }

    // Test 3: Complex Chain with Multiple Branches
    #[test]
    fn test_complex_chain_with_multiple_branches() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "intermediate1".to_string(),
            create_transform(
                "root",
                "intermediate1",
                Isometry3::translation(1.0, 0.0, 0.0),
            ),
        );
        buffer.insert(
            "intermediate2".to_string(),
            create_transform(
                "intermediate1",
                "intermediate2",
                Isometry3::translation(0.0, 1.0, 0.0),
            ),
        );
        buffer.insert(
            "branch".to_string(),
            create_transform(
                "intermediate1",
                "branch",
                Isometry3::translation(0.0, 0.0, 1.0),
            ),
        );
        buffer.insert(
            "child".to_string(),
            create_transform(
                "intermediate2",
                "child",
                Isometry3::translation(1.0, 1.0, 1.0),
            ),
        );

        let result = root_to_child("child", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        let expected_transform = Isometry3::translation(2.0, 2.0, 1.0);
        assert_eq!(transform.translation, expected_transform.translation);
    }

    #[test]
    fn test_simple_direct_parent() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "child".to_string(),
            create_transform("root", "child", Isometry3::translation(1.0, 0.0, 0.0)),
        );

        let result = parent_to_root("child", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        let expected_transform = Isometry3::translation(-1.0, 0.0, 0.0); // Inverse of the translation
        assert_eq!(transform.translation, expected_transform.translation);
    }

    // Test 2: Intermediate Frames
    #[test]
    fn test_intermediate_frames_2() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "intermediate".to_string(),
            create_transform(
                "root",
                "intermediate",
                Isometry3::translation(1.0, 1.0, 0.0),
            ),
        );
        buffer.insert(
            "child".to_string(),
            create_transform(
                "intermediate",
                "child",
                Isometry3::translation(1.0, 0.0, 1.0),
            ),
        );

        let result = parent_to_root("child", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        let expected_transform = Isometry3::translation(-2.0, -1.0, -1.0); // Inverse of the combined translation
        assert_eq!(transform.translation, expected_transform.translation);
    }

    // Test 3: Complex Chain with Multiple Branches
    #[test]
    fn test_complex_chain_with_multiple_branches_2() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "intermediate1".to_string(),
            create_transform(
                "root",
                "intermediate1",
                Isometry3::translation(1.0, 0.0, 0.0),
            ),
        );
        buffer.insert(
            "intermediate2".to_string(),
            create_transform(
                "intermediate1",
                "intermediate2",
                Isometry3::translation(0.0, 1.0, 0.0),
            ),
        );
        buffer.insert(
            "branch".to_string(),
            create_transform(
                "intermediate1",
                "branch",
                Isometry3::translation(0.0, 0.0, 1.0),
            ),
        );
        buffer.insert(
            "child".to_string(),
            create_transform(
                "intermediate2",
                "child",
                Isometry3::translation(1.0, 1.0, 1.0),
            ),
        );

        let result = parent_to_root("child", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        let expected_transform = Isometry3::translation(-2.0, -2.0, -1.0); // Inverse of the chosen path
        assert_eq!(transform.translation, expected_transform.translation);
    }

    #[test]
    fn test_complex_transform_chain() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "frame1".to_string(),
            create_transform("root", "frame1", Isometry3::translation(1.0, 2.0, 0.0)),
        );
        buffer.insert(
            "frame2".to_string(),
            create_transform("frame1", "frame2", Isometry3::translation(0.0, 3.0, 1.0)),
        );
        buffer.insert(
            "frame3".to_string(),
            create_transform("frame2", "frame3", Isometry3::translation(2.0, 0.0, -1.0)),
        );

        let result = lookup_transform_with_root("frame1", "frame3", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        assert_eq!(transform.parent_frame_id, "frame1");
        assert_eq!(transform.child_frame_id, "frame3");

        // The expected transform is the result of the chain: T1 -> T2 -> T3
        let expected_transform = isometry_to_sp_transform(Isometry3::translation(2.0, 3.0, 0.0));
        assert_eq!(
            transform.transform.translation,
            expected_transform.translation
        );
    }

    // Test 5: Multiple Intermediate Frames
    #[test]
    fn test_multiple_intermediate_frames() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "frameA".to_string(),
            create_transform("root", "frameA", Isometry3::translation(1.0, 1.0, 1.0)),
        );
        buffer.insert(
            "frameB".to_string(),
            create_transform("frameA", "frameB", Isometry3::translation(1.0, 0.0, 0.0)),
        );
        buffer.insert(
            "frameC".to_string(),
            create_transform("frameB", "frameC", Isometry3::translation(0.0, 2.0, 0.0)),
        );
        buffer.insert(
            "frameD".to_string(),
            create_transform("frameC", "frameD", Isometry3::translation(0.0, 0.0, 3.0)),
        );

        let result = lookup_transform_with_root("root", "frameD", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        assert_eq!(transform.parent_frame_id, "root");
        assert_eq!(transform.child_frame_id, "frameD");

        // The expected transform is the result of the chain: T1 -> T2 -> T3 -> T4
        let expected_transform = isometry_to_sp_transform(Isometry3::translation(2.0, 3.0, 4.0));
        assert_eq!(
            transform.transform.translation,
            expected_transform.translation
        );
    }

    // Test 6: Mixed Transformations with Rotations
    #[test]
    fn test_mixed_transformations_with_rotations() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "frame1".to_string(),
            create_transform("root", "frame1", Isometry3::translation(0.0, 0.0, 1.0)),
        );

        let rot = Isometry3::rotation(Vector3::new(0.5, 0.0, 0.0));
        let rot2 = Isometry3::rotation(Vector3::new(0.5, 0.5, 0.0));

        buffer.insert(
            "frame2".to_string(),
            create_transform("frame1", "frame2", rot), // Assume rotation around X-axis
        );
        buffer.insert(
            "frame3".to_string(),
            create_transform("frame2", "frame3", Isometry3::translation(1.0, 0.0, 0.0)),
        );
        buffer.insert(
            "frame4".to_string(),
            create_transform("frame3", "frame4", rot2),
        );

        let result = lookup_transform_with_root("frame1", "frame4", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        assert_eq!(transform.parent_frame_id, "frame1");
        assert_eq!(transform.child_frame_id, "frame4");

        // The expected transform combines translation and rotation
        let expected_translation = isometry_to_sp_transform(Isometry3::translation(1.0, 0.0, 0.0));
        assert_eq!(transform.transform.translation, expected_translation.translation);
        println!("{:?}", transform.transform.rotation);
    }

    #[test]
    fn test_parent_to_root() {
        let test_buffer = HashMap::from([
            (
                "finger".to_string(),
                SPTransformStamped {
                    active_transform: true,
                    enable_transform: true,
                    time_stamp: SystemTime::now(),
                    child_frame_id: "finger".to_string(),
                    parent_frame_id: "hand".to_string(),
                    transform: isometry_to_sp_transform(Isometry3 {
                        translation: Translation {
                            vector: Vector3::new(0.0, 0.0, 0.0),
                        },
                        rotation: UnitQuaternion::from_quaternion(Quaternion::new(
                            1.0, 0.0, 0.0, 0.0,
                        )),
                    }),
                    metadata: MapOrUnknown::UNKNOWN,
                },
            ),
            (
                "hand".to_string(),
                SPTransformStamped {
                    active_transform: true,
                    enable_transform: true,
                    time_stamp: SystemTime::now(),
                    child_frame_id: "hand".to_string(),
                    parent_frame_id: "elbow".to_string(),
                    transform: isometry_to_sp_transform(Isometry3 {
                        translation: Translation {
                            vector: Vector3::new(1.0, 0.0, 0.0),
                        },
                        rotation: UnitQuaternion::from_quaternion(Quaternion::new(
                            0.7071, 0.7071, 0.0, 0.0,
                        )),
                    }),
                    metadata: MapOrUnknown::UNKNOWN
                },
            ),
            (
                "elbow".to_string(),
                SPTransformStamped {
                    active_transform: true,
                    enable_transform: true,
                    time_stamp: SystemTime::now(),
                    child_frame_id: "elbow".to_string(),
                    parent_frame_id: "shoulder".to_string(),
                    transform: isometry_to_sp_transform(Isometry3 {
                        translation: Translation {
                            vector: Vector3::new(0.0, 1.0, 0.0),
                        },
                        rotation: UnitQuaternion::from_quaternion(Quaternion::new(
                            0.7071, 0.0, 0.7071, 0.0,
                        )),
                    }),
                    metadata: MapOrUnknown::UNKNOWN
                },
            ),
            (
                "shoulder".to_string(),
                SPTransformStamped {
                    active_transform: false,
                    enable_transform: true,
                    time_stamp: SystemTime::now(),
                    child_frame_id: "shoulder".to_string(),
                    parent_frame_id: "world".to_string(),
                    transform: isometry_to_sp_transform(Isometry3 {
                        translation: Translation {
                            vector: Vector3::new(0.0, 0.0, 1.0),
                        },
                        rotation: UnitQuaternion::from_quaternion(Quaternion::new(
                            0.7071, 0.0, 0.0, 0.7071,
                        )),
                    }),
                    metadata: MapOrUnknown::UNKNOWN
                },
            ),
        ]);

        let res = parent_to_root("hand", "world", &test_buffer);
        assert!(!res.is_none());
        println!("{}", res.unwrap());
        // TODO: verify if this is correct and test
    }

    fn dummy_1_frame() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: false,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "world".to_string(),
            child_frame_id: "dummy_1".to_string(),
            transform: isometry_to_sp_transform(Isometry3::default()),
            metadata: MapOrUnknown::UNKNOWN
        }
    }

    fn dummy_2_frame() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "dummy_1".to_string(),
            child_frame_id: "dummy_2".to_string(),
            transform: isometry_to_sp_transform(Isometry3::default()),
            metadata: MapOrUnknown::UNKNOWN
        }
    }

    fn dummy_3_frame() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: "dummy_1".to_string(),
            child_frame_id: "dummy_3".to_string(),
            transform: isometry_to_sp_transform(Isometry3::default()),
            metadata: MapOrUnknown::UNKNOWN
        }
    }

    #[test]
    fn test_get_frame_children() {
        let mut buffer = HashMap::<String, SPTransformStamped>::new();
        buffer.insert("dummy_1".to_string(), dummy_1_frame());

        //          w
        //          |
        //          d1

        assert_eq!(
            get_frame_children("world", &buffer)
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<String>>(),
            vec!("dummy_1")
        );

        buffer.insert("dummy_2".to_string(), dummy_2_frame());

        //          w
        //          |
        //          d1
        //          |
        //          d2

        assert_eq!(
            get_frame_children("dummy_1", &buffer)
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<String>>(),
            vec!("dummy_2")
        );

        assert_eq!(
            get_frame_children("world", &buffer)
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<String>>(),
            vec!("dummy_1")
        );

        assert_eq!(
            get_frame_children("dummy_2", &buffer)
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<String>>(),
            Vec::<String>::new()
        );

        buffer.insert("dummy_3".to_string(), dummy_3_frame());

        //          w
        //          |
        //          d1
        //         /  \
        //       d2    d3

        assert_eq!(
            get_frame_children("world", &buffer)
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<String>>()
                .sort(),
            vec!("dummy_2", "dummy_3").sort()
        );
    }

    fn create_transform(
        parent_frame_id: &str,
        child_frame_id: &str,
        transform: Isometry3<f64>,
    ) -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: parent_frame_id.to_string(),
            child_frame_id: child_frame_id.to_string(),
            transform: isometry_to_sp_transform(transform),
            metadata: MapOrUnknown::UNKNOWN
        }
    }

    // Successful Transform Lookup
    #[test]
    fn test_successful_transform_lookup() {
        let mut buffer = HashMap::new();
        buffer.insert(
            "parent".to_string(),
            create_transform("root", "parent", Isometry3::translation(1.0, 0.0, 0.0)),
        );
        buffer.insert(
            "child".to_string(),
            create_transform("parent", "child", Isometry3::translation(0.0, 1.0, 0.0)),
        );

        let result = lookup_transform_with_root("parent", "child", "root", &buffer);

        assert!(result.is_some());
        let transform = result.unwrap();
        assert_eq!(transform.parent_frame_id, "parent");
        assert_eq!(transform.child_frame_id, "child");

        // We expect the result to be a combined transform of (1, 1, 0)
        let expected_transform = Isometry3::translation(0.0, 1.0, 0.0);
        assert_eq!(
            transform.transform.translation,
            isometry_to_sp_transform(expected_transform).translation
        );
    }
}

/// Lookup failure and depth limits.
///
/// The happy paths - direct parent, direct child, chains, branches - are well
/// covered above. What is not is what happens when the buffer is *broken*: a
/// frame whose parent is not in the buffer, and a chain longer than
/// `MAX_TRANSFORM_CHAIN`. Both matter because the buffer is assembled from JSON
/// files and from whatever other processes have written into Redis, so a
/// dangling parent is a realistic state, and the depth limit is the only thing
/// standing between a malformed buffer and an unbounded walk.
#[cfg(test)]
mod failure_tests {
    use crate::*;
    use nalgebra::Isometry3;
    use std::collections::HashMap;
    use std::time::SystemTime;

    fn transform(child: &str, parent: &str, x: f64) -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: parent.to_string(),
            child_frame_id: child.to_string(),
            transform: SPTransform {
                translation: SPTranslation {
                    x: ordered_float::OrderedFloat(x),
                    y: ordered_float::OrderedFloat(0.0),
                    z: ordered_float::OrderedFloat(0.0),
                },
                rotation: SPTransform::default().rotation,
            },
            metadata: MapOrUnknown::UNKNOWN,
        }
    }

    fn buffer(edges: &[(&str, &str)]) -> HashMap<String, SPTransformStamped> {
        edges
            .iter()
            .map(|(child, parent)| (child.to_string(), transform(child, parent, 1.0)))
            .collect()
    }

    /// Walking up to a root that is not reachable - because a link in the chain
    /// is missing from the buffer - fails rather than looping or returning a
    /// partial chain.
    #[test]
    fn walking_to_a_root_that_is_not_there_fails() {
        // b -> a, but 'a' itself is not in the buffer, so the walk from b
        // cannot reach 'world'.
        let buffer = buffer(&[("b", "a")]);
        assert_eq!(parent_to_root("b", "world", &buffer), None);
    }

    /// A frame is its own root - the identity case the walk short-circuits on
    /// before it does any work at all.
    #[test]
    fn a_frame_is_its_own_root() {
        let buffer = buffer(&[("a", "world")]);
        assert_eq!(parent_to_root("world", "world", &buffer), Some(Isometry3::identity()));
    }

    /// A cycle in the buffer would walk forever; `MAX_TRANSFORM_CHAIN` is what
    /// stops it. `TransformsManager` refuses to create one (see
    /// `transforms::cycles`), so this is the belt to that braces - and the only
    /// thing that saves the lookup if a cycle ever arrives another way.
    #[test]
    fn a_cycle_is_stopped_by_the_chain_limit_rather_than_looping_forever() {
        let buffer = buffer(&[("a", "b"), ("b", "a")]);

        // Neither direction can reach an outside root, and neither hangs.
        assert_eq!(parent_to_root("a", "world", &buffer), None);
        assert_eq!(parent_to_root("b", "world", &buffer), None);
    }

    /// A chain longer than the limit is refused even though it is perfectly
    /// well formed - the limit is a hard bound, not a cycle detector.
    #[test]
    fn a_chain_longer_than_the_limit_is_refused() {
        let mut buffer = HashMap::new();
        let depth = MAX_TRANSFORM_CHAIN + 5;
        for i in 0..depth {
            let parent = if i == 0 {
                "world".to_string()
            } else {
                format!("f{}", i - 1)
            };
            let child = format!("f{}", i);
            buffer.insert(child.clone(), transform(&child, &parent, 1.0));
        }

        let deepest = format!("f{}", depth - 1);
        assert_eq!(
            parent_to_root(&deepest, "world", &buffer),
            None,
            "a chain of {depth} must hit the limit"
        );

        // And a chain just inside the limit still resolves.
        assert!(parent_to_root("f2", "world", &buffer).is_some());
    }

    /// A lookup between two frames with no common root fails rather than
    /// returning an arbitrary transform.
    #[test]
    fn a_lookup_across_two_disconnected_trees_fails() {
        let buffer = buffer(&[("a", "world"), ("x", "other_world")]);
        assert!(lookup_transform_with_root("a", "x", "world", &buffer).is_none());
    }

    /// A lookup naming a frame that is not in the buffer at all fails.
    #[test]
    fn a_lookup_of_an_unknown_frame_fails() {
        let buffer = buffer(&[("a", "world")]);
        assert!(lookup_transform_with_root("world", "nope", "world", &buffer).is_none());
        assert!(lookup_transform_with_root("nope", "a", "world", &buffer).is_none());
    }

    /// `lookup_transform_with_root` refuses outright, before walking anything,
    /// when the buffer itself contains a cycle - a mutual parent/child pair
    /// here, which `is_cyclic_all` flags regardless of which two frames are
    /// actually being looked up.
    #[test]
    fn a_lookup_over_a_cyclic_buffer_fails() {
        let buffer = buffer(&[("a", "b"), ("b", "a")]);
        assert!(
            lookup_transform_with_root("a", "b", "world", &buffer).is_none(),
            "a cyclic buffer must short-circuit to None rather than walk"
        );
    }

    /// `root_to_child`'s BFS has the same `MAX_TRANSFORM_CHAIN` bound as
    /// `parent_to_root` - a straight chain longer than the limit must fail
    /// rather than search forever, and a chain just inside it still resolves.
    #[test]
    fn root_to_child_is_stopped_by_the_chain_limit_rather_than_looping_forever() {
        let mut buf = HashMap::new();
        let depth = MAX_TRANSFORM_CHAIN + 5;
        for i in 0..depth {
            let parent = if i == 0 {
                "world".to_string()
            } else {
                format!("f{}", i - 1)
            };
            let child = format!("f{}", i);
            buf.insert(child.clone(), transform(&child, &parent, 1.0));
        }

        let deepest = format!("f{}", depth - 1);
        assert_eq!(
            root_to_child(&deepest, "world", &buf),
            None,
            "a chain of {depth} must hit the limit"
        );

        // And a chain just inside the limit still resolves.
        assert!(root_to_child("f2", "world", &buf).is_some());
    }
}
