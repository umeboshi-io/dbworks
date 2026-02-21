use uuid::Uuid;

use crate::domain::repository::UserRepository;
use crate::domain::user::AppUser;

use super::UsecaseError;
use super::error::require_super_admin;

pub async fn create_user(
    user_repo: &dyn UserRepository,
    caller: &AppUser,
    org_id: &Uuid,
    name: &str,
    email: &str,
    role: &str,
) -> Result<AppUser, UsecaseError> {
    require_super_admin(caller)?;
    user_repo
        .create(org_id, name, email, role)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn list_users(
    user_repo: &dyn UserRepository,
    org_id: &Uuid,
) -> Result<Vec<AppUser>, UsecaseError> {
    user_repo
        .list_by_org(org_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
