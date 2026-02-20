use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};

use crate::dto::CreateOrganizationRequest;
use crate::infrastructure::database::organization_repo;
use crate::presentation::state::AppState;

pub async fn create_organization(
    State(state): State<AppState>,
    Json(req): Json<CreateOrganizationRequest>,
) -> impl IntoResponse {
    tracing::info!(name = %req.name, "POST /api/organizations");
    match organization_repo::create_organization(&state.pool, &req).await {
        Ok(org) => (StatusCode::CREATED, Json(serde_json::json!(org))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create organization");
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}

pub async fn list_organizations(State(state): State<AppState>) -> impl IntoResponse {
    match organization_repo::list_organizations(&state.pool).await {
        Ok(orgs) => Json(serde_json::json!(orgs)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list organizations");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response()
        }
    }
}
