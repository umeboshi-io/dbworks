use serde::Deserialize;
use uuid::Uuid;

// ============================================================
// Organization
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
}

// ============================================================
// User
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String {
    "member".to_string()
}

// ============================================================
// Group
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddGroupMemberRequest {
    pub user_id: Uuid,
}

// ============================================================
// Connection
// ============================================================

/// Request body when creating a new connection
#[derive(Debug, Deserialize)]
pub struct ConnectionRequest {
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub database: String,
    pub user: String,
    pub password: String,
}

// ============================================================
// User Permissions
// ============================================================

#[derive(Debug, Deserialize)]
pub struct GrantUserConnectionPermissionRequest {
    pub user_id: Uuid,
    pub permission: String,
    #[serde(default = "default_true")]
    pub all_tables: bool,
}

#[derive(Debug, Deserialize)]
pub struct GrantUserTablePermissionRequest {
    pub table_name: String,
    pub permission: String,
}

// ============================================================
// Group Permissions
// ============================================================

#[derive(Debug, Deserialize)]
pub struct GrantGroupConnectionPermissionRequest {
    pub group_id: Uuid,
    pub permission: String,
    #[serde(default = "default_true")]
    pub all_tables: bool,
}

#[derive(Debug, Deserialize)]
pub struct GrantGroupTablePermissionRequest {
    pub table_name: String,
    pub permission: String,
}

fn default_true() -> bool {
    true
}

// ============================================================
// Rows query
// ============================================================

/// Query parameters for listing rows
#[derive(Debug, Deserialize)]
pub struct RowsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub filter: Option<String>,
}
