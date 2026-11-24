//! Comprehensive audit logging for security-relevant events.
//!
//! This module defines the taxonomy of security events and provides an
//! append-only, immutable audit log. Entries can be queried and reported on
//! for compliance audits and incident investigations.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// The taxonomy of security-relevant events captured by the audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityEvent {
    /// A login attempt, successful or not.
    Login {
        user: String,
        success: bool,
        reason: Option<String>,
    },
    /// A logout event.
    Logout { user: String },
    /// Read access to data, with the stated purpose.
    DataAccess {
        user: String,
        resource: String,
        purpose: String,
    },
    /// Modification of data, with the stated purpose.
    DataModification {
        user: String,
        resource: String,
        purpose: String,
    },
    /// A permission or role change, capturing old and new values.
    PermissionChange {
        actor: String,
        subject: String,
        old_value: String,
        new_value: String,
    },
}

impl SecurityEvent {
    /// A short, stable label for the event kind, useful for reporting.
    pub fn kind(&self) -> &'static str {
        match self {
            SecurityEvent::Login { .. } => "login",
            SecurityEvent::Logout { .. } => "logout",
            SecurityEvent::DataAccess { .. } => "data_access",
            SecurityEvent::DataModification { .. } => "data_modification",
            SecurityEvent::PermissionChange { .. } => "permission_change",
        }
    }

    /// The primary user associated with the event.
    pub fn user(&self) -> &str {
        match self {
            SecurityEvent::Login { user, .. } => user,
            SecurityEvent::Logout { user } => user,
            SecurityEvent::DataAccess { user, .. } => user,
            SecurityEvent::DataModification { user, .. } => user,
            SecurityEvent::PermissionChange { actor, .. } => actor,
        }
    }
}

/// A single immutable audit log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    /// Monotonic sequence number assigned at append time.
    pub sequence: u64,
    /// Unix timestamp (seconds) at which the event was recorded.
    pub timestamp: u64,
    /// The recorded security event.
    pub event: SecurityEvent,
}

/// An append-only audit log.
///
/// Entries are never mutated or removed through the public API, satisfying the
/// immutability requirement. Retention is bounded by `retention` entries so
/// that storage meets compliance needs without unbounded growth.
#[derive(Debug)]
pub struct AuditLog {
    entries: Mutex<VecDeque<AuditEntry>>,
    next_sequence: Mutex<u64>,
    retention: usize,
}

impl AuditLog {
    /// Create a new audit log retaining up to `retention` entries.
    pub fn new(retention: usize) -> Self {
        AuditLog {
            entries: Mutex::new(VecDeque::new()),
            next_sequence: Mutex::new(0),
            retention: retention.max(1),
        }
    }

    /// Record a security event, returning the immutable entry that was stored.
    pub fn record(&self, event: SecurityEvent) -> AuditEntry {
        let mut seq = self.next_sequence.lock().expect("audit sequence poisoned");
        let sequence = *seq;
        *seq += 1;
        drop(seq);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = AuditEntry {
            sequence,
            timestamp,
            event,
        };

        let mut entries = self.entries.lock().expect("audit log poisoned");
        entries.push_back(entry.clone());
        while entries.len() > self.retention {
            entries.pop_front();
        }
        entry
    }

    /// Convenience helper for recording a login attempt.
    pub fn record_login(&self, user: &str, success: bool, reason: Option<&str>) -> AuditEntry {
        self.record(SecurityEvent::Login {
            user: user.to_string(),
            success,
            reason: reason.map(|r| r.to_string()),
        })
    }

    /// Convenience helper for recording data access.
    pub fn record_data_access(&self, user: &str, resource: &str, purpose: &str) -> AuditEntry {
        self.record(SecurityEvent::DataAccess {
            user: user.to_string(),
            resource: resource.to_string(),
            purpose: purpose.to_string(),
        })
    }

    /// Convenience helper for recording data modification.
    pub fn record_data_modification(
        &self,
        user: &str,
        resource: &str,
        purpose: &str,
    ) -> AuditEntry {
        self.record(SecurityEvent::DataModification {
            user: user.to_string(),
            resource: resource.to_string(),
            purpose: purpose.to_string(),
        })
    }

    /// Convenience helper for recording a permission or role change.
    pub fn record_permission_change(
        &self,
        actor: &str,
        subject: &str,
        old_value: &str,
        new_value: &str,
    ) -> AuditEntry {
        self.record(SecurityEvent::PermissionChange {
            actor: actor.to_string(),
            subject: subject.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        })
    }

    /// Return a snapshot of all retained entries, oldest first.
    pub fn entries(&self) -> Vec<AuditEntry> {
        self.entries
            .lock()
            .expect("audit log poisoned")
            .iter()
            .cloned()
            .collect()
    }

    /// Query entries matching a predicate, oldest first.
    pub fn query<F>(&self, predicate: F) -> Vec<AuditEntry>
    where
        F: Fn(&AuditEntry) -> bool,
    {
        self.entries
            .lock()
            .expect("audit log poisoned")
            .iter()
            .filter(|e| predicate(e))
            .cloned()
            .collect()
    }

    /// Query entries for a specific user.
    pub fn query_by_user(&self, user: &str) -> Vec<AuditEntry> {
        self.query(|e| e.event.user() == user)
    }

    /// Query entries of a specific event kind (see [`SecurityEvent::kind`]).
    pub fn query_by_kind(&self, kind: &str) -> Vec<AuditEntry> {
        self.query(|e| e.event.kind() == kind)
    }

    /// Produce a simple per-kind count report for retained entries.
    pub fn report(&self) -> Vec<(&'static str, usize)> {
        let entries = self.entries.lock().expect("audit log poisoned");
        let mut counts: Vec<(&'static str, usize)> = Vec::new();
        for entry in entries.iter() {
            let kind = entry.event.kind();
            match counts.iter_mut().find(|(k, _)| *k == kind) {
                Some((_, count)) => *count += 1,
                None => counts.push((kind, 1)),
            }
        }
        counts
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        // Default retention of 100_000 entries.
        AuditLog::new(100_000)
    }
}
