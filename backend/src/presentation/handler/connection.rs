use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::presentation::middleware::{get_current_user, require_super_admin};
use crate::presentation::request::ConnectionRequest;
use crate::presentation::state::AppState;

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

    let current_user = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await
    {
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

pub async fn list_connections(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ConnectionListParams>,
) -> impl IntoResponse {
    tracing::debug!(scope = ?params.scope, "GET /api/connections");

    let current_user = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await
    {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    let connections = match params.scope.as_deref() {
        Some("personal") => {
            state
                .connection_manager
                .list_personal(&current_user.id)
                .await
        }
        Some(s) if s.starts_with("org:") => {
            let org_id_str = &s[4..];
            match Uuid::parse_str(org_id_str) {
                Ok(org_id) => state.connection_manager.list_by_org(&org_id).await,
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": "Invalid org ID in scope" })),
                    )
                        .into_response();
                }
            }
        }
        _ => {
            // Default: return all connections accessible to this user
            state.connection_manager.list().await
        }
    };

    Json(serde_json::json!(connections)).into_response()
}

pub async fn delete_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(conn_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!(connection_id = %conn_id, "DELETE /api/connections/:conn_id");

    let current_user = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await
    {
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
