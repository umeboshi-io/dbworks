use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use sqlx::PgPool;

use crate::domain::connection::ConnectionInfo;
use crate::infrastructure::auth::oauth::OAuthClients;
use crate::infrastructure::crypto::Encryptor;
use crate::infrastructure::database::connection_repo;
use crate::infrastructure::datasource::DataSource;
use crate::infrastructure::datasource::postgres::PostgresDataSource;

pub struct AppStateInner {
    pub connection_manager: ConnectionManager,
    pub pool: PgPool,
    pub oauth_clients: OAuthClients,
    pub jwt_secret: String,
}

pub type AppState = Arc<AppStateInner>;

// ============================================================
// Connection Manager
// ============================================================

pub struct ConnectionManager {
    connections: RwLock<HashMap<Uuid, ConnectionEntry>>,
    pool: Option<PgPool>,
    encryptor: Option<Encryptor>,
}

struct ConnectionEntry {
    pub info: ConnectionInfo,
    pub datasource: Arc<dyn DataSource>,
}

impl ConnectionManager {
    pub fn new(pool: Option<PgPool>, encryptor: Option<Encryptor>) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            pool,
            encryptor,
        }
    }

    /// Load all saved connections from the database and establish live connections.
    pub async fn load_saved_connections(&self) -> anyhow::Result<()> {
        let pool = match &self.pool {
            Some(p) => p,
            None => {
                tracing::warn!("No app DB pool configured, skipping connection loading");
                return Ok(());
            }
        };
        let encryptor = match &self.encryptor {
            Some(e) => e,
            None => {
                tracing::warn!("No encryptor configured, skipping connection loading");
                return Ok(());
            }
        };

        let saved = connection_repo::list_saved_connections(pool).await?;
        tracing::info!(count = saved.len(), "Loading saved connections from DB");

        for row in &saved {
            let password = match encryptor.decrypt(&row.encrypted_password) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!(
                        connection_id = %row.id,
                        name = %row.name,
                        error = %e,
                        "Failed to decrypt password, skipping"
                    );
                    continue;
                }
            };

            let conn_string = format!(
                "postgres://{}:{}@{}:{}/{}",
                row.username, password, row.host, row.port, row.database_name
            );

            match PostgresDataSource::new(&conn_string).await {
                Ok(ds) => {
                    let info = ConnectionInfo {
                        id: row.id,
                        name: row.name.clone(),
                        host: row.host.clone(),
                        port: row.port as u16,
                        database: row.database_name.clone(),
                        user: row.username.clone(),
                        password,
                        organization_id: row.organization_id,
                        owner_user_id: row.owner_user_id,
                    };
                    let entry = ConnectionEntry {
                        info,
                        datasource: Arc::new(ds),
                    };
                    self.connections.write().await.insert(row.id, entry);
                    tracing::info!(
                        connection_id = %row.id,
                        name = %row.name,
                        "Loaded saved connection"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        connection_id = %row.id,
                        name = %row.name,
                        error = %e,
                        "Failed to connect to saved connection, skipping"
                    );
                }
            }
        }

        Ok(())
    }

    /// Register a new PostgreSQL connection and persist it.
    pub async fn add_postgres(
        &self,
        name: String,
        host: String,
        port: u16,
        database: String,
        user: String,
        password: String,
        organization_id: Option<Uuid>,
        owner_user_id: Option<Uuid>,
    ) -> anyhow::Result<ConnectionInfo> {
        let conn_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        );

        tracing::info!(
            name = %name,
            host = %host,
            port = %port,
            database = %database,
            user = %user,
            "Attempting to connect to PostgreSQL..."
        );

        let datasource = match PostgresDataSource::new(&conn_string).await {
            Ok(ds) => {
                tracing::info!(name = %name, "Successfully connected to PostgreSQL");
                ds
            }
            Err(e) => {
                tracing::error!(
                    name = %name,
                    host = %host,
                    port = %port,
                    database = %database,
                    error = %e,
                    "Failed to connect to PostgreSQL"
                );
                return Err(e);
            }
        };

        let id = Uuid::new_v4();
        let info = ConnectionInfo {
            id,
            name,
            host,
            port,
            database,
            user,
            password: password.clone(),
            organization_id,
            owner_user_id,
        };

        // Persist to DB if configured
        if let (Some(pool), Some(encryptor)) = (&self.pool, &self.encryptor) {
            if let Err(e) = connection_repo::save_connection(
                pool,
                encryptor,
                organization_id.as_ref(),
                owner_user_id.as_ref(),
                &info,
            )
            .await
            {
                tracing::error!(error = %e, "Failed to persist connection to DB");
                return Err(e);
            }
            tracing::info!(connection_id = %id, "Connection persisted to DB");
        }

        let entry = ConnectionEntry {
            info: info.clone(),
            datasource: Arc::new(datasource),
        };

        self.connections.write().await.insert(id, entry);
        tracing::info!(connection_id = %id, "Connection registered");
        Ok(info)
    }

    /// Get a datasource by connection ID
    pub async fn get_datasource(&self, id: &Uuid) -> Option<Arc<dyn DataSource>> {
        let result = self
            .connections
            .read()
            .await
            .get(id)
            .map(|e| e.datasource.clone());

        if result.is_none() {
            tracing::warn!(connection_id = %id, "Connection not found");
        }

        result
    }

    /// List all connection infos
    pub async fn list(&self) -> Vec<ConnectionInfo> {
        let connections: Vec<ConnectionInfo> = self
            .connections
            .read()
            .await
            .values()
            .map(|e| e.info.clone())
            .collect();
        tracing::debug!(count = connections.len(), "Listed connections");
        connections
    }

    /// List connections belonging to a specific organization
    pub async fn list_by_org(&self, org_id: &Uuid) -> Vec<ConnectionInfo> {
        self.connections
            .read()
            .await
            .values()
            .filter(|e| e.info.organization_id.as_ref() == Some(org_id))
            .map(|e| e.info.clone())
            .collect()
    }

    /// List personal connections owned by a specific user
    pub async fn list_personal(&self, user_id: &Uuid) -> Vec<ConnectionInfo> {
        self.connections
            .read()
            .await
            .values()
            .filter(|e| e.info.owner_user_id.as_ref() == Some(user_id))
            .map(|e| e.info.clone())
            .collect()
    }

    /// Remove a connection (also deletes from DB)
    pub async fn remove(&self, id: &Uuid) -> bool {
        let removed = self.connections.write().await.remove(id).is_some();
        if removed {
            // Delete from DB
            if let Some(pool) = &self.pool {
                if let Err(e) = connection_repo::delete_saved_connection(pool, id).await {
                    tracing::error!(connection_id = %id, error = %e, "Failed to delete connection from DB");
                }
            }
            tracing::info!(connection_id = %id, "Connection removed");
        } else {
            tracing::warn!(connection_id = %id, "Attempted to remove non-existent connection");
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::domain::data::{RowsResponse, TableInfo, TableSchema};
    use crate::presentation::request::RowsQuery;
    use async_trait::async_trait;

    /// Minimal mock DataSource for testing (no real DB)
    struct MockDataSource;

    #[async_trait]
    impl DataSource for MockDataSource {
        async fn list_tables(&self) -> anyhow::Result<Vec<TableInfo>> {
            Ok(vec![])
        }
        async fn get_table_schema(&self, _: &str) -> anyhow::Result<TableSchema> {
            anyhow::bail!("mock")
        }
        async fn list_rows(&self, _: &str, _: &RowsQuery) -> anyhow::Result<RowsResponse> {
            anyhow::bail!("mock")
        }
        async fn get_row(&self, _: &str, _: &str) -> anyhow::Result<serde_json::Value> {
            anyhow::bail!("mock")
        }
        async fn insert_row(
            &self,
            _: &str,
            _: &serde_json::Value,
        ) -> anyhow::Result<serde_json::Value> {
            anyhow::bail!("mock")
        }
        async fn update_row(
            &self,
            _: &str,
            _: &str,
            _: &serde_json::Value,
        ) -> anyhow::Result<serde_json::Value> {
            anyhow::bail!("mock")
        }
        async fn delete_row(&self, _: &str, _: &str) -> anyhow::Result<()> {
            anyhow::bail!("mock")
        }
    }

    fn make_entry(org_id: Option<Uuid>, owner_id: Option<Uuid>) -> (Uuid, ConnectionEntry) {
        let id = Uuid::new_v4();
        let info = ConnectionInfo {
            id,
            name: "test".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "db".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
            organization_id: org_id,
            owner_user_id: owner_id,
        };
        let entry = ConnectionEntry {
            info,
            datasource: Arc::new(MockDataSource),
        };
        (id, entry)
    }

    #[tokio::test]
    async fn list_empty() {
        let cm = ConnectionManager::new(None, None);
        assert!(cm.list().await.is_empty());
    }

    #[tokio::test]
    async fn list_by_org_filters_correctly() {
        let cm = ConnectionManager::new(None, None);
        let org_a = Uuid::new_v4();
        let org_b = Uuid::new_v4();

        let (id1, entry1) = make_entry(Some(org_a), None);
        let (id2, entry2) = make_entry(Some(org_b), None);
        let (id3, entry3) = make_entry(Some(org_a), None);

        {
            let mut map = cm.connections.write().await;
            map.insert(id1, entry1);
            map.insert(id2, entry2);
            map.insert(id3, entry3);
        }

        let org_a_conns = cm.list_by_org(&org_a).await;
        assert_eq!(org_a_conns.len(), 2);
        assert!(org_a_conns.iter().all(|c| c.organization_id == Some(org_a)));

        let org_b_conns = cm.list_by_org(&org_b).await;
        assert_eq!(org_b_conns.len(), 1);

        let org_c_conns = cm.list_by_org(&Uuid::new_v4()).await;
        assert!(org_c_conns.is_empty());
    }

    #[tokio::test]
    async fn list_personal_filters_correctly() {
        let cm = ConnectionManager::new(None, None);
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();

        let (id1, entry1) = make_entry(None, Some(user_a));
        let (id2, entry2) = make_entry(None, Some(user_b));

        {
            let mut map = cm.connections.write().await;
            map.insert(id1, entry1);
            map.insert(id2, entry2);
        }

        let personal = cm.list_personal(&user_a).await;
        assert_eq!(personal.len(), 1);
        assert_eq!(personal[0].owner_user_id, Some(user_a));
    }

    #[tokio::test]
    async fn get_datasource_unknown_id_returns_none() {
        let cm = ConnectionManager::new(None, None);
        assert!(cm.get_datasource(&Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn get_datasource_existing_id_returns_some() {
        let cm = ConnectionManager::new(None, None);
        let (id, entry) = make_entry(None, None);
        cm.connections.write().await.insert(id, entry);
        assert!(cm.get_datasource(&id).await.is_some());
    }
}
