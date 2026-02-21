use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::connection::{ConnectionInfo, SavedConnectionRow};

#[async_trait]
pub trait ConnectionRepository: Send + Sync {
    async fn save(
        &self,
        org_id: Option<&Uuid>,
        owner_user_id: Option<&Uuid>,
        info: &ConnectionInfo,
    ) -> anyhow::Result<SavedConnectionRow>;
    async fn list(&self) -> anyhow::Result<Vec<SavedConnectionRow>>;
    async fn delete(&self, conn_id: &Uuid) -> anyhow::Result<bool>;
}
