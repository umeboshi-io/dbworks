use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::middleware::get_current_user;
use crate::presentation::request::RowsQuery;
use crate::presentation::state::AppState;
use crate::usecase;

use super::into_response;

// ============================================================
// Table Introspection
// ============================================================

pub async fn list_tables(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, "GET /api/connections/:conn_id/tables");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::data::list_tables(
        &*state.permission_repo,
        &state.connection_manager,
        &caller,
        &conn_id,
    )
    .await
    {
        Ok(tables) => Json(serde_json::json!(tables)).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn get_table_schema(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, table = %table, "GET schema");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::data::get_table_schema(
        &*state.permission_repo,
        &state.connection_manager,
        &caller,
        &conn_id,
        &table,
    )
    .await
    {
        Ok(schema) => Json(serde_json::json!(schema)).into_response(),
        Err(e) => into_response(e),
    }
}

// ============================================================
// Row CRUD
// ============================================================

pub async fn list_rows(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table)): Path<(Uuid, String)>,
    Query(query): Query<RowsQuery>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, table = %table, "GET rows");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::data::list_rows(
        &*state.permission_repo,
        &state.connection_manager,
        &caller,
        &conn_id,
        &table,
        &query,
    )
    .await
    {
        Ok(response) => Json(serde_json::json!(response)).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn create_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table)): Path<(Uuid, String)>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, table = %table, "POST row");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::data::create_row(
        &*state.permission_repo,
        &state.connection_manager,
        &caller,
        &conn_id,
        &table,
        &data,
    )
    .await
    {
        Ok(row) => (StatusCode::CREATED, Json(row)).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn get_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table, pk)): Path<(Uuid, String, String)>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::data::get_row(
        &*state.permission_repo,
        &state.connection_manager,
        &caller,
        &conn_id,
        &table,
        &pk,
    )
    .await
    {
        Ok(row) => Json(row).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn update_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table, pk)): Path<(Uuid, String, String)>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::data::update_row(
        &*state.permission_repo,
        &state.connection_manager,
        &caller,
        &conn_id,
        &table,
        &pk,
        &data,
    )
    .await
    {
        Ok(row) => Json(row).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn delete_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((conn_id, table, pk)): Path<(Uuid, String, String)>,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::data::delete_row(
        &*state.permission_repo,
        &state.connection_manager,
        &caller,
        &conn_id,
        &table,
        &pk,
    )
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => into_response(e),
    }
}
