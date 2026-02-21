use uuid::Uuid;

use crate::domain::repository::GroupRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;

pub async fn list_group_members(
    group_repo: &dyn GroupRepository,
    group_id: &Uuid,
) -> Result<Vec<AppUser>, UsecaseError> {
    group_repo
        .list_members(group_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
