use uuid::Uuid;

use crate::domain::repository::{
    ConnectionRepository, OrganizationMemberRepository, PermissionRepository,
};
use crate::domain::user::AppUser;
use crate::usecase::error::{UsecaseError, require_conn_owner};

pub async fn revoke_user_connection_permission(
    permission_repo: &dyn PermissionRepository,
    org_member_repo: &dyn OrganizationMemberRepository,
    conn_repo: &dyn ConnectionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_conn_owner(org_member_repo, conn_repo, &caller.id, conn_id).await?;
    let revoked = permission_repo
        .revoke_user_connection_permission(conn_id, user_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))?;
    if revoked {
        Ok(())
    } else {
        Err(UsecaseError::NotFound("Permission not found".to_string()))
    }
}
