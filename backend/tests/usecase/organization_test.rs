use crate::common;
use dbworks_backend::domain::repository::OrganizationRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::usecase::organization;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn create_organization_via_usecase() {
    let pool = common::setup_test_db().await;
    let repo = PgOrganizationRepository::new(pool);

    let org = organization::create_organization(&repo, "Usecase Org")
        .await
        .unwrap();

    assert_eq!(org.name, "Usecase Org");
    assert!(org.created_at.is_some());
}

#[tokio::test]
#[serial]
async fn list_organizations_via_usecase() {
    let pool = common::setup_test_db().await;
    let repo = PgOrganizationRepository::new(pool);

    for name in ["Alpha", "Beta"] {
        repo.create(name).await.unwrap();
    }

    let orgs = organization::list_organizations(&repo).await.unwrap();
    assert_eq!(orgs.len(), 2);
}
