use uuid::Uuid;

use crate::domain::permission::UserConnectionPermission;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

pub async fn grant_user_connection_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
    permission: &str,
    all_tables: bool,
) -> Result<UserConnectionPermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_user_connection_permission(conn_id, user_id, permission, all_tables)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
