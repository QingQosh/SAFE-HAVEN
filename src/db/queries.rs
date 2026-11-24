//! Database query layer.
//!
//! Provides paginated access to list queries. Pagination uses an
//! offset/limit standard: callers pass a `Pagination` describing the
//! requested page, and receive a `Paginated<T>` containing the page of
//! results plus the total number of matching rows.

use serde::{Deserialize, Serialize};

/// Default number of items returned when the client does not specify a limit.
pub const DEFAULT_PAGE_SIZE: u32 = 100;

/// Hard upper bound on the number of items a single page may return.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Pagination parameters accepted by list endpoints.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Pagination {
    /// Zero-based page index. Defaults to the first page.
    #[serde(default)]
    pub page: u32,
    /// Requested page size. Clamped to `MAX_PAGE_SIZE`.
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 0,
            limit: DEFAULT_PAGE_SIZE,
        }
    }
}

impl Pagination {
    /// Build a validated pagination request, enforcing the maximum page size.
    pub fn new(page: u32, limit: u32) -> Self {
        Self {
            page,
            limit: limit.clamp(1, MAX_PAGE_SIZE),
        }
    }

    /// Number of rows to skip before returning results.
    pub fn offset(&self) -> u64 {
        u64::from(self.page) * u64::from(self.limit)
    }

    /// Number of rows to return for this page.
    pub fn limit(&self) -> u32 {
        self.limit.clamp(1, MAX_PAGE_SIZE)
    }
}

/// A page of results together with pagination metadata.
#[derive(Debug, Clone, Serialize)]
pub struct Paginated<T> {
    /// Items contained in the current page.
    pub items: Vec<T>,
    /// Total number of rows matching the query, ignoring pagination.
    pub total: u64,
    /// Zero-based page index of this response.
    pub page: u32,
    /// Page size used for this response.
    pub limit: u32,
}

impl<T> Paginated<T> {
    /// Assemble a paginated response from a page of items and the total count.
    pub fn new(items: Vec<T>, total: u64, pagination: Pagination) -> Self {
        Self {
            items,
            total,
            page: pagination.page,
            limit: pagination.limit(),
        }
    }

    /// Whether another page of results exists after this one.
    pub fn has_next(&self) -> bool {
        self.offset_end() < self.total
    }

    fn offset_end(&self) -> u64 {
        u64::from(self.page) * u64::from(self.limit) + self.items.len() as u64
    }
}

/// Build the `LIMIT`/`OFFSET` clause for a paginated query.
///
/// Returns a fragment safe to append to a `SELECT` statement along with the
/// bound values, so callers never interpolate user input into SQL.
pub fn limit_offset_clause(pagination: Pagination) -> (&'static str, u32, u64) {
    (" LIMIT ? OFFSET ?", pagination.limit(), pagination.offset())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_page_size_is_capped() {
        let p = Pagination::default();
        assert_eq!(p.limit(), DEFAULT_PAGE_SIZE);
        assert!(p.limit() <= MAX_PAGE_SIZE);
    }

    #[test]
    fn limit_is_clamped_to_max() {
        let p = Pagination::new(0, 10_000);
        assert_eq!(p.limit(), MAX_PAGE_SIZE);
    }

    #[test]
    fn limit_is_at_least_one() {
        let p = Pagination::new(0, 0);
        assert_eq!(p.limit(), 1);
    }

    #[test]
    fn offset_is_page_times_limit() {
        let p = Pagination::new(3, 25);
        assert_eq!(p.offset(), 75);
    }

    #[test]
    fn paginated_reports_total_and_next() {
        let p = Pagination::new(0, 10);
        let page = Paginated::new(vec![1, 2, 3], 42, p);
        assert_eq!(page.total, 42);
        assert_eq!(page.limit, 10);
        assert!(page.has_next());
    }

    #[test]
    fn last_page_has_no_next() {
        let p = Pagination::new(4, 10);
        let page = Paginated::new(vec![1, 2], 42, p);
        assert!(!page.has_next());
    }

    #[test]
    fn clause_binds_limit_and_offset() {
        let (sql, limit, offset) = limit_offset_clause(Pagination::new(2, 20));
        assert_eq!(sql, " LIMIT ? OFFSET ?");
        assert_eq!(limit, 20);
        assert_eq!(offset, 40);
    }
}
