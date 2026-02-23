use uuid::Uuid;

use crate::domain::repository::{OrganizationMemberRepository, UserRepository};
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;
use crate::usecase::error::require_super_admin;

pub async fn create_user(
    user_repo: &dyn UserRepository,
    org_member_repo: &dyn OrganizationMemberRepository,
    caller: &AppUser,
    org_id: &Uuid,
    name: &str,
    email: &str,
    role: &str,
) -> Result<AppUser, UsecaseError> {
    require_super_admin(caller)?;

    // Create the user
    let user = user_repo
        .create(name, email, role)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))?;

    // Add user to organization as member
    org_member_repo
        .add_member(org_id, &user.id, "member")
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))?;

    Ok(user)
}
