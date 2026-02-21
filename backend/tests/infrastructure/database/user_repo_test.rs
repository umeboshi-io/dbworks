use crate::common;
use dbworks_backend::domain::repository::{OrganizationRepository, UserRepository};
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use serial_test::serial;

async fn create_test_org(
    org_repo: &PgOrganizationRepository,
) -> dbworks_backend::domain::organization::Organization {
    org_repo.create("Test Org").await.unwrap()
}

#[tokio::test]
#[serial]
async fn create_user() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool);
    let org = create_test_org(&org_repo).await;

    let user = user_repo
        .create(&org.id, "Alice", "alice@example.com", "member")
        .await
        .unwrap();

    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.role, "member");
    assert_eq!(user.organization_id, Some(org.id));
}

#[tokio::test]
#[serial]
async fn list_users_by_org() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool);

    let org_a = create_test_org(&org_repo).await;
    let org_b = org_repo.create("Org B").await.unwrap();

    // Create 2 users in org_a, 1 in org_b
    for (name, email, org_id) in [
        ("Alice", "alice@a.com", &org_a.id),
        ("Bob", "bob@a.com", &org_a.id),
        ("Charlie", "charlie@b.com", &org_b.id),
    ] {
        user_repo
            .create(org_id, name, email, "member")
            .await
            .unwrap();
    }

    let users_a = user_repo.list_by_org(&org_a.id).await.unwrap();
    assert_eq!(users_a.len(), 2);

    let users_b = user_repo.list_by_org(&org_b.id).await.unwrap();
    assert_eq!(users_b.len(), 1);
}

#[tokio::test]
#[serial]
async fn get_user_found_and_not_found() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool);
    let org = create_test_org(&org_repo).await;

    let created = user_repo
        .create(&org.id, "Alice", "alice@example.com", "member")
        .await
        .unwrap();

    let found = user_repo.get(&created.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, "alice@example.com");

    let not_found = user_repo.get(&uuid::Uuid::new_v4()).await.unwrap();
    assert!(not_found.is_none());
}
