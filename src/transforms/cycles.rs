//! Keeping the transform tree a tree.
//!
//! A parent-child cycle would make a lookup walk forever, so every call that
//! changes a frame's parent asks [`check_would_produce_cycle`] first, and
//! [`crate::lookup_transform_with_root`] refuses a buffer that already contains
//! one.

use transforms::lookup::get_frame_children;

use crate::*;
use std::collections::HashMap;

/// Whether the tree segment reachable from `frame` contains a cycle.
///
/// Walks downwards through children only, so it sees nothing above `frame`; use
/// [`is_cyclic_all`] to cover a buffer whose tree is segmented.
pub fn is_cyclic(frame: &str, buffer: &HashMap<String, SPTransformStamped>) -> bool {
    let mut stack = vec![frame.to_string()];
    let mut visited = vec![];

    loop {
        match stack.pop() {
            Some(current_frame) => {
                if visited.contains(&current_frame) && buffer.contains_key(&current_frame) {
                    break true;
                } else {
                    visited.push(current_frame.clone());

                    for child in get_frame_children(&current_frame, buffer) {
                        stack.push(child.1.child_frame_id);
                    }
                }
            }
            None => break false,
        }
    }
}

/// Whether `frames` contains a cycle anywhere.
///
/// Starts a walk from every frame in turn, so a segmented tree - several
/// disconnected roots in one buffer - is covered too.
pub fn is_cyclic_all(frames: &HashMap<String, SPTransformStamped>) -> bool {
    for (k, _) in frames {
        if is_cyclic(k, frames) {
            return true;
        } else {
            continue;
        }
    }
    false
}

/// Whether inserting `frame` into `buffer` would create a cycle.
///
/// The guard on every write path that sets a frame's parent. `buffer` is left
/// untouched - the check is done on a copy.
///
/// ```
/// use micro_sp::*;
/// use std::collections::HashMap;
/// use std::time::SystemTime;
///
/// fn frame(child: &str, parent: &str) -> SPTransformStamped {
///     SPTransformStamped {
///         active_transform: true,
///         enable_transform: true,
///         time_stamp: SystemTime::now(),
///         parent_frame_id: parent.to_string(),
///         child_frame_id: child.to_string(),
///         transform: SPTransform::default(),
///         metadata: MapOrUnknown::UNKNOWN,
///     }
/// }
///
/// // world -> a -> b
/// let mut buffer = HashMap::new();
/// buffer.insert("a".to_string(), frame("a", "world"));
/// buffer.insert("b".to_string(), frame("b", "a"));
/// assert!(!is_cyclic_all(&buffer));
///
/// // Reparenting 'a' under its own descendant 'b' closes the loop.
/// assert!(check_would_produce_cycle(&frame("a", "b"), &buffer));
/// // ... while hanging a fresh frame off 'b' is fine.
/// assert!(!check_would_produce_cycle(&frame("c", "b"), &buffer));
/// ```
pub fn check_would_produce_cycle(
    frame: &SPTransformStamped,
    buffer: &HashMap<String, SPTransformStamped>,
) -> bool {
    let mut buffer_local = buffer.clone();
    buffer_local.insert(frame.child_frame_id.clone(), frame.clone());
    is_cyclic_all(&buffer_local)
}

#[cfg(test)]
mod tests {

    use nalgebra::Isometry3;
    use transforms::cycles::{check_would_produce_cycle, is_cyclic, is_cyclic_all};
    use std::{collections::HashMap, time::SystemTime};

    use crate::*;

    fn dummy_1_frame() -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
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
            parent_frame_id: "dummy_2".to_string(),
            child_frame_id: "dummy_3".to_string(),
            transform: isometry_to_sp_transform(Isometry3::default()),
            metadata: MapOrUnknown::UNKNOWN
        }
    }

    #[test]
    fn test_is_not_cyclic() {
        let mut buffer = HashMap::<String, SPTransformStamped>::new();
        buffer.insert("dummy_1".to_string(), dummy_1_frame());

        //          w
        //          |
        //          d1

        let res = is_cyclic("dummy_1", &buffer);
        assert_eq!(res, false);

        buffer.insert("dummy_2".to_string(), dummy_2_frame());

        //          w
        //          |
        //          d1
        //          |
        //          d2

        let res = is_cyclic("world", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_1", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_2", &buffer);
        assert_eq!(res, false);
    }

    #[test]
    fn test_is_cyclic() {
        let mut buffer = HashMap::<String, SPTransformStamped>::new();
        buffer.insert("dummy_1".to_string(), dummy_1_frame());
        buffer.insert("dummy_2".to_string(), dummy_2_frame());
        buffer.insert(
            "dummy_1".to_string(),
            SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "dummy_2".to_string(),
                child_frame_id: "dummy_1".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            },
        );

        //          w
        //          
        //          d1
        //          ||
        //          d2

        let res = is_cyclic("world", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_1", &buffer);
        assert_eq!(res, true);
        let res = is_cyclic("dummy_2", &buffer);
        assert_eq!(res, true);
    }

    #[test]
    fn test_is_cyclic_triangle() {
        let mut buffer = HashMap::<String, SPTransformStamped>::new();
        buffer.insert("dummy_1".to_string(), dummy_1_frame());
        buffer.insert("dummy_2".to_string(), dummy_2_frame());
        buffer.insert("dummy_3".to_string(), dummy_3_frame());

        //          w
        //          |
        //          d1
        //         /
        //       d2 -- d3

        let res = is_cyclic("world", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_1", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_2", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_3", &buffer);
        assert_eq!(res, false);

        buffer.insert(
            "dummy_1".to_string(),
            SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "dummy_3".to_string(),
                child_frame_id: "dummy_1".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            },
        );

        //          w
        //          
        //          d1
        //         /  \
        //       d2 -- d3

        let res = is_cyclic("world", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_1", &buffer);
        assert_eq!(res, true);
        let res = is_cyclic("dummy_2", &buffer);
        assert_eq!(res, true);
        let res = is_cyclic("dummy_3", &buffer);
        assert_eq!(res, true);
    }


    #[test]
    fn test_is_cyclic_all() {
        let mut buffer = HashMap::<String, SPTransformStamped>::new();
        buffer.insert("dummy_1".to_string(), dummy_1_frame());
        buffer.insert("dummy_2".to_string(), dummy_2_frame());
        buffer.insert("dummy_3".to_string(), dummy_3_frame());

        //          w
        //          |
        //          d1
        //         /
        //       d2 -- d3

        let res = is_cyclic_all(&buffer);
        assert_eq!(res, false);

        buffer.insert(
            "dummy_5".to_string(),
            SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "dummy_4".to_string(),
                child_frame_id: "dummy_5".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            },
        );

        buffer.insert(
            "dummy_6".to_string(),
            SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "dummy_5".to_string(),
                child_frame_id: "dummy_6".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            },
        );

        //          w           d4
        //          |           |
        //          d1          d5
        //         /            |
        //       d2 -- d3       d6

        let res = is_cyclic_all(&buffer);
        assert_eq!(res, false);

        buffer.insert(
            "dummy_4".to_string(),
            SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "dummy_6".to_string(),
                child_frame_id: "dummy_4".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            },
        );

        //          w           d4
        //          |          /  \
        //          d1       d5 -- d6
        //         /            
        //       d2 -- d3       

        let res = is_cyclic_all(&buffer);
        assert_eq!(res, true);

        buffer.insert(
            "dummy_4".to_string(),
            SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "world".to_string(),
                child_frame_id: "dummy_4".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            },
        );

        //          w --------- d4
        //          |          /  
        //          d1       d5 -- d6
        //         /            
        //       d2 -- d3   

        let res = is_cyclic_all(&buffer);
        assert_eq!(res, false);
    }

    #[test]
    fn test_would_produce_cycle() {
        let mut buffer = HashMap::<String, SPTransformStamped>::new();
        buffer.insert("dummy_1".to_string(), dummy_1_frame());
        buffer.insert("dummy_2".to_string(), dummy_2_frame());
        buffer.insert("dummy_3".to_string(), dummy_3_frame());

        //          w
        //          |
        //          d1
        //         /
        //       d2 -- d3

        let res = is_cyclic("world", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_1", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_2", &buffer);
        assert_eq!(res, false);
        let res = is_cyclic("dummy_3", &buffer);
        assert_eq!(res, false);

        assert_eq!(check_would_produce_cycle(
            &SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "dummy_4".to_string(),
                child_frame_id: "dummy_1".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            }, 
            &buffer), false
        );

        assert_eq!(check_would_produce_cycle(
            &SPTransformStamped {
                active_transform: true,
                enable_transform: true,
                time_stamp: SystemTime::now(),
                parent_frame_id: "dummy_3".to_string(),
                child_frame_id: "dummy_1".to_string(),
                transform: isometry_to_sp_transform(Isometry3::default()),
                metadata: MapOrUnknown::UNKNOWN
            }, 
            &buffer), true
        );
    }


}
