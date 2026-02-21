use crate::domain::organization::Organization;
use crate::domain::repository::OrganizationRepository;
use crate::usecase::UsecaseError;

pub async fn create_organization(
    repo: &dyn OrganizationRepository,
    name: &str,
) -> Result<Organization, UsecaseError> {
    repo.create(name)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
