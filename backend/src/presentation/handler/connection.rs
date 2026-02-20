use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::request::ConnectionRequest;
use crate::presentation::middleware::{get_current_user, require_super_admin};
use crate::presentation::state::AppState;

pub async fn create_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ConnectionRequest>,
) -> impl IntoResponse {
    let port = req.port.unwrap_or(5432);
    tracing::info!(name = %req.name, host = %req.host, port = port, database = %req.database, "POST /api/connections");

    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    // Determine ownership: org user → org connection (requires super_admin), no org → personal connection
    let (organization_id, owner_user_id) = if let Some(org_id) = current_user.organization_id {
        // User belongs to an org → org-level connection, requires super_admin
        if let Err(resp) = require_super_admin(&current_user) {
            return resp.into_response();
        }
        (Some(org_id), None)
    } else {
        // No org → personal connection, any authenticated user can create
        (None, Some(current_user.id))
    };

    match state
        .connection_manager
        .add_postgres(
            req.name,
            req.host,
            port,
            req.database,
            req.user,
            req.password,
            organization_id,
            owner_user_id,
        )
        .await
    {
        Ok(info) => {
            tracing::info!(connection_id = %info.id, "Connection created successfully");
            (StatusCode::CREATED, Json(serde_json::json!(info))).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create connection");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

pub async fn list_connections(State(state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("GET /api/connections");
    let connections = state.connection_manager.list().await;
    Json(serde_json::json!(connections))
}

pub async fn delete_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, "DELETE /api/connections/:conn_id");

    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    if state.connection_manager.remove(&conn_id).await {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Connection not found" })),
        )
            .into_response()
    }
}
