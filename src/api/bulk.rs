//! Bulk operations API (issue #983).
//!
//! Provides endpoints for bulk create/update/delete of items. Each request
//! accepts an array of items, validates them individually, and reports
//! per-item errors alongside aggregate success/failure counts. Progress is
//! tracked in-memory for large operations.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Maximum number of items accepted in a single bulk request.
pub const MAX_BULK_ITEMS: usize = 1000;

/// A single item submitted to a bulk operation.
#[derive(Debug, Clone)]
pub struct BulkItem {
    pub id: Option<String>,
    pub data: serde_json::Value,
}

/// Per-item validation/processing error.
#[derive(Debug, Clone)]
pub struct ItemError {
    pub index: usize,
    pub id: Option<String>,
    pub message: String,
}

/// Aggregate result of a bulk operation.
#[derive(Debug, Clone)]
pub struct BulkResult {
    pub success_count: usize,
    pub failure_count: usize,
    pub errors: Vec<ItemError>,
}

impl BulkResult {
    fn new() -> Self {
        BulkResult {
            success_count: 0,
            failure_count: 0,
            errors: Vec::new(),
        }
    }

    fn record_error(&mut self, index: usize, id: Option<String>, message: String) {
        self.failure_count += 1;
        self.errors.push(ItemError {
            index,
            id,
            message,
        });
    }
}

/// Progress tracking for a bulk operation.
#[derive(Debug, Clone)]
pub struct BulkProgress {
    pub total: usize,
    pub processed: Arc<AtomicU64>,
}

impl BulkProgress {
    pub fn new(total: usize) -> Self {
        BulkProgress {
            total,
            processed: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn processed(&self) -> u64 {
        self.processed.load(Ordering::Relaxed)
    }

    pub fn is_complete(&self) -> bool {
        self.processed() as usize >= self.total
    }

    fn advance(&self) {
        self.processed.fetch_add(1, Ordering::Relaxed);
    }
}

/// In-memory progress registry keyed by operation id.
#[derive(Debug, Default, Clone)]
pub struct ProgressRegistry {
    inner: Arc<Mutex<HashMap<String, BulkProgress>>>,
}

impl ProgressRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, op_id: impl Into<String>, total: usize) -> BulkProgress {
        let progress = BulkProgress::new(total);
        if let Ok(mut map) = self.inner.lock() {
            map.insert(op_id.into(), progress.clone());
        }
        progress
    }

    pub fn get(&self, op_id: &str) -> Option<BulkProgress> {
        self.inner.lock().ok().and_then(|map| map.get(op_id).cloned())
    }
}

/// Error returned when a bulk request itself is invalid (not per-item).
#[derive(Debug, Clone)]
pub enum BulkRequestError {
    Empty,
    TooManyItems { count: usize, max: usize },
}

impl std::fmt::Display for BulkRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkRequestError::Empty => write!(f, "bulk request must contain at least one item"),
            BulkRequestError::TooManyItems { count, max } => write!(
                f,
                "bulk request contains {count} items, exceeding the maximum of {max}"
            ),
        }
    }
}

impl std::error::Error for BulkRequestError {}

fn validate_request(items: &[BulkItem]) -> Result<(), BulkRequestError> {
    if items.is_empty() {
        return Err(BulkRequestError::Empty);
    }
    if items.len() > MAX_BULK_ITEMS {
        return Err(BulkRequestError::TooManyItems {
            count: items.len(),
            max: MAX_BULK_ITEMS,
        });
    }
    Ok(())
}

fn validate_item(item: &BulkItem) -> Result<(), String> {
    if item.data.is_null() {
        return Err("item data must not be null".to_string());
    }
    Ok(())
}

/// Bulk create: each item must not carry an existing id.
pub fn bulk_create(
    items: &[BulkItem],
    progress: Option<&BulkProgress>,
) -> Result<BulkResult, BulkRequestError> {
    validate_request(items)?;
    let mut result = BulkResult::new();
    for (index, item) in items.iter().enumerate() {
        if let Some(id) = &item.id {
            result.record_error(
                index,
                Some(id.clone()),
                "create items must not specify an id".to_string(),
            );
        } else if let Err(msg) = validate_item(item) {
            result.record_error(index, None, msg);
        } else {
            result.success_count += 1;
        }
        if let Some(p) = progress {
            p.advance();
        }
    }
    Ok(result)
}

/// Bulk update: each item must carry an id.
pub fn bulk_update(
    items: &[BulkItem],
    progress: Option<&BulkProgress>,
) -> Result<BulkResult, BulkRequestError> {
    validate_request(items)?;
    let mut result = BulkResult::new();
    for (index, item) in items.iter().enumerate() {
        match &item.id {
            None => result.record_error(
                index,
                None,
                "update items must specify an id".to_string(),
            ),
            Some(id) => {
                if let Err(msg) = validate_item(item) {
                    result.record_error(index, Some(id.clone()), msg);
                } else {
                    result.success_count += 1;
                }
            }
        }
        if let Some(p) = progress {
            p.advance();
        }
    }
    Ok(result)
}

/// Bulk delete: each item must carry an id.
pub fn bulk_delete(
    items: &[BulkItem],
    progress: Option<&BulkProgress>,
) -> Result<BulkResult, BulkRequestError> {
    validate_request(items)?;
    let mut result = BulkResult::new();
    for (index, item) in items.iter().enumerate() {
        match &item.id {
            None => result.record_error(
                index,
                None,
                "delete items must specify an id".to_string(),
            ),
            Some(id) => result.success_count += 1,
        }
        if let Some(p) = progress {
            p.advance();
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(id: Option<&str>, data: serde_json::Value) -> BulkItem {
        BulkItem {
            id: id.map(|s| s.to_string()),
            data,
        }
    }

    #[test]
    fn rejects_empty_request() {
        assert!(matches!(bulk_create(&[], None), Err(BulkRequestError::Empty)));
    }

    #[test]
    fn rejects_too_many_items() {
        let items: Vec<BulkItem> = (0..MAX_BULK_ITEMS + 1)
            .map(|i| item(None, json!({ "n": i })))
            .collect();
        assert!(matches!(
            bulk_create(&items, None),
            Err(BulkRequestError::TooManyItems { .. })
        ));
    }

    #[test]
    fn create_reports_per_item_errors() {
        let items = vec![
            item(None, json!({ "a": 1 })),
            item(Some("x"), json!({ "a": 2 })),
            item(None, serde_json::Value::Null),
        ];
        let result = bulk_create(&items, None).unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 2);
        assert_eq!(result.errors[0].index, 1);
        assert_eq!(result.errors[1].index, 2);
    }

    #[test]
    fn update_requires_ids() {
        let items = vec![item(None, json!({})), item(Some("a"), json!({}))];
        let result = bulk_update(&items, None).unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 1);
    }

    #[test]
    fn delete_requires_ids() {
        let items = vec![item(Some("a"), json!({})), item(None, json!({}))];
        let result = bulk_delete(&items, None).unwrap();
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 1);
    }

    #[test]
    fn progress_tracks_processed_items() {
        let items: Vec<BulkItem> = (0..10).map(|i| item(None, json!({ "n": i }))).collect();
        let progress = BulkProgress::new(items.len());
        let result = bulk_create(&items, Some(&progress)).unwrap();
        assert_eq!(result.success_count, 10);
        assert_eq!(progress.processed(), 10);
        assert!(progress.is_complete());
    }

    #[test]
    fn handles_max_items() {
        let items: Vec<BulkItem> = (0..MAX_BULK_ITEMS)
            .map(|i| item(None, json!({ "n": i })))
            .collect();
        let result = bulk_create(&items, None).unwrap();
        assert_eq!(result.success_count, MAX_BULK_ITEMS);
        assert_eq!(result.failure_count, 0);
    }
}
