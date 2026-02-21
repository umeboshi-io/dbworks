use uuid::Uuid;

use crate::domain::data::RowsResponse;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::presentation::request::RowsQuery;
use crate::presentation::state::ConnectionManager;
use crate::usecase::UsecaseError;

use super::{get_datasource, require_table_read};

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
