use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::middleware::get_current_user;
use crate::presentation::request::{AddGroupMemberRequest, CreateGroupRequest};
use crate::presentation::state::AppState;
use crate::usecase;

use super::into_response;

pub async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    tracing::info!(org_id = %org_id, name = %req.name, "POST /api/organizations/:org_id/groups");
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::group::create_group(
        &*state.group_repo,
        &caller,
        &org_id,
        &req.name,
        req.description.as_deref(),
    )
    .await
    {
        Ok(group) => (StatusCode::CREATED, Json(serde_json::json!(group))).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_groups(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    match usecase::group::list_groups(&*state.group_repo, &org_id).await {
        Ok(groups) => Json(serde_json::json!(groups)).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn add_group_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
    Json(req): Json<AddGroupMemberRequest>,
) -> impl IntoResponse {
    tracing::info!(group_id = %group_id, user_id = %req.user_id, "POST /api/groups/:group_id/members");
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::group::add_group_member(&*state.group_repo, &caller, &group_id, &req.user_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn remove_group_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    tracing::info!(group_id = %group_id, user_id = %user_id, "DELETE /api/groups/:group_id/members/:user_id");
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::group::remove_group_member(&*state.group_repo, &caller, &group_id, &user_id)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Member not found" })),
        )
            .into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_group_members(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    match usecase::group::list_group_members(&*state.group_repo, &group_id).await {
        Ok(members) => Json(serde_json::json!(members)).into_response(),
        Err(e) => into_response(e),
    }
}
