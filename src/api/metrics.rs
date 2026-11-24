use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Per-endpoint rate limit configuration.
#[derive(Clone, Copy, Debug)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed within the window.
    pub max_requests: u64,
    /// Length of the rate limit window.
    pub window: Duration,
}

impl RateLimitConfig {
    pub const fn new(max_requests: u64, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }
}

/// Result of a rate limit check for a single request.
#[derive(Clone, Copy, Debug)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub limit: u64,
    pub remaining: u64,
    /// Seconds until the current window resets.
    pub reset_after_secs: u64,
}

struct WindowState {
    window_start: Instant,
    count: u64,
}

/// Tracks request counts per client key (user id or API key) and endpoint.
///
/// This is intentionally simple and in-process; it provides the tracking and
/// metrics surface required by the rate limiting middleware.
#[derive(Default)]
pub struct RateLimiter {
    windows: Mutex<HashMap<(String, String), WindowState>>,
    /// Total number of requests that were rejected due to rate limiting.
    violations: AtomicU64,
    /// Total number of requests observed by the limiter.
    total_requests: AtomicU64,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check and record a request for the given client key and endpoint.
    pub fn check(&self, client_key: &str, endpoint: &str, config: RateLimitConfig) -> RateLimitDecision {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let now = Instant::now();
        let mut windows = self.windows.lock().expect("rate limiter mutex poisoned");
        let key = (client_key.to_string(), endpoint.to_string());

        let state = windows.entry(key).or_insert_with(|| WindowState {
            window_start: now,
            count: 0,
        });

        if now.duration_since(state.window_start) >= config.window {
            state.window_start = now;
            state.count = 0;
        }

        let reset_after_secs = config
            .window
            .saturating_sub(now.duration_since(state.window_start))
            .as_secs();

        if state.count >= config.max_requests {
            self.violations.fetch_add(1, Ordering::Relaxed);
            return RateLimitDecision {
                allowed: false,
                limit: config.max_requests,
                remaining: 0,
                reset_after_secs,
            };
        }

        state.count += 1;
        let remaining = config.max_requests.saturating_sub(state.count);

        RateLimitDecision {
            allowed: true,
            limit: config.max_requests,
            remaining,
            reset_after_secs,
        }
    }

    /// Number of requests rejected due to exceeding a rate limit.
    pub fn violations(&self) -> u64 {
        self.violations.load(Ordering::Relaxed)
    }

    /// Total number of requests observed by the limiter.
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }
}

impl RateLimitDecision {
    /// Build the standard rate limit response headers.
    pub fn headers(&self) -> Vec<(String, String)> {
        vec![
            ("X-RateLimit-Limit".to_string(), self.limit.to_string()),
            ("X-RateLimit-Remaining".to_string(), self.remaining.to_string()),
            (
                "X-RateLimit-Reset".to_string(),
                self.reset_after_secs.to_string(),
            ),
        ]
    }
}
