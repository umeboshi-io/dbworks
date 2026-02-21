use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::usecase::UsecaseError;

pub mod connection;
pub mod data;
pub mod group;
pub mod organization;
pub mod permission;
pub mod user;

/// Map a `UsecaseError` to an HTTP response.
pub fn into_response(err: UsecaseError) -> axum::response::Response {
    let (status, message) = match &err {
        UsecaseError::Unauthorized => (StatusCode::UNAUTHORIZED, err.to_string()),
        UsecaseError::Forbidden(_) => (StatusCode::FORBIDDEN, err.to_string()),
        UsecaseError::NotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
        UsecaseError::BadRequest(_) => (StatusCode::BAD_REQUEST, err.to_string()),
        UsecaseError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    };
    tracing::error!(error = %message, "Usecase error");
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}
