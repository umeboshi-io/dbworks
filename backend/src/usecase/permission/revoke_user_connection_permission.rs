use uuid::Uuid;

use crate::domain::repository::PermissionRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

pub async fn revoke_user_connection_permission(
    permission_repo: &dyn PermissionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
    user_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
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
