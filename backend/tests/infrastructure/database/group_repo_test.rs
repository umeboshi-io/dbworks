use crate::common;
use dbworks_backend::infrastructure::database::{group_repo, organization_repo, user_repo};
use dbworks_backend::presentation::request::{
    CreateGroupRequest, CreateOrganizationRequest, CreateUserRequest,
};
use serial_test::serial;

async fn setup_org_and_users(
    pool: &sqlx::PgPool,
) -> (
    dbworks_backend::domain::organization::Organization,
    dbworks_backend::domain::user::AppUser,
    dbworks_backend::domain::user::AppUser,
) {
    let org = organization_repo::create_organization(
        pool,
        &CreateOrganizationRequest {
            name: "Test Org".to_string(),
        },
    )
    .await
    .unwrap();

    let alice = user_repo::create_user(
        pool,
        &org.id,
        &CreateUserRequest {
            name: "Alice".to_string(),
            email: "alice@test.com".to_string(),
            role: "member".to_string(),
        },
    )
    .await
    .unwrap();

    let bob = user_repo::create_user(
        pool,
        &org.id,
        &CreateUserRequest {
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            role: "member".to_string(),
        },
    )
    .await
    .unwrap();

    (org, alice, bob)
}

#[tokio::test]
#[serial]
async fn create_and_list_groups() {
    let pool = common::setup_test_db().await;
    let (org, _, _) = setup_org_and_users(&pool).await;

    let req = CreateGroupRequest {
        name: "Engineers".to_string(),
        description: Some("Engineering team".to_string()),
    };
    let group = group_repo::create_group(&pool, &org.id, &req)
        .await
        .unwrap();

    assert_eq!(group.name, "Engineers");
    assert_eq!(group.organization_id, org.id);
    assert_eq!(group.description, Some("Engineering team".to_string()));

    let groups = group_repo::list_groups_by_org(&pool, &org.id)
        .await
        .unwrap();
    assert_eq!(groups.len(), 1);
}

#[tokio::test]
#[serial]
async fn add_and_list_group_members() {
    let pool = common::setup_test_db().await;
    let (org, alice, bob) = setup_org_and_users(&pool).await;

    let group = group_repo::create_group(
        &pool,
        &org.id,
        &CreateGroupRequest {
            name: "Team".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    group_repo::add_group_member(&pool, &group.id, &alice.id)
        .await
        .unwrap();
    group_repo::add_group_member(&pool, &group.id, &bob.id)
        .await
        .unwrap();

    let members = group_repo::list_group_members(&pool, &group.id)
        .await
        .unwrap();
    assert_eq!(members.len(), 2);

    let names: Vec<&str> = members.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));
}

#[tokio::test]
#[serial]
async fn add_group_member_idempotent() {
    let pool = common::setup_test_db().await;
    let (org, alice, _) = setup_org_and_users(&pool).await;

    let group = group_repo::create_group(
        &pool,
        &org.id,
        &CreateGroupRequest {
            name: "Team".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    // Adding same member twice should not fail (ON CONFLICT DO NOTHING)
    group_repo::add_group_member(&pool, &group.id, &alice.id)
        .await
        .unwrap();
    group_repo::add_group_member(&pool, &group.id, &alice.id)
        .await
        .unwrap();

    let members = group_repo::list_group_members(&pool, &group.id)
        .await
        .unwrap();
    assert_eq!(members.len(), 1);
}

#[tokio::test]
#[serial]
async fn remove_group_member() {
    let pool = common::setup_test_db().await;
    let (org, alice, _) = setup_org_and_users(&pool).await;

    let group = group_repo::create_group(
        &pool,
        &org.id,
        &CreateGroupRequest {
            name: "Team".to_string(),
            description: None,
        },
    )
    .await
    .unwrap();

    group_repo::add_group_member(&pool, &group.id, &alice.id)
        .await
        .unwrap();

    let removed = group_repo::remove_group_member(&pool, &group.id, &alice.id)
        .await
        .unwrap();
    assert!(removed);

    let members = group_repo::list_group_members(&pool, &group.id)
        .await
        .unwrap();
    assert_eq!(members.len(), 0);

    // Removing again should return false
    let removed_again = group_repo::remove_group_member(&pool, &group.id, &alice.id)
        .await
        .unwrap();
    assert!(!removed_again);
}
