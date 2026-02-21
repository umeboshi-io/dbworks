use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::middleware::get_current_user;
use crate::presentation::request::*;
use crate::presentation::state::AppState;
use crate::usecase;

use super::into_response;

// ============================================================
// User Connection Permissions
// ============================================================

pub async fn grant_user_conn_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
    Json(req): Json<GrantUserConnectionPermissionRequest>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::grant_user_connection_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &req.user_id,
        &req.permission,
        req.all_tables,
    )
    .await
    {
        Ok(p) => (StatusCode::CREATED, Json(serde_json::json!(p))).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn revoke_user_conn_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::revoke_user_connection_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &user_id,
    )
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_user_conn_permissions(
    State(state): State<AppState>,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    match usecase::permission::list_user_connection_permissions(&*state.permission_repo, &conn_id)
        .await
    {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => into_response(e),
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
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::grant_user_table_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &user_id,
        &req.table_name,
        &req.permission,
    )
    .await
    {
        Ok(p) => (StatusCode::CREATED, Json(serde_json::json!(p))).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn revoke_user_table_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, user_id, table)): Path<(Uuid, Uuid, String)>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::revoke_user_table_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &user_id,
        &table,
    )
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_user_table_permissions(
    State(state): State<AppState>,
    Path((conn_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match usecase::permission::list_user_table_permissions(
        &*state.permission_repo,
        &conn_id,
        &user_id,
    )
    .await
    {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => into_response(e),
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
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::grant_group_connection_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &req.group_id,
        &req.permission,
        req.all_tables,
    )
    .await
    {
        Ok(p) => (StatusCode::CREATED, Json(serde_json::json!(p))).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn revoke_group_conn_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, group_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::revoke_group_connection_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &group_id,
    )
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_group_conn_permissions(
    State(state): State<AppState>,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    match usecase::permission::list_group_connection_permissions(&*state.permission_repo, &conn_id)
        .await
    {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => into_response(e),
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
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::grant_group_table_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &group_id,
        &req.table_name,
        &req.permission,
    )
    .await
    {
        Ok(p) => (StatusCode::CREATED, Json(serde_json::json!(p))).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn revoke_group_table_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, group_id, table)): Path<(Uuid, Uuid, String)>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    match usecase::permission::revoke_group_table_permission(
        &*state.permission_repo,
        &caller,
        &conn_id,
        &group_id,
        &table,
    )
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_group_table_permissions(
    State(state): State<AppState>,
    Path((conn_id, group_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match usecase::permission::list_group_table_permissions(
        &*state.permission_repo,
        &conn_id,
        &group_id,
    )
    .await
    {
        Ok(perms) => Json(serde_json::json!(perms)).into_response(),
        Err(e) => into_response(e),
    }
}
