use uuid::Uuid;

use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;

use super::{get_datasource, require_table_write};

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
