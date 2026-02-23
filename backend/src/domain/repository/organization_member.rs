use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::organization_member::OrganizationMember;

#[async_trait]
pub trait OrganizationMemberRepository: Send + Sync {
    async fn add_member(
        &self,
        org_id: &Uuid,
        user_id: &Uuid,
        role: &str,
    ) -> anyhow::Result<OrganizationMember>;

    async fn remove_member(&self, org_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool>;

    async fn list_members(&self, org_id: &Uuid) -> anyhow::Result<Vec<OrganizationMember>>;

    async fn get_user_orgs(&self, user_id: &Uuid) -> anyhow::Result<Vec<OrganizationMember>>;
}
