use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::organization::Organization;
use crate::dto::CreateOrganizationRequest;

pub async fn create_organization(
    pool: &PgPool,
    req: &CreateOrganizationRequest,
) -> anyhow::Result<Organization> {
    let org = sqlx::query_as::<_, Organization>(
        "INSERT INTO organizations (name) VALUES ($1) RETURNING *",
    )
    .bind(&req.name)
    .fetch_one(pool)
    .await?;
    Ok(org)
}

pub async fn list_organizations(pool: &PgPool) -> anyhow::Result<Vec<Organization>> {
    let orgs = sqlx::query_as::<_, Organization>("SELECT * FROM organizations ORDER BY created_at")
        .fetch_all(pool)
        .await?;
    Ok(orgs)
}

#[allow(dead_code)]
pub async fn get_organization(pool: &PgPool, id: &Uuid) -> anyhow::Result<Option<Organization>> {
    let org =
        sqlx::query_as::<_, Organization>("SELECT * FROM organizations WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(org)
}
