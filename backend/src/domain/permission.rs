use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// User Connection Permission
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserConnectionPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub permission: String,
    pub all_tables: bool,
    pub granted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================
// User Table Permission
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserTablePermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub table_name: String,
    pub permission: String,
    pub granted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================
// Group Connection Permission
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupConnectionPermission {
    pub id: Uuid,
    pub group_id: Uuid,
    pub connection_id: Uuid,
    pub permission: String,
    pub all_tables: bool,
    pub granted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================
// Group Table Permission
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupTablePermission {
    pub id: Uuid,
    pub group_id: Uuid,
    pub connection_id: Uuid,
    pub table_name: String,
    pub permission: String,
    pub granted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================
// Resolved Permission Level (value object)
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    None,
    Read,
    Write,
    Admin,
}

impl PermissionLevel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "admin" => Self::Admin,
            "write" => Self::Write,
            "read" => Self::Read,
            _ => Self::None,
        }
    }

    pub fn can_read(&self) -> bool {
        matches!(self, Self::Read | Self::Write | Self::Admin)
    }

    pub fn can_write(&self) -> bool {
        matches!(self, Self::Write | Self::Admin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_admin() {
        assert_eq!(PermissionLevel::from_str("admin"), PermissionLevel::Admin);
    }

    #[test]
    fn from_str_write() {
        assert_eq!(PermissionLevel::from_str("write"), PermissionLevel::Write);
    }

    #[test]
    fn from_str_read() {
        assert_eq!(PermissionLevel::from_str("read"), PermissionLevel::Read);
    }

    #[test]
    fn from_str_unknown_returns_none() {
        assert_eq!(PermissionLevel::from_str("unknown"), PermissionLevel::None);
        assert_eq!(PermissionLevel::from_str(""), PermissionLevel::None);
        assert_eq!(PermissionLevel::from_str("Admin"), PermissionLevel::None); // case-sensitive
    }

    #[test]
    fn can_read_levels() {
        assert!(!PermissionLevel::None.can_read());
        assert!(PermissionLevel::Read.can_read());
        assert!(PermissionLevel::Write.can_read());
        assert!(PermissionLevel::Admin.can_read());
    }

    #[test]
    fn can_write_levels() {
        assert!(!PermissionLevel::None.can_write());
        assert!(!PermissionLevel::Read.can_write());
        assert!(PermissionLevel::Write.can_write());
        assert!(PermissionLevel::Admin.can_write());
    }

    #[test]
    fn ordering() {
        assert!(PermissionLevel::None < PermissionLevel::Read);
        assert!(PermissionLevel::Read < PermissionLevel::Write);
        assert!(PermissionLevel::Write < PermissionLevel::Admin);
    }
}
