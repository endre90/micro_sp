use std::sync::{Arc, Mutex};

use log::*;
use r2r::Context;
use r2r_transforms::*;
use serde_json::Value;
use tokio::time::{Duration, Instant};

pub static VISUALIZE_TREE_REFRESH_RATE: u64 = 100; // milliseconds

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fn initialize_logging() {
        std::env::set_var("RUST_LOG", "warn");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    initialize_logging();

    log::info!("Starting the r2r_transforms example...");

    let context = Context::create()?;
    let node = r2r::Node::create(context, "r2r_transforms", "")?;
    let arc_node = Arc::new(Mutex::new(node));

    let arc_node_clone = arc_node.clone();
    let buffer = RosSpaceTreeServer::new("test", &arc_node_clone);

    let buffer_clone = buffer.clone();
    tokio::task::spawn(async move {
        match space_tree_manipulation_example(&buffer_clone).await {
            Ok(()) => (),
            Err(e) => error!("Space tree manipulation task failed with: '{}'.", e),
        };
    });

    // let buffer_clone = buffer.clone();
    // let arc_node_clone = arc_node.clone();
    // tokio::task::spawn(async move {
    //     match buffer_clone.connect_to_ros(&arc_node_clone).await {
    //         Ok(()) => (),
    //         Err(e) => error!("Connecting the buffer to ros failed with: '{}'.", e),
    //     };
    // });

    // Keep the node alive
    let arc_node_clone: Arc<Mutex<r2r::Node>> = arc_node.clone();
    let handle = std::thread::spawn(move || loop {
        arc_node_clone
            .lock()
            .unwrap()
            .spin_once(std::time::Duration::from_millis(100));
    });

    r2r::log_info!("r2r_transforms", "Node started.");

    handle.join().unwrap();

    Ok(())
}

pub async fn space_tree_manipulation_example(
    buffer: &RosSpaceTreeServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

    let path = format!("{}/examples/data", manifest_dir);


    tokio::time::sleep(Duration::from_millis(500)).await;
    buffer.load_scenario(&path, false);
    buffer.apply_changes();

    println!("Open RViz for visualizing the ROS connection.");
    tokio::time::sleep(Duration::from_millis(5000)).await;

    // let _ = visualize_tree_once(&buffer);

    tokio::time::sleep(Duration::from_millis(500)).await;
    let new_transform = TransformStamped {
        active: true,
        time_stamp: Instant::now(),
        parent_frame_id: "frame_1".to_string(),
        child_frame_id: "asdfasdf".to_string(),
        transform: json_transform_to_isometry(JsonTransform::default()),
        metadata: Value::default()
    };
    

    tokio::time::sleep(Duration::from_millis(5000)).await;

    buffer.insert_transform("asdfasdf", new_transform.clone());
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.insert_transform("frame_7", new_transform);
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.move_transform("frame_6", json_transform_to_isometry(JsonTransform::default()));
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);

    buffer.remove_transform("frame_7");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.rename_transform("frame_6", "frame_8");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_6", "child_7");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.clone_transform("frame_6", "child_7");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.clone_transform("frame_8", "frame_9");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_8", "child_7");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_3", "frame_8");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_8", "world");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.remove_transform("frame_9");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.remove_transform("frame_5");
    buffer.apply_changes();

    // let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;


    println!("{:?}", buffer.global_buffer.lock().unwrap().clone());

    // buffer.delete_all_transforms();
    // buffer.apply_changes();

    // // let _ = visualize_tree_once(&buffer);
    // tokio::time::sleep(Duration::from_millis(500)).await;

    Ok(())
}
