use crate::common;
use crate::presentation::helpers::{build_test_app, seed_connection};

use dbworks_backend::domain::repository::{
    OrganizationRepository, PermissionRepository, UserRepository,
};
use dbworks_backend::infrastructure::database::organization_repo::PgOrganizationRepository;
use dbworks_backend::infrastructure::database::permission_repo::PgPermissionRepository;
use dbworks_backend::infrastructure::database::user_repo::PgUserRepository;
use http::Request;
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

/// data endpoints require a valid connection. With no connection registered,
/// list_tables should return 401 for unknown user.
#[tokio::test]
#[serial]
async fn list_tables_returns_401_for_unknown_user() {
    let pool = common::setup_test_db().await;
    let app = build_test_app(pool);

    let conn_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/api/connections/{}/tables", conn_id))
        .header("X-User-Id", uuid::Uuid::new_v4().to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

/// Authenticated user but no permission → 403
#[tokio::test]
#[serial]
async fn list_tables_returns_403_for_no_permission() {
    let pool = common::setup_test_db().await;
    let org_repo = PgOrganizationRepository::new(pool.clone());
    let user_repo = PgUserRepository::new(pool.clone());

    let org = org_repo.create("Org").await.unwrap();
    let member = user_repo
        .create(&org.id, "Member", "member@test.com", "member")
        .await
        .unwrap();

    let app = build_test_app(pool);

    let conn_id = uuid::Uuid::new_v4();
    let req = Request::builder()
        .uri(format!("/api/connections/{}/tables", conn_id))
        .header("X-User-Id", member.id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

/// Authenticated + read permission but connection not found in connection_manager → 404
#[tokio::test]
#[serial]
async fn list_tables_returns_404_for_missing_connection() {
    let pool = common::setup_test_db().await;
    let (org_id, admin_id) = seed(&pool).await;

    // Seed a connection record for FK, then grant permission
    let conn_id = seed_connection(&pool, &org_id).await;

    let permission_repo = PgPermissionRepository::new(pool.clone());
    permission_repo
        .grant_user_connection_permission(&conn_id, &admin_id, "read", true)
        .await
        .unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .uri(format!("/api/connections/{}/tables", conn_id))
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    // Connection not registered in ConnectionManager → 404
    assert_eq!(resp.status(), 404);
}
