use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::presentation::middleware::get_current_user;
use crate::presentation::request::CreateUserRequest;
use crate::presentation::state::AppState;
use crate::usecase;

use super::into_response;

pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    tracing::info!(org_id = %org_id, name = %req.name, "POST /api/organizations/:org_id/users");
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::user::create_user(
        &*state.user_repo,
        &caller,
        &org_id,
        &req.name,
        &req.email,
        &req.role,
    )
    .await
    {
        Ok(user) => (StatusCode::CREATED, Json(serde_json::json!(user))).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_users(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    match usecase::user::list_users(&*state.user_repo, &org_id).await {
        Ok(users) => Json(serde_json::json!(users)).into_response(),
        Err(e) => into_response(e),
    }
}
