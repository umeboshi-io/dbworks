use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::presentation::middleware::get_current_user;
use crate::presentation::request::ConnectionRequest;
use crate::presentation::state::AppState;
use crate::usecase;

use super::into_response;

#[derive(Debug, Deserialize)]
pub struct ConnectionListParams {
    pub scope: Option<String>,
}

pub async fn create_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ConnectionRequest>,
) -> impl IntoResponse {
    let port = req.port.unwrap_or(5432);
    tracing::info!(name = %req.name, host = %req.host, port = port, database = %req.database, "POST /api/connections");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::connection::create_connection(
        &state.connection_manager,
        &caller,
        req.name,
        req.host,
        port,
        req.database,
        req.user,
        req.password,
    )
    .await
    {
        Ok(info) => {
            tracing::info!(connection_id = %info.id, "Connection created successfully");
            (StatusCode::CREATED, Json(serde_json::json!(info))).into_response()
        }
        Err(e) => into_response(e),
    }
}

pub async fn list_connections(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ConnectionListParams>,
) -> impl IntoResponse {
    tracing::debug!(scope = ?params.scope, "GET /api/connections");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::connection::list_connections(
        &state.connection_manager,
        &caller,
        params.scope.as_deref(),
    )
    .await
    {
        Ok(connections) => Json(serde_json::json!(connections)).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn delete_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, "DELETE /api/connections/:conn_id");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::connection::delete_connection(&state.connection_manager, &caller, &conn_id).await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => into_response(e),
    }
}
