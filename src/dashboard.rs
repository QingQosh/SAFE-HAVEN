//! Dashboard data loading with caching and lazy widget support.
//!
//! Issue #975: the dashboard previously aggregated every widget's data in a
//! single synchronous pass, which made the initial load slow and prone to
//! timeouts. This module splits the work into:
//!
//! * a cheap, cached *summary* payload that is returned on the initial load,
//! * per-widget data that can be fetched lazily by the client,
//! * a small TTL cache so repeated loads reuse previously computed results.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Default time-to-live for cached dashboard data.
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(30);

/// Identifies a dashboard widget whose data can be loaded lazily.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidgetId {
    Summary,
    Metrics,
    Activity,
    Alerts,
}

impl WidgetId {
    /// All widgets that make up a dashboard, in display order.
    pub fn all() -> &'static [WidgetId] {
        &[
            WidgetId::Summary,
            WidgetId::Metrics,
            WidgetId::Activity,
            WidgetId::Alerts,
        ]
    }

    /// Stable string key used by the cache and the HTTP layer.
    pub fn as_str(&self) -> &'static str {
        match self {
            WidgetId::Summary => "summary",
            WidgetId::Metrics => "metrics",
            WidgetId::Activity => "activity",
            WidgetId::Alerts => "alerts",
        }
    }

    /// Parse a widget key coming from a request path/query.
    pub fn parse(key: &str) -> Option<WidgetId> {
        match key {
            "summary" => Some(WidgetId::Summary),
            "metrics" => Some(WidgetId::Metrics),
            "activity" => Some(WidgetId::Activity),
            "alerts" => Some(WidgetId::Alerts),
            _ => None,
        }
    }
}

/// A single widget's rendered data.
#[derive(Debug, Clone)]
pub struct WidgetData {
    pub widget: WidgetId,
    pub payload: String,
}

/// Initial dashboard payload: only the summary widget is computed eagerly so
/// the page can render immediately; the rest are loaded lazily.
#[derive(Debug, Clone)]
pub struct DashboardSummary {
    pub summary: WidgetData,
    pub pending_widgets: Vec<&'static str>,
}

/// Source of raw dashboard data. Implementations perform the actual
/// aggregation; the dashboard layer is responsible for caching and splitting
/// the work so slow aggregations never block the initial load.
pub trait DashboardSource: Send + Sync {
    /// Compute the data for a single widget.
    fn load_widget(&self, widget: &WidgetId) -> String;
}

struct CacheEntry {
    data: WidgetData,
    inserted_at: Instant,
}

/// Caches per-widget data with a TTL so repeated dashboard loads reuse
/// previously computed aggregations instead of re-running them.
pub struct DashboardCache {
    ttl: Duration,
    entries: Mutex<HashMap<WidgetId, CacheEntry>>,
}

impl DashboardCache {
    pub fn new(ttl: Duration) -> Self {
        DashboardCache {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Return cached data for `widget` if it is still fresh.
    fn get(&self, widget: &WidgetId) -> Option<WidgetData> {
        let entries = self.entries.lock().ok()?;
        let entry = entries.get(widget)?;
        if entry.inserted_at.elapsed() < self.ttl {
            Some(entry.data.clone())
        } else {
            None
        }
    }

    fn put(&self, data: WidgetData) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(
                data.widget.clone(),
                CacheEntry {
                    data,
                    inserted_at: Instant::now(),
                },
            );
        }
    }

    /// Drop all cached entries (e.g. after a write invalidates the dashboard).
    pub fn invalidate(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }
}

impl Default for DashboardCache {
    fn default() -> Self {
        DashboardCache::new(DEFAULT_CACHE_TTL)
    }
}

/// Loads dashboard data, caching per-widget results and supporting lazy
/// loading of individual widgets.
pub struct DashboardLoader {
    source: Arc<dyn DashboardSource>,
    cache: DashboardCache,
}

impl DashboardLoader {
    pub fn new(source: Arc<dyn DashboardSource>) -> Self {
        DashboardLoader {
            source,
            cache: DashboardCache::default(),
        }
    }

    pub fn with_cache(source: Arc<dyn DashboardSource>, cache: DashboardCache) -> Self {
        DashboardLoader { source, cache }
    }

    /// Load a single widget, using the cache when possible.
    pub fn load_widget(&self, widget: &WidgetId) -> WidgetData {
        if let Some(cached) = self.cache.get(widget) {
            return cached;
        }
        let data = WidgetData {
            widget: widget.clone(),
            payload: self.source.load_widget(widget),
        };
        self.cache.put(data.clone());
        data
    }

    /// Load the initial dashboard payload. Only the summary widget is computed
    /// eagerly; the remaining widgets are reported as pending so the client can
    /// fetch them lazily and render the page quickly.
    pub fn load_summary(&self) -> DashboardSummary {
        let summary = self.load_widget(&WidgetId::Summary);
        let pending_widgets = WidgetId::all()
            .iter()
            .filter(|w| **w != WidgetId::Summary)
            .map(|w| w.as_str())
            .collect();
        DashboardSummary {
            summary,
            pending_widgets,
        }
    }

    /// Invalidate cached dashboard data.
    pub fn invalidate(&self) {
        self.cache.invalidate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingSource {
        calls: AtomicUsize,
    }

    impl DashboardSource for CountingSource {
        fn load_widget(&self, widget: &WidgetId) -> String {
            self.calls.fetch_add(1, Ordering::SeqCst);
            format!("data-{}", widget.as_str())
        }
    }

    #[test]
    fn summary_only_loads_summary_widget() {
        let source = Arc::new(CountingSource {
            calls: AtomicUsize::new(0),
        });
        let loader = DashboardLoader::new(source.clone());
        let summary = loader.load_summary();
        assert_eq!(summary.summary.widget, WidgetId::Summary);
        assert_eq!(summary.pending_widgets.len(), 3);
        assert_eq!(source.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn repeated_loads_use_cache() {
        let source = Arc::new(CountingSource {
            calls: AtomicUsize::new(0),
        });
        let loader = DashboardLoader::new(source.clone());
        loader.load_widget(&WidgetId::Metrics);
        loader.load_widget(&WidgetId::Metrics);
        assert_eq!(source.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn invalidate_forces_reload() {
        let source = Arc::new(CountingSource {
            calls: AtomicUsize::new(0),
        });
        let loader = DashboardLoader::new(source.clone());
        loader.load_widget(&WidgetId::Alerts);
        loader.invalidate();
        loader.load_widget(&WidgetId::Alerts);
        assert_eq!(source.calls.load(Ordering::SeqCst), 2);
    }
}
