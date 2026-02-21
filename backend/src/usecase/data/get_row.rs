use uuid::Uuid;

use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;

use super::{get_datasource, require_table_read};

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
