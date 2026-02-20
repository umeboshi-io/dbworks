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
