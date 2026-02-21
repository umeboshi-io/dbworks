use axum::{
    Json,
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::domain::repository::UserRepository;
use crate::domain::user::AppUser;
use crate::infrastructure::auth::jwt::{Claims, extract_bearer_token};

/// Authenticate user from JWT, falling back to X-User-Id for backward compat.
pub async fn authenticate_user(
    user_repo: &dyn UserRepository,
    jwt_secret: &str,
    headers: &HeaderMap,
) -> Result<AppUser, StatusCode> {
    // Try JWT first
    if let Some(token) = extract_bearer_token(headers) {
        let claims = Claims::decode(&token, jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
        return user_repo
            .get(&user_id)
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

    user_repo
        .get(&user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

pub async fn get_current_user(
    user_repo: &dyn UserRepository,
    jwt_secret: &str,
    headers: &HeaderMap,
) -> Result<AppUser, StatusCode> {
    authenticate_user(user_repo, jwt_secret, headers).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_user(role: &str) -> AppUser {
        AppUser {
            id: Uuid::new_v4(),
            organization_id: None,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            role: role.to_string(),
            auth_provider: None,
            provider_id: None,
            avatar_url: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn super_admin_passes() {
        let user = make_user("super_admin");
        assert!(require_super_admin(&user).is_ok());
    }

    #[test]
    fn member_is_rejected() {
        let user = make_user("member");
        let err = require_super_admin(&user).unwrap_err();
        assert_eq!(err.0, StatusCode::FORBIDDEN);
    }

    #[test]
    fn admin_is_rejected() {
        let user = make_user("admin");
        assert!(require_super_admin(&user).is_err());
    }

    #[test]
    fn empty_role_is_rejected() {
        let user = make_user("");
        assert!(require_super_admin(&user).is_err());
    }
}
