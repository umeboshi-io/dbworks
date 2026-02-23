use uuid::Uuid;

use crate::domain::repository::{ConnectionRepository, OrganizationMemberRepository};
use crate::domain::user::AppUser;
use crate::presentation::state::ConnectionManager;
use crate::usecase::error::{UsecaseError, require_conn_owner};

pub async fn delete_connection(
    connection_manager: &ConnectionManager,
    org_member_repo: &dyn OrganizationMemberRepository,
    conn_repo: &dyn ConnectionRepository,
    caller: &AppUser,
    conn_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_conn_owner(org_member_repo, conn_repo, &caller.id, conn_id).await?;
    if connection_manager.remove(conn_id).await {
        Ok(())
    } else {
        Err(UsecaseError::NotFound("Connection not found".to_string()))
    }
}
