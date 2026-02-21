use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::repository::GroupRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

pub async fn create_group(
    group_repo: &dyn GroupRepository,
    caller: &AppUser,
    org_id: &Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Group, UsecaseError> {
    require_super_admin(caller)?;
    group_repo
        .create(org_id, name, description)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
