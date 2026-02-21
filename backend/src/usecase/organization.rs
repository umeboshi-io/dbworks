use crate::domain::organization::Organization;
use crate::domain::repository::OrganizationRepository;

use super::UsecaseError;

pub async fn create_organization(
    repo: &dyn OrganizationRepository,
    name: &str,
) -> Result<Organization, UsecaseError> {
    repo.create(name)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}

pub async fn list_organizations(
    repo: &dyn OrganizationRepository,
) -> Result<Vec<Organization>, UsecaseError> {
    repo.list()
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
