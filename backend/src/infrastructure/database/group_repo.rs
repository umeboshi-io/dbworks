use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::group::Group;
use crate::domain::user::AppUser;
use crate::dto::CreateGroupRequest;

pub async fn create_group(
    pool: &PgPool,
    org_id: &Uuid,
    req: &CreateGroupRequest,
) -> anyhow::Result<Group> {
    let group = sqlx::query_as::<_, Group>(
        "INSERT INTO groups (organization_id, name, description) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.description)
    .fetch_one(pool)
    .await?;
    Ok(group)
}

pub async fn list_groups_by_org(pool: &PgPool, org_id: &Uuid) -> anyhow::Result<Vec<Group>> {
    let groups = sqlx::query_as::<_, Group>(
        "SELECT * FROM groups WHERE organization_id = $1 ORDER BY created_at",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await?;
    Ok(groups)
}

pub async fn add_group_member(
    pool: &PgPool,
    group_id: &Uuid,
    user_id: &Uuid,
) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(group_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_group_member(
    pool: &PgPool,
    group_id: &Uuid,
    user_id: &Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
        .bind(group_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_group_members(pool: &PgPool, group_id: &Uuid) -> anyhow::Result<Vec<AppUser>> {
    let users = sqlx::query_as::<_, AppUser>(
        "SELECT u.* FROM app_users u INNER JOIN group_members gm ON u.id = gm.user_id WHERE gm.group_id = $1 ORDER BY u.name",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;
    Ok(users)
}
