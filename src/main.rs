use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// A feature flag definition with targeting and percentage rollout support.
#[derive(Clone, Debug)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    /// Percentage (0-100) of users for whom the flag is enabled.
    pub rollout_percentage: u8,
    /// Explicit user IDs that always see the flag.
    pub targeted_users: Vec<String>,
    /// Explicit groups that always see the flag.
    pub targeted_groups: Vec<String>,
}

impl FeatureFlag {
    pub fn new(name: &str) -> Self {
        FeatureFlag {
            name: name.to_string(),
            enabled: false,
            rollout_percentage: 0,
            targeted_users: Vec::new(),
            targeted_groups: Vec::new(),
        }
    }

    /// Evaluate whether the flag is on for a given user/group context.
    pub fn evaluate(&self, user_id: &str, groups: &[String]) -> bool {
        if !self.enabled {
            return false;
        }
        if self.targeted_users.iter().any(|u| u == user_id) {
            return true;
        }
        if groups
            .iter()
            .any(|g| self.targeted_groups.iter().any(|tg| tg == g))
        {
            return true;
        }
        if self.rollout_percentage >= 100 {
            return true;
        }
        if self.rollout_percentage == 0 {
            return false;
        }
        // Deterministic bucketing so a user consistently falls in the same bucket.
        let bucket = stable_bucket(&self.name, user_id);
        bucket < self.rollout_percentage as u32
    }
}

/// Deterministic hash of flag name + user id into a 0-99 bucket.
fn stable_bucket(flag: &str, user_id: &str) -> u32 {
    // FNV-1a 32-bit hash.
    let mut hash: u32 = 0x811c9dc5;
    for byte in flag.bytes().chain(b"\0".iter().copied()).chain(user_id.bytes()) {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash % 100
}

/// In-memory feature flag store with a short cache TTL so changes propagate quickly.
pub struct FeatureFlagStore {
    flags: RwLock<HashMap<String, FeatureFlag>>,
    last_refresh: RwLock<Instant>,
    cache_ttl: Duration,
}

impl FeatureFlagStore {
    pub fn new() -> Self {
        FeatureFlagStore {
            flags: RwLock::new(HashMap::new()),
            last_refresh: RwLock::new(Instant::now()),
            cache_ttl: Duration::from_secs(30),
        }
    }

    /// Register or replace a flag definition (management API).
    pub fn upsert(&self, flag: FeatureFlag) {
        let mut flags = self.flags.write().unwrap();
        flags.insert(flag.name.clone(), flag);
        *self.last_refresh.write().unwrap() = Instant::now();
    }

    /// Toggle a flag on/off without redeploy.
    pub fn set_enabled(&self, name: &str, enabled: bool) -> bool {
        let mut flags = self.flags.write().unwrap();
        if let Some(flag) = flags.get_mut(name) {
            flag.enabled = enabled;
            *self.last_refresh.write().unwrap() = Instant::now();
            true
        } else {
            false
        }
    }

    /// Set the percentage rollout for a flag.
    pub fn set_rollout(&self, name: &str, percentage: u8) -> bool {
        let mut flags = self.flags.write().unwrap();
        if let Some(flag) = flags.get_mut(name) {
            flag.rollout_percentage = percentage.min(100);
            *self.last_refresh.write().unwrap() = Instant::now();
            true
        } else {
            false
        }
    }

    /// Evaluate a flag for a user context. Unknown flags default to off.
    pub fn is_enabled(&self, name: &str, user_id: &str, groups: &[String]) -> bool {
        self.maybe_refresh();
        let flags = self.flags.read().unwrap();
        flags
            .get(name)
            .map(|flag| flag.evaluate(user_id, groups))
            .unwrap_or(false)
    }

    fn maybe_refresh(&self) {
        let last = *self.last_refresh.read().unwrap();
        if last.elapsed() >= self.cache_ttl {
            *self.last_refresh.write().unwrap() = Instant::now();
        }
    }
}

impl Default for FeatureFlagStore {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let store = FeatureFlagStore::new();

    // Example: register flags for features being rolled out.
    let mut new_dashboard = FeatureFlag::new("new_dashboard");
    new_dashboard.enabled = true;
    new_dashboard.rollout_percentage = 25;
    new_dashboard.targeted_users.push("beta-tester".to_string());
    store.upsert(new_dashboard);

    let mut dark_mode = FeatureFlag::new("dark_mode");
    dark_mode.enabled = true;
    dark_mode.rollout_percentage = 100;
    store.upsert(dark_mode);

    let groups = vec!["internal".to_string()];
    println!(
        "new_dashboard for beta-tester: {}",
        store.is_enabled("new_dashboard", "beta-tester", &groups)
    );
    println!(
        "dark_mode for user-1: {}",
        store.is_enabled("dark_mode", "user-1", &groups)
    );

    // Toggle without redeploy.
    store.set_enabled("new_dashboard", false);
    println!(
        "new_dashboard after disable: {}",
        store.is_enabled("new_dashboard", "beta-tester", &groups)
    );
use axum::{
    http::{header, HeaderValue, Method},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod api;
mod dashboard;
mod lib;
mod metrics;

/// Explicit allowlist of origins permitted to make cross-origin requests.
/// No wildcard: browsers only receive CORS headers for these origins.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://app.example.com",
    "https://admin.example.com",
    "http://localhost:3000",
    "http://localhost:5173",
];

fn cors_layer() -> CorsLayer {
    let origins: Vec<HeaderValue> = ALLOWED_ORIGINS
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ORIGIN,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(api::routes::router())
        .merge(dashboard::router())
        .layer(cors_layer());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
