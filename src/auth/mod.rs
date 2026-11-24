use std::collections::HashSet;
use std::fmt;

/// Application roles. At least three roles are defined as required by #971.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Operator,
    Viewer,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Viewer => "viewer",
        }
    }

    pub fn from_str(value: &str) -> Option<Role> {
        match value.trim().to_ascii_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "operator" => Some(Role::Operator),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    /// Permissions granted to this role.
    pub fn permissions(&self) -> HashSet<Permission> {
        let mut perms = HashSet::new();
        match self {
            Role::Admin => {
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
                perms.insert(Permission::Delete);
                perms.insert(Permission::ManageUsers);
                perms.insert(Permission::ManageRoles);
            }
            Role::Operator => {
                perms.insert(Permission::Read);
                perms.insert(Permission::Write);
            }
            Role::Viewer => {
                perms.insert(Permission::Read);
            }
        }
        perms
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions().contains(&permission)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Fine-grained permissions checked by middleware and endpoint guards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    ManageUsers,
    ManageRoles,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::Read => "read",
            Permission::Write => "write",
            Permission::Delete => "delete",
            Permission::ManageUsers => "manage_users",
            Permission::ManageRoles => "manage_roles",
        }
    }
}

/// An authenticated principal with an assigned role.
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub role: Role,
}

impl User {
    pub fn new(id: impl Into<String>, role: Role) -> Self {
        User {
            id: id.into(),
            role,
        }
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.role.has_permission(permission)
    }
}

/// Result of an authorization check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    Unauthenticated,
    Forbidden(Permission),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Unauthenticated => f.write_str("authentication required"),
            AuthError::Forbidden(p) => write!(f, "missing permission: {}", p.as_str()),
        }
    }
}

impl std::error::Error for AuthError {}

/// Permission-checking middleware: enforces that the current user holds the
/// required permission, returning 403 (Forbidden) when unauthorized.
pub fn authorize(user: Option<&User>, required: Permission) -> Result<(), AuthError> {
    match user {
        None => Err(AuthError::Unauthenticated),
        Some(u) if u.has_permission(required) => Ok(()),
        Some(_) => Err(AuthError::Forbidden(required)),
    }
}

/// Map an authorization result to an HTTP status code.
/// Unauthenticated -> 401, Forbidden -> 403, Ok -> 200.
pub fn status_for(result: &Result<(), AuthError>) -> u16 {
    match result {
        Ok(()) => 200,
        Err(AuthError::Unauthenticated) => 401,
        Err(AuthError::Forbidden(_)) => 403,
    }
}

/// Endpoint guard decorator: wraps a handler so it only runs when the user
/// holds the required permission. Unauthorized access yields 403.
pub fn require_permission<F, T>(user: Option<&User>, required: Permission, handler: F) -> Result<T, AuthError>
where
    F: FnOnce(&User) -> T,
{
    authorize(user, required)?;
    // Safe: authorize() guarantees a user is present.
    let user = user.expect("authorize guarantees a user");
    Ok(handler(user))
}

/// Role management: assign a role to a user. Only principals with the
/// ManageRoles permission (admins) may perform this action.
pub fn assign_role(actor: Option<&User>, target: &mut User, new_role: Role) -> Result<(), AuthError> {
    authorize(actor, Permission::ManageRoles)?;
    target.role = new_role;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_have_expected_permissions() {
        assert!(Role::Admin.has_permission(Permission::ManageRoles));
        assert!(Role::Operator.has_permission(Permission::Write));
        assert!(!Role::Operator.has_permission(Permission::Delete));
        assert!(Role::Viewer.has_permission(Permission::Read));
        assert!(!Role::Viewer.has_permission(Permission::Write));
    }

    #[test]
    fn unauthorized_returns_403() {
        let viewer = User::new("u1", Role::Viewer);
        let result = authorize(Some(&viewer), Permission::Write);
        assert_eq!(status_for(&result), 403);
    }

    #[test]
    fn admin_can_assign_roles() {
        let admin = User::new("a1", Role::Admin);
        let mut target = User::new("u2", Role::Viewer);
        assert!(assign_role(Some(&admin), &mut target, Role::Operator).is_ok());
        assert_eq!(target.role, Role::Operator);
    }

    #[test]
    fn non_admin_cannot_assign_roles() {
        let operator = User::new("o1", Role::Operator);
        let mut target = User::new("u2", Role::Viewer);
        assert!(assign_role(Some(&operator), &mut target, Role::Admin).is_err());
        assert_eq!(target.role, Role::Viewer);
    }
}
