use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repository::UserRepository;
use crate::domain::user::AppUser;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, name: &str, email: &str, role: &str) -> anyhow::Result<AppUser> {
        let user = sqlx::query_as::<_, AppUser>(
            "INSERT INTO app_users (name, email, role) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(name)
        .bind(email)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn list_by_org(&self, org_id: &Uuid) -> anyhow::Result<Vec<AppUser>> {
        let users = sqlx::query_as::<_, AppUser>(
            r#"SELECT u.* FROM app_users u
               INNER JOIN organization_members om ON om.user_id = u.id
               WHERE om.organization_id = $1
               ORDER BY u.created_at"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    async fn get(&self, user_id: &Uuid) -> anyhow::Result<Option<AppUser>> {
        let user = sqlx::query_as::<_, AppUser>("SELECT * FROM app_users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }
}
