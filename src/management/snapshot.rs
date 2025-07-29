use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use crate::{StateManager, ConnectionManager, State};

pub struct SnapshotManager {
    snapshot_path: PathBuf,
    in_memory_snapshot: Arc<RwLock<Option<State>>>,
    connection_manager: Arc<ConnectionManager>,
    sync_interval_ms: u64
    // keys_to_monitor: Vec<String>,
}

impl SnapshotManager {
    /// Creates a new manager and immediately tries to load a snapshot from disk.
    pub async fn new(
        snapshot_path: impl AsRef<Path>,
        connection_manager: Arc<ConnectionManager>,
        sync_interval_ms: u64
        // keys_to_monitor: Vec<String>,
    ) -> Self {
        let manager = Self {
            snapshot_path: snapshot_path.as_ref().to_path_buf(),
            in_memory_snapshot: Arc::new(RwLock::new(None)),
            connection_manager,
            sync_interval_ms
            // keys_to_monitor,
        };
        manager.load_from_disk().await;
        manager
    }

    /// Loads the last snapshot from the JSON file into memory.
    async fn load_from_disk(&self) {
        if let Ok(data) = fs::read_to_string(&self.snapshot_path).await {
            if let Ok(state) = serde_json::from_str::<State>(&data) {
                *self.in_memory_snapshot.write().await = Some(state);
                log::info!(target: "snapshot_manager", "Successfully loaded snapshot from disk.");
            }
        }
    }

    /// Saves the current in-memory snapshot to the JSON file.
    async fn save_to_disk(&self) {
        let snapshot = self.in_memory_snapshot.read().await;
        if let Some(state) = &*snapshot {
            let data = serde_json::to_string_pretty(state).unwrap();
            if fs::write(&self.snapshot_path, data).await.is_ok() {
                log::info!(target: "snapshot_manager", "Successfully saved snapshot to disk.");
            }
        }
    }

    /// The main task loop that periodically syncs state between Redis and the snapshot.
    pub async fn run_periodic_task(self) {
        let log_target = "snapshot_manager";
        let mut sync_interval = interval(Duration::from_millis(self.sync_interval_ms));

        log::info!(target: log_target, "Periodic snapshot task started.");

        loop {
            sync_interval.tick().await;
            let mut con = self.connection_manager.get_connection().await;

            // match redis_get_state_for_keys(&mut con, &self.keys_to_monitor).await {
                match StateManager::get_full_state(&mut con).await {
                Some(current_redis_state) => {
                    let snapshot_is_different = {
                        let snapshot = self.in_memory_snapshot.read().await;
                        snapshot.as_ref() != Some(&current_redis_state)
                    };

                    if snapshot_is_different {
                        *self.in_memory_snapshot.write().await = Some(current_redis_state);
                        self.save_to_disk().await;
                    }
                }
                // Redis is empty. Restore from snapshot. Expermental
                None => {
                    let snapshot = self.in_memory_snapshot.read().await;
                    if let Some(state_to_restore) = &*snapshot {
                        log::warn!(target: log_target, "Redis is empty. Repopulating from snapshot.");
                        let _ = StateManager::set_state(&mut con, state_to_restore).await;
                    }
                }
            }
        }
    }
}