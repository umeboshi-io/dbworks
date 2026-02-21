use crate::common;
use dbworks_backend::domain::repository::{
    GroupRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::infrastructure::database::group_repo::PgGroupRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use serial_test::serial;

async fn setup_org_and_users(
    pool: &sqlx::PgPool,
) -> (
    dbworks_backend::domain::organization::Organization,
    dbworks_backend::domain::user::AppUser,
    dbworks_backend::domain::user::AppUser,
) {
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let org = org_repo.create("Test Org").await.unwrap();

    let alice = user_repo
        .create(&org.id, "Alice", "alice@test.com", "member")
        .await
        .unwrap();

    let bob = user_repo
        .create(&org.id, "Bob", "bob@test.com", "member")
        .await
        .unwrap();

    (org, alice, bob)
}

#[tokio::test]
#[serial]
async fn create_and_list_groups() {
    let pool = common::setup_test_db().await;
    let (org, _, _) = setup_org_and_users(&pool).await;
    let group_repo = PgGroupRepository::new(pool);

    let group = group_repo
        .create(&org.id, "Engineers", Some("Engineering team"))
        .await
        .unwrap();

    assert_eq!(group.name, "Engineers");
    assert_eq!(group.organization_id, org.id);
    assert_eq!(group.description, Some("Engineering team".to_string()));

    let groups = group_repo.list_by_org(&org.id).await.unwrap();
    assert_eq!(groups.len(), 1);
}

#[tokio::test]
#[serial]
async fn add_and_list_group_members() {
    let pool = common::setup_test_db().await;
    let (org, alice, bob) = setup_org_and_users(&pool).await;
    let group_repo = PgGroupRepository::new(pool);

    let group = group_repo.create(&org.id, "Team", None).await.unwrap();

    group_repo.add_member(&group.id, &alice.id).await.unwrap();
    group_repo.add_member(&group.id, &bob.id).await.unwrap();

    let members = group_repo.list_members(&group.id).await.unwrap();
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
    let group_repo = PgGroupRepository::new(pool);

    let group = group_repo.create(&org.id, "Team", None).await.unwrap();

    // Adding same member twice should not fail (ON CONFLICT DO NOTHING)
    group_repo.add_member(&group.id, &alice.id).await.unwrap();
    group_repo.add_member(&group.id, &alice.id).await.unwrap();

    let members = group_repo.list_members(&group.id).await.unwrap();
    assert_eq!(members.len(), 1);
}

#[tokio::test]
#[serial]
async fn remove_group_member() {
    let pool = common::setup_test_db().await;
    let (org, alice, _) = setup_org_and_users(&pool).await;
    let group_repo = PgGroupRepository::new(pool);

    let group = group_repo.create(&org.id, "Team", None).await.unwrap();

    group_repo.add_member(&group.id, &alice.id).await.unwrap();

    let removed = group_repo
        .remove_member(&group.id, &alice.id)
        .await
        .unwrap();
    assert!(removed);

    let members = group_repo.list_members(&group.id).await.unwrap();
    assert_eq!(members.len(), 0);

    // Removing again should return false
    let removed_again = group_repo
        .remove_member(&group.id, &alice.id)
        .await
        .unwrap();
    assert!(!removed_again);
}
