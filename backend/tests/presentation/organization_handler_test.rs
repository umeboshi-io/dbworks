use crate::common;
use crate::presentation::helpers::build_test_app;

use http::Request;
use http_body_util::BodyExt;
use serial_test::serial;
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn create_organization_returns_201() {
    let pool = common::setup_test_db().await;
    let app = build_test_app(pool);

    let body = serde_json::json!({ "name": "Test Org" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/organizations")
        .header("Content-Type", "application/json")
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
    let org_repo =
        dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository::new(
            pool.clone(),
        );

    use dbworks_backend::domain::repository::OrganizationRepository;
    org_repo.create("Alpha").await.unwrap();
    org_repo.create("Beta").await.unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .uri("/api/organizations")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);
}
