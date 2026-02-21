use uuid::Uuid;

use crate::domain::permission::GroupTablePermission;
use crate::domain::repository::PermissionRepository;
use crate::usecase::UsecaseError;

pub async fn list_group_table_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
    group_id: &Uuid,
) -> Result<Vec<GroupTablePermission>, UsecaseError> {
    permission_repo
        .list_group_table_permissions(conn_id, group_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
