use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Per-endpoint rate limit configuration.
#[derive(Clone, Copy, Debug)]
pub struct RateLimit {
    /// Maximum number of requests allowed within the window.
    pub max_requests: u32,
    /// Length of the sliding window.
    pub window: Duration,
}

impl RateLimit {
    pub const fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }
}

/// Rate limits for each endpoint, configurable per environment.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub default: RateLimit,
    pub endpoints: HashMap<String, RateLimit>,
    /// Clients whose requests bypass rate limiting entirely.
    pub trusted_clients: Vec<String>,
}

impl RateLimitConfig {
    /// Build a configuration appropriate for the given environment.
    pub fn for_environment(env: &str) -> Self {
        let (default, endpoints) = match env {
            "production" => (
                RateLimit::new(100, Duration::from_secs(60)),
                HashMap::from([
                    ("/api/auth/login".to_string(), RateLimit::new(5, Duration::from_secs(60))),
                    ("/api/auth/register".to_string(), RateLimit::new(3, Duration::from_secs(60))),
                    ("/api/search".to_string(), RateLimit::new(30, Duration::from_secs(60))),
                ]),
            ),
            "staging" => (
                RateLimit::new(500, Duration::from_secs(60)),
                HashMap::from([
                    ("/api/auth/login".to_string(), RateLimit::new(20, Duration::from_secs(60))),
                    ("/api/auth/register".to_string(), RateLimit::new(10, Duration::from_secs(60))),
                    ("/api/search".to_string(), RateLimit::new(100, Duration::from_secs(60))),
                ]),
            ),
            // Development and any unknown environment: permissive limits.
            _ => (
                RateLimit::new(10_000, Duration::from_secs(60)),
                HashMap::from([
                    ("/api/auth/login".to_string(), RateLimit::new(1_000, Duration::from_secs(60))),
                    ("/api/auth/register".to_string(), RateLimit::new(1_000, Duration::from_secs(60))),
                    ("/api/search".to_string(), RateLimit::new(5_000, Duration::from_secs(60))),
                ]),
            ),
        };

        Self {
            default,
            endpoints,
            trusted_clients: Vec::new(),
        }
    }

    /// Resolve the limit that applies to a given request path.
    pub fn limit_for(&self, path: &str) -> RateLimit {
        self.endpoints
            .get(path)
            .copied()
            .unwrap_or(self.default)
    }

    /// Whether the given client identifier is trusted and should bypass limits.
    pub fn is_trusted(&self, client_id: &str) -> bool {
        self.trusted_clients.iter().any(|c| c == client_id)
    }
}

/// Tracks request counts per client and endpoint within a sliding window.
#[derive(Default)]
pub struct RateLimitTracker {
    entries: Mutex<HashMap<(String, String), (Instant, u32)>>,
}

impl RateLimitTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a request and return the number of remaining requests in the window.
    /// Returns `None` when the limit has been exceeded.
    fn record(&self, client_id: &str, path: &str, limit: RateLimit) -> Option<u32> {
        let mut entries = self.entries.lock().expect("rate limit tracker poisoned");
        let key = (client_id.to_string(), path.to_string());
        let now = Instant::now();

        let entry = entries.entry(key).or_insert((now, 0));
        if now.duration_since(entry.0) >= limit.window {
            *entry = (now, 0);
        }

        if entry.1 >= limit.max_requests {
            return None;
        }

        entry.1 += 1;
        Some(limit.max_requests.saturating_sub(entry.1))
    }

    /// Seconds until the current window resets for a client/endpoint pair.
    fn reset_after(&self, client_id: &str, path: &str, limit: RateLimit) -> u64 {
        let entries = self.entries.lock().expect("rate limit tracker poisoned");
        let key = (client_id.to_string(), path.to_string());
        match entries.get(&key) {
            Some((start, _)) => {
                let elapsed = Instant::now().duration_since(*start);
                limit.window.saturating_sub(elapsed).as_secs()
            }
            None => limit.window.as_secs(),
        }
    }
}

/// Extract a stable client identifier from the request.
/// Prefers an API key, then falls back to the remote address.
fn client_id<B>(req: &Request<B>) -> String {
    if let Some(key) = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
    {
        return format!("key:{key}");
    }

    if let Some(addr) = req
        .extensions()
        .get::<std::net::SocketAddr>()
    {
        return format!("ip:{addr}");
    }

    "anonymous".to_string()
}

/// Axum middleware enforcing per-client, per-endpoint rate limits.
pub async fn rate_limit_middleware(
    State((config, tracker)): State<(RateLimitConfig, std::sync::Arc<RateLimitTracker>)>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let client = client_id(&req);

    // Trusted clients bypass rate limiting entirely.
    if config.is_trusted(&client) {
        return next.run(req).await;
    }

    let limit = config.limit_for(&path);
    let reset = tracker.reset_after(&client, &path, limit);

    match tracker.record(&client, &path, limit) {
        Some(remaining) => {
            let mut response = next.run(req).await;
            let headers = response.headers_mut();
            headers.insert("x-ratelimit-limit", limit.max_requests.into());
            headers.insert("x-ratelimit-remaining", remaining.into());
            headers.insert("x-ratelimit-reset", reset.into());
            response
        }
        None => {
            tracing::warn!(
                client = %client,
                path = %path,
                limit = limit.max_requests,
                "rate limit exceeded"
            );

            let body = format!(
                "{{\"error\":\"rate_limit_exceeded\",\"retry_after\":{reset}}}"
            );
            let mut response = (StatusCode::TOO_MANY_REQUESTS, body).into_response();
            let headers = response.headers_mut();
            headers.insert("x-ratelimit-limit", limit.max_requests.into());
            headers.insert("x-ratelimit-remaining", 0.into());
            headers.insert("x-ratelimit-reset", reset.into());
            headers.insert("retry-after", reset.into());
            response
        }
    }
}
