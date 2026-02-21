use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::repository::GroupRepository;
use crate::domain::user::AppUser;

use super::UsecaseError;
use super::error::require_super_admin;

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

pub async fn list_groups(
    group_repo: &dyn GroupRepository,
    org_id: &Uuid,
) -> Result<Vec<Group>, UsecaseError> {
    group_repo
        .list_by_org(org_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}

pub async fn add_group_member(
    group_repo: &dyn GroupRepository,
    caller: &AppUser,
    group_id: &Uuid,
    user_id: &Uuid,
) -> Result<(), UsecaseError> {
    require_super_admin(caller)?;
    group_repo
        .add_member(group_id, user_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn remove_group_member(
    group_repo: &dyn GroupRepository,
    caller: &AppUser,
    group_id: &Uuid,
    user_id: &Uuid,
) -> Result<bool, UsecaseError> {
    require_super_admin(caller)?;
    group_repo
        .remove_member(group_id, user_id)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn list_group_members(
    group_repo: &dyn GroupRepository,
    group_id: &Uuid,
) -> Result<Vec<AppUser>, UsecaseError> {
    group_repo
        .list_members(group_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
