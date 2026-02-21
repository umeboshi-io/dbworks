use uuid::Uuid;

use crate::domain::data::TableInfo;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;

use super::{get_datasource, require_connection_read};

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
