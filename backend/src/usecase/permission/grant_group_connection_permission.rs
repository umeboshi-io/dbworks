use uuid::Uuid;

use crate::domain::permission::GroupConnectionPermission;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

pub async fn grant_group_connection_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    group_id: &Uuid,
    permission: &str,
    all_tables: bool,
) -> Result<GroupConnectionPermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_group_connection_permission(conn_id, group_id, permission, all_tables)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
