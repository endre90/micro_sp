use redis::aio::MultiplexedConnection;
use redis::{Client, RedisResult};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

pub struct ConnectionManager {
    connection: Arc<RwLock<MultiplexedConnection>>,
    redis_addr: String,
}

impl ConnectionManager {
    pub async fn new() -> Self {
        let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
        let redis_addr = format!("redis://{}:{}", redis_host, redis_port);
        
        log::info!(target: "redis_manager", "Connecting to Redis at {}...", redis_addr);
        
        loop {
            match Self::try_connect(&redis_addr).await {
                Ok(connection) => {
                    log::info!(target: "redis_manager", "Redis connection established.");
                    return Self {
                        connection: Arc::new(RwLock::new(connection)),
                        redis_addr,
                    };
                }
                Err(e) => {
                    log::error!(target: "redis_manager", "Initial connection failed: {}. Retrying in 5s...", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn try_connect(redis_addr: &str) -> RedisResult<MultiplexedConnection> {
        let client = Client::open(redis_addr.to_string())?;
        client.get_multiplexed_async_connection().await
    }

    pub async fn get_connection(&self) -> MultiplexedConnection {
        self.connection.read().await.clone()
    }

    // Replaces the dead connection with a new one
    pub async fn reconnect(&self) {
        log::warn!(target: "redis_manager", "Redis connection lost. Attempting to reconnect...");
        
        // Get a write lock to replace the connection.
        // This ensures only one task tries to reconnect at a time
        let mut connection_guard = self.connection.write().await;

        loop {
            match Self::try_connect(&self.redis_addr).await {
                Ok(new_connection) => {
                    *connection_guard = new_connection;
                    log::info!(target: "redis_manager", "Redis re-connection successful.");
                    return; 
                }
                Err(e) => {
                    log::error!(target: "redis_manager", "Reconnect failed: {}. Retrying in 3s...", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }
}

pub async fn handle_redis_error(
    e: &redis::RedisError,
    log_target: &str,
    connection_manager: &Arc<ConnectionManager>,
) {
    if e.is_io_error() {
        log::error!(target: log_target, "Redis command failed, triggering reconnect.");
        connection_manager.reconnect().await;
    } else {
        log::error!(target: log_target, "An unexpected Redis error occurred: {}", e);
    }
}