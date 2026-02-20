use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::request::CreateUserRequest;
use crate::infrastructure::database::user_repo;
use crate::presentation::middleware::{get_current_user, require_super_admin};
use crate::presentation::state::AppState;

pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    tracing::info!(org_id = %org_id, name = %req.name, "POST /api/organizations/:org_id/users");
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match user_repo::create_user(&state.pool, &org_id, &req).await {
        Ok(user) => (StatusCode::CREATED, Json(serde_json::json!(user))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user");
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn list_users(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    match user_repo::list_users_by_org(&state.pool, &org_id).await {
        Ok(users) => Json(serde_json::json!(users)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list users");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}
