use crate::common;
use crate::presentation::helpers::{build_test_app, seed_org_and_owner};

use dbworks_backend::domain::repository::{
    OrganizationMemberRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::infrastructure::database::organization_member_repo::PgOrganizationMemberRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use http::Request;
use http_body_util::BodyExt;
use serial_test::serial;
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn create_organization_returns_201() {
    let pool = common::setup_test_db().await;
    let user_repo = PgUserRepository::new(pool.clone());
    let user = user_repo
        .create("Admin", "admin@test.com", "member")
        .await
        .unwrap();

    let app = build_test_app(pool);

    let body = serde_json::json!({ "name": "Test Org" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/organizations")
        .header("Content-Type", "application/json")
        .header("X-User-Id", user.id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Test Org");
    assert!(json["id"].is_string());
}

#[tokio::test]
#[serial]
async fn list_organizations_returns_200() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let org_member_repo = PgOrganizationMemberRepository::new(pool.clone());

    let org_a = org_repo.create("Alpha").await.unwrap();
    let org_b = org_repo.create("Beta").await.unwrap();

    let user = user_repo
        .create("Admin", "admin@test.com", "member")
        .await
        .unwrap();
    // Add user to both orgs
    org_member_repo
        .add_member(&org_a.id, &user.id, "owner")
        .await
        .unwrap();
    org_member_repo
        .add_member(&org_b.id, &user.id, "member")
        .await
        .unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .uri("/api/organizations")
        .header("X-User-Id", user.id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}
