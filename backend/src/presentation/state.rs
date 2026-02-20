use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use sqlx::PgPool;

use crate::domain::connection::ConnectionInfo;
use crate::infrastructure::auth::oauth::OAuthClients;
use crate::infrastructure::crypto::Encryptor;
use crate::infrastructure::datasource::DataSource;
use crate::infrastructure::datasource::postgres::PostgresDataSource;
use crate::infrastructure::database::connection_repo;

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
                        organization_id: Some(row.organization_id),
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
        };

        // Persist to DB if configured
        if let (Some(pool), Some(encryptor)) = (&self.pool, &self.encryptor) {
            let org_id = organization_id.unwrap_or(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
            if let Err(e) = connection_repo::save_connection(pool, encryptor, &org_id, &info).await {
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
