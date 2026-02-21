use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::user::AppUser;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        org_id: &Uuid,
        name: &str,
        email: &str,
        role: &str,
    ) -> anyhow::Result<AppUser>;
    async fn list_by_org(&self, org_id: &Uuid) -> anyhow::Result<Vec<AppUser>>;
    async fn get(&self, user_id: &Uuid) -> anyhow::Result<Option<AppUser>>;
}
