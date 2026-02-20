use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::request::RowsQuery;
use crate::infrastructure::database::permission_repo;
use crate::presentation::middleware::get_current_user;
use crate::presentation::state::AppState;

// ============================================================
// Table Introspection (with permission checks)
// ============================================================

pub async fn list_tables(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, "GET /api/connections/:conn_id/tables");

    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };

    // Check connection-level permission
    let (conn_perm, _) = match permission_repo::resolve_connection_permission(&state.pool, &current_user, &conn_id).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    if !conn_perm.can_read() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "No access to this connection" }))).into_response();
    }

    let ds = match state.connection_manager.get_datasource(&conn_id).await {
        Some(ds) => ds,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Connection not found" }))).into_response(),
    };

    match ds.list_tables().await {
        Ok(tables) => Json(serde_json::json!(tables)).into_response(),
        Err(e) => {
            tracing::error!(connection_id = %conn_id, error = %e, "Failed to list tables");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn get_table_schema(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, table = %table, "GET schema");

    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };

    let table_perm = match permission_repo::resolve_table_permission(&state.pool, &current_user, &conn_id, &table).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    if !table_perm.can_read() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "No access to this table" }))).into_response();
    }

    let ds = match state.connection_manager.get_datasource(&conn_id).await {
        Some(ds) => ds,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Connection not found" }))).into_response(),
    };

    match ds.get_table_schema(&table).await {
        Ok(schema) => Json(serde_json::json!(schema)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// ============================================================
// Row CRUD (with permission checks)
// ============================================================

pub async fn list_rows(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table)): Path<(Uuid, String)>,
    Query(query): Query<RowsQuery>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, table = %table, "GET rows");

    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };

    let table_perm = match permission_repo::resolve_table_permission(&state.pool, &current_user, &conn_id, &table).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    if !table_perm.can_read() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "No access to this table" }))).into_response();
    }

    let ds = match state.connection_manager.get_datasource(&conn_id).await {
        Some(ds) => ds,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Connection not found" }))).into_response(),
    };

    match ds.list_rows(&table, &query).await {
        Ok(response) => Json(serde_json::json!(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn create_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table)): Path<(Uuid, String)>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, table = %table, "POST row");

    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };

    let table_perm = match permission_repo::resolve_table_permission(&state.pool, &current_user, &conn_id, &table).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    if !table_perm.can_write() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Write access required" }))).into_response();
    }

    let ds = match state.connection_manager.get_datasource(&conn_id).await {
        Some(ds) => ds,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Connection not found" }))).into_response(),
    };

    match ds.insert_row(&table, &data).await {
        Ok(row) => (StatusCode::CREATED, Json(row)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn get_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table, pk)): Path<(Uuid, String, String)>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };

    let table_perm = match permission_repo::resolve_table_permission(&state.pool, &current_user, &conn_id, &table).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    if !table_perm.can_read() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "No access" }))).into_response();
    }

    let ds = match state.connection_manager.get_datasource(&conn_id).await {
        Some(ds) => ds,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Connection not found" }))).into_response(),
    };

    match ds.get_row(&table, &pk).await {
        Ok(row) => Json(row).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table, pk)): Path<(Uuid, String, String)>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };

    let table_perm = match permission_repo::resolve_table_permission(&state.pool, &current_user, &conn_id, &table).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    if !table_perm.can_write() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Write access required" }))).into_response();
    }

    let ds = match state.connection_manager.get_datasource(&conn_id).await {
        Some(ds) => ds,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Connection not found" }))).into_response(),
    };

    match ds.update_row(&table, &pk, &data).await {
        Ok(row) => Json(row).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn delete_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table, pk)): Path<(Uuid, String, String)>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };

    let table_perm = match permission_repo::resolve_table_permission(&state.pool, &current_user, &conn_id, &table).await {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };
    if !table_perm.can_write() {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Write access required" }))).into_response();
    }

    let ds = match state.connection_manager.get_datasource(&conn_id).await {
        Some(ds) => ds,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Connection not found" }))).into_response(),
    };

    match ds.delete_row(&table, &pk).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
