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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_connection_info() -> ConnectionInfo {
        ConnectionInfo {
            id: Uuid::nil(),
            name: "test".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            user: "postgres".to_string(),
            password: "secret123".to_string(),
            organization_id: None,
            owner_user_id: None,
        }
    }

    #[test]
    fn password_is_not_serialized() {
        let info = sample_connection_info();
        let json = serde_json::to_value(&info).unwrap();
        assert!(
            json.get("password").is_none(),
            "password should be skipped in serialization"
        );
        assert_eq!(json["name"], "test");
        assert_eq!(json["host"], "localhost");
    }

    #[test]
    fn connection_info_with_org() {
        let org_id = Uuid::new_v4();
        let info = ConnectionInfo {
            organization_id: Some(org_id),
            ..sample_connection_info()
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["organization_id"], org_id.to_string());
    }

    #[test]
    fn connection_info_with_owner() {
        let owner_id = Uuid::new_v4();
        let info = ConnectionInfo {
            owner_user_id: Some(owner_id),
            ..sample_connection_info()
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["owner_user_id"], owner_id.to_string());
    }

    #[test]
    fn saved_connection_response_from_row() {
        let row = SavedConnectionRow {
            id: Uuid::new_v4(),
            organization_id: Some(Uuid::new_v4()),
            name: "prod-db".to_string(),
            host: "db.example.com".to_string(),
            port: 5433,
            database_name: "production".to_string(),
            username: "admin".to_string(),
            encrypted_password: "encrypted_data".to_string(),
            created_by: Some(Uuid::new_v4()),
            owner_user_id: None,
            created_at: None,
            updated_at: None,
        };

        let response = SavedConnectionResponse::from(&row);
        assert_eq!(response.id, row.id);
        assert_eq!(response.name, "prod-db");
        assert_eq!(response.host, "db.example.com");
        assert_eq!(response.port, 5433);
        assert_eq!(response.database_name, "production");
        assert_eq!(response.username, "admin");
        assert_eq!(response.organization_id, row.organization_id);
        assert_eq!(response.created_by, row.created_by);
    }
}
