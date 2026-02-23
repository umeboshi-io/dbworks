use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use dbworks_backend::infrastructure::auth::oauth::OAuthClients;
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::connection_repo::PgConnectionRepository;
use dbworks_backend::infrastructure::database::group_repo::PgGroupRepository;
use dbworks_backend::infrastructure::database::organization_member_repo::PgOrganizationMemberRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::permission_repo::PgPermissionRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::presentation::routes::create_router;
use dbworks_backend::presentation::state::{AppStateInner, ConnectionManager};

/// Build a fully wired axum Router backed by the test database pool.
pub fn build_test_app(pool: PgPool) -> Router {
    let organization_repo = Arc::new(PgOrganizationRepository::new(pool.clone()));
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let group_repo = Arc::new(PgGroupRepository::new(pool.clone()));
    let permission_repo = Arc::new(PgPermissionRepository::new(pool.clone()));
    let org_member_repo = Arc::new(PgOrganizationMemberRepository::new(pool.clone()));
    let encryptor = {
        unsafe { std::env::set_var("ENCRYPTION_KEY", "dGVzdGtleXRlc3RrZXl0ZXN0a2V5dGVzdGtleTEy") };
        Encryptor::from_env().unwrap()
    };
    let conn_repo = Arc::new(PgConnectionRepository::new(pool.clone(), encryptor));
    let connection_manager = ConnectionManager::new(None, None);

    let oauth_clients = OAuthClients {
        google: None,
        github: None,
    };

    let state = Arc::new(AppStateInner {
        connection_manager,
        pool,
        oauth_clients,
        jwt_secret: "test-secret".to_string(),
        organization_repo,
        user_repo,
        group_repo,
        permission_repo,
        org_member_repo,
        conn_repo,
    });

    create_router().with_state(state)
}

/// Insert a minimal `saved_connections` row so that permission FK constraints
/// are satisfied. Returns the connection UUID.
pub async fn seed_connection(pool: &sqlx::PgPool, org_id: &uuid::Uuid) -> uuid::Uuid {
    let conn_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO saved_connections (id, organization_id, name, host, port, database_name, username, encrypted_password)
           VALUES ($1, $2, 'test-conn', 'localhost', 5432, 'testdb', 'test', 'encrypted')"#,
    )
    .bind(conn_id)
    .bind(org_id)
    .execute(pool)
    .await
    .expect("Failed to seed saved_connection");
    conn_id
}
