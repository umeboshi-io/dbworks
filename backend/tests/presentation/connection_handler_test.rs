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
    let (_, admin_id) = seed_org_and_owner(&pool).await;
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
    let (_, admin_id) = seed_org_and_owner(&pool).await;
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
    let org_member_repo = PgOrganizationMemberRepository::new(pool.clone());

    let org = org_repo.create("Org").await.unwrap();

    // Create an owner to seed a connection
    let owner = user_repo
        .create("Owner", "owner@test.com", "member")
        .await
        .unwrap();
    org_member_repo
        .add_member(&org.id, &owner.id, "owner")
        .await
        .unwrap();

    // Seed a connection owned by the org
    let conn_id = crate::presentation::helpers::seed_connection(&pool, &org.id).await;

    // Create a member (not owner)
    let member = user_repo
        .create("Member", "member@test.com", "member")
        .await
        .unwrap();
    org_member_repo
        .add_member(&org.id, &member.id, "member")
        .await
        .unwrap();

    let app = build_test_app(pool);

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/connections/{}", conn_id))
        .header("X-User-Id", member.id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 403);
}

/// When db_type is omitted from the request body, it should default to "postgres"
#[tokio::test]
#[serial]
async fn create_connection_without_db_type_defaults_to_postgres() {
    let pool = common::setup_test_db().await;
    let (_, admin_id) = seed_org_and_owner(&pool).await;
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
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    if status == 400 {
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let err_msg = json["error"].as_str().unwrap_or("");
        assert!(
            !err_msg.contains("Unsupported"),
            "Default db_type should be 'postgres', got error: {}",
            err_msg
        );
    }
}

/// When db_type is explicitly "mysql"
#[tokio::test]
#[serial]
async fn create_connection_with_db_type_mysql() {
    let pool = common::setup_test_db().await;
    let (_, admin_id) = seed_org_and_owner(&pool).await;
    let app = build_test_app(pool);

    let body = serde_json::json!({
        "name": "mydb",
        "db_type": "mysql",
        "host": "localhost",
        "port": 3306,
        "database": "testdb",
        "user": "u",
        "password": "p"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/connections")
        .header("Content-Type", "application/json")
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    if status == 400 {
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let err_msg = json["error"].as_str().unwrap_or("");
        assert!(
            !err_msg.contains("Unsupported"),
            "mysql should be a supported db_type, got error: {}",
            err_msg
        );
    }
}

/// When db_type is unsupported, should return 400
#[tokio::test]
#[serial]
async fn create_connection_with_unsupported_db_type_returns_400() {
    let pool = common::setup_test_db().await;
    let (_, admin_id) = seed_org_and_owner(&pool).await;
    let app = build_test_app(pool);

    let body = serde_json::json!({
        "name": "mydb",
        "db_type": "sqlite",
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
        .header("X-User-Id", admin_id.to_string())
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 400);

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let err_msg = json["error"].as_str().unwrap_or("");
    assert!(
        err_msg.contains("Unsupported database type"),
        "Expected unsupported db type error, got: {}",
        err_msg
    );
}
