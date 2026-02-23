use crate::domain::organization::Organization;
use crate::domain::repository::OrganizationRepository;
use crate::domain::user::AppUser;
use crate::usecase::UsecaseError;

pub async fn list_organizations(
    repo: &dyn OrganizationRepository,
    caller: &AppUser,
) -> Result<Vec<Organization>, UsecaseError> {
    repo.list_by_user(&caller.id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
