use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::connection::{ConnectionInfo, SavedConnectionRow};
use crate::infrastructure::crypto::Encryptor;

pub async fn save_connection(
    pool: &PgPool,
    encryptor: &Encryptor,
    org_id: Option<&Uuid>,
    owner_user_id: Option<&Uuid>,
    info: &ConnectionInfo,
) -> anyhow::Result<SavedConnectionRow> {
    let encrypted_password = encryptor.encrypt(&info.password)?;
    let row = sqlx::query_as::<_, SavedConnectionRow>(
        r#"INSERT INTO saved_connections (id, organization_id, name, host, port, database_name, username, encrypted_password, created_by, owner_user_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           RETURNING *"#,
    )
    .bind(info.id)
    .bind(org_id)
    .bind(&info.name)
    .bind(&info.host)
    .bind(info.port as i32)
    .bind(&info.database)
    .bind(&info.user)
    .bind(&encrypted_password)
    .bind::<Option<Uuid>>(None) // created_by
    .bind(owner_user_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_saved_connections(pool: &PgPool) -> anyhow::Result<Vec<SavedConnectionRow>> {
    let rows = sqlx::query_as::<_, SavedConnectionRow>(
        "SELECT * FROM saved_connections ORDER BY created_at",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn delete_saved_connection(pool: &PgPool, conn_id: &Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM saved_connections WHERE id = $1")
        .bind(conn_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
