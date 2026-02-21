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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn response_status(err: UsecaseError) -> StatusCode {
        let resp = into_response(err);
        resp.status()
    }

    async fn response_body(err: UsecaseError) -> serde_json::Value {
        let resp = into_response(err);
        let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn unauthorized_maps_to_401() {
        let status = response_status(UsecaseError::Unauthorized).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn forbidden_maps_to_403() {
        let status = response_status(UsecaseError::Forbidden("denied".into())).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn not_found_maps_to_404() {
        let status = response_status(UsecaseError::NotFound("gone".into())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn bad_request_maps_to_400() {
        let status = response_status(UsecaseError::BadRequest("bad".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn internal_maps_to_500() {
        let status = response_status(UsecaseError::Internal("oops".into())).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn response_body_contains_error_field() {
        let body = response_body(UsecaseError::BadRequest("invalid input".into())).await;
        assert_eq!(body["error"], "invalid input");
    }

    #[tokio::test]
    async fn unauthorized_body_contains_error_field() {
        let body = response_body(UsecaseError::Unauthorized).await;
        assert_eq!(body["error"], "Unauthorized");
    }
}
