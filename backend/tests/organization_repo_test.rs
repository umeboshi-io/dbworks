mod common;

use dbworks_backend::infrastructure::database::organization_repo;
use dbworks_backend::presentation::request::CreateOrganizationRequest;

#[tokio::test]
async fn create_organization() {
    let pool = common::setup_test_db().await;

    let req = CreateOrganizationRequest {
        name: "Test Org".to_string(),
    };
    let org = organization_repo::create_organization(&pool, &req)
        .await
        .unwrap();

    assert_eq!(org.name, "Test Org");
    assert!(org.created_at.is_some());
}

#[tokio::test]
async fn list_organizations_returns_all() {
    let pool = common::setup_test_db().await;

    for name in ["Alpha", "Beta", "Gamma"] {
        let req = CreateOrganizationRequest {
            name: name.to_string(),
        };
        organization_repo::create_organization(&pool, &req)
            .await
            .unwrap();
    }

    let orgs = organization_repo::list_organizations(&pool).await.unwrap();
    assert_eq!(orgs.len(), 3);
}

#[tokio::test]
async fn get_organization_found() {
    let pool = common::setup_test_db().await;

    let req = CreateOrganizationRequest {
        name: "FindMe".to_string(),
    };
    let created = organization_repo::create_organization(&pool, &req)
        .await
        .unwrap();

    let found = organization_repo::get_organization(&pool, &created.id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "FindMe");
}

#[tokio::test]
async fn get_organization_not_found() {
    let pool = common::setup_test_db().await;

    let found = organization_repo::get_organization(&pool, &uuid::Uuid::new_v4())
        .await
        .unwrap();
    assert!(found.is_none());
}
