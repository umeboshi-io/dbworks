use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AppUser {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub auth_provider: Option<String>,
    pub provider_id: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
