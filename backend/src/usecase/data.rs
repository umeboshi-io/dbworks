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
