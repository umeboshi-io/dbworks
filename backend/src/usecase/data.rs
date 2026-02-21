use std::sync::Arc;

use uuid::Uuid;

use crate::domain::data::{RowsResponse, TableInfo, TableSchema};
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::infrastructure::datasource::DataSource;
use crate::presentation::request::RowsQuery;
use crate::presentation::state::ConnectionManager;

use super::UsecaseError;

// ============================================================
// Helpers
// ============================================================

/// Check that the caller has at least read access to the connection.
async fn require_connection_read(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
) -> Result<(), UsecaseError> {
    let (perm, _) = permission_repo
        .resolve_connection_permission(caller, conn_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))?;
    if !perm.can_read() {
        return Err(UsecaseError::Forbidden(
            "No access to this connection".to_string(),
        ));
    }
    Ok(())
}

/// Check that the caller has at least read access to a specific table.
async fn require_table_read(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
) -> Result<(), UsecaseError> {
    let perm = permission_repo
        .resolve_table_permission(caller, conn_id, table)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))?;
    if !perm.can_read() {
        return Err(UsecaseError::Forbidden(
            "No access to this table".to_string(),
        ));
    }
    Ok(())
}

/// Check that the caller has write access to a specific table.
async fn require_table_write(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
) -> Result<(), UsecaseError> {
    let perm = permission_repo
        .resolve_table_permission(caller, conn_id, table)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))?;
    if !perm.can_write() {
        return Err(UsecaseError::Forbidden("Write access required".to_string()));
    }
    Ok(())
}

/// Get a datasource by connection ID, returning NotFound if absent.
async fn get_datasource(
    connection_manager: &ConnectionManager,
    conn_id: &Uuid,
) -> Result<Arc<dyn DataSource>, UsecaseError> {
    connection_manager
        .get_datasource(conn_id)
        .await
        .ok_or_else(|| UsecaseError::NotFound("Connection not found".to_string()))
}

// ============================================================
// Table Introspection
// ============================================================

pub async fn list_tables(
    permission_repo: &dyn PermissionRepository,
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
) -> Result<Vec<TableInfo>, UsecaseError> {
    require_connection_read(permission_repo, caller, conn_id).await?;
    let ds = get_datasource(connection_manager, conn_id).await?;
    ds.list_tables()
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

pub async fn get_table_schema(
    permission_repo: &dyn PermissionRepository,
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
) -> Result<TableSchema, UsecaseError> {
    require_table_read(permission_repo, caller, conn_id, table).await?;
    let ds = get_datasource(connection_manager, conn_id).await?;
    ds.get_table_schema(table)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

// ============================================================
// Row CRUD
// ============================================================

pub async fn list_rows(
    permission_repo: &dyn PermissionRepository,
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
    query: &RowsQuery,
) -> Result<RowsResponse, UsecaseError> {
    require_table_read(permission_repo, caller, conn_id, table).await?;
    let ds = get_datasource(connection_manager, conn_id).await?;
    ds.list_rows(table, query)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

pub async fn create_row(
    permission_repo: &dyn PermissionRepository,
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
    data: &serde_json::Value,
) -> Result<serde_json::Value, UsecaseError> {
    require_table_write(permission_repo, caller, conn_id, table).await?;
    let ds = get_datasource(connection_manager, conn_id).await?;
    ds.insert_row(table, data)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn get_row(
    permission_repo: &dyn PermissionRepository,
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
    pk: &str,
) -> Result<serde_json::Value, UsecaseError> {
    require_table_read(permission_repo, caller, conn_id, table).await?;
    let ds = get_datasource(connection_manager, conn_id).await?;
    ds.get_row(table, pk)
        .await
        .map_err(|e| UsecaseError::NotFound(e.to_string()))
}

pub async fn update_row(
    permission_repo: &dyn PermissionRepository,
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
    pk: &str,
    data: &serde_json::Value,
) -> Result<serde_json::Value, UsecaseError> {
    require_table_write(permission_repo, caller, conn_id, table).await?;
    let ds = get_datasource(connection_manager, conn_id).await?;
    ds.update_row(table, pk, data)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn delete_row(
    permission_repo: &dyn PermissionRepository,
    connection_manager: &ConnectionManager,
    caller: &AppUser,
    conn_id: &Uuid,
    table: &str,
    pk: &str,
) -> Result<(), UsecaseError> {
    require_table_write(permission_repo, caller, conn_id, table).await?;
    let ds = get_datasource(connection_manager, conn_id).await?;
    ds.delete_row(table, pk)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::permission::*;
    use async_trait::async_trait;

    fn make_user(role: &str) -> AppUser {
        AppUser {
            id: Uuid::new_v4(),
            organization_id: None,
            name: "Caller".to_string(),
            email: "caller@test.com".to_string(),
            role: role.to_string(),
            auth_provider: None,
            provider_id: None,
            avatar_url: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Mock that returns configurable permission levels.
    struct MockPermissionRepo {
        conn_level: PermissionLevel,
        table_level: PermissionLevel,
    }

    #[async_trait]
    impl PermissionRepository for MockPermissionRepo {
        async fn grant_user_connection_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
            _: &str,
            _: bool,
        ) -> anyhow::Result<UserConnectionPermission> {
            unimplemented!()
        }
        async fn revoke_user_connection_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
        ) -> anyhow::Result<bool> {
            unimplemented!()
        }
        async fn list_user_connection_permissions(
            &self,
            _: &Uuid,
        ) -> anyhow::Result<Vec<UserConnectionPermission>> {
            unimplemented!()
        }
        async fn grant_user_table_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
            _: &str,
            _: &str,
        ) -> anyhow::Result<UserTablePermission> {
            unimplemented!()
        }
        async fn revoke_user_table_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
            _: &str,
        ) -> anyhow::Result<bool> {
            unimplemented!()
        }
        async fn list_user_table_permissions(
            &self,
            _: &Uuid,
            _: &Uuid,
        ) -> anyhow::Result<Vec<UserTablePermission>> {
            unimplemented!()
        }
        async fn grant_group_connection_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
            _: &str,
            _: bool,
        ) -> anyhow::Result<GroupConnectionPermission> {
            unimplemented!()
        }
        async fn revoke_group_connection_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
        ) -> anyhow::Result<bool> {
            unimplemented!()
        }
        async fn list_group_connection_permissions(
            &self,
            _: &Uuid,
        ) -> anyhow::Result<Vec<GroupConnectionPermission>> {
            unimplemented!()
        }
        async fn grant_group_table_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
            _: &str,
            _: &str,
        ) -> anyhow::Result<GroupTablePermission> {
            unimplemented!()
        }
        async fn revoke_group_table_permission(
            &self,
            _: &Uuid,
            _: &Uuid,
            _: &str,
        ) -> anyhow::Result<bool> {
            unimplemented!()
        }
        async fn list_group_table_permissions(
            &self,
            _: &Uuid,
            _: &Uuid,
        ) -> anyhow::Result<Vec<GroupTablePermission>> {
            unimplemented!()
        }

        async fn resolve_connection_permission(
            &self,
            _user: &AppUser,
            _conn_id: &Uuid,
        ) -> anyhow::Result<(PermissionLevel, bool)> {
            Ok((self.conn_level.clone(), false))
        }

        async fn resolve_table_permission(
            &self,
            _user: &AppUser,
            _conn_id: &Uuid,
            _table_name: &str,
        ) -> anyhow::Result<PermissionLevel> {
            Ok(self.table_level.clone())
        }
    }

    // ============================================================
    // Permission Helper Tests
    // ============================================================

    #[tokio::test]
    async fn require_connection_read_with_read() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::Read,
            table_level: PermissionLevel::None,
        };
        let caller = make_user("member");
        let result = require_connection_read(&repo, &caller, &Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn require_connection_read_with_none() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::None,
        };
        let caller = make_user("member");
        let result = require_connection_read(&repo, &caller, &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn require_table_read_with_read() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::Read,
        };
        let caller = make_user("member");
        let result = require_table_read(&repo, &caller, &Uuid::new_v4(), "users").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn require_table_read_with_none() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::None,
        };
        let caller = make_user("member");
        let result = require_table_read(&repo, &caller, &Uuid::new_v4(), "users").await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn require_table_write_with_write() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::Write,
        };
        let caller = make_user("member");
        let result = require_table_write(&repo, &caller, &Uuid::new_v4(), "users").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn require_table_write_with_read_only() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::Read,
        };
        let caller = make_user("member");
        let result = require_table_write(&repo, &caller, &Uuid::new_v4(), "users").await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    // ============================================================
    // Datasource Not Found Tests
    // ============================================================

    #[tokio::test]
    async fn list_tables_connection_not_found() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::Read,
            table_level: PermissionLevel::None,
        };
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member");
        let result = list_tables(&repo, &cm, &caller, &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
    }

    #[tokio::test]
    async fn list_tables_no_permission() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::None,
        };
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member");
        let result = list_tables(&repo, &cm, &caller, &Uuid::new_v4()).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn create_row_write_required() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::Read, // read only
        };
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member");
        let data = serde_json::json!({"name": "test"});
        let result = create_row(&repo, &cm, &caller, &Uuid::new_v4(), "users", &data).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn delete_row_write_required() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::Read,
        };
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member");
        let result = delete_row(&repo, &cm, &caller, &Uuid::new_v4(), "users", "1").await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }

    #[tokio::test]
    async fn get_table_schema_no_permission() {
        let repo = MockPermissionRepo {
            conn_level: PermissionLevel::None,
            table_level: PermissionLevel::None,
        };
        let cm = ConnectionManager::new(None, None);
        let caller = make_user("member");
        let result = get_table_schema(&repo, &cm, &caller, &Uuid::new_v4(), "users").await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
    }
}
