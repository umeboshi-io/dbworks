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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user_default_role() {
        let json = r#"{"name": "Alice", "email": "alice@example.com"}"#;
        let req: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.role, "member");
    }

    #[test]
    fn create_user_custom_role() {
        let json = r#"{"name": "Alice", "email": "alice@example.com", "role": "super_admin"}"#;
        let req: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.role, "super_admin");
    }

    #[test]
    fn user_conn_permission_default_all_tables() {
        let json = r#"{"user_id": "00000000-0000-0000-0000-000000000001", "permission": "read"}"#;
        let req: GrantUserConnectionPermissionRequest = serde_json::from_str(json).unwrap();
        assert!(req.all_tables);
    }

    #[test]
    fn group_conn_permission_default_all_tables() {
        let json = r#"{"group_id": "00000000-0000-0000-0000-000000000001", "permission": "write"}"#;
        let req: GrantGroupConnectionPermissionRequest = serde_json::from_str(json).unwrap();
        assert!(req.all_tables);
    }

    #[test]
    fn connection_request_port_optional() {
        let json = r#"{"name": "test", "host": "localhost", "database": "db", "user": "u", "password": "p"}"#;
        let req: ConnectionRequest = serde_json::from_str(json).unwrap();
        assert!(req.port.is_none());
    }

    #[test]
    fn connection_request_with_port() {
        let json = r#"{"name": "test", "host": "localhost", "port": 3306, "database": "db", "user": "u", "password": "p"}"#;
        let req: ConnectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.port, Some(3306));
    }

    #[test]
    fn create_organization_request() {
        let json = r#"{"name": "My Org"}"#;
        let req: CreateOrganizationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "My Org");
    }
}
