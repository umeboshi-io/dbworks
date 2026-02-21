use crate::common;
use dbworks_backend::domain::connection::ConnectionInfo;
use dbworks_backend::domain::repository::{
    ConnectionRepository, OrganizationRepository, PermissionRepository, UserRepository,
};
use dbworks_backend::domain::user::AppUser;
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::connection_repo::PgConnectionRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::permission_repo::PgPermissionRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::presentation::state::ConnectionManager;
use dbworks_backend::usecase::{self, UsecaseError};
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[allow(dead_code)]
struct TestFixture {
    admin: AppUser,
    /// User who is the connection owner (gets Admin from ownership)
    owner: AppUser,
    /// User with a granted read permission (not owner)
    reader: AppUser,
    /// User with no permissions at all
    no_perm_user: AppUser,
    conn_id: Uuid,
    permission_repo: PgPermissionRepository,
    cm: ConnectionManager,
}

async fn setup() -> TestFixture {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let permission_repo = PgPermissionRepository::new(pool.clone());

    unsafe {
        std::env::set_var(
            "ENCRYPTION_KEY",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[42u8; 32]),
        );
    }
    let enc = Encryptor::from_env().unwrap();
    let conn_repo = PgConnectionRepository::new(pool.clone(), enc.clone());

    let org = org_repo.create("Test Org").await.unwrap();

    let admin = user_repo
        .create(&org.id, "Admin", "admin@test.com", "super_admin")
        .await
        .unwrap();
    let owner = user_repo
        .create(&org.id, "Owner", "owner@test.com", "member")
        .await
        .unwrap();
    let reader = user_repo
        .create(&org.id, "Reader", "reader@test.com", "member")
        .await
        .unwrap();
    let no_perm_user = user_repo
        .create(&org.id, "NoPerm", "noperm@test.com", "member")
        .await
        .unwrap();

    // Save a connection record (owner is the `owner` user)
    let info = ConnectionInfo {
        id: Uuid::new_v4(),
        name: "test-conn".to_string(),
        host: "localhost".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        user: "testuser".to_string(),
        password: "pass".to_string(),
        organization_id: Some(org.id),
        owner_user_id: Some(owner.id),
    };
    let saved = conn_repo
        .save(Some(&org.id), Some(&owner.id), &info)
        .await
        .unwrap();

    // Grant `reader` read permission (NOT owner, so they actually get read-level)
    permission_repo
        .grant_user_connection_permission(&saved.id, &reader.id, "read", true)
        .await
        .unwrap();

    let cm = ConnectionManager::new(Some(Arc::new(conn_repo)), Some(enc));

    TestFixture {
        admin,
        owner,
        reader,
        no_perm_user,
        conn_id: saved.id,
        permission_repo,
        cm,
    }
}

// ============================================================
// Permission Gating Tests
// ============================================================

#[tokio::test]
#[serial]
async fn list_tables_no_permission_forbidden() {
    let f = setup().await;

    let result =
        usecase::data::list_tables(&f.permission_repo, &f.cm, &f.no_perm_user, &f.conn_id).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn list_tables_super_admin_allowed_but_no_datasource() {
    let f = setup().await;

    // super_admin has Admin permission but no live datasource was loaded
    let result = usecase::data::list_tables(&f.permission_repo, &f.cm, &f.admin, &f.conn_id).await;

    // Passes permission check, fails because no live datasource
    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}

#[tokio::test]
#[serial]
async fn list_tables_reader_with_read_no_datasource() {
    let f = setup().await;

    // reader has read permission, but no live datasource
    let result = usecase::data::list_tables(&f.permission_repo, &f.cm, &f.reader, &f.conn_id).await;

    // Passes permission check → fails on get_datasource (NotFound)
    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}

#[tokio::test]
#[serial]
async fn get_table_schema_no_permission() {
    let f = setup().await;

    let result = usecase::data::get_table_schema(
        &f.permission_repo,
        &f.cm,
        &f.no_perm_user,
        &f.conn_id,
        "users",
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn create_row_read_only_forbidden() {
    let f = setup().await;

    // reader has read-only permission — create requires write
    let data = serde_json::json!({"name": "test"});
    let result = usecase::data::create_row(
        &f.permission_repo,
        &f.cm,
        &f.reader,
        &f.conn_id,
        "users",
        &data,
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn update_row_read_only_forbidden() {
    let f = setup().await;

    let data = serde_json::json!({"name": "updated"});
    let result = usecase::data::update_row(
        &f.permission_repo,
        &f.cm,
        &f.reader,
        &f.conn_id,
        "users",
        "1",
        &data,
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn delete_row_read_only_forbidden() {
    let f = setup().await;

    let result = usecase::data::delete_row(
        &f.permission_repo,
        &f.cm,
        &f.reader,
        &f.conn_id,
        "users",
        "1",
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

// ============================================================
// Nonexistent Connection Tests
// ============================================================

#[tokio::test]
#[serial]
async fn list_tables_nonexistent_connection_forbidden() {
    let f = setup().await;

    // super_admin + nonexistent conn_id → admin permission, then NotFound for datasource
    let result =
        usecase::data::list_tables(&f.permission_repo, &f.cm, &f.admin, &Uuid::new_v4()).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}
