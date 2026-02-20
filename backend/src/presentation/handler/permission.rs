use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::dto::*;
use crate::infrastructure::database::permission_repo;
use crate::presentation::middleware::{get_current_user, require_super_admin};
use crate::presentation::state::AppState;

// ============================================================
// User Connection Permissions
// ============================================================

pub async fn grant_user_conn_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
    Json(req): Json<GrantUserConnectionPermissionRequest>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::grant_user_connection_permission(&state.pool, &conn_id, &req).await {
        Ok(perm) => (StatusCode::CREATED, Json(serde_json::json!(perm))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn revoke_user_conn_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::revoke_user_connection_permission(&state.pool, &conn_id, &user_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Permission not found" }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_user_conn_permissions(
    State(state): State<AppState>,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    match permission_repo::list_user_connection_permissions(&state.pool, &conn_id).await {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ============================================================
// User Table Permissions
// ============================================================

pub async fn grant_user_table_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<GrantUserTablePermissionRequest>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::grant_user_table_permission(&state.pool, &conn_id, &user_id, &req).await {
        Ok(perm) => (StatusCode::CREATED, Json(serde_json::json!(perm))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn revoke_user_table_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, user_id, table)): Path<(Uuid, Uuid, String)>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::revoke_user_table_permission(&state.pool, &conn_id, &user_id, &table).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Permission not found" }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_user_table_permissions(
    State(state): State<AppState>,
    Path((conn_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match permission_repo::list_user_table_permissions(&state.pool, &conn_id, &user_id).await {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ============================================================
// Group Connection Permissions
// ============================================================

pub async fn grant_group_conn_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
    Json(req): Json<GrantGroupConnectionPermissionRequest>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::grant_group_connection_permission(&state.pool, &conn_id, &req).await {
        Ok(perm) => (StatusCode::CREATED, Json(serde_json::json!(perm))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn revoke_group_conn_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, group_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::revoke_group_connection_permission(&state.pool, &conn_id, &group_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Permission not found" }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_group_conn_permissions(
    State(state): State<AppState>,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    match permission_repo::list_group_connection_permissions(&state.pool, &conn_id).await {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ============================================================
// Group Table Permissions
// ============================================================

pub async fn grant_group_table_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, group_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<GrantGroupTablePermissionRequest>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::grant_group_table_permission(&state.pool, &conn_id, &group_id, &req).await {
        Ok(perm) => (StatusCode::CREATED, Json(serde_json::json!(perm))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn revoke_group_table_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, group_id, table)): Path<(Uuid, Uuid, String)>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match permission_repo::revoke_group_table_permission(&state.pool, &conn_id, &group_id, &table).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Permission not found" }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn list_group_table_permissions(
    State(state): State<AppState>,
    Path((conn_id, group_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match permission_repo::list_group_table_permissions(&state.pool, &conn_id, &group_id).await {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
