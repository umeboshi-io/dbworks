use crate::common;
use crate::presentation::helpers::{build_test_app, seed_connection};

use dbworks_backend::domain::repository::{
    OrganizationRepository, PermissionRepository, UserRepository,
};
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::permission_repo::PgPermissionRepository;
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
async fn grant_user_conn_permission_returns_201() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;

    let user_repo = PgUserRepository::new(pool.clone());
    let target = user_repo
        .create(&org_id, "Target", "target@test.com", "member")
        .await
        .unwrap();

    // Seed a connection record to satisfy FK constraint
    let conn_id = seed_connection(&pool, &org_id).await;

    let app = build_test_app(pool);

    let body = serde_json::json!({
        "user_id": target.id,
        "permission": "read",
        "all_tables": true
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/connections/{}/user-permissions", conn_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["permission"], "read");
    assert_eq!(json["all_tables"], true);
}

#[tokio::test]
#[serial]
async fn grant_user_conn_permission_as_member_returns_403() {
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
        "user_id": uuid::Uuid::new_v4(),
        "permission": "read"
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/connections/{}/user-permissions",
            uuid::Uuid::new_v4()
        ))
        .header("Content-Type", "application/json")
        .header("X-User-Id", member.id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
#[serial]
async fn list_user_conn_permissions_returns_200() {
    let pool = common::setup_test_db().await;
    let app = build_test_app(pool);

    let conn_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/api/connections/{}/user-permissions", conn_id))
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
async fn revoke_user_conn_permission_not_found_returns_404() {
    let pool = common::setup_test_db().await;
    let (_, admin_id) = seed(&pool).await;
    let app = build_test_app(pool);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/connections/{}/user-permissions/{}",
            uuid::Uuid::new_v4(),
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
async fn grant_and_revoke_user_conn_permission() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;

    let user_repo = PgUserRepository::new(pool.clone());
    let permission_repo = PgPermissionRepository::new(pool.clone());
    let target = user_repo
        .create(&org_id, "Target", "target@test.com", "member")
        .await
        .unwrap();

    // Seed a connection record to satisfy FK
    let conn_id = seed_connection(&pool, &org_id).await;

    // Grant permission via repo
    permission_repo
        .grant_user_connection_permission(&conn_id, &target.id, "write", true)
        .await
        .unwrap();

    let app = build_test_app(pool);

    // Revoke via API
    let req = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/api/connections/{}/user-permissions/{}",
            conn_id, target.id
        ))
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
#[serial]
async fn grant_group_conn_permission_returns_201() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;

    use dbworks_backend::domain::repository::GroupRepository;
    use dbworks_backend::infrastructure::database::group_repo::PgGroupRepository;

    let group_repo = PgGroupRepository::new(pool.clone());
    let group = group_repo.create(&org_id, "Team", None).await.unwrap();

    // Seed a connection record to satisfy FK
    let conn_id = seed_connection(&pool, &org_id).await;

    let app = build_test_app(pool);

    let body = serde_json::json!({
        "group_id": group.id,
        "permission": "read",
        "all_tables": false
    });
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/connections/{}/group-permissions", conn_id))
        .header("Content-Type", "application/json")
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["permission"], "read");
    assert_eq!(json["all_tables"], false);
}
