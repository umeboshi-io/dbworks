#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stored in DB (encrypted_password is AES-GCM encrypted)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SavedConnectionRow {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub encrypted_password: String,
    pub created_by: Option<Uuid>,
    pub owner_user_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Returned to API (no password)
#[derive(Debug, Clone, Serialize)]
pub struct SavedConnectionResponse {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub created_by: Option<Uuid>,
    pub owner_user_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<&SavedConnectionRow> for SavedConnectionResponse {
    fn from(row: &SavedConnectionRow) -> Self {
        Self {
            id: row.id,
            organization_id: row.organization_id,
            name: row.name.clone(),
            host: row.host.clone(),
            port: row.port,
            database_name: row.database_name.clone(),
            username: row.username.clone(),
            created_by: row.created_by,
            owner_user_id: row.owner_user_id,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub organization_id: Option<Uuid>,
    pub owner_user_id: Option<Uuid>,
}
