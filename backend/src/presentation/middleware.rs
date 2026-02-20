use axum::{Json, http::{HeaderMap, StatusCode}};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::user::AppUser;
use crate::infrastructure::auth::jwt::{Claims, extract_bearer_token};
use crate::infrastructure::database::user_repo;

/// Authenticate user from JWT, falling back to X-User-Id for backward compat.
pub async fn authenticate_user(
    pool: &PgPool,
    jwt_secret: &str,
    headers: &HeaderMap,
) -> Result<AppUser, StatusCode> {
    // Try JWT first
    if let Some(token) = extract_bearer_token(headers) {
        let claims = Claims::decode(&token, jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
        return user_repo::get_user(pool, &user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED);
    }

    // Fallback: X-User-Id header (for dev/testing)
    let user_id_str = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("00000000-0000-0000-0000-000000000001");

    let user_id = Uuid::parse_str(user_id_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    user_repo::get_user(pool, &user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

pub async fn get_current_user(pool: &PgPool, jwt_secret: &str, headers: &HeaderMap) -> Result<AppUser, StatusCode> {
    authenticate_user(pool, jwt_secret, headers).await
}

pub fn require_super_admin(user: &AppUser) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if user.role != "super_admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "SuperAdmin role required" })),
        ));
    }
    Ok(())
}
