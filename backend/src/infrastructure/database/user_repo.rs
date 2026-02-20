use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::user::AppUser;
use crate::dto::CreateUserRequest;

pub async fn create_user(
    pool: &PgPool,
    org_id: &Uuid,
    req: &CreateUserRequest,
) -> anyhow::Result<AppUser> {
    let user = sqlx::query_as::<_, AppUser>(
        "INSERT INTO app_users (organization_id, name, email, role) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.email)
    .bind(&req.role)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn list_users_by_org(pool: &PgPool, org_id: &Uuid) -> anyhow::Result<Vec<AppUser>> {
    let users = sqlx::query_as::<_, AppUser>(
        "SELECT * FROM app_users WHERE organization_id = $1 ORDER BY created_at",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await?;
    Ok(users)
}

pub async fn get_user(pool: &PgPool, user_id: &Uuid) -> anyhow::Result<Option<AppUser>> {
    let user = sqlx::query_as::<_, AppUser>("SELECT * FROM app_users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}
