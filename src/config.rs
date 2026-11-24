use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single feature flag definition with targeting and rollout rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// Unique flag name, e.g. "new_dashboard".
    pub name: String,
    /// Master on/off switch. When false the flag is off for everyone.
    pub enabled: bool,
    /// Percentage (0-100) of users for whom the flag is enabled.
    #[serde(default)]
    pub rollout_percentage: u8,
    /// Explicit user IDs that always get the flag.
    #[serde(default)]
    pub target_users: Vec<String>,
    /// Explicit group names that always get the flag.
    #[serde(default)]
    pub target_groups: Vec<String>,
}

impl FeatureFlag {
    /// Evaluate the flag for a given user and their groups.
    ///
    /// Targeting takes precedence over percentage rollout. Percentage
    /// rollout is deterministic per user so a user always sees a stable
    /// result for a given flag.
    pub fn evaluate(&self, user_id: &str, groups: &[String]) -> bool {
        if !self.enabled {
            return false;
        }
        if self.target_users.iter().any(|u| u == user_id) {
            return true;
        }
        if self
            .target_groups
            .iter()
            .any(|g| groups.iter().any(|ug| ug == g))
        {
            return true;
        }
        if self.rollout_percentage == 0 {
            return false;
        }
        if self.rollout_percentage >= 100 {
            return true;
        }
        bucket_for(&self.name, user_id) < self.rollout_percentage
    }
}

/// Deterministic bucket (0-99) for a user within a flag's rollout.
fn bucket_for(flag_name: &str, user_id: &str) -> u8 {
    // FNV-1a hash keeps evaluation stable across processes without
    // pulling in extra dependencies.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in flag_name.bytes().chain(b"\0").chain(user_id.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash % 100) as u8
}

/// Collection of feature flags keyed by name.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureFlags {
    #[serde(default)]
    pub flags: HashMap<String, FeatureFlag>,
}

impl FeatureFlags {
    /// Returns true when the named flag is enabled for the user.
    /// Unknown flags evaluate to false.
    pub fn is_enabled(&self, name: &str, user_id: &str, groups: &[String]) -> bool {
        self.flags
            .get(name)
            .map(|flag| flag.evaluate(user_id, groups))
            .unwrap_or(false)
    }

    /// Toggle a flag on or off without a redeploy.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        match self.flags.get_mut(name) {
            Some(flag) => {
                flag.enabled = enabled;
                true
            }
            None => false,
        }
    }

    /// Update the percentage rollout for a flag (clamped to 0-100).
    pub fn set_rollout_percentage(&mut self, name: &str, percentage: u8) -> bool {
        match self.flags.get_mut(name) {
            Some(flag) => {
                flag.rollout_percentage = percentage.min(100);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag(name: &str, enabled: bool, pct: u8) -> FeatureFlag {
        FeatureFlag {
            name: name.to_string(),
            enabled,
            rollout_percentage: pct,
            target_users: Vec::new(),
            target_groups: Vec::new(),
        }
    }

    #[test]
    fn disabled_flag_is_off() {
        assert!(!flag("f", false, 100).evaluate("u1", &[]));
    }

    #[test]
    fn full_rollout_is_on() {
        assert!(flag("f", true, 100).evaluate("u1", &[]));
    }

    #[test]
    fn zero_rollout_is_off() {
        assert!(!flag("f", true, 0).evaluate("u1", &[]));
    }

    #[test]
    fn targeted_user_is_on() {
        let mut f = flag("f", true, 0);
        f.target_users.push("u1".to_string());
        assert!(f.evaluate("u1", &[]));
        assert!(!f.evaluate("u2", &[]));
    }

    #[test]
    fn targeted_group_is_on() {
        let mut f = flag("f", true, 0);
        f.target_groups.push("beta".to_string());
        assert!(f.evaluate("u1", &["beta".to_string()]));
        assert!(!f.evaluate("u1", &[]));
    }

    #[test]
    fn percentage_rollout_is_deterministic() {
        let f = flag("f", true, 50);
        let first = f.evaluate("u1", &[]);
        assert_eq!(first, f.evaluate("u1", &[]));
    }

    #[test]
    fn toggle_without_redeploy() {
        let mut flags = FeatureFlags::default();
        flags.flags.insert("f".to_string(), flag("f", false, 100));
        assert!(!flags.is_enabled("f", "u1", &[]));
        assert!(flags.set_enabled("f", true));
        assert!(flags.is_enabled("f", "u1", &[]));
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per window for a single user/API key.
    pub max_requests: u64,
    /// Length of the rate limit window in seconds.
    pub window_seconds: u64,
    /// Whether rate limiting is enabled for this environment.
    pub enabled: bool,
    /// API keys that are exempt from rate limiting (trusted clients).
    pub trusted_api_keys: Vec<String>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            enabled: true,
            trusted_api_keys: Vec::new(),
        }
    }
}

impl RateLimitConfig {
    /// Build a rate limit config from environment variables, falling back to
    /// sensible defaults so limits are configurable per environment.
    pub fn from_env() -> Self {
        let defaults = Self::default();

        let max_requests = env::var("RATE_LIMIT_MAX_REQUESTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.max_requests);

        let window_seconds = env::var("RATE_LIMIT_WINDOW_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(defaults.window_seconds);

        let enabled = env::var("RATE_LIMIT_ENABLED")
            .ok()
            .map(|v| v != "false" && v != "0")
            .unwrap_or(defaults.enabled);

        let trusted_api_keys = env::var("RATE_LIMIT_TRUSTED_API_KEYS")
            .ok()
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or(defaults.trusted_api_keys);

        Self {
            max_requests,
            window_seconds,
            enabled,
            trusted_api_keys,
        }
    }

    /// Returns true when the given API key is a trusted client that should
    /// bypass rate limiting.
    pub fn is_trusted(&self, api_key: &str) -> bool {
        self.trusted_api_keys.iter().any(|k| k == api_key)
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_default(),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            rate_limit: RateLimitConfig::from_env(),
use std::time::Duration;

/// Database connection pool configuration.
///
/// Pool sizing is capped at 50 concurrent connections per the acceptance
/// criteria for issue #967. Acquisition timeout is kept well under the 10ms
/// target so that normal load never surfaces timeout errors.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of concurrent connections held by the pool.
    pub max_size: u32,
    /// Minimum number of idle connections kept warm.
    pub min_idle: Option<u32>,
    /// How long an idle connection may live before being reaped.
    pub idle_timeout: Option<Duration>,
    /// Maximum lifetime of a connection before it is recycled.
    pub max_lifetime: Option<Duration>,
    /// How long a caller waits to acquire a connection.
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 50,
            min_idle: Some(5),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
            acquire_timeout: Duration::from_millis(10),
        }
    }
}

impl PoolConfig {
    /// Build a pool config from explicit sizing, clamped to the 50-connection cap.
    pub fn new(max_size: u32) -> Self {
        Self {
            max_size: max_size.min(50),
            ..Self::default()
        }
    }
}
