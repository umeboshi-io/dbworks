use crate::common;
use dbworks_backend::domain::repository::OrganizationRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn create_organization() {
    let pool = common::setup_test_db().await;
    let repo = PgOrganizationRepository::new(pool);

    let org = repo.create("Test Org").await.unwrap();

    assert_eq!(org.name, "Test Org");
    assert!(org.created_at.is_some());
}

#[tokio::test]
#[serial]
async fn list_organizations_returns_all() {
    let pool = common::setup_test_db().await;
    let repo = PgOrganizationRepository::new(pool);

    for name in ["Alpha", "Beta", "Gamma"] {
        repo.create(name).await.unwrap();
    }

    let orgs = repo.list().await.unwrap();
    assert_eq!(orgs.len(), 3);
}

#[tokio::test]
#[serial]
async fn get_organization_found() {
    let pool = common::setup_test_db().await;
    let repo = PgOrganizationRepository::new(pool);

    let created = repo.create("FindMe").await.unwrap();

    let found = repo.get(&created.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "FindMe");
}

#[tokio::test]
#[serial]
async fn get_organization_not_found() {
    let pool = common::setup_test_db().await;
    let repo = PgOrganizationRepository::new(pool);

    let found = repo.get(&uuid::Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}
