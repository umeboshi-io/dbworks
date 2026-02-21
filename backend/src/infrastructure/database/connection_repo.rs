use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::connection::{ConnectionInfo, SavedConnectionRow};
use crate::domain::repository::ConnectionRepository;
use crate::infrastructure::crypto::Encryptor;

pub struct PgConnectionRepository {
    pool: PgPool,
    encryptor: Encryptor,
}

impl PgConnectionRepository {
    pub fn new(pool: PgPool, encryptor: Encryptor) -> Self {
        Self { pool, encryptor }
    }
}

#[async_trait]
impl ConnectionRepository for PgConnectionRepository {
    async fn save(
        &self,
        org_id: Option<&Uuid>,
        owner_user_id: Option<&Uuid>,
        info: &ConnectionInfo,
    ) -> anyhow::Result<SavedConnectionRow> {
        let encrypted_password = self.encryptor.encrypt(&info.password)?;
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
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn list(&self) -> anyhow::Result<Vec<SavedConnectionRow>> {
        let rows = sqlx::query_as::<_, SavedConnectionRow>(
            "SELECT * FROM saved_connections ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn delete(&self, conn_id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM saved_connections WHERE id = $1")
            .bind(conn_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
