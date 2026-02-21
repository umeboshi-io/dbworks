use uuid::Uuid;

use crate::domain::repository::GroupRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

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
