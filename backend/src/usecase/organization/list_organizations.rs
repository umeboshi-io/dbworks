use crate::domain::organization::Organization;
use crate::domain::repository::OrganizationRepository;
use crate::usecase::UsecaseError;

pub async fn list_organizations(
    repo: &dyn OrganizationRepository,
) -> Result<Vec<Organization>, UsecaseError> {
    repo.list()
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
