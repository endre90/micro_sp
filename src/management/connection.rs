use redis::aio::{ConnectionManager as RedisConnectionManager, ConnectionManagerConfig};
use redis::{Client, RedisResult};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

use crate::{State, StateManager};

/// The Redis handle that is passed around the crate.
///
/// This is `redis::aio::ConnectionManager`, which is:
///   - cheap to clone (it is internally reference counted), so every runner can
///     hold its own long-lived handle instead of re-fetching one per tick;
///   - multiplexed, so concurrent tasks share one socket without locking;
///   - self-healing - if the connection drops it reconnects in the background
///     with exponential backoff *and jitter*, and retries the in-flight command.
///
/// Because reconnection is handled by the driver, nothing in this crate needs
/// to pre-flight a `PING` before issuing a command. Grab a connection once when
/// a runner starts and keep using it; a dead socket surfaces as an error on the
/// command itself and is repaired underneath you.
///
/// This is an alias rather than a concrete type on purpose: swapping the
/// transport (for a cluster connection, a pooled connection, or a test double)
/// only requires changing this line.
pub type SPConnection = RedisConnectionManager;

/// Each individual TCP connect + handshake attempt is abandoned after this.
/// Only applies to *establishing* a connection, never to running commands.
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Reconnect backoff. The driver waits `rand(0 .. FACTOR_MS * (BASE ^ attempt))`
/// milliseconds between attempts, capped at `MAX_DELAY_MS`. The jitter matters:
/// it stops every runner in the process from retrying in lockstep after a blip.
const RECONNECT_EXPONENT_BASE: u64 = 2;
const RECONNECT_FACTOR_MS: u64 = 100;
const RECONNECT_MAX_DELAY_MS: u64 = 2_000;
const RECONNECT_RETRIES: usize = 6;

/// Delay between attempts of the initial, blocking startup connect loop.
const INITIAL_CONNECT_RETRY_DELAY: Duration = Duration::from_secs(5);

/// How often the optional background health monitor pings Redis. This is the
/// replacement for the old per-tick, per-runner `PING`: one round trip every
/// few seconds for the whole process instead of ~50 per second.
pub const DEFAULT_HEALTH_CHECK_PERIOD: Duration = Duration::from_secs(5);

/// Owns the process-wide Redis connection and hands out cheap clones of it.
///
/// Note there is deliberately no `RwLock` here any more. The previous design
/// wrapped a `MultiplexedConnection` in `Arc<RwLock<..>>` so that a reconnect
/// could swap it out, which meant every single access on the hot path took an
/// async read guard, and a reconnect blocked every runner for the duration of
/// the retry loop. `SPConnection` replaces its own inner connection on failure,
/// so a plain field is both faster and more robust.
pub struct ConnectionManager {
    connection: SPConnection,
    redis_addr: String,
}

impl ConnectionManager {
    /// Connect to Redis, retrying forever. Intended to be called once at
    /// startup, before any runner is spawned.
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
                        connection,
                        redis_addr,
                    };
                }
                Err(e) => {
                    log::error!(
                        target: "redis_manager",
                        "Initial connection failed: {}. Retrying in {}s...",
                        e,
                        INITIAL_CONNECT_RETRY_DELAY.as_secs()
                    );
                    tokio::time::sleep(INITIAL_CONNECT_RETRY_DELAY).await;
                }
            }
        }
    }

    async fn try_connect(redis_addr: &str) -> RedisResult<SPConnection> {
        let client = Client::open(redis_addr.to_string())?;

        // Note: `response_timeout` is intentionally left unset. Capping how long
        // a *command* may take would abort long-running calls such as `KEYS *`
        // or `FLUSHDB` on a large keyspace and surface them as connection
        // errors. Set it here if you later bound the size of those operations.
        let config = ConnectionManagerConfig::new()
            .set_exponent_base(RECONNECT_EXPONENT_BASE)
            .set_factor(RECONNECT_FACTOR_MS)
            .set_number_of_retries(RECONNECT_RETRIES)
            .set_max_delay(RECONNECT_MAX_DELAY_MS)
            .set_connection_timeout(CONNECTION_TIMEOUT);

        RedisConnectionManager::new_with_config(client, config).await
    }

    /// Get a handle to Redis.
    ///
    /// This is now just a reference-count bump - no lock, no I/O - so it is
    /// safe to call anywhere. Even so, prefer calling it *once* before a
    /// runner's loop and reusing the handle: the returned connection stays
    /// valid across reconnects, so there is no reason to re-fetch per tick.
    ///
    /// Kept `async` purely so existing call sites (`get_connection().await`)
    /// keep compiling; see [`ConnectionManager::connection`] for the sync form.
    pub async fn get_connection(&self) -> SPConnection {
        self.connection.clone()
    }

    /// Synchronous equivalent of [`ConnectionManager::get_connection`].
    pub fn connection(&self) -> SPConnection {
        self.connection.clone()
    }

    /// The `redis://host:port` address this manager was built from.
    pub fn redis_addr(&self) -> &str {
        &self.redis_addr
    }

    /// Ping Redis once and report whether it answered.
    ///
    /// This is a diagnostic / startup probe, **not** something to call on a
    /// tick. It used to run before every iteration of every runner, which cost
    /// a full round trip per runner per tick (~40-60 RTTs/second across the
    /// process) and, because it was awaited before the real work, added that
    /// latency directly to every state update. Recovery no longer depends on
    /// it: `SPConnection` reconnects on its own. Use
    /// [`ConnectionManager::spawn_health_monitor`] if you want a liveness
    /// signal in the logs.
    pub async fn check_redis_health(&self, log_target: &str) -> RedisResult<()> {
        let mut con = self.connection();
        match redis::cmd("PING").query_async::<()>(&mut con).await {
            Ok(()) => Ok(()),
            Err(e) => {
                if e.is_io_error() {
                    log::error!(
                        target: log_target,
                        "Pinging Redis failed: {}. The connection manager will reconnect automatically.",
                        e
                    );
                } else {
                    log::error!(target: log_target, "An unexpected Redis error occurred: {}", e);
                }
                Err(e)
            }
        }
    }

    /// Block until Redis answers again.
    ///
    /// Reconnection itself is automatic, so this no longer *performs* a
    /// reconnect - it waits for one to succeed, which is the only part callers
    /// ever actually needed. Unlike the previous implementation it holds no
    /// lock, so waiting here never stalls the other runners.
    pub async fn reconnect(&self, log_target: &str) {
        log::warn!(target: log_target, "Waiting for the Redis connection to recover...");

        let mut delay = Duration::from_millis(RECONNECT_FACTOR_MS);
        loop {
            if self.check_redis_health(log_target).await.is_ok() {
                log::info!(target: log_target, "Redis connection recovered.");
                return;
            }
            tokio::time::sleep(delay).await;
            delay = (delay * RECONNECT_EXPONENT_BASE as u32)
                .min(Duration::from_millis(RECONNECT_MAX_DELAY_MS));
        }
    }

    /// Log a Redis error from a command.
    ///
    /// Callers no longer need to trigger anything: an I/O error means the
    /// driver is already reconnecting in the background and the next command
    /// will go over the new socket. Skipping the current tick is the correct
    /// response.
    pub async fn handle_redis_error(&self, e: &redis::RedisError, log_target: &str) {
        if e.is_io_error() {
            log::error!(
                target: log_target,
                "Redis command failed: {}. Reconnecting automatically, skipping this tick.",
                e
            );
        } else {
            log::error!(target: log_target, "An unexpected Redis error occurred: {}", e);
        }
    }

    /// Spawn a single background task that pings Redis on an interval.
    ///
    /// This replaces the old per-runner, per-tick health check with one probe
    /// for the whole process. It exists only for observability - nothing
    /// depends on it to recover.
    pub fn spawn_health_monitor(
        self: &Arc<Self>,
        log_target: &str,
        period: Duration,
    ) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(self);
        let log_target = log_target.to_string();
        tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(period);
            let mut was_healthy = true;
            loop {
                interval.tick().await;
                let healthy = manager.check_redis_health(&log_target).await.is_ok();
                if healthy != was_healthy {
                    if healthy {
                        log::info!(target: &log_target, "Redis is reachable again.");
                    } else {
                        log::warn!(target: &log_target, "Redis is unreachable.");
                    }
                    was_healthy = healthy;
                }
            }
        })
    }
}

pub async fn restore_state_from_snapshot(
    con: &mut SPConnection,
    last_known_state: &Arc<RwLock<Option<State>>>,
    log_target: &str,
) {
    let snapshot = last_known_state.read().await;

    if let Some(state_to_restore) = &*snapshot {
        log::warn!(
            target: log_target,
            "Redis is empty. Repopulating with the last known state."
        );
        StateManager::set_state(con, state_to_restore).await;
    } else {
        log::debug!(
            target: log_target,
            "Redis is empty and no snapshot exists yet. Waiting for initial state."
        );
    }
}
