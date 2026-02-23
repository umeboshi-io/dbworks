use crate::common;
use dbworks_backend::domain::repository::{
    ConnectionRepository, OrganizationMemberRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::domain::user::AppUser;
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::connection_repo::PgConnectionRepository;
use dbworks_backend::infrastructure::database::organization_member_repo::PgOrganizationMemberRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::presentation::state::ConnectionManager;
use dbworks_backend::usecase::{self, UsecaseError};
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

struct TestFixture {
    admin: AppUser,
    member: AppUser,
    cm: ConnectionManager,
    org_id: Uuid,
    org_member_repo: Arc<PgOrganizationMemberRepository>,
    conn_repo: Arc<PgConnectionRepository>,
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
    let org_member_repo = Arc::new(PgOrganizationMemberRepository::new(pool.clone()));
    let enc = test_encryptor();
    let conn_repo = Arc::new(PgConnectionRepository::new(pool.clone(), enc.clone()));

    let org = org_repo.create("Test Org").await.unwrap();

    let admin = user_repo
        .create("Admin", "admin@test.com", "member")
        .await
        .unwrap();
    // Make admin an org owner
    org_member_repo
        .add_member(&org.id, &admin.id, "owner")
        .await
        .unwrap();

    let member = user_repo
        .create("Member", "member@test.com", "member")
        .await
        .unwrap();
    // Make member an org member (not owner)
    org_member_repo
        .add_member(&org.id, &member.id, "member")
        .await
        .unwrap();

    let cm = ConnectionManager::new(
        Some(conn_repo.clone() as Arc<dyn ConnectionRepository>),
        Some(enc),
    );

    TestFixture {
        admin,
        member,
        cm,
        org_id: org.id,
        org_member_repo,
        conn_repo,
    }
}

#[tokio::test]
#[serial]
async fn create_connection_org_owner() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    let conn = usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.admin,
        "test-conn".into(),
        "postgres".into(),
        host,
        port,
        database,
        user,
        password,
        Some(f.org_id),
    )
    .await
    .unwrap();

    assert_eq!(conn.name, "test-conn");
    assert_eq!(conn.organization_id, Some(f.org_id));
    assert!(conn.owner_user_id.is_none());
}

#[tokio::test]
#[serial]
async fn create_connection_org_member_forbidden() {
    let f = setup().await;

    let result = usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.member,
        "test-conn".into(),
        "postgres".into(),
        "localhost".into(),
        5432,
        "testdb".into(),
        "user".into(),
        "pass".into(),
        Some(f.org_id),
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn create_connection_personal() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    // Anyone can create a personal connection (no org scope)
    let conn = usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.member,
        "personal-conn".into(),
        "postgres".into(),
        host,
        port,
        database,
        user,
        password,
        None,
    )
    .await
    .unwrap();

    assert!(conn.organization_id.is_none());
    assert_eq!(conn.owner_user_id, Some(f.member.id));
}

#[tokio::test]
#[serial]
async fn list_connections_all() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.admin,
        "test-conn".into(),
        "postgres".into(),
        host,
        port,
        database,
        user,
        password,
        Some(f.org_id),
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
async fn delete_connection_as_org_owner() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    let conn = usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.admin,
        "to-delete".into(),
        "postgres".into(),
        host,
        port,
        database,
        user,
        password,
        Some(f.org_id),
    )
    .await
    .unwrap();

    usecase::connection::delete_connection(
        &f.cm,
        &*f.org_member_repo,
        &*f.conn_repo,
        &f.admin,
        &conn.id,
    )
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
    let (host, port, database, user, password) = parse_db_url();

    let conn = usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.admin,
        "to-delete".into(),
        "postgres".into(),
        host,
        port,
        database,
        user,
        password,
        Some(f.org_id),
    )
    .await
    .unwrap();

    let result = usecase::connection::delete_connection(
        &f.cm,
        &*f.org_member_repo,
        &*f.conn_repo,
        &f.member,
        &conn.id,
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn delete_connection_not_found() {
    let f = setup().await;

    let result = usecase::connection::delete_connection(
        &f.cm,
        &*f.org_member_repo,
        &*f.conn_repo,
        &f.admin,
        &Uuid::new_v4(),
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}

#[tokio::test]
#[serial]
async fn create_connection_unsupported_db_type_error() {
    let f = setup().await;

    let result = usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.admin,
        "test-conn".into(),
        "sqlite".into(),
        "localhost".into(),
        5432,
        "testdb".into(),
        "user".into(),
        "pass".into(),
        None,
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::BadRequest(_)));
}

#[tokio::test]
#[serial]
async fn create_connection_postgres_has_db_type_field() {
    let f = setup().await;
    let (host, port, database, user, password) = parse_db_url();

    let conn = usecase::connection::create_connection(
        &f.cm,
        &*f.org_member_repo,
        &f.admin,
        "pg-conn".into(),
        "postgres".into(),
        host,
        port,
        database,
        user,
        password,
        None,
    )
    .await
    .unwrap();

    assert_eq!(conn.db_type, "postgres");
}
