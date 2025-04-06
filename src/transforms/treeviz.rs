use std::collections::HashMap;
use std::time::Duration;

use crate::*;

// use termtree::Tree;

// pub fn build_tree_recursive(
//     node_id: &str,
//     transforms: &HashMap<String, TransformStamped>,
//     parent_map: &HashMap<String, Vec<String>>,
//     current_depth: u64,
// ) -> Tree<String> {
//     if current_depth > MAX_RECURSION_DEPTH {
//         eprintln!("Maximum recursion depth reached for node ID {}", node_id);
//         return Tree::new(format!("{} (depth limit reached)", node_id));
//     }

//     let mut tree = Tree::new(node_id.to_string());

//     if let Some(mut children) = parent_map.get(node_id).cloned() {
//         children.sort_unstable();
//         for child_id in children {
//             let child_tree =
//                 build_tree_recursive(&child_id, transforms, parent_map, current_depth + 1);
//             tree.push(child_tree);
//         }
//     }

//     tree
// }


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

// pub async fn vizualize_tree(
//     buffer: &SpaceTreeServer,
//     refresh_rate: u64,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     loop {
//         let _ = visualize_tree_once(buffer);
//         tokio::time::sleep(Duration::from_millis(refresh_rate)).await;
//     }
// }

// pub fn visualize_tree_once(
//     buffer: &SpaceTreeServer,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let buffer_local = buffer.local_buffer.lock().unwrap().clone();

//     let mut parent_map: HashMap<String, Vec<String>> = HashMap::new();
//     for transform in buffer_local.values() {
//         parent_map
//             .entry(transform.parent_frame_id.clone())
//             .or_default()
//             .push(transform.child_frame_id.clone());
//     }

//     match get_tree_root(&buffer_local) {
//         Some(root) => println!(
//             "{}",
//             build_tree_recursive(&root, &buffer_local, &parent_map, 0)
//         ),
//         None => println!("No tree root."),
//     }
//     Ok(())
// }

// #[cfg(test)]
// mod tests {

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
//         let mut transforms: HashMap<String, TransformStamped> = HashMap::new();
//         transforms.insert(
//             "child1".to_string(),
//             TransformStamped {
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
//             TransformStamped {
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
//             TransformStamped {
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
//             TransformStamped {
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
//             TransformStamped {
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
//         let mut transforms: HashMap<String, TransformStamped> = HashMap::new();
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
//                 TransformStamped {
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
