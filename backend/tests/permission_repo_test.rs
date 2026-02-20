mod common;

use dbworks_backend::domain::connection::ConnectionInfo;
use dbworks_backend::infrastructure::crypto::Encryptor;
use dbworks_backend::infrastructure::database::{
    connection_repo, group_repo, organization_repo, permission_repo, user_repo,
};
use dbworks_backend::presentation::request::*;
use uuid::Uuid;

// ============================================================
// Helpers
// ============================================================

struct TestFixture {
    pool: sqlx::PgPool,
    org: dbworks_backend::domain::organization::Organization,
    admin: dbworks_backend::domain::user::AppUser,
    member: dbworks_backend::domain::user::AppUser,
    conn_id: Uuid,
}

async fn setup() -> TestFixture {
    let pool = common::setup_test_db().await;

    let org = organization_repo::create_organization(
        &pool,
        &CreateOrganizationRequest {
            name: "Test Org".to_string(),
        },
    )
    .await
    .unwrap();

    let admin = user_repo::create_user(
        &pool,
        &org.id,
        &CreateUserRequest {
            name: "Admin".to_string(),
            email: "admin@test.com".to_string(),
            role: "super_admin".to_string(),
        },
    )
    .await
    .unwrap();

    let member = user_repo::create_user(
        &pool,
        &org.id,
        &CreateUserRequest {
            name: "Member".to_string(),
            email: "member@test.com".to_string(),
            role: "member".to_string(),
        },
    )
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

    let saved =
        connection_repo::save_connection(&pool, &enc, Some(&org.id), Some(&member.id), &info)
            .await
            .unwrap();

    TestFixture {
        pool,
        org,
        admin,
        member,
        conn_id: saved.id,
    }
}

// ============================================================
// User Connection Permissions
// ============================================================

#[tokio::test]
async fn grant_and_list_user_connection_permission() {
    let f = setup().await;

    let req = GrantUserConnectionPermissionRequest {
        user_id: f.member.id,
        permission: "read".to_string(),
        all_tables: true,
    };
    let perm = permission_repo::grant_user_connection_permission(&f.pool, &f.conn_id, &req)
        .await
        .unwrap();

    assert_eq!(perm.user_id, f.member.id);
    assert_eq!(perm.connection_id, f.conn_id);
    assert_eq!(perm.permission, "read");
    assert!(perm.all_tables);

    let list = permission_repo::list_user_connection_permissions(&f.pool, &f.conn_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn grant_user_connection_permission_upserts() {
    let f = setup().await;

    let req_read = GrantUserConnectionPermissionRequest {
        user_id: f.member.id,
        permission: "read".to_string(),
        all_tables: true,
    };
    permission_repo::grant_user_connection_permission(&f.pool, &f.conn_id, &req_read)
        .await
        .unwrap();

    // Update to write
    let req_write = GrantUserConnectionPermissionRequest {
        user_id: f.member.id,
        permission: "write".to_string(),
        all_tables: false,
    };
    let updated =
        permission_repo::grant_user_connection_permission(&f.pool, &f.conn_id, &req_write)
            .await
            .unwrap();

    assert_eq!(updated.permission, "write");
    assert!(!updated.all_tables);

    // Still only 1 record
    let list = permission_repo::list_user_connection_permissions(&f.pool, &f.conn_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn revoke_user_connection_permission() {
    let f = setup().await;

    let req = GrantUserConnectionPermissionRequest {
        user_id: f.member.id,
        permission: "read".to_string(),
        all_tables: true,
    };
    permission_repo::grant_user_connection_permission(&f.pool, &f.conn_id, &req)
        .await
        .unwrap();

    let revoked =
        permission_repo::revoke_user_connection_permission(&f.pool, &f.conn_id, &f.member.id)
            .await
            .unwrap();
    assert!(revoked);

    let list = permission_repo::list_user_connection_permissions(&f.pool, &f.conn_id)
        .await
        .unwrap();
    assert!(list.is_empty());
}

// ============================================================
// User Table Permissions
// ============================================================

#[tokio::test]
async fn grant_and_list_user_table_permission() {
    let f = setup().await;

    let req = GrantUserTablePermissionRequest {
        table_name: "users".to_string(),
        permission: "write".to_string(),
    };
    let perm =
        permission_repo::grant_user_table_permission(&f.pool, &f.conn_id, &f.member.id, &req)
            .await
            .unwrap();

    assert_eq!(perm.table_name, "users");
    assert_eq!(perm.permission, "write");

    let list = permission_repo::list_user_table_permissions(&f.pool, &f.conn_id, &f.member.id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn revoke_user_table_permission() {
    let f = setup().await;

    let req = GrantUserTablePermissionRequest {
        table_name: "orders".to_string(),
        permission: "read".to_string(),
    };
    permission_repo::grant_user_table_permission(&f.pool, &f.conn_id, &f.member.id, &req)
        .await
        .unwrap();

    let revoked =
        permission_repo::revoke_user_table_permission(&f.pool, &f.conn_id, &f.member.id, "orders")
            .await
            .unwrap();
    assert!(revoked);
}

// ============================================================
// Group Connection Permissions
// ============================================================

#[tokio::test]
async fn grant_and_list_group_connection_permission() {
    let f = setup().await;

    let group = group_repo::create_group(
        &f.pool,
        &f.org.id,
        &CreateGroupRequest {
            name: "Engineers".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    let req = GrantGroupConnectionPermissionRequest {
        group_id: group.id,
        permission: "admin".to_string(),
        all_tables: true,
    };
    let perm = permission_repo::grant_group_connection_permission(&f.pool, &f.conn_id, &req)
        .await
        .unwrap();

    assert_eq!(perm.group_id, group.id);
    assert_eq!(perm.permission, "admin");

    let list = permission_repo::list_group_connection_permissions(&f.pool, &f.conn_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn revoke_group_connection_permission() {
    let f = setup().await;

    let group = group_repo::create_group(
        &f.pool,
        &f.org.id,
        &CreateGroupRequest {
            name: "Team".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    let req = GrantGroupConnectionPermissionRequest {
        group_id: group.id,
        permission: "read".to_string(),
        all_tables: true,
    };
    permission_repo::grant_group_connection_permission(&f.pool, &f.conn_id, &req)
        .await
        .unwrap();

    let revoked =
        permission_repo::revoke_group_connection_permission(&f.pool, &f.conn_id, &group.id)
            .await
            .unwrap();
    assert!(revoked);
}

// ============================================================
// Group Table Permissions
// ============================================================

#[tokio::test]
async fn grant_and_list_group_table_permission() {
    let f = setup().await;

    let group = group_repo::create_group(
        &f.pool,
        &f.org.id,
        &CreateGroupRequest {
            name: "Team".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    let req = GrantGroupTablePermissionRequest {
        table_name: "products".to_string(),
        permission: "write".to_string(),
    };
    let perm = permission_repo::grant_group_table_permission(&f.pool, &f.conn_id, &group.id, &req)
        .await
        .unwrap();

    assert_eq!(perm.table_name, "products");

    let list = permission_repo::list_group_table_permissions(&f.pool, &f.conn_id, &group.id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
}

// ============================================================
// Permission Resolution
// ============================================================

#[tokio::test]
async fn resolve_connection_permission_super_admin() {
    let f = setup().await;

    let (level, all_tables) =
        permission_repo::resolve_connection_permission(&f.pool, &f.admin, &f.conn_id)
            .await
            .unwrap();

    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Admin
    );
    assert!(all_tables);
}

#[tokio::test]
async fn resolve_connection_permission_owner() {
    let f = setup().await;

    // member is the owner_user_id of the connection
    let (level, all_tables) =
        permission_repo::resolve_connection_permission(&f.pool, &f.member, &f.conn_id)
            .await
            .unwrap();

    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Admin
    );
    assert!(all_tables);
}

#[tokio::test]
async fn resolve_connection_permission_user_level() {
    let f = setup().await;

    // Create a second user who is NOT the owner
    let other = user_repo::create_user(
        &f.pool,
        &f.org.id,
        &CreateUserRequest {
            name: "Other".to_string(),
            email: "other@test.com".to_string(),
            role: "member".to_string(),
        },
    )
    .await
    .unwrap();

    // No permission yet
    let (level, _) = permission_repo::resolve_connection_permission(&f.pool, &other, &f.conn_id)
        .await
        .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::None
    );

    // Grant read
    let req = GrantUserConnectionPermissionRequest {
        user_id: other.id,
        permission: "read".to_string(),
        all_tables: false,
    };
    permission_repo::grant_user_connection_permission(&f.pool, &f.conn_id, &req)
        .await
        .unwrap();

    let (level, all_tables) =
        permission_repo::resolve_connection_permission(&f.pool, &other, &f.conn_id)
            .await
            .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Read
    );
    assert!(!all_tables);
}

#[tokio::test]
async fn resolve_connection_permission_group_level() {
    let f = setup().await;

    // Create a user with no direct permission
    let user = user_repo::create_user(
        &f.pool,
        &f.org.id,
        &CreateUserRequest {
            name: "GroupUser".to_string(),
            email: "groupuser@test.com".to_string(),
            role: "member".to_string(),
        },
    )
    .await
    .unwrap();

    // Create group and add user
    let group = group_repo::create_group(
        &f.pool,
        &f.org.id,
        &CreateGroupRequest {
            name: "Team".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();
    group_repo::add_group_member(&f.pool, &group.id, &user.id)
        .await
        .unwrap();

    // Grant group permission
    let req = GrantGroupConnectionPermissionRequest {
        group_id: group.id,
        permission: "write".to_string(),
        all_tables: true,
    };
    permission_repo::grant_group_connection_permission(&f.pool, &f.conn_id, &req)
        .await
        .unwrap();

    let (level, all_tables) =
        permission_repo::resolve_connection_permission(&f.pool, &user, &f.conn_id)
            .await
            .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Write
    );
    assert!(all_tables);
}

#[tokio::test]
async fn resolve_table_permission_super_admin() {
    let f = setup().await;

    let level =
        permission_repo::resolve_table_permission(&f.pool, &f.admin, &f.conn_id, "any_table")
            .await
            .unwrap();

    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Admin
    );
}

#[tokio::test]
async fn resolve_table_permission_with_table_override() {
    let f = setup().await;

    // Create user with connection-level write + all_tables
    let user = user_repo::create_user(
        &f.pool,
        &f.org.id,
        &CreateUserRequest {
            name: "TableUser".to_string(),
            email: "tableuser@test.com".to_string(),
            role: "member".to_string(),
        },
    )
    .await
    .unwrap();

    let conn_req = GrantUserConnectionPermissionRequest {
        user_id: user.id,
        permission: "write".to_string(),
        all_tables: true,
    };
    permission_repo::grant_user_connection_permission(&f.pool, &f.conn_id, &conn_req)
        .await
        .unwrap();

    // Override specific table to read-only
    let table_req = GrantUserTablePermissionRequest {
        table_name: "sensitive".to_string(),
        permission: "read".to_string(),
    };
    permission_repo::grant_user_table_permission(&f.pool, &f.conn_id, &user.id, &table_req)
        .await
        .unwrap();

    // General tables → write (from connection level)
    let level =
        permission_repo::resolve_table_permission(&f.pool, &user, &f.conn_id, "normal_table")
            .await
            .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Write
    );

    // Sensitive table → read (from table override)
    let level = permission_repo::resolve_table_permission(&f.pool, &user, &f.conn_id, "sensitive")
        .await
        .unwrap();
    assert_eq!(
        level,
        dbworks_backend::domain::permission::PermissionLevel::Read
    );
}
