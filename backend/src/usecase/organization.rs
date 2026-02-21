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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use uuid::Uuid;

    struct MockOrgRepo {
        fail: bool,
    }

    #[async_trait]
    impl OrganizationRepository for MockOrgRepo {
        async fn create(&self, name: &str) -> anyhow::Result<Organization> {
            if self.fail {
                anyhow::bail!("create failed");
            }
            Ok(Organization {
                id: Uuid::new_v4(),
                name: name.to_string(),
                created_at: None,
                updated_at: None,
            })
        }

        async fn list(&self) -> anyhow::Result<Vec<Organization>> {
            if self.fail {
                anyhow::bail!("list failed");
            }
            Ok(vec![Organization {
                id: Uuid::new_v4(),
                name: "Org A".to_string(),
                created_at: None,
                updated_at: None,
            }])
        }

        async fn get(&self, _id: &Uuid) -> anyhow::Result<Option<Organization>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn create_organization_success() {
        let repo = MockOrgRepo { fail: false };
        let result = create_organization(&repo, "Test Org").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Test Org");
    }

    #[tokio::test]
    async fn create_organization_failure() {
        let repo = MockOrgRepo { fail: true };
        let result = create_organization(&repo, "Fail").await;
        assert!(matches!(result.unwrap_err(), UsecaseError::BadRequest(_)));
    }

    #[tokio::test]
    async fn list_organizations_success() {
        let repo = MockOrgRepo { fail: false };
        let result = list_organizations(&repo).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn list_organizations_failure() {
        let repo = MockOrgRepo { fail: true };
        let result = list_organizations(&repo).await;
        assert!(matches!(result.unwrap_err(), UsecaseError::Internal(_)));
    }
}
