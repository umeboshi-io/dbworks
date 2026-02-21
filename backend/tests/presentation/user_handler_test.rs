use crate::common;
use crate::presentation::helpers::build_test_app;

use dbworks_backend::domain::repository::{OrganizationRepository, UserRepository};
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use http::Request;
use http_body_util::BodyExt;
use serial_test::serial;
use tower::ServiceExt;

/// Seed an organization + super_admin user and return (org_id, user_id).
async fn seed_org_and_admin(pool: &sqlx::PgPool) -> (uuid::Uuid, uuid::Uuid) {
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let org = org_repo.create("Test Org").await.unwrap();
    let user = user_repo
        .create(&org.id, "Admin", "admin@test.com", "super_admin")
        .await
        .unwrap();
    (org.id, user.id)
}

#[tokio::test]
#[serial]
async fn create_user_returns_201() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed_org_and_admin(&pool).await;

    let app = build_test_app(pool);

    let body = serde_json::json!({
        "name": "Alice",
        "email": "alice@test.com",
        "role": "member"
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/organizations/{}/users", org_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Alice");
    assert_eq!(json["email"], "alice@test.com");
}

#[tokio::test]
#[serial]
async fn create_user_without_auth_returns_401() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let org = org_repo.create("Org").await.unwrap();

    let app = build_test_app(pool);

    let body = serde_json::json!({
        "name": "Bob",
        "email": "bob@test.com"
    });
    // Use a non-existent user ID to trigger 401
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/organizations/{}/users", org.id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", uuid::Uuid::new_v4().to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
#[serial]
async fn create_user_as_member_returns_403() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let org = org_repo.create("Org").await.unwrap();
    let member = user_repo
        .create(&org.id, "Member", "member@test.com", "member")
        .await
        .unwrap();

    let app = build_test_app(pool);

    let body = serde_json::json!({
        "name": "New",
        "email": "new@test.com"
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/organizations/{}/users", org.id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", member.id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
#[serial]
async fn list_users_returns_200() {
    let pool = common::setup_test_db().await;
    let (org_id, _admin_id) = seed_org_and_admin(&pool).await;

    let app = build_test_app(pool);

    let req = Request::builder()
        .uri(format!("/api/organizations/{}/users", org_id))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    // Should have the seeded admin
    assert_eq!(arr.len(), 1);
}
