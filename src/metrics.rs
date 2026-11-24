//! System health monitoring: metrics collection, dashboard, alerting, and
//! historical storage (issue #973).
//!
//! This module provides a lightweight, in-process metrics collection system
//! that records API response times, error rates/types, database performance,
//! and uptime. It exposes a dashboard renderer and threshold-based alerting,
//! and retains historical samples for the last 30 days.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How long historical metrics are retained (30 days).
const RETENTION: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// A single recorded API request observation.
#[derive(Debug, Clone)]
pub struct RequestSample {
    pub timestamp: SystemTime,
    pub duration: Duration,
    pub status: u16,
    pub error_type: Option<String>,
}

/// A single recorded database operation observation.
#[derive(Debug, Clone)]
pub struct DbSample {
    pub timestamp: SystemTime,
    pub duration: Duration,
    pub query: String,
    pub failed: bool,
}

/// Aggregated API response-time statistics for a window.
#[derive(Debug, Clone, Default)]
pub struct ResponseTimeStats {
    pub count: usize,
    pub avg_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
}

/// Aggregated error statistics for a window.
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    pub total_requests: usize,
    pub total_errors: usize,
    pub error_rate: f64,
    pub by_type: HashMap<String, usize>,
}

/// Aggregated database performance statistics for a window.
#[derive(Debug, Clone, Default)]
pub struct DbStats {
    pub count: usize,
    pub avg_ms: f64,
    pub max_ms: f64,
    pub failures: usize,
}

/// A critical threshold that, when breached, raises an alert.
#[derive(Debug, Clone)]
pub struct Threshold {
    pub name: &'static str,
    pub max_error_rate: f64,
    pub max_avg_response_ms: f64,
    pub max_db_avg_ms: f64,
}

impl Default for Threshold {
    fn default() -> Self {
        Threshold {
            name: "service_degradation",
            max_error_rate: 0.05,
            max_avg_response_ms: 500.0,
            max_db_avg_ms: 250.0,
        }
    }
}

/// An alert raised when a threshold is breached.
#[derive(Debug, Clone)]
pub struct Alert {
    pub name: String,
    pub message: String,
    pub raised_at: SystemTime,
}

/// Thread-safe metrics collector with bounded historical storage.
#[derive(Clone)]
pub struct MetricsCollector {
    inner: Arc<Mutex<Inner>>,
    started_at: Instant,
    threshold: Threshold,
}

struct Inner {
    requests: VecDeque<RequestSample>,
    db: VecDeque<DbSample>,
    alerts: Vec<Alert>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::with_threshold(Threshold::default())
    }

    pub fn with_threshold(threshold: Threshold) -> Self {
        MetricsCollector {
            inner: Arc::new(Mutex::new(Inner {
                requests: VecDeque::new(),
                db: VecDeque::new(),
                alerts: Vec::new(),
            })),
            started_at: Instant::now(),
            threshold,
        }
    }

    /// Uptime of the process since the collector was created.
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Record an API request observation.
    pub fn record_request(&self, duration: Duration, status: u16, error_type: Option<&str>) {
        let sample = RequestSample {
            timestamp: SystemTime::now(),
            duration,
            status,
            error_type: error_type.map(|s| s.to_string()),
        };
        let mut inner = self.inner.lock().unwrap();
        inner.requests.push_back(sample);
        Self::prune(&mut inner.requests, |s| s.timestamp);
        Self::evaluate_alerts(&mut inner, &self.threshold);
    }

    /// Record a database operation observation.
    pub fn record_db(&self, duration: Duration, query: &str, failed: bool) {
        let sample = DbSample {
            timestamp: SystemTime::now(),
            duration,
            query: query.to_string(),
            failed,
        };
        let mut inner = self.inner.lock().unwrap();
        inner.db.push_back(sample);
        Self::prune(&mut inner.db, |s| s.timestamp);
        Self::evaluate_alerts(&mut inner, &self.threshold);
    }

    /// Response-time statistics over the last `window`.
    pub fn response_time_stats(&self, window: Duration) -> ResponseTimeStats {
        let inner = self.inner.lock().unwrap();
        let cutoff = SystemTime::now() - window;
        let mut durations: Vec<f64> = inner
            .requests
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .map(|s| s.duration.as_secs_f64() * 1000.0)
            .collect();
        if durations.is_empty() {
            return ResponseTimeStats::default();
        }
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let count = durations.len();
        let sum: f64 = durations.iter().sum();
        let p95_idx = ((count as f64) * 0.95).ceil() as usize;
        let p95 = durations[p95_idx.saturating_sub(1).min(count - 1)];
        ResponseTimeStats {
            count,
            avg_ms: sum / count as f64,
            p95_ms: p95,
            max_ms: *durations.last().unwrap(),
        }
    }

    /// Error statistics over the last `window`.
    pub fn error_stats(&self, window: Duration) -> ErrorStats {
        let inner = self.inner.lock().unwrap();
        let cutoff = SystemTime::now() - window;
        let mut stats = ErrorStats::default();
        for s in inner.requests.iter().filter(|s| s.timestamp >= cutoff) {
            stats.total_requests += 1;
            if s.status >= 400 || s.error_type.is_some() {
                stats.total_errors += 1;
                let key = s
                    .error_type
                    .clone()
                    .unwrap_or_else(|| format!("http_{}", s.status));
                *stats.by_type.entry(key).or_insert(0) += 1;
            }
        }
        if stats.total_requests > 0 {
            stats.error_rate = stats.total_errors as f64 / stats.total_requests as f64;
        }
        stats
    }

    /// Database performance statistics over the last `window`.
    pub fn db_stats(&self, window: Duration) -> DbStats {
        let inner = self.inner.lock().unwrap();
        let cutoff = SystemTime::now() - window;
        let mut stats = DbStats::default();
        let mut sum = 0.0;
        for s in inner.db.iter().filter(|s| s.timestamp >= cutoff) {
            let ms = s.duration.as_secs_f64() * 1000.0;
            stats.count += 1;
            sum += ms;
            stats.max_ms = stats.max_ms.max(ms);
            if s.failed {
                stats.failures += 1;
            }
        }
        if stats.count > 0 {
            stats.avg_ms = sum / stats.count as f64;
        }
        stats
    }

    /// Currently active alerts.
    pub fn alerts(&self) -> Vec<Alert> {
        self.inner.lock().unwrap().alerts.clone()
    }

    /// Render a plain-text monitoring dashboard for the last `window`.
    pub fn render_dashboard(&self, window: Duration) -> String {
        let rt = self.response_time_stats(window);
        let err = self.error_stats(window);
        let db = self.db_stats(window);
        let alerts = self.alerts();

        let mut out = String::new();
        out.push_str("=== System Health Dashboard ===\n");
        out.push_str(&format!("Uptime: {}s\n\n", self.uptime().as_secs()));

        out.push_str("-- API Response Time --\n");
        out.push_str(&format!(
            "requests: {}  avg: {:.1}ms  p95: {:.1}ms  max: {:.1}ms\n\n",
            rt.count, rt.avg_ms, rt.p95_ms, rt.max_ms
        ));

        out.push_str("-- Error Rate --\n");
        out.push_str(&format!(
            "errors: {}/{} ({:.2}%)\n",
            err.total_errors,
            err.total_requests,
            err.error_rate * 100.0
        ));
        if !err.by_type.is_empty() {
            out.push_str("by type:\n");
            let mut types: Vec<_> = err.by_type.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            for (name, count) in types {
                out.push_str(&format!("  {}: {}\n", name, count));
            }
        }
        out.push('\n');

        out.push_str("-- Database Performance --\n");
        out.push_str(&format!(
            "queries: {}  avg: {:.1}ms  max: {:.1}ms  failures: {}\n\n",
            db.count, db.avg_ms, db.max_ms, db.failures
        ));

        out.push_str("-- Alerts --\n");
        if alerts.is_empty() {
            out.push_str("none\n");
        } else {
            for a in alerts {
                out.push_str(&format!("  [{}] {}\n", a.name, a.message));
            }
        }
        out
    }

    fn prune<T, F>(queue: &mut VecDeque<T>, ts: F)
    where
        F: Fn(&T) -> SystemTime,
    {
        let cutoff = SystemTime::now() - RETENTION;
        while let Some(front) = queue.front() {
            if ts(front) < cutoff {
                queue.pop_front();
            } else {
                break;
            }
        }
    }

    fn evaluate_alerts(inner: &mut Inner, threshold: &Threshold) {
        let window = Duration::from_secs(60);
        let cutoff = SystemTime::now() - window;

        let mut total = 0usize;
        let mut errors = 0usize;
        let mut resp_sum = 0.0;
        for s in inner.requests.iter().filter(|s| s.timestamp >= cutoff) {
            total += 1;
            resp_sum += s.duration.as_secs_f64() * 1000.0;
            if s.status >= 400 || s.error_type.is_some() {
                errors += 1;
            }
        }
        let error_rate = if total > 0 { errors as f64 / total as f64 } else { 0.0 };
        let avg_resp = if total > 0 { resp_sum / total as f64 } else { 0.0 };

        let mut db_total = 0usize;
        let mut db_sum = 0.0;
        for s in inner.db.iter().filter(|s| s.timestamp >= cutoff) {
            db_total += 1;
            db_sum += s.duration.as_secs_f64() * 1000.0;
        }
        let db_avg = if db_total > 0 { db_sum / db_total as f64 } else { 0.0 };

        let mut messages = Vec::new();
        if error_rate > threshold.max_error_rate {
            messages.push(format!("error rate {:.2}% exceeds {:.2}%", error_rate * 100.0, threshold.max_error_rate * 100.0));
        }
        if avg_resp > threshold.max_avg_response_ms {
            messages.push(format!("avg response {:.1}ms exceeds {:.1}ms", avg_resp, threshold.max_avg_response_ms));
        }
        if db_avg > threshold.max_db_avg_ms {
            messages.push(format!("db avg {:.1}ms exceeds {:.1}ms", db_avg, threshold.max_db_avg_ms));
        }

        if !messages.is_empty() {
            inner.alerts.push(Alert {
                name: threshold.name.to_string(),
                message: messages.join("; "),
                raised_at: SystemTime::now(),
            });
            // Keep alert history bounded.
            while inner.alerts.len() > 100 {
                inner.alerts.remove(0);
            }
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience: current unix timestamp in seconds (used by dashboards/exporters).
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
