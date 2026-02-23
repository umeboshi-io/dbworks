use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::presentation::middleware::get_current_user;
use crate::presentation::request::CreateOrganizationRequest;
use crate::presentation::state::AppState;
use crate::usecase;

use super::into_response;

pub async fn create_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateOrganizationRequest>,
) -> impl IntoResponse {
    tracing::info!(name = %req.name, "POST /api/organizations");

    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::organization::create_organization(&*state.organization_repo, &req.name).await {
        Ok(org) => {
            // Auto-add creator as owner
            if let Err(e) = state
                .org_member_repo
                .add_member(&org.id, &caller.id, "owner")
                .await
            {
                tracing::error!(error = %e, "Failed to add creator as org owner");
            }
            (StatusCode::CREATED, Json(serde_json::json!(org))).into_response()
        }
        Err(e) => into_response(e),
    }
}

pub async fn list_organizations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
        Ok(u) => u,
        Err(status) => {
            return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
        }
    };

    match usecase::organization::list_organizations(&*state.organization_repo, &caller).await {
        Ok(orgs) => Json(serde_json::json!(orgs)).into_response(),
        Err(e) => into_response(e),
    }
}
