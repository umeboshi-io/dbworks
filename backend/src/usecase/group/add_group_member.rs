use uuid::Uuid;

use crate::domain::repository::{GroupRepository, OrganizationMemberRepository};
use crate::domain::user::AppUser;
use crate::usecase::error::{UsecaseError, require_org_owner};

pub async fn add_group_member(
    group_repo: &dyn GroupRepository,
    org_member_repo: &dyn OrganizationMemberRepository,
    caller: &AppUser,
    group_id: &Uuid,
    user_id: &Uuid,
) -> Result<(), UsecaseError> {
    let org_id = group_repo
        .get_org_id(group_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))?
        .ok_or_else(|| UsecaseError::NotFound("Group not found".to_string()))?;
    require_org_owner(org_member_repo, &caller.id, &org_id).await?;
    group_repo
        .add_member(group_id, user_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
