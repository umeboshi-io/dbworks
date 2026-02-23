use uuid::Uuid;

use crate::domain::permission::GroupConnectionPermission;
use crate::domain::repository::{
    ConnectionRepository, OrganizationMemberRepository, PermissionRepository,
};
use crate::domain::user::AppUser;
use crate::usecase::error::{UsecaseError, require_conn_owner};

#[allow(clippy::too_many_arguments)]
pub async fn grant_group_connection_permission(
    permission_repo: &dyn PermissionRepository,
    org_member_repo: &dyn OrganizationMemberRepository,
    conn_repo: &dyn ConnectionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    group_id: &Uuid,
    permission: &str,
    all_tables: bool,
) -> Result<GroupConnectionPermission, UsecaseError> {
    require_conn_owner(org_member_repo, conn_repo, &caller.id, conn_id).await?;
    permission_repo
        .grant_group_connection_permission(conn_id, group_id, permission, all_tables)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
