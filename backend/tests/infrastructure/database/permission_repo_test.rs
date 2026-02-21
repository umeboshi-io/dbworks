use crate::common;
use dbworks_backend::domain::connection::ConnectionInfo;
use dbworks_backend::domain::repository::{
    ConnectionRepository, GroupRepository, OrganizationRepository, PermissionRepository,
    UserRepository,
};
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::connection_repo::PgConnectionRepository;
use dbworks_backend::infrastructure::database::group_repo::PgGroupRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::permission_repo::PgPermissionRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use serial_test::serial;
use uuid::Uuid;

// ============================================================
// Helpers
// ============================================================

#[allow(dead_code)]
struct TestFixture {
    org: dbworks_backend::domain::organization::Organization,
    admin: dbworks_backend::domain::user::AppUser,
    member: dbworks_backend::domain::user::AppUser,
    conn_id: Uuid,
    org_repo: PgOrganizationRepository,
    user_repo: PgUserRepository,
    group_repo: PgGroupRepository,
    permission_repo: PgPermissionRepository,
}

async fn setup() -> TestFixture {
    let pool = common::setup_test_db().await;

    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let group_repo = PgGroupRepository::new(pool.clone());
    let permission_repo = PgPermissionRepository::new(pool.clone());

    let org = org_repo.create("Test Org").await.unwrap();

    let admin = user_repo
        .create(&org.id, "Admin", "admin@test.com", "super_admin")
        .await
        .unwrap();

    let member = user_repo
        .create(&org.id, "Member", "member@test.com", "member")
        .await
        .unwrap();

    // Create a saved connection
    unsafe {
        std::env::set_var(
            "ENCRYPTION_KEY",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[42u8; 32]),
        );
    }
    let enc = Encryptor::from_env().unwrap();
    let conn_repo = PgConnectionRepository::new(pool, enc);

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
        org,
        admin,
        member,
        conn_id: saved.id,
        org_repo,
        user_repo,
        group_repo,
        permission_repo,
    }
}

// ============================================================
// User Connection Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_and_list_user_connection_permission() {
    let f = setup().await;

    let perm = f
        .permission_repo
        .grant_user_connection_permission(&f.conn_id, &f.member.id, "read", true)
        .await
        .unwrap();

    assert_eq!(perm.user_id, f.member.id);
    assert_eq!(perm.connection_id, f.conn_id);
    assert_eq!(perm.permission, "read");
    assert!(perm.all_tables);

    let list = f
        .permission_repo
        .list_user_connection_permissions(&f.conn_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
#[serial]
async fn grant_user_connection_permission_upserts() {
    let f = setup().await;

    f.permission_repo
        .grant_user_connection_permission(&f.conn_id, &f.member.id, "read", true)
        .await
        .unwrap();

    // Update to write
    let updated = f
        .permission_repo
        .grant_user_connection_permission(&f.conn_id, &f.member.id, "write", false)
        .await
        .unwrap();

    assert_eq!(updated.permission, "write");
    assert!(!updated.all_tables);

    // Still only 1 record
    let list = f
        .permission_repo
        .list_user_connection_permissions(&f.conn_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
#[serial]
async fn revoke_user_connection_permission() {
    let f = setup().await;

    f.permission_repo
        .grant_user_connection_permission(&f.conn_id, &f.member.id, "read", true)
        .await
        .unwrap();

    let revoked = f
        .permission_repo
        .revoke_user_connection_permission(&f.conn_id, &f.member.id)
        .await
        .unwrap();
    assert!(revoked);

    let list = f
        .permission_repo
        .list_user_connection_permissions(&f.conn_id)
        .await
        .unwrap();
    assert!(list.is_empty());
}

// ============================================================
// User Table Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_and_list_user_table_permission() {
    let f = setup().await;

    let perm = f
        .permission_repo
        .grant_user_table_permission(&f.conn_id, &f.member.id, "users", "write")
        .await
        .unwrap();

    assert_eq!(perm.table_name, "users");
    assert_eq!(perm.permission, "write");

    let list = f
        .permission_repo
        .list_user_table_permissions(&f.conn_id, &f.member.id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
#[serial]
async fn revoke_user_table_permission() {
    let f = setup().await;

    f.permission_repo
        .grant_user_table_permission(&f.conn_id, &f.member.id, "orders", "read")
        .await
        .unwrap();

    let revoked = f
        .permission_repo
        .revoke_user_table_permission(&f.conn_id, &f.member.id, "orders")
        .await
        .unwrap();
    assert!(revoked);
}

// ============================================================
// Group Connection Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_and_list_group_connection_permission() {
    let f = setup().await;

    let group = f
        .group_repo
        .create(&f.org.id, "Engineers", None)
        .await
        .unwrap();

    let perm = f
        .permission_repo
        .grant_group_connection_permission(&f.conn_id, &group.id, "admin", true)
        .await
        .unwrap();

    assert_eq!(perm.group_id, group.id);
    assert_eq!(perm.permission, "admin");

    let list = f
        .permission_repo
        .list_group_connection_permissions(&f.conn_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
#[serial]
async fn revoke_group_connection_permission() {
    let f = setup().await;

    let group = f.group_repo.create(&f.org.id, "Team", None).await.unwrap();

    f.permission_repo
        .grant_group_connection_permission(&f.conn_id, &group.id, "read", true)
        .await
        .unwrap();

    let revoked = f
        .permission_repo
        .revoke_group_connection_permission(&f.conn_id, &group.id)
        .await
        .unwrap();
    assert!(revoked);
}

// ============================================================
// Group Table Permissions
// ============================================================

#[tokio::test]
#[serial]
async fn grant_and_list_group_table_permission() {
    let f = setup().await;

    let group = f.group_repo.create(&f.org.id, "Team", None).await.unwrap();

    let perm = f
        .permission_repo
        .grant_group_table_permission(&f.conn_id, &group.id, "products", "write")
        .await
        .unwrap();

    assert_eq!(perm.table_name, "products");

    let list = f
        .permission_repo
        .list_group_table_permissions(&f.conn_id, &group.id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

// ============================================================
// Permission Resolution
// ============================================================

#[tokio::test]
#[serial]
async fn resolve_connection_permission_super_admin() {
    let f = setup().await;

    let (level, all_tables) = f
        .permission_repo
        .resolve_connection_permission(&f.admin, &f.conn_id)
        .await
        .unwrap();

    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Admin
    );
    assert!(all_tables);
}

#[tokio::test]
#[serial]
async fn resolve_connection_permission_owner() {
    let f = setup().await;

    // member is the owner_user_id of the connection
    let (level, all_tables) = f
        .permission_repo
        .resolve_connection_permission(&f.member, &f.conn_id)
        .await
        .unwrap();

    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Admin
    );
    assert!(all_tables);
}

#[tokio::test]
#[serial]
async fn resolve_connection_permission_user_level() {
    let f = setup().await;

    // Create a second user who is NOT the owner
    let other = f
        .user_repo
        .create(&f.org.id, "Other", "other@test.com", "member")
        .await
        .unwrap();

    // No permission yet
    let (level, _) = f
        .permission_repo
        .resolve_connection_permission(&other, &f.conn_id)
        .await
        .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::None
    );

    // Grant read
    f.permission_repo
        .grant_user_connection_permission(&f.conn_id, &other.id, "read", false)
        .await
        .unwrap();

    let (level, all_tables) = f
        .permission_repo
        .resolve_connection_permission(&other, &f.conn_id)
        .await
        .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Read
    );
    assert!(!all_tables);
}

#[tokio::test]
#[serial]
async fn resolve_connection_permission_group_level() {
    let f = setup().await;

    // Create a user with no direct permission
    let user = f
        .user_repo
        .create(&f.org.id, "GroupUser", "groupuser@test.com", "member")
        .await
        .unwrap();

    // Create group and add user
    let group = f.group_repo.create(&f.org.id, "Team", None).await.unwrap();
    f.group_repo.add_member(&group.id, &user.id).await.unwrap();

    // Grant group permission
    f.permission_repo
        .grant_group_connection_permission(&f.conn_id, &group.id, "write", true)
        .await
        .unwrap();

    let (level, all_tables) = f
        .permission_repo
        .resolve_connection_permission(&user, &f.conn_id)
        .await
        .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Write
    );
    assert!(all_tables);
}

#[tokio::test]
#[serial]
async fn resolve_table_permission_super_admin() {
    let f = setup().await;

    let level = f
        .permission_repo
        .resolve_table_permission(&f.admin, &f.conn_id, "any_table")
        .await
        .unwrap();

    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Admin
    );
}

#[tokio::test]
#[serial]
async fn resolve_table_permission_with_table_override() {
    let f = setup().await;

    // Create user with connection-level write + all_tables
    let user = f
        .user_repo
        .create(&f.org.id, "TableUser", "tableuser@test.com", "member")
        .await
        .unwrap();

    f.permission_repo
        .grant_user_connection_permission(&f.conn_id, &user.id, "write", true)
        .await
        .unwrap();

    // Override specific table to read-only
    f.permission_repo
        .grant_user_table_permission(&f.conn_id, &user.id, "sensitive", "read")
        .await
        .unwrap();

    // General tables → write (from connection level)
    let level = f
        .permission_repo
        .resolve_table_permission(&user, &f.conn_id, "normal_table")
        .await
        .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Write
    );

    // Sensitive table → read (from table override)
    let level = f
        .permission_repo
        .resolve_table_permission(&user, &f.conn_id, "sensitive")
        .await
        .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Read
    );
}
