use uuid::Uuid;

use crate::domain::permission::UserConnectionPermission;
use crate::domain::repository::PermissionRepository;
use crate::usecase::UsecaseError;

pub async fn list_user_connection_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
) -> Result<Vec<UserConnectionPermission>, UsecaseError> {
    permission_repo
        .list_user_connection_permissions(conn_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
