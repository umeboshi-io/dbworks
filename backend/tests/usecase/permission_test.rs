use crate::common;
use dbworks_backend::domain::connection::ConnectionInfo;
use dbworks_backend::domain::repository::{
    ConnectionRepository, GroupRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::domain::user::AppUser;
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::connection_repo::PgConnectionRepository;
use dbworks_backend::infrastructure::database::group_repo::PgGroupRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::permission_repo::PgPermissionRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use dbworks_backend::usecase::{self, UsecaseError};
use serial_test::serial;
use uuid::Uuid;

#[allow(dead_code)]
struct TestFixture {
    admin: AppUser,
    member: AppUser,
    other: AppUser,
    conn_id: Uuid,
    org_id: Uuid,
    permission_repo: PgPermissionRepository,
    group_repo: PgGroupRepository,
}

async fn setup() -> TestFixture {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let group_repo = PgGroupRepository::new(pool.clone());
    let permission_repo = PgPermissionRepository::new(pool.clone());

    unsafe {
        std::env::set_var(
            "ENCRYPTION_KEY",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[42u8; 32]),
        );
    }
    let enc = Encryptor::from_env().unwrap();
    let conn_repo = PgConnectionRepository::new(pool, enc);

    let org = org_repo.create("Test Org").await.unwrap();

    let admin = user_repo
        .create(&org.id, "Admin", "admin@test.com", "super_admin")
        .await
        .unwrap();
    let member = user_repo
        .create(&org.id, "Member", "member@test.com", "member")
        .await
        .unwrap();
    let other = user_repo
        .create(&org.id, "Other", "other@test.com", "member")
        .await
        .unwrap();

    let info = ConnectionInfo {
        id: Uuid::new_v4(),
        name: "test-conn".to_string(),
        host: "localhost".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        user: "testuser".to_string(),
        password: "pass".to_string(),
        organization_id: Some(org.id),
        owner_user_id: Some(member.id),
    };
    let saved = conn_repo
        .save(Some(&org.id), Some(&member.id), &info)
        .await
        .unwrap();

    TestFixture {
        admin,
        member,
        other,
        conn_id: saved.id,
        org_id: org.id,
        permission_repo,
        group_repo,
    }
}

// ============================================================
// User Connection Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_user_connection_permission_as_admin() {
    let f = setup().await;

    let perm = usecase::permission::grant_user_connection_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &f.other.id,
        "read",
        true,
    )
    .await
    .unwrap();

    assert_eq!(perm.permission, "read");
    assert!(perm.all_tables);
    assert_eq!(perm.user_id, f.other.id);
}

#[tokio::test]
#[serial]
async fn grant_user_connection_permission_as_member_forbidden() {
    let f = setup().await;

    let result = usecase::permission::grant_user_connection_permission(
        &f.permission_repo,
        &f.member,
        &f.conn_id,
        &f.other.id,
        "read",
        true,
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::Forbidden(_)));
}

#[tokio::test]
#[serial]
async fn revoke_user_connection_permission_found() {
    let f = setup().await;

    usecase::permission::grant_user_connection_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &f.other.id,
        "read",
        true,
    )
    .await
    .unwrap();

    usecase::permission::revoke_user_connection_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &f.other.id,
    )
    .await
    .unwrap();

    // Verify it's gone
    let list =
        usecase::permission::list_user_connection_permissions(&f.permission_repo, &f.conn_id)
            .await
            .unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
#[serial]
async fn revoke_user_connection_permission_not_found() {
    let f = setup().await;

    let result = usecase::permission::revoke_user_connection_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &f.other.id,
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}

#[tokio::test]
#[serial]
async fn list_user_connection_permissions_via_usecase() {
    let f = setup().await;

    usecase::permission::grant_user_connection_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &f.other.id,
        "write",
        false,
    )
    .await
    .unwrap();

    let list =
        usecase::permission::list_user_connection_permissions(&f.permission_repo, &f.conn_id)
            .await
            .unwrap();

    assert_eq!(list.len(), 1);
    assert_eq!(list[0].permission, "write");
}

// ============================================================
// User Table Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_user_table_permission_via_usecase() {
    let f = setup().await;

    let perm = usecase::permission::grant_user_table_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &f.other.id,
        "users",
        "read",
    )
    .await
    .unwrap();

    assert_eq!(perm.table_name, "users");
    assert_eq!(perm.permission, "read");
}

#[tokio::test]
#[serial]
async fn revoke_user_table_permission_not_found() {
    let f = setup().await;

    let result = usecase::permission::revoke_user_table_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &f.other.id,
        "nonexistent",
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}

// ============================================================
// Group Connection Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_group_connection_permission_via_usecase() {
    let f = setup().await;

    let group = f.group_repo.create(&f.org_id, "Team", None).await.unwrap();

    let perm = usecase::permission::grant_group_connection_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &group.id,
        "admin",
        true,
    )
    .await
    .unwrap();

    assert_eq!(perm.permission, "admin");
    assert_eq!(perm.group_id, group.id);
}

#[tokio::test]
#[serial]
async fn revoke_group_connection_permission_not_found() {
    let f = setup().await;

    let group = f.group_repo.create(&f.org_id, "Team", None).await.unwrap();

    let result = usecase::permission::revoke_group_connection_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &group.id,
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}

// ============================================================
// Group Table Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_group_table_permission_via_usecase() {
    let f = setup().await;

    let group = f.group_repo.create(&f.org_id, "Team", None).await.unwrap();

    let perm = usecase::permission::grant_group_table_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &group.id,
        "orders",
        "write",
    )
    .await
    .unwrap();

    assert_eq!(perm.table_name, "orders");
    assert_eq!(perm.permission, "write");
}

#[tokio::test]
#[serial]
async fn revoke_group_table_permission_not_found() {
    let f = setup().await;

    let group = f.group_repo.create(&f.org_id, "Team", None).await.unwrap();

    let result = usecase::permission::revoke_group_table_permission(
        &f.permission_repo,
        &f.admin,
        &f.conn_id,
        &group.id,
        "nonexistent",
    )
    .await;

    assert!(matches!(result.unwrap_err(), UsecaseError::NotFound(_)));
}
