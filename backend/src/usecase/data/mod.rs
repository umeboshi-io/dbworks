mod create_row;
mod delete_row;
mod get_row;
mod get_table_schema;
mod list_rows;
mod list_tables;
mod update_row;

pub use create_row::create_row;
pub use delete_row::delete_row;
pub use get_row::get_row;
pub use get_table_schema::get_table_schema;
pub use list_rows::list_rows;
pub use list_tables::list_tables;
pub use update_row::update_row;

// ============================================================
// Shared helpers used by individual function files
// ============================================================

use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::infrastructure::datasource::DataSource;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;

/// Check that the caller has at least read access to the connection.
pub(super) async fn require_connection_read(
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
pub(super) async fn require_table_read(
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
pub(super) async fn require_table_write(
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
pub(super) async fn get_datasource(
    connection_manager: &ConnectionManager,
    conn_id: &Uuid,
) -> Result<Arc<dyn DataSource>, UsecaseError> {
    connection_manager
        .get_datasource(conn_id)
        .await
        .ok_or_else(|| UsecaseError::NotFound("Connection not found".to_string()))
}
