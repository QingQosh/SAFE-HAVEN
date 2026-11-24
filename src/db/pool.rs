//! Database connection pooling.
//!
//! Provides a reusable pool of database connections so that requests do not
//! open a fresh connection each time. Pool size is capped at 50 concurrent
//! connections, idle connections are recycled, and utilization metrics are
//! exposed for monitoring.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Maximum number of concurrent connections the pool will ever open.
pub const MAX_POOL_SIZE: usize = 50;
/// Minimum number of connections kept warm in the pool.
pub const MIN_POOL_SIZE: usize = 5;
/// How long an idle connection may sit in the pool before being closed.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(300);
/// Maximum lifetime of a connection before it is recycled.
pub const MAX_LIFETIME: Duration = Duration::from_secs(1800);
/// How long a caller will wait to acquire a connection before timing out.
pub const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Configuration for the connection pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_size: usize,
    pub min_size: usize,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            max_size: MAX_POOL_SIZE,
            min_size: MIN_POOL_SIZE,
            idle_timeout: IDLE_TIMEOUT,
            max_lifetime: MAX_LIFETIME,
            acquire_timeout: ACQUIRE_TIMEOUT,
        }
    }
}

/// A single pooled connection.
pub struct Connection {
    id: u64,
    created_at: Instant,
    last_used: Instant,
}

impl Connection {
    fn new(id: u64) -> Self {
        let now = Instant::now();
        Connection {
            id,
            created_at: now,
            last_used: now,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    fn is_expired(&self, config: &PoolConfig) -> bool {
        self.created_at.elapsed() > config.max_lifetime
            || self.last_used.elapsed() > config.idle_timeout
    }
}

/// Snapshot of pool utilization for monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolMetrics {
    pub active: usize,
    pub idle: usize,
    pub total: usize,
    pub waiters: usize,
    pub max_size: usize,
    pub acquisitions: u64,
    pub timeouts: u64,
    pub last_acquire_micros: u64,
}

struct PoolState {
    idle: Vec<Connection>,
    active: usize,
    waiters: usize,
    next_id: u64,
}

/// A thread-safe connection pool.
pub struct ConnectionPool {
    config: PoolConfig,
    state: Mutex<PoolState>,
    available: Condvar,
    acquisitions: AtomicU64,
    timeouts: AtomicU64,
    last_acquire_micros: AtomicU64,
}

impl ConnectionPool {
    /// Create a new pool with the given configuration.
    pub fn new(config: PoolConfig) -> Arc<Self> {
        let max_size = config.max_size.max(1);
        let min_size = config.min_size.min(max_size);
        let config = PoolConfig {
            max_size,
            min_size,
            ..config
        };

        let mut idle = Vec::with_capacity(min_size);
        for id in 0..min_size as u64 {
            idle.push(Connection::new(id));
        }

        Arc::new(ConnectionPool {
            state: Mutex::new(PoolState {
                idle,
                active: 0,
                waiters: 0,
                next_id: min_size as u64,
            }),
            available: Condvar::new(),
            acquisitions: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
            last_acquire_micros: AtomicU64::new(0),
            config,
        })
    }

    /// Create a pool with default settings (max 50 connections).
    pub fn with_defaults() -> Arc<Self> {
        Self::new(PoolConfig::default())
    }

    /// Acquire a connection from the pool, reusing an idle one when possible.
    ///
    /// Returns `None` if no connection becomes available within the configured
    /// acquisition timeout.
    pub fn acquire(&self) -> Option<PooledConnection> {
        let start = Instant::now();
        let deadline = start + self.config.acquire_timeout;
        let mut state = self.state.lock().unwrap();

        loop {
            // Reuse an idle connection if one is available and still valid.
            while let Some(conn) = state.idle.pop() {
                if conn.is_expired(&self.config) {
                    continue;
                }
                state.active += 1;
                drop(state);
                self.record_acquire(start);
                return Some(PooledConnection {
                    pool: self as *const ConnectionPool,
                    conn: Some(conn),
                });
            }

            // Open a new connection if we are below the cap.
            if state.active < self.config.max_size {
                let conn = Connection::new(state.next_id);
                state.next_id += 1;
                state.active += 1;
                drop(state);
                self.record_acquire(start);
                return Some(PooledConnection {
                    pool: self as *const ConnectionPool,
                    conn: Some(conn),
                });
            }

            // Pool is saturated: wait for a connection to be returned.
            let now = Instant::now();
            if now >= deadline {
                self.timeouts.fetch_add(1, Ordering::Relaxed);
                return None;
            }
            state.waiters += 1;
            let (guard, timeout) = self
                .available
                .wait_timeout(state, deadline - now)
                .unwrap();
            state = guard;
            state.waiters = state.waiters.saturating_sub(1);
            if timeout.timed_out() && state.idle.is_empty() && state.active >= self.config.max_size {
                self.timeouts.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        }
    }

    fn record_acquire(&self, start: Instant) {
        self.acquisitions.fetch_add(1, Ordering::Relaxed);
        let micros = start.elapsed().as_micros() as u64;
        self.last_acquire_micros.store(micros, Ordering::Relaxed);
    }

    fn release(&self, mut conn: Connection) {
        let mut state = self.state.lock().unwrap();
        state.active = state.active.saturating_sub(1);
        conn.last_used = Instant::now();
        if !conn.is_expired(&self.config) && state.idle.len() < self.config.max_size {
            state.idle.push(conn);
        }
        drop(state);
        self.available.notify_one();
    }

    /// Current pool utilization metrics for monitoring.
    pub fn metrics(&self) -> PoolMetrics {
        let state = self.state.lock().unwrap();
        PoolMetrics {
            active: state.active,
            idle: state.idle.len(),
            total: state.active + state.idle.len(),
            waiters: state.waiters,
            max_size: self.config.max_size,
            acquisitions: self.acquisitions.load(Ordering::Relaxed),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            last_acquire_micros: self.last_acquire_micros.load(Ordering::Relaxed),
        }
    }

    /// Number of connections currently checked out.
    pub fn active_connections(&self) -> usize {
        self.state.lock().unwrap().active
    }

    /// Number of idle connections available for reuse.
    pub fn idle_connections(&self) -> usize {
        self.state.lock().unwrap().idle.len()
    }
}

/// A connection checked out from the pool. Returning it to the pool happens
/// automatically on drop so connections are always reused.
pub struct PooledConnection {
    pool: *const ConnectionPool,
    conn: Option<Connection>,
}

impl PooledConnection {
    pub fn id(&self) -> u64 {
        self.conn.as_ref().map(|c| c.id).unwrap_or(0)
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            // SAFETY: the pool outlives all connections it hands out; the raw
            // pointer is only dereferenced while the pool is alive.
            unsafe { (*self.pool).release(conn) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_idle_connections() {
        let pool = ConnectionPool::with_defaults();
        let first = pool.acquire().expect("acquire");
        let id = first.id();
        drop(first);
        let second = pool.acquire().expect("acquire");
        assert_eq!(second.id(), id);
    }

    #[test]
    fn caps_concurrent_connections() {
        let pool = ConnectionPool::new(PoolConfig {
            max_size: 2,
            min_size: 0,
            acquire_timeout: Duration::from_millis(10),
            ..PoolConfig::default()
        });
        let _a = pool.acquire().expect("a");
        let _b = pool.acquire().expect("b");
        assert!(pool.acquire().is_none());
        assert_eq!(pool.metrics().timeouts, 1);
    }
}
