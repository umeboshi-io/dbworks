use crate::common;
use dbworks_backend::infrastructure::database::{organization_repo, user_repo};
use dbworks_backend::presentation::request::{CreateOrganizationRequest, CreateUserRequest};
use serial_test::serial;

async fn create_test_org(
    pool: &sqlx::PgPool,
) -> dbworks_backend::domain::organization::Organization {
    let req = CreateOrganizationRequest {
        name: "Test Org".to_string(),
    };
    organization_repo::create_organization(pool, &req)
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
async fn create_user() {
    let pool = common::setup_test_db().await;
    let org = create_test_org(&pool).await;

    let req = CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        role: "member".to_string(),
    };
    let user = user_repo::create_user(&pool, &org.id, &req).await.unwrap();

    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.role, "member");
    assert_eq!(user.organization_id, Some(org.id));
}

#[tokio::test]
#[serial]
async fn list_users_by_org() {
    let pool = common::setup_test_db().await;
    let org_a = create_test_org(&pool).await;

    let org_b_req = CreateOrganizationRequest {
        name: "Org B".to_string(),
    };
    let org_b = organization_repo::create_organization(&pool, &org_b_req)
        .await
        .unwrap();

    // Create 2 users in org_a, 1 in org_b
    for (name, email, org_id) in [
        ("Alice", "alice@a.com", &org_a.id),
        ("Bob", "bob@a.com", &org_a.id),
        ("Charlie", "charlie@b.com", &org_b.id),
    ] {
        let req = CreateUserRequest {
            name: name.to_string(),
            email: email.to_string(),
            role: "member".to_string(),
        };
        user_repo::create_user(&pool, org_id, &req).await.unwrap();
    }

    let users_a = user_repo::list_users_by_org(&pool, &org_a.id)
        .await
        .unwrap();
    assert_eq!(users_a.len(), 2);

    let users_b = user_repo::list_users_by_org(&pool, &org_b.id)
        .await
        .unwrap();
    assert_eq!(users_b.len(), 1);
}

#[tokio::test]
#[serial]
async fn get_user_found_and_not_found() {
    let pool = common::setup_test_db().await;
    let org = create_test_org(&pool).await;

    let req = CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        role: "member".to_string(),
    };
    let created = user_repo::create_user(&pool, &org.id, &req).await.unwrap();

    let found = user_repo::get_user(&pool, &created.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, "alice@example.com");

    let not_found = user_repo::get_user(&pool, &uuid::Uuid::new_v4())
        .await
        .unwrap();
    assert!(not_found.is_none());
}
