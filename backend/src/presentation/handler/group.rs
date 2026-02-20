use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::request::{AddGroupMemberRequest, CreateGroupRequest};
use crate::infrastructure::database::group_repo;
use crate::presentation::middleware::{get_current_user, require_super_admin};
use crate::presentation::state::AppState;

pub async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    tracing::info!(org_id = %org_id, name = %req.name, "POST /api/organizations/:org_id/groups");
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match group_repo::create_group(&state.pool, &org_id, &req).await {
        Ok(group) => (StatusCode::CREATED, Json(serde_json::json!(group))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create group");
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn list_groups(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    match group_repo::list_groups_by_org(&state.pool, &org_id).await {
        Ok(groups) => Json(serde_json::json!(groups)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list groups");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn add_group_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
    Json(req): Json<AddGroupMemberRequest>,
) -> impl IntoResponse {
    tracing::info!(group_id = %group_id, user_id = %req.user_id, "POST /api/groups/:group_id/members");
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match group_repo::add_group_member(&state.pool, &group_id, &req.user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to add group member");
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn remove_group_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    tracing::info!(group_id = %group_id, user_id = %user_id, "DELETE /api/groups/:group_id/members/:user_id");
    let current_user = match get_current_user(&state.pool, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response(),
    };
    if let Err(resp) = require_super_admin(&current_user) {
        return resp.into_response();
    }

    match group_repo::remove_group_member(&state.pool, &group_id, &user_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Member not found" }))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to remove group member");
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn list_group_members(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    match group_repo::list_group_members(&state.pool, &group_id).await {
        Ok(members) => Json(serde_json::json!(members)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list group members");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}
