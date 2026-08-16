use std::collections::HashMap;
use crate::*;

use termtree::Tree;

pub fn build_tree_recursive(
    node_id: &str,
    transforms: &HashMap<String, SPTransformStamped>,
    parent_map: &HashMap<String, Vec<String>>,
    current_depth: u64,
) -> Tree<String> {
    if current_depth > MAX_RECURSION_DEPTH {
        eprintln!("Maximum recursion depth reached for node ID {}", node_id);
        return Tree::new(format!("{} (depth limit reached)", node_id));
    }

    let mut tree = Tree::new(node_id.to_string());

    if let Some(mut children) = parent_map.get(node_id).cloned() {
        children.sort_unstable();
        for child_id in children {
            let child_tree =
                build_tree_recursive(&child_id, transforms, parent_map, current_depth + 1);
            tree.push(child_tree);
        }
    }

    tree
}

pub fn get_tree_root(buffer: &HashMap<String, SPTransformStamped>) -> Option<String> {
    if let Some(start_frame) = buffer.keys().next() {
        let mut current_frame = start_frame.clone();
        while let Some(transform) = buffer.get(&current_frame) {
            if !buffer.contains_key(&transform.parent_frame_id) {
                return Some(transform.parent_frame_id.clone());
            }
            current_frame = transform.parent_frame_id.clone();
        }
    }
    None
}

// pub async fn update_tree_visualization(
//     buffer: &HashMap<String, SPTransformStamped>,
//     refresh_rate: u64,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     loop {
//         let tree_data = update_tree_visualization_once(buffer);

//         tokio::time::sleep(Duration::from_millis(refresh_rate)).await;
//     }
// }

pub fn update_tree_visualization_once(
    buffer: &HashMap<String, SPTransformStamped>,
) -> String {
    let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();
    for transform in buffer.values() {
        parent_map
            .entry(transform.parent_frame_id.clone())
            .or_default()
            .push(transform.child_frame_id.clone());
    }

    match get_tree_root(&buffer) {
        Some(root) => format!(
            "{}",
            build_tree_recursive(&root, &buffer, &parent_map, 0)
        ),
        None => format!("No tree root."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn transform(child: &str, parent: &str) -> SPTransformStamped {
        SPTransformStamped {
            active_transform: true,
            enable_transform: true,
            time_stamp: SystemTime::now(),
            parent_frame_id: parent.to_string(),
            child_frame_id: child.to_string(),
            transform: SPTransform::default(),
            metadata: MapOrUnknown::Map(vec![]),
        }
    }

    fn buffer(edges: &[(&str, &str)]) -> HashMap<String, SPTransformStamped> {
        edges
            .iter()
            .map(|(child, parent)| (child.to_string(), transform(child, parent)))
            .collect()
    }

    fn parent_map(
        buffer: &HashMap<String, SPTransformStamped>,
    ) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for tf in buffer.values() {
            map.entry(tf.parent_frame_id.clone())
                .or_default()
                .push(tf.child_frame_id.clone());
        }
        map
    }

    /// The rendered shape, including the ordering: children are sorted, so the
    /// same buffer always renders identically no matter what order the
    /// `HashMap` iterates in. Without the sort this test would be flaky, which
    /// is precisely why it is worth pinning.
    #[test]
    fn a_tree_renders_its_children_in_sorted_order() {
        let buffer = buffer(&[
            ("child1", "root"),
            ("child2", "child1"),
            ("child3", "child1"),
            ("child5", "child3"),
            ("child4", "root"),
        ]);

        let rendered = build_tree_recursive("root", &buffer, &parent_map(&buffer), 0).to_string();

        assert_eq!(
            rendered,
            "root\n├── child1\n│   ├── child2\n│   └── child3\n│       └── child5\n└── child4\n"
        );
    }

    /// Repeated renders of the same buffer must be byte-identical - the sort is
    /// the only thing standing between this and `HashMap` iteration order.
    #[test]
    fn rendering_is_deterministic() {
        let buffer = buffer(&[
            ("a", "root"),
            ("b", "root"),
            ("c", "root"),
            ("d", "root"),
            ("e", "root"),
            ("f", "root"),
            ("g", "root"),
            ("h", "root"),
        ]);

        let first = update_tree_visualization_once(&buffer);
        for _ in 0..10 {
            assert_eq!(update_tree_visualization_once(&buffer), first);
        }
    }

    #[test]
    fn a_leaf_with_no_children_renders_as_itself() {
        let buffer = buffer(&[("only", "root")]);
        let rendered = build_tree_recursive("only", &buffer, &parent_map(&buffer), 0).to_string();
        assert_eq!(rendered, "only\n");
    }

    /// The root is the one frame that is a parent but is not itself a child of
    /// anything in the buffer - it is the frame the whole chain hangs from.
    #[test]
    fn the_root_is_the_frame_nothing_in_the_buffer_parents() {
        let buffer = buffer(&[("a", "world"), ("b", "a"), ("c", "b")]);
        assert_eq!(get_tree_root(&buffer), Some("world".to_string()));
    }

    #[test]
    fn an_empty_buffer_has_no_root() {
        assert_eq!(get_tree_root(&HashMap::new()), None);
        assert_eq!(update_tree_visualization_once(&HashMap::new()), "No tree root.");
    }

    /// Two separate chains still produce a single tree - `get_tree_root` walks
    /// up from an arbitrary starting frame, so it finds *a* root, and only the
    /// part of the forest hanging off that root is rendered. This is a real
    /// limitation of the visualisation rather than a bug in these functions,
    /// and it is worth having written down.
    #[test]
    fn a_forest_renders_only_the_tree_the_walk_landed_in() {
        let buffer = buffer(&[("a", "world"), ("b", "a"), ("x", "other_world")]);
        let rendered = update_tree_visualization_once(&buffer);

        let root = get_tree_root(&buffer).unwrap();
        assert!(
            root == "world" || root == "other_world",
            "the root has to be one of the two, got {root}"
        );
        if root == "world" {
            assert!(rendered.contains("a") && !rendered.contains("x"));
        } else {
            assert!(rendered.contains("x") && !rendered.contains("── a"));
        }
    }

    /// A chain longer than `MAX_RECURSION_DEPTH` has to stop and say so rather
    /// than blow the stack. Note this guards the *depth of the walk*, not
    /// cycles: `build_tree_recursive` following a cycle would also hit this
    /// limit, but `get_tree_root` would spin forever before ever getting here
    /// (see the note on `a_cycle_never_reaches_the_renderer`).
    #[test]
    fn a_chain_deeper_than_the_limit_is_cut_off() {
        let mut edges: Vec<(String, String)> = vec![("node0".to_string(), "root".to_string())];
        for i in 1..(MAX_RECURSION_DEPTH + 2) {
            edges.push((format!("node{}", i), format!("node{}", i - 1)));
        }
        let buffer: HashMap<String, SPTransformStamped> = edges
            .iter()
            .map(|(child, parent)| (child.clone(), transform(child, parent)))
            .collect();

        let rendered =
            build_tree_recursive("root", &buffer, &parent_map(&buffer), 0).to_string();

        assert!(
            rendered.contains("(depth limit reached)"),
            "a chain of {} frames should have hit the depth limit",
            MAX_RECURSION_DEPTH + 2
        );
    }

    /// A cycle among frames that are all present in the buffer makes
    /// `get_tree_root` loop forever: it walks `current_frame = parent` and the
    /// loop only ends when a parent is *not* in the buffer, which never happens
    /// in a cycle.
    ///
    /// This is why `TransformsManager` refuses to insert an edge that would
    /// create one (`transforms::cycles`) - the check is what keeps this
    /// function from hanging, so the test asserts the check, not the hang.
    #[test]
    fn a_cycle_never_reaches_the_renderer() {
        let mut buffer = buffer(&[("a", "b"), ("b", "a")]);

        // `check_would_produce_cycle` is the guard on the write path.
        let offending = transform("a", "b");
        assert!(
            check_would_produce_cycle(&offending, &buffer),
            "inserting a -> b on top of b -> a must be rejected as a cycle"
        );

        // And with the cycle broken, the same buffer renders fine.
        buffer.insert("b".to_string(), transform("b", "world"));
        assert_eq!(get_tree_root(&buffer), Some("world".to_string()));
    }
}

// #[cfg(test)]
// mod old_tests {

//     use nalgebra::Isometry3;
//     use serde_json::Value;
//     use std::collections::HashMap;
//     use tokio::time::Instant;

//     use rand::distributions::{Distribution, Uniform};
//     use rand::{thread_rng, Rng};
//     use termtree::Tree;

//     use crate::*;

//     #[test]
//     fn test_build_tree_recursive() {
//         let mut transforms: HashMap<String, SPTransformStamped> = HashMap::new();
//         transforms.insert(
//             "child1".to_string(),
//             SPTransformStamped {
//                 active: true,
//                 time_stamp: Instant::now(),
//                 parent_frame_id: "root".to_string(),
//                 child_frame_id: "child1".to_string(),
//                 transform: Isometry3::default(),
//                 metadata: Value::default()
//             },
//         );
//         transforms.insert(
//             "child2".to_string(),
//             SPTransformStamped {
//                 active: true,
//                 time_stamp: Instant::now(),
//                 parent_frame_id: "child1".to_string(),
//                 child_frame_id: "child2".to_string(),
//                 transform: Isometry3::default(),
//                 metadata: Value::default()
//             },
//         );
//         transforms.insert(
//             "child3".to_string(),
//             SPTransformStamped {
//                 active: true,
//                 time_stamp: Instant::now(),
//                 parent_frame_id: "child1".to_string(),
//                 child_frame_id: "child3".to_string(),
//                 transform: Isometry3::default(),
//                 metadata: Value::default()
//             },
//         );
//         transforms.insert(
//             "child5".to_string(),
//             SPTransformStamped {
//                 active: true,
//                 time_stamp: Instant::now(),
//                 parent_frame_id: "child3".to_string(),
//                 child_frame_id: "child5".to_string(),
//                 transform: Isometry3::default(),
//                 metadata: Value::default()
//             },
//         );

//         transforms.insert(
//             "child4".to_string(),
//             SPTransformStamped {
//                 active: true,
//                 time_stamp: Instant::now(),
//                 parent_frame_id: "root".to_string(),
//                 child_frame_id: "child4".to_string(),
//                 transform: Isometry3::default(),
//                 metadata: Value::default()
//             },
//         );

//         let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();
//         for transform in transforms.values() {
//             parent_map
//                 .entry(transform.parent_frame_id.clone())
//                 .or_default()
//                 .push(transform.child_frame_id.clone());
//         }

//         if let Some(_) = parent_map.get("root") {
//             let tree = build_tree_recursive("root", &transforms, &parent_map, 0);
//             assert_eq!(tree.to_string(), "root\n├── child1\n│   ├── child2\n│   └── child3\n│       └── child5\n└── child4\n")
//         }
//     }

//     #[test]
//     fn test_tree_maximum_recursion_depth() {
//         let mut transforms: HashMap<String, SPTransformStamped> = HashMap::new();
//         let max_depth = MAX_RECURSION_DEPTH + 1; // We exceed MAX_DEPTH to trigger the limit
//         let parent_id_base = "node";

//         // Create a linear hierarchy of nodes exceeding the maximum depth
//         for i in 0..max_depth {
//             let parent_id = if i == 0 {
//                 "root".to_string()
//             } else {
//                 format!("{}{}", parent_id_base, i - 1)
//             };
//             let child_id = format!("{}{}", parent_id_base, i);

//             transforms.insert(
//                 child_id.clone(),
//                 SPTransformStamped {
//                     active: true,
//                     time_stamp: Instant::now(),
//                     parent_frame_id: parent_id,
//                     child_frame_id: child_id.clone(),
//                     transform: Isometry3::default(),
//                     metadata: Value::default()
//                 },
//             );
//         }

//         let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();
//         for transform in transforms.values() {
//             parent_map
//                 .entry(transform.parent_frame_id.clone())
//                 .or_default()
//                 .push(transform.child_frame_id.clone());
//         }

//         // Start the tree building from the root, which is the start of our chain
//         let tree = build_tree_recursive("root", &transforms, &parent_map, 0);

//         // Check for depth limit indication in the output
//         let output = format!("{}", tree);
//         assert!(
//             output.contains("(depth limit reached)"),
//             "Tree should indicate that the maximum depth was reached"
//         );
//     }

//     fn generate_random_tree(depth: usize, num_nodes: usize) -> Tree<String> {
//         let mut rng = rand::thread_rng();
//         let current_depth = 1; // We start at depth 1 with the root node
//         build_random_tree(&mut rng, depth, &mut (num_nodes - 1), current_depth)
//     }

//     fn build_random_tree<R: Rng + ?Sized>(
//         rng: &mut R,
//         max_depth: usize,
//         remaining_nodes: &mut usize,
//         current_depth: usize,
//     ) -> Tree<String> {
//         // Create a root node for this subtree
//         let node_label = format!("Node {}", rng.gen::<u32>());
//         let mut tree = Tree::new(node_label);

//         if *remaining_nodes > 0 && current_depth < max_depth {
//             let num_children = if *remaining_nodes < max_depth - current_depth {
//                 rng.gen_range(0..=*remaining_nodes)
//             } else {
//                 // Decide how many children this node should have
//                 let dist = Uniform::from(0..=(max_depth - current_depth));
//                 dist.sample(rng)
//             };

//             for _ in 0..num_children {
//                 if *remaining_nodes > 0 {
//                     let subtree =
//                         build_random_tree(rng, max_depth, remaining_nodes, current_depth + 1);
//                     tree.push(subtree);
//                     if *remaining_nodes != 0 {
//                         *remaining_nodes -= 1;
//                     }
//                 }
//             }
//         }

//         tree
//     }

//     #[test]
//     fn test_visualize_random_tree() {
//         let mut rng = thread_rng();
//         let depth = rng.gen_range(1..20);
//         let num_nodes = rng.gen_range(1..20);
//         let tree = generate_random_tree(depth as usize, num_nodes as usize);
//         println!("{}", tree);
//     }

//     // TODO: need a test for the async function
// }
