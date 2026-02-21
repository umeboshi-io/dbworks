use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::repository::GroupRepository;
use crate::domain::user::AppUser;

pub struct PgGroupRepository {
    pool: PgPool,
}

impl PgGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepository for PgGroupRepository {
    async fn create(
        &self,
        org_id: &Uuid,
        name: &str,
        description: Option<&str>,
    ) -> anyhow::Result<Group> {
        let group = sqlx::query_as::<_, Group>(
            "INSERT INTO groups (organization_id, name, description) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(org_id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;
        Ok(group)
    }

    async fn list_by_org(&self, org_id: &Uuid) -> anyhow::Result<Vec<Group>> {
        let groups = sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE organization_id = $1 ORDER BY created_at",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(groups)
    }

    async fn add_member(&self, group_id: &Uuid, user_id: &Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(group_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_member(&self, group_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_members(&self, group_id: &Uuid) -> anyhow::Result<Vec<AppUser>> {
        let users = sqlx::query_as::<_, AppUser>(
            "SELECT u.* FROM app_users u INNER JOIN group_members gm ON u.id = gm.user_id WHERE gm.group_id = $1 ORDER BY u.name",
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
}
