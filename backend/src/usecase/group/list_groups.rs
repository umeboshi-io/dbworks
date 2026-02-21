use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::repository::GroupRepository;
use crate::usecase::UsecaseError;

pub async fn list_groups(
    group_repo: &dyn GroupRepository,
    org_id: &Uuid,
) -> Result<Vec<Group>, UsecaseError> {
    group_repo
        .list_by_org(org_id)
        .await
        .map_err(|e| UsecaseError::Internal(e.to_string()))
}
