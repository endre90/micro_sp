use log::*;
use r2r_transforms::*;
use serde_json::Value;
use tokio::time::{Duration, Instant};

pub static VISUALIZE_TREE_REFRESH_RATE: u64 = 100; // milliseconds

#[tokio::main]
async fn main() -> () {
    fn initialize_logging() {
        std::env::set_var("RUST_LOG", "warn");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    initialize_logging();

    log::info!("Starting the r2r_transforms example...");

    let buffer = SpaceTreeServer::new("test");

    let buffer_clone = buffer.clone();
    tokio::task::spawn(async move {
        match space_tree_manipulation_example(&buffer_clone).await {
            Ok(()) => (),
            Err(e) => error!("Space tree manipulation task failed with: '{}'.", e),
        };
    });

    let handle = std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(1000));
    });

    handle.join().unwrap();
}

pub async fn space_tree_manipulation_example(
    buffer: &SpaceTreeServer,
    // refresh_rate: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

    let path = format!("{}/examples/data", manifest_dir);

    let _ = visualize_tree_once(&buffer);

    // loop {
    tokio::time::sleep(Duration::from_millis(500)).await;
    buffer.load_scenario(&path, false);
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);

    tokio::time::sleep(Duration::from_millis(500)).await;
    let new_transform = TransformStamped {
        active: true,
        time_stamp: Instant::now(),
        parent_frame_id: "frame_5".to_string(),
        child_frame_id: "frame_6".to_string(),
        transform: json_transform_to_isometry(JsonTransform::default()),
        metadata: Value::default()
    };
    
    buffer.insert_transform("frame_6", new_transform.clone());
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.insert_transform("frame_7", new_transform);
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.move_transform("frame_6", json_transform_to_isometry(JsonTransform::default()));
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);

    buffer.remove_transform("frame_7");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.rename_transform("frame_6", "frame_8");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_6", "child_7");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.clone_transform("frame_6", "child_7");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.clone_transform("frame_8", "frame_9");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_8", "child_7");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_3", "frame_8");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.reparent_transform("frame_8", "frame_3");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.remove_transform("frame_9");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.remove_transform("frame_5");
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    buffer.delete_all_transforms();
    buffer.apply_changes();

    let _ = visualize_tree_once(&buffer);
    tokio::time::sleep(Duration::from_millis(500)).await;

    Ok(())
}
