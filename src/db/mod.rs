use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, Semaphore};

/// Maximum number of concurrent database connections allowed by the pool.
pub const MAX_POOL_SIZE: usize = 50;
/// Minimum number of connections kept warm in the pool.
pub const MIN_POOL_SIZE: usize = 5;
/// How long an idle connection may live before being closed.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(300);
/// Maximum lifetime of a pooled connection before it is recycled.
pub const MAX_LIFETIME: Duration = Duration::from_secs(1800);
/// Maximum time to wait for a connection from the pool.
pub const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Configuration for the database connection pool.
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
        Self {
            max_size: MAX_POOL_SIZE,
            min_size: MIN_POOL_SIZE,
            idle_timeout: IDLE_TIMEOUT,
            max_lifetime: MAX_LIFETIME,
            acquire_timeout: ACQUIRE_TIMEOUT,
        }
    }
}

/// Snapshot of pool utilization metrics for monitoring.
#[derive(Debug, Clone, Default)]
pub struct PoolMetrics {
    pub active: usize,
    pub idle: usize,
    pub total: usize,
    pub waiters: usize,
    pub acquisitions: u64,
    pub timeouts: u64,
    pub last_acquire_micros: u64,
}

struct PooledConnection {
    conn: Connection,
    created_at: Instant,
    last_used: Instant,
}

/// A reusable database connection pool.
///
/// Connections are created lazily up to `max_size` and reused across requests
/// instead of opening a new connection per request.
pub struct ConnectionPool {
    config: PoolConfig,
    idle: Mutex<Vec<PooledConnection>>,
    permits: Arc<Semaphore>,
    acquisitions: AtomicU64,
    timeouts: AtomicU64,
    last_acquire_micros: AtomicU64,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        let permits = Arc::new(Semaphore::new(config.max_size));
        Self {
            config,
            idle: Mutex::new(Vec::new()),
            permits,
            acquisitions: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
            last_acquire_micros: AtomicU64::new(0),
        }
    }

    /// Acquire a connection from the pool, reusing an idle one when available.
    pub async fn acquire(&self) -> Result<PooledConnectionGuard<'_>, PoolError> {
        let start = Instant::now();
        let permit = tokio::time::timeout(
            self.config.acquire_timeout,
            self.permits.clone().acquire_owned(),
        )
        .await
        .map_err(|_| {
            self.timeouts.fetch_add(1, Ordering::Relaxed);
            PoolError::Timeout
        })?
        .map_err(|_| PoolError::Closed)?;

        let conn = {
            let mut idle = self.idle.lock().await;
            let now = Instant::now();
            idle.retain(|c| {
                now.duration_since(c.last_used) < self.config.idle_timeout
                    && now.duration_since(c.created_at) < self.config.max_lifetime
            });
            idle.pop()
        };

        let conn = match conn {
            Some(c) => c.conn,
            None => Connection::connect().await.map_err(PoolError::Connect)?,
        };

        self.acquisitions.fetch_add(1, Ordering::Relaxed);
        self.last_acquire_micros
            .store(start.elapsed().as_micros() as u64, Ordering::Relaxed);

        Ok(PooledConnectionGuard {
            pool: self,
            conn: Some(conn),
            _permit: permit,
        })
    }

    /// Return a connection to the idle set for reuse.
    async fn release(&self, conn: Connection) {
        let mut idle = self.idle.lock().await;
        if idle.len() < self.config.max_size {
            idle.push(PooledConnection {
                conn,
                created_at: Instant::now(),
                last_used: Instant::now(),
            });
        }
    }

    /// Collect current pool utilization metrics for monitoring.
    pub async fn metrics(&self) -> PoolMetrics {
        let idle = self.idle.lock().await;
        let idle_count = idle.len();
        let available = self.permits.available_permits();
        let active = self.config.max_size.saturating_sub(available);
        PoolMetrics {
            active,
            idle: idle_count,
            total: active + idle_count,
            waiters: self.config.max_size.saturating_sub(available + idle_count),
            acquisitions: self.acquisitions.load(Ordering::Relaxed),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            last_acquire_micros: self.last_acquire_micros.load(Ordering::Relaxed),
        }
    }
}

/// Guard that returns its connection to the pool on drop.
pub struct PooledConnectionGuard<'a> {
    pool: &'a ConnectionPool,
    conn: Option<Connection>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl PooledConnectionGuard<'_> {
    pub fn connection(&mut self) -> &mut Connection {
        self.conn.as_mut().expect("connection present until drop")
    }
}

impl Drop for PooledConnectionGuard<'_> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let pool = self.pool;
            tokio::spawn(async move {
                pool.release(conn).await;
            });
        }
    }
}

#[derive(Debug)]
pub enum PoolError {
    Timeout,
    Closed,
    Connect(ConnectionError),
}
