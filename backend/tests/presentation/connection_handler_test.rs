use crate::common;
use crate::presentation::helpers::build_test_app;

use dbworks_backend::domain::repository::{OrganizationRepository, UserRepository};
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use http::Request;
use http_body_util::BodyExt;
use serial_test::serial;
use tower::ServiceExt;

async fn seed(pool: &sqlx::PgPool) -> (uuid::Uuid, uuid::Uuid) {
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());
    let org = org_repo.create("Org").await.unwrap();
    let admin = user_repo
        .create(&org.id, "Admin", "admin@test.com", "super_admin")
        .await
        .unwrap();
    (org.id, admin.id)
}

#[tokio::test]
#[serial]
async fn create_connection_returns_401_for_unknown_user() {
    let pool = common::setup_test_db().await;
    let app = build_test_app(pool);

    let body = serde_json::json!({
        "name": "mydb",
        "host": "localhost",
        "port": 5432,
        "database": "testdb",
        "user": "u",
        "password": "p"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/connections")
        .header("Content-Type", "application/json")
        .header("X-User-Id", uuid::Uuid::new_v4().to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
#[serial]
async fn list_connections_returns_200() {
    let pool = common::setup_test_db().await;
    let (_, admin_id) = seed(&pool).await;
    let app = build_test_app(pool);

    let req = Request::builder()
        .uri("/api/connections")
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
#[serial]
async fn delete_connection_not_found_returns_404() {
    let pool = common::setup_test_db().await;
    let (_, admin_id) = seed(&pool).await;
    let app = build_test_app(pool);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/connections/{}", uuid::Uuid::new_v4()))
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
#[serial]
async fn delete_connection_as_member_returns_403() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let org = org_repo.create("Org").await.unwrap();
    let member = user_repo
        .create(&org.id, "Member", "member@test.com", "member")
        .await
        .unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/connections/{}", uuid::Uuid::new_v4()))
        .header("X-User-Id", member.id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}
