use uuid::Uuid;

use crate::domain::permission::UserTablePermission;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

pub async fn grant_user_table_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
    table_name: &str,
    permission: &str,
) -> Result<UserTablePermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_user_table_permission(conn_id, user_id, table_name, permission)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
