use uuid::Uuid;

use crate::domain::permission::UserTablePermission;
use crate::domain::repository::PermissionRepository;
use crate::usecase::UsecaseError;

pub async fn list_user_table_permissions(
    permission_repo: &dyn PermissionRepository,
    conn_id: &Uuid,
    user_id: &Uuid,
) -> Result<Vec<UserTablePermission>, UsecaseError> {
    permission_repo
        .list_user_table_permissions(conn_id, user_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
