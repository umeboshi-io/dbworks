use uuid::Uuid;

use crate::domain::permission::GroupTablePermission;
use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

pub async fn grant_group_table_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    group_id: &Uuid,
    table_name: &str,
    permission: &str,
) -> Result<GroupTablePermission, UsecaseError> {
    require_super_admin(caller)?;
    permission_repo
        .grant_group_table_permission(conn_id, group_id, table_name, permission)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
