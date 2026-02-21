use crate::common;
use dbworks_backend::domain::repository::{OrganizationRepository, UserRepository};
use dbworks_backend::domain::user::AppUser;
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::connection_repo::PgConnectionRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::presentation::state::ConnectionManager;
use dbworks_backend::usecase::{self, UsecaseError};
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[allow(dead_code)]
struct TestFixture {
    admin: AppUser,
    member: AppUser,
    cm: ConnectionManager,
    db_url: String,
}

fn test_encryptor() -> Encryptor {
    unsafe {
        std::env::set_var(
            "ENCRYPTION_KEY",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[42u8; 32]),
        );
    }
    Encryptor::from_env().unwrap()
}

/// Parse the test database URL into (host, port, database, user, password).
fn parse_db_url() -> (String, u16, String, String, String) {
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dbworks:dbworks@localhost:5432/dbworks_test".to_string());
    // Format: postgres://user:password@host:port/database
    let without_scheme = url.strip_prefix("postgres://").unwrap_or(&url);
    let (creds, rest) = without_scheme.split_once('@').expect("Missing @ in DB URL");
    let (user, password) = creds.split_once(':').expect("Missing : in credentials");
    let (host_port, database) = rest.split_once('/').expect("Missing / in DB URL");
    let (host, port_str) = host_port.split_once(':').unwrap_or((host_port, "5432"));
    let port: u16 = port_str.parse().unwrap_or(5432);
    (
        host.to_string(),
        port,
        database.to_string(),
        user.to_string(),
        password.to_string(),
    )
}

async fn setup() -> TestFixture {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let enc = test_encryptor();
    let conn_repo = PgConnectionRepository::new(pool.clone(), enc.clone());

    let org = org_repo.create("Test Org").await.unwrap();

    let admin = user_repo
        .create(&org.id, "Admin", "admin@test.com", "super_admin")
        .await
        .unwrap();

    let member = user_repo
        .create(&org.id, "Member", "member@test.com", "member")
        .await
        .unwrap();

    let cm = ConnectionManager::new(Some(Arc::new(conn_repo)), Some(enc));

    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dbworks:dbworks@localhost:5432/dbworks_test".to_string());

    TestFixture {
        admin,
        member,
        cm,
        db_url,
    }
}

#[tokio::test]
#[serial]
async fn create_connection_org_admin() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    let conn = usecase::connection::create_connection(
        &f.cm,
        &f.admin,
        "test-conn".into(),
        host,
        port,
        database,
        user,
        password,
    )
    .await
    .unwrap();

    assert_eq!(conn.name, "test-conn");
    // Org connection owned by org
    assert!(conn.organization_id.is_some());
    assert!(conn.owner_user_id.is_none());
}

#[tokio::test]
#[serial]
async fn create_connection_org_member_forbidden() {
    let f = setup().await;

    let result = usecase::connection::create_connection(
        &f.cm,
        &f.member,
        "test-conn".into(),
        "localhost".into(),
        5432,
        "testdb".into(),
        "user".into(),
        "pass".into(),
    )
    .await;

    // Forbidden because member role + org connection
    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn list_connections_all() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    usecase::connection::create_connection(
        &f.cm,
        &f.admin,
        "test-conn".into(),
        host,
        port,
        database,
        user,
        password,
    )
    .await
    .unwrap();

    let connections = usecase::connection::list_connections(&f.cm, &f.admin, None)
        .await
        .unwrap();

    assert_eq!(connections.len(), 1);
}

#[tokio::test]
#[serial]
async fn list_connections_by_org() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    let conn = usecase::connection::create_connection(
        &f.cm,
        &f.admin,
        "org-conn".into(),
        host,
        port,
        database,
        user,
        password,
    )
    .await
    .unwrap();

    let org_id = conn.organization_id.unwrap();
    let scope = format!("org:{}", org_id);
    let connections = usecase::connection::list_connections(&f.cm, &f.admin, Some(&scope))
        .await
        .unwrap();

    assert_eq!(connections.len(), 1);
}

#[tokio::test]
#[serial]
async fn list_connections_invalid_org_scope() {
    let f = setup().await;

    let result = usecase::connection::list_connections(&f.cm, &f.admin, Some("org:invalid")).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::BadRequest(_)));
}

#[tokio::test]
#[serial]
async fn delete_connection_as_super_admin() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    let conn = usecase::connection::create_connection(
        &f.cm,
        &f.admin,
        "to-delete".into(),
        host,
        port,
        database,
        user,
        password,
    )
    .await
    .unwrap();

    usecase::connection::delete_connection(&f.cm, &f.admin, &conn.id)
        .await
        .unwrap();

    let connections = usecase::connection::list_connections(&f.cm, &f.admin, None)
        .await
        .unwrap();

    assert!(connections.is_empty());
}

#[tokio::test]
#[serial]
async fn delete_connection_as_member_forbidden() {
    let f = setup().await;

    let result = usecase::connection::delete_connection(&f.cm, &f.member, &Uuid::new_v4()).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn delete_connection_not_found() {
    let f = setup().await;

    let result = usecase::connection::delete_connection(&f.cm, &f.admin, &Uuid::new_v4()).await;

    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}
