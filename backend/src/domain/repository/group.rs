use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::user::AppUser;

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn create(
        &self,
        org_id: &Uuid,
        name: &str,
        description: Option<&str>,
    ) -> anyhow::Result<Group>;
    async fn list_by_org(&self, org_id: &Uuid) -> anyhow::Result<Vec<Group>>;
    async fn add_member(&self, group_id: &Uuid, user_id: &Uuid) -> anyhow::Result<()>;
    async fn remove_member(&self, group_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool>;
    async fn list_members(&self, group_id: &Uuid) -> anyhow::Result<Vec<AppUser>>;
}
