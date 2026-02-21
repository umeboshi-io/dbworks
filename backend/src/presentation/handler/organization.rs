use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::presentation::request::CreateOrganizationRequest;
use crate::presentation::state::AppState;
use crate::usecase;

use super::into_response;

pub async fn create_organization(
    State(state): State<AppState>,
    Json(req): Json<CreateOrganizationRequest>,
) -> impl IntoResponse {
    tracing::info!(name = %req.name, "POST /api/organizations");
    match usecase::organization::create_organization(&*state.organization_repo, &req.name).await {
        Ok(org) => (StatusCode::CREATED, Json(serde_json::json!(org))).into_response(),
        Err(e) => into_response(e),
    }
}

pub async fn list_organizations(State(state): State<AppState>) -> impl IntoResponse {
    match usecase::organization::list_organizations(&*state.organization_repo).await {
        Ok(orgs) => Json(serde_json::json!(orgs)).into_response(),
        Err(e) => into_response(e),
    }
}
