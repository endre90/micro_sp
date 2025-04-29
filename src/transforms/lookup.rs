use crate::{
    is_cyclic_all, MapOrUnknown, SPRotation, SPTransform, SPTransformStamped, SPTranslation,
};
use nalgebra::{Isometry3, Quaternion, UnitQuaternion, Vector3};
use ordered_float::OrderedFloat;
// use serde_sp_transform::Value;
use std::collections::HashMap;
use std::time::SystemTime;

// use tokio::time::Instant;

pub static MAX_TRANSFORM_CHAIN: u64 = 1000;

pub fn isometry_chain_product(vec: Vec<Isometry3<f64>>) -> Isometry3<f64> {
    vec.iter().fold(Isometry3::identity(), |a, &b| a * b)
}

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

pub fn lookup_transform_with_root(
    parent_frame_id: &str,
    child_frame_id: &str,
    root_frame_id: &str,
    buffer: &HashMap<String, SPTransformStamped>,
) -> Option<SPTransformStamped> {
    // let buffer_local = buffer.lock().unwrap().clone();
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

// Go upstream to the root
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
                None => break None,
            }
        }
    };

    match res {
        Some(chain) => Some(isometry_chain_product(chain)),
        None => None,
    }
}

// BFS to get the path to the child
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
                None => break None,
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

// The frame whose children we are searching for don't have to exist in the transform buffer
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
    // use serde_sp_transform::Value;
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
