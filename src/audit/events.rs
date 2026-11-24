//! Audit logging for security-relevant events.
//!
//! Defines the taxonomy of security events and the immutable records that are
//! captured for compliance audits and incident investigations.

use std::fmt;

/// Classification of a security-relevant action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityEventKind {
    /// An authentication attempt (successful or failed).
    Login,
    /// A logout / session termination.
    Logout,
    /// Reading or querying data.
    DataAccess,
    /// Creating, updating, or deleting data.
    DataModification,
    /// A change to a permission grant.
    PermissionChange,
    /// A change to a user's role assignment.
    RoleChange,
}

impl SecurityEventKind {
    /// Stable string identifier suitable for persistence and querying.
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityEventKind::Login => "login",
            SecurityEventKind::Logout => "logout",
            SecurityEventKind::DataAccess => "data_access",
            SecurityEventKind::DataModification => "data_modification",
            SecurityEventKind::PermissionChange => "permission_change",
            SecurityEventKind::RoleChange => "role_change",
        }
    }
}

impl fmt::Display for SecurityEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Outcome of an authentication attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginResult {
    Success,
    Failure,
}

impl LoginResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginResult::Success => "success",
            LoginResult::Failure => "failure",
        }
    }
}

/// The actor responsible for an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    /// Identifier of the user (or service account) performing the action.
    pub user_id: String,
    /// Optional source address (IP / host) the action originated from.
    pub source: Option<String>,
}

impl Actor {
    pub fn new(user_id: impl Into<String>) -> Self {
        Actor {
            user_id: user_id.into(),
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// A before/after pair capturing a change to a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueChange {
    pub old: Option<String>,
    pub new: Option<String>,
}

impl ValueChange {
    pub fn new(old: Option<String>, new: Option<String>) -> Self {
        ValueChange { old, new }
    }
}

/// The payload describing what happened in a security event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityEventDetails {
    /// Authentication attempt with its result.
    Login { result: LoginResult },
    /// Session termination.
    Logout,
    /// Data access, recording the resource and the stated purpose.
    DataAccess { resource: String, purpose: String },
    /// Data modification, recording the resource and the stated purpose.
    DataModification { resource: String, purpose: String },
    /// Permission change with old and new values.
    PermissionChange {
        resource: String,
        change: ValueChange,
    },
    /// Role change with old and new values.
    RoleChange { change: ValueChange },
}

impl SecurityEventDetails {
    /// The event kind implied by these details.
    pub fn kind(&self) -> SecurityEventKind {
        match self {
            SecurityEventDetails::Login { .. } => SecurityEventKind::Login,
            SecurityEventDetails::Logout => SecurityEventKind::Logout,
            SecurityEventDetails::DataAccess { .. } => SecurityEventKind::DataAccess,
            SecurityEventDetails::DataModification { .. } => SecurityEventKind::DataModification,
            SecurityEventDetails::PermissionChange { .. } => SecurityEventKind::PermissionChange,
            SecurityEventDetails::RoleChange { .. } => SecurityEventKind::RoleChange,
        }
    }
}

/// An immutable audit log record.
///
/// Records are append-only: all fields are private and there are no mutators,
/// so once constructed a record cannot be altered. Retention is enforced by the
/// backing store, which must never delete records before the compliance window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    timestamp: u64,
    actor: Actor,
    details: SecurityEventDetails,
}

impl AuditEvent {
    /// Create a new audit record stamped with the given Unix timestamp (seconds).
    pub fn new(timestamp: u64, actor: Actor, details: SecurityEventDetails) -> Self {
        AuditEvent {
            timestamp,
            actor,
            details,
        }
    }

    /// Convenience constructor for a login attempt.
    pub fn login(timestamp: u64, actor: Actor, result: LoginResult) -> Self {
        AuditEvent::new(timestamp, actor, SecurityEventDetails::Login { result })
    }

    /// Convenience constructor for a data access event.
    pub fn data_access(
        timestamp: u64,
        actor: Actor,
        resource: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        AuditEvent::new(
            timestamp,
            actor,
            SecurityEventDetails::DataAccess {
                resource: resource.into(),
                purpose: purpose.into(),
            },
        )
    }

    /// Convenience constructor for a data modification event.
    pub fn data_modification(
        timestamp: u64,
        actor: Actor,
        resource: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        AuditEvent::new(
            timestamp,
            actor,
            SecurityEventDetails::DataModification {
                resource: resource.into(),
                purpose: purpose.into(),
            },
        )
    }

    /// Convenience constructor for a permission change event.
    pub fn permission_change(
        timestamp: u64,
        actor: Actor,
        resource: impl Into<String>,
        change: ValueChange,
    ) -> Self {
        AuditEvent::new(
            timestamp,
            actor,
            SecurityEventDetails::PermissionChange {
                resource: resource.into(),
                change,
            },
        )
    }

    /// Convenience constructor for a role change event.
    pub fn role_change(timestamp: u64, actor: Actor, change: ValueChange) -> Self {
        AuditEvent::new(timestamp, actor, SecurityEventDetails::RoleChange { change })
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn details(&self) -> &SecurityEventDetails {
        &self.details
    }

    pub fn kind(&self) -> SecurityEventKind {
        self.details.kind()
    }
}
