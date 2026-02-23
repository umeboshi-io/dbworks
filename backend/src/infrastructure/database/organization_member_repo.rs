use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::organization_member::OrganizationMember;
use crate::domain::repository::OrganizationMemberRepository;

pub struct PgOrganizationMemberRepository {
    pool: PgPool,
}

impl PgOrganizationMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationMemberRepository for PgOrganizationMemberRepository {
    async fn add_member(
        &self,
        org_id: &Uuid,
        user_id: &Uuid,
        role: &str,
    ) -> anyhow::Result<OrganizationMember> {
        let member = sqlx::query_as::<_, OrganizationMember>(
            r#"INSERT INTO organization_members (organization_id, user_id, role)
               VALUES ($1, $2, $3)
               ON CONFLICT (organization_id, user_id) DO UPDATE SET role = $3
               RETURNING *"#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;
        Ok(member)
    }

    async fn remove_member(&self, org_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM organization_members WHERE organization_id = $1 AND user_id = $2",
        )
        .bind(org_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_members(&self, org_id: &Uuid) -> anyhow::Result<Vec<OrganizationMember>> {
        let members = sqlx::query_as::<_, OrganizationMember>(
            "SELECT * FROM organization_members WHERE organization_id = $1 ORDER BY joined_at",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(members)
    }

    async fn get_user_orgs(&self, user_id: &Uuid) -> anyhow::Result<Vec<OrganizationMember>> {
        let memberships = sqlx::query_as::<_, OrganizationMember>(
            "SELECT * FROM organization_members WHERE user_id = $1 ORDER BY joined_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(memberships)
    }
}
