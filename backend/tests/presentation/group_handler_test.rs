use crate::common;
use crate::presentation::helpers::build_test_app;

use dbworks_backend::domain::repository::{
    GroupRepository, OrganizationRepository, UserRepository,
};
use dbworks_backend::infrastructure::database::group_repo::PgGroupRepository;
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use http::Request;
use http_body_util::BodyExt;
use serial_test::serial;
use tower::ServiceExt;

/// Seed org + super_admin and return (org_id, admin_id).
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
async fn create_group_returns_201() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;
    let app = build_test_app(pool);

    let body = serde_json::json!({ "name": "Dev Team", "description": "Developers" });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/organizations/{}/groups", org_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "Dev Team");
}

#[tokio::test]
#[serial]
async fn list_groups_returns_200() {
    let pool = common::setup_test_db().await;
    let (org_id, _) = seed(&pool).await;

    let group_repo = PgGroupRepository::new(pool.clone());
    group_repo.create(&org_id, "G1", None).await.unwrap();
    group_repo.create(&org_id, "G2", None).await.unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .uri(format!("/api/organizations/{}/groups", org_id))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[tokio::test]
#[serial]
async fn add_group_member_returns_204() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;

    let group_repo = PgGroupRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let group = group_repo.create(&org_id, "Team", None).await.unwrap();
    let member = user_repo
        .create(&org_id, "Member", "member@test.com", "member")
        .await
        .unwrap();

    let app = build_test_app(pool);

    let body = serde_json::json!({ "user_id": member.id });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/groups/{}/members", group.id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
#[serial]
async fn remove_group_member_returns_204() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;

    let group_repo = PgGroupRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let group = group_repo.create(&org_id, "Team", None).await.unwrap();
    let member = user_repo
        .create(&org_id, "Member", "member@test.com", "member")
        .await
        .unwrap();
    group_repo.add_member(&group.id, &member.id).await.unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/groups/{}/members/{}", group.id, member.id))
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
#[serial]
async fn remove_nonexistent_member_returns_404() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;

    let group_repo = PgGroupRepository::new(pool.clone());
    let group = group_repo.create(&org_id, "Team", None).await.unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/groups/{}/members/{}",
            group.id,
            uuid::Uuid::new_v4()
        ))
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
#[serial]
async fn list_group_members_returns_200() {
    let pool = common::setup_test_db().await;
    let (org_id, _) = seed(&pool).await;

    let group_repo = PgGroupRepository::new(pool.clone());
    let group = group_repo.create(&org_id, "Team", None).await.unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .uri(format!("/api/groups/{}/members", group.id))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.as_array().unwrap().is_empty());
}
