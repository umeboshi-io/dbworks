use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::organization::Organization;
use crate::domain::repository::OrganizationRepository;

pub struct PgOrganizationRepository {
    pool: PgPool,
}

impl PgOrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for PgOrganizationRepository {
    async fn create(&self, name: &str) -> anyhow::Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            "INSERT INTO organizations (name) VALUES ($1) RETURNING *",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await?;
        Ok(org)
    }

    async fn list(&self) -> anyhow::Result<Vec<Organization>> {
        let orgs =
            sqlx::query_as::<_, Organization>("SELECT * FROM organizations ORDER BY created_at")
                .fetch_all(&self.pool)
                .await?;
        Ok(orgs)
    }

    async fn get(&self, id: &Uuid) -> anyhow::Result<Option<Organization>> {
        let org = sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(org)
    }
}
