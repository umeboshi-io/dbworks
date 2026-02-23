use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::repository::{GroupRepository, OrganizationMemberRepository};
use crate::domain::user::AppUser;
use crate::usecase::error::{UsecaseError, require_org_owner};

pub async fn create_group(
    group_repo: &dyn GroupRepository,
    org_member_repo: &dyn OrganizationMemberRepository,
    caller: &AppUser,
    org_id: &Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Group, UsecaseError> {
    require_org_owner(org_member_repo, &caller.id, org_id).await?;
    group_repo
        .create(org_id, name, description)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
