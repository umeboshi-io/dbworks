use uuid::Uuid;

use crate::domain::repository::UserRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;

pub async fn list_users(
    user_repo: &dyn UserRepository,
    org_id: &Uuid,
) -> Result<Vec<AppUser>, UsecaseError> {
    user_repo
        .list_by_org(org_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
