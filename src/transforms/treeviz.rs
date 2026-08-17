//! Rendering the transform tree for a terminal.
//!
//! [`update_tree_visualization_once`] turns a transform buffer into the
//! familiar `├──` ASCII tree. Purely for looking at - nothing in the runtime
//! reads it back.

use std::collections::HashMap;
use crate::*;

use termtree::Tree;

/// Build the subtree rooted at `node_id`.
///
/// `parent_map` maps a parent frame id to its children. Children are sorted, so
/// the same buffer always renders identically. Recursion stops at
/// [`MAX_RECURSION_DEPTH`], leaving a `(depth limit reached)` marker in the
/// output instead of overflowing the stack.
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

/// Find the tree's root: the first frame id named as a parent that is not
/// itself a frame in `buffer`.
///
/// `None` for an empty buffer. A buffer containing a cycle has no such frame
/// and would loop, so check with [`is_cyclic_all`] first.
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

/// Render a whole transform buffer as an ASCII tree.
///
/// Returns `"No tree root."` when the buffer is empty. Frames not reachable
/// from the root are not drawn.
///
/// ```
/// use micro_sp::*;
/// use std::collections::HashMap;
/// use std::time::SystemTime;
///
/// let mut buffer = HashMap::new();
/// for (child, parent) in [("base", "world"), ("arm", "base"), ("tool", "arm")] {
///     buffer.insert(
///         child.to_string(),
///         SPTransformStamped {
///             active_transform: true,
///             enable_transform: true,
///             time_stamp: SystemTime::now(),
///             parent_frame_id: parent.to_string(),
///             child_frame_id: child.to_string(),
///             transform: SPTransform::default(),
///             metadata: MapOrUnknown::UNKNOWN,
///         },
///     );
/// }
///
/// assert_eq!(
///     update_tree_visualization_once(&buffer),
///     "world\n└── base\n    └── arm\n        └── tool\n"
/// );
/// ```
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
