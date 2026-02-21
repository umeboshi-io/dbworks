use crate::common;
use dbworks_backend::domain::connection::ConnectionInfo;
use dbworks_backend::domain::repository::{
    ConnectionRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::connection_repo::PgConnectionRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use serial_test::serial;
use uuid::Uuid;

async fn setup_org_and_user(
    pool: &sqlx::PgPool,
) -> (
    dbworks_backend::domain::organization::Organization,
    dbworks_backend::domain::user::AppUser,
) {
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let org = org_repo.create("Test Org").await.unwrap();
    let user = user_repo
        .create(&org.id, "Alice", "alice@test.com", "member")
        .await
        .unwrap();

    (org, user)
}

fn test_encryptor() -> Encryptor {
    // Use a fixed key for testing (base64 of 32 zero bytes)
    unsafe {
        std::env::set_var(
            "ENCRYPTION_KEY",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[42u8; 32]),
        );
    }
    Encryptor::from_env().unwrap()
}

fn make_connection_info(org_id: Option<Uuid>, owner_id: Option<Uuid>) -> ConnectionInfo {
    ConnectionInfo {
        id: Uuid::new_v4(),
        name: "test-conn".to_string(),
        host: "localhost".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        user: "testuser".to_string(),
        password: "super_secret".to_string(),
        organization_id: org_id,
        owner_user_id: owner_id,
    }
}

#[tokio::test]
#[serial]
async fn save_and_list_connections() {
    let pool = common::setup_test_db().await;
    let (org, user) = setup_org_and_user(&pool).await;
    let enc = test_encryptor();
    let conn_repo = PgConnectionRepository::new(pool, enc.clone());

    let info = make_connection_info(Some(org.id), Some(user.id));
    let saved = conn_repo
        .save(Some(&org.id), Some(&user.id), &info)
        .await
        .unwrap();

    assert_eq!(saved.name, "test-conn");
    assert_eq!(saved.host, "localhost");
    assert_eq!(saved.port, 5432);
    assert_eq!(saved.database_name, "testdb");
    assert_eq!(saved.username, "testuser");
    assert_eq!(saved.organization_id, Some(org.id));

    // Password should be encrypted, not plaintext
    assert_ne!(saved.encrypted_password, "super_secret");
    assert!(!saved.encrypted_password.is_empty());

    // Verify decryption works
    let decrypted = enc.decrypt(&saved.encrypted_password).unwrap();
    assert_eq!(decrypted, "super_secret");

    let all = conn_repo.list().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, saved.id);
}

#[tokio::test]
#[serial]
async fn save_connection_with_owner_user_id() {
    let pool = common::setup_test_db().await;
    let (org, user) = setup_org_and_user(&pool).await;
    let enc = test_encryptor();
    let conn_repo = PgConnectionRepository::new(pool, enc);

    let info = make_connection_info(Some(org.id), Some(user.id));
    let saved = conn_repo
        .save(Some(&org.id), Some(&user.id), &info)
        .await
        .unwrap();

    assert_eq!(saved.owner_user_id, Some(user.id));
}

#[tokio::test]
#[serial]
async fn delete_saved_connection() {
    let pool = common::setup_test_db().await;
    let (org, user) = setup_org_and_user(&pool).await;
    let enc = test_encryptor();
    let conn_repo = PgConnectionRepository::new(pool, enc);

    let info = make_connection_info(Some(org.id), Some(user.id));
    let saved = conn_repo
        .save(Some(&org.id), Some(&user.id), &info)
        .await
        .unwrap();

    let deleted = conn_repo.delete(&saved.id).await.unwrap();
    assert!(deleted);

    let all = conn_repo.list().await.unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
#[serial]
async fn delete_nonexistent_connection_returns_false() {
    let pool = common::setup_test_db().await;
    let enc = test_encryptor();
    let conn_repo = PgConnectionRepository::new(pool, enc);

    let deleted = conn_repo.delete(&Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}
