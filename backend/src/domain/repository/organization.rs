use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::organization::Organization;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn create(&self, name: &str) -> anyhow::Result<Organization>;
    async fn list(&self) -> anyhow::Result<Vec<Organization>>;
    async fn get(&self, id: &Uuid) -> anyhow::Result<Option<Organization>>;
}
