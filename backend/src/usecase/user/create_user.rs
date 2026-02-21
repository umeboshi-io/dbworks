use uuid::Uuid;

use crate::domain::repository::UserRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

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
