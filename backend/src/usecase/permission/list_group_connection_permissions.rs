use uuid::Uuid;

use crate::domain::permission::GroupConnectionPermission;
use crate::domain::repository::PermissionRepository;
use crate::usecase::UsecaseError;

pub async fn list_group_connection_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
) -> Result<Vec<GroupConnectionPermission>, UsecaseError> {
    permission_repo
        .list_group_connection_permissions(conn_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
