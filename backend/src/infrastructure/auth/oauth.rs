use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient, reqwest::async_http_client,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::user::AppUser;
use crate::presentation::state::AppState;

use super::jwt::Claims;

// ============================================================
// OAuth2 Clients
// ============================================================

pub struct OAuthClients {
    pub google: Option<BasicClient>,
    pub github: Option<BasicClient>,
}

impl OAuthClients {
    pub fn from_env() -> Self {
        let google = Self::build_google();
        let github = Self::build_github();
        Self { google, github }
    }

    fn build_google() -> Option<BasicClient> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").ok()?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok()?;

        Some(
            BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
                Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
            )
            .set_redirect_uri(
                RedirectUrl::new(format!(
                    "{}/api/auth/google/callback",
                    std::env::var("BACKEND_URL")
                        .unwrap_or_else(|_| "http://localhost:3001".to_string())
                ))
                .unwrap(),
            ),
        )
    }

    fn build_github() -> Option<BasicClient> {
        let client_id = std::env::var("GITHUB_CLIENT_ID").ok()?;
        let client_secret = std::env::var("GITHUB_CLIENT_SECRET").ok()?;

        Some(
            BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
                Some(
                    TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                        .unwrap(),
                ),
            )
            .set_redirect_uri(
                RedirectUrl::new(format!(
                    "{}/api/auth/github/callback",
                    std::env::var("BACKEND_URL")
                        .unwrap_or_else(|_| "http://localhost:3001".to_string())
                ))
                .unwrap(),
            ),
        )
    }
}

// ============================================================
// OAuth profile types
// ============================================================

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    name: Option<String>,
    email: String,
    picture: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

// ============================================================
// Callback query
// ============================================================

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    #[allow(dead_code)]
    state: Option<String>,
}

// ============================================================
// Handlers
// ============================================================

/// GET /api/auth/google — redirect to Google consent screen
pub async fn google_login(State(state): State<AppState>) -> impl IntoResponse {
    let client = match &state.oauth_clients.google {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Google OAuth not configured" })),
            )
                .into_response();
        }
    };

    let (auth_url, _csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Redirect::temporary(auth_url.as_str()).into_response()
}

/// GET /api/auth/google/callback
pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthCallbackQuery>,
) -> impl IntoResponse {
    let client = match &state.oauth_clients.google {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Google OAuth not configured" })),
            )
                .into_response();
        }
    };

    // Exchange code for token
    let token = match client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "Google token exchange failed");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Token exchange failed" })),
            )
                .into_response();
        }
    };

    // Fetch Google user info
    let http_client = reqwest::Client::new();
    let user_info: GoogleUserInfo = match http_client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
    {
        Ok(res) => match res.json().await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse Google user info");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Failed to get user info" })),
                )
                    .into_response();
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch Google user info");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Failed to get user info" })),
            )
                .into_response();
        }
    };

    let name = user_info.name.unwrap_or_else(|| user_info.email.clone());

    // Find or create user
    let user = match find_or_create_user(
        &state.pool,
        "google",
        &user_info.sub,
        &name,
        &user_info.email,
        user_info.picture.as_deref(),
    )
    .await
    {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(error = %e, "Failed to find/create user");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "User creation failed" })),
            )
                .into_response();
        }
    };

    // Generate JWT
    let token = match Claims::generate_token(&user, &state.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate JWT");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "JWT generation failed" })),
            )
                .into_response();
        }
    };

    // Redirect to frontend with token
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Redirect::temporary(&format!("{}?token={}", frontend_url, token)).into_response()
}

/// GET /api/auth/github — redirect to GitHub consent screen
pub async fn github_login(State(state): State<AppState>) -> impl IntoResponse {
    let client = match &state.oauth_clients.github {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "GitHub OAuth not configured" })),
            )
                .into_response();
        }
    };

    let (auth_url, _csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    Redirect::temporary(auth_url.as_str()).into_response()
}

/// GET /api/auth/github/callback
pub async fn github_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthCallbackQuery>,
) -> impl IntoResponse {
    let client = match &state.oauth_clients.github {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "GitHub OAuth not configured" })),
            )
                .into_response();
        }
    };

    // Exchange code for token
    let token = match client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "GitHub token exchange failed");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Token exchange failed" })),
            )
                .into_response();
        }
    };

    let http_client = reqwest::Client::new();

    // Fetch GitHub user info
    let user_info: GitHubUserInfo = match http_client
        .get("https://api.github.com/user")
        .bearer_auth(token.access_token().secret())
        .header("User-Agent", "dbworks")
        .send()
        .await
    {
        Ok(res) => match res.json().await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse GitHub user info");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Failed to get user info" })),
                )
                    .into_response();
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch GitHub user info");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Failed to get user info" })),
            )
                .into_response();
        }
    };

    // Get primary email if not in profile
    let email = if let Some(email) = &user_info.email {
        email.clone()
    } else {
        // Fetch emails from GitHub API
        match http_client
            .get("https://api.github.com/user/emails")
            .bearer_auth(token.access_token().secret())
            .header("User-Agent", "dbworks")
            .send()
            .await
        {
            Ok(res) => {
                let emails: Vec<GitHubEmail> = res.json().await.unwrap_or_default();
                emails
                    .into_iter()
                    .find(|e| e.primary && e.verified)
                    .map(|e| e.email)
                    .unwrap_or_else(|| format!("{}@github.local", user_info.login))
            }
            Err(_) => format!("{}@github.local", user_info.login),
        }
    };

    let name = user_info.name.unwrap_or_else(|| user_info.login.clone());

    let provider_id = user_info.id.to_string();

    // Find or create user
    let user = match find_or_create_user(
        &state.pool,
        "github",
        &provider_id,
        &name,
        &email,
        user_info.avatar_url.as_deref(),
    )
    .await
    {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(error = %e, "Failed to find/create user");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "User creation failed" })),
            )
                .into_response();
        }
    };

    // Generate JWT
    let jwt = match Claims::generate_token(&user, &state.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate JWT");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "JWT generation failed" })),
            )
                .into_response();
        }
    };

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Redirect::temporary(&format!("{}?token={}", frontend_url, jwt)).into_response()
}

/// GET /api/auth/me — return current user from JWT
pub async fn me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let token = match super::jwt::extract_bearer_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "No token provided" })),
            )
                .into_response();
        }
    };

    let claims = match Claims::decode(&token, &state.jwt_secret) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Invalid JWT");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid token" })),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid token" })),
            )
                .into_response();
        }
    };

    match state.user_repo.get(&user_id).await {
        Ok(Some(user)) => Json(serde_json::json!(user)).into_response(),
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "User not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal error" })),
            )
                .into_response()
        }
    }
}

// ============================================================
// Helpers
// ============================================================

async fn find_or_create_user(
    pool: &PgPool,
    provider: &str,
    provider_id: &str,
    name: &str,
    email: &str,
    avatar_url: Option<&str>,
) -> anyhow::Result<AppUser> {
    // 1. Try to find by provider
    let existing = sqlx::query_as::<_, AppUser>(
        "SELECT * FROM app_users WHERE auth_provider = $1 AND provider_id = $2",
    )
    .bind(provider)
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    if let Some(mut user) = existing {
        // Update name/avatar if changed
        sqlx::query(
            "UPDATE app_users SET name = $1, avatar_url = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(name)
        .bind(avatar_url)
        .bind(user.id)
        .execute(pool)
        .await?;
        user.name = name.to_string();
        user.avatar_url = avatar_url.map(|s| s.to_string());
        return Ok(user);
    }

    // 2. Try to find by email — link account
    let by_email = sqlx::query_as::<_, AppUser>(
        "SELECT * FROM app_users WHERE email = $1 AND organization_id IS NULL",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    if let Some(user) = by_email {
        // Link OAuth provider to existing user
        sqlx::query(
            "UPDATE app_users SET auth_provider = $1, provider_id = $2, avatar_url = $3, updated_at = NOW() WHERE id = $4",
        )
        .bind(provider)
        .bind(provider_id)
        .bind(avatar_url)
        .bind(user.id)
        .execute(pool)
        .await?;
        return Ok(AppUser {
            auth_provider: Some(provider.to_string()),
            provider_id: Some(provider_id.to_string()),
            avatar_url: avatar_url.map(|s| s.to_string()),
            ..user
        });
    }

    // 3. Create new user (no organization)
    let user = sqlx::query_as::<_, AppUser>(
        r#"INSERT INTO app_users (name, email, role, auth_provider, provider_id, avatar_url)
           VALUES ($1, $2, 'member', $3, $4, $5)
           RETURNING *"#,
    )
    .bind(name)
    .bind(email)
    .bind(provider)
    .bind(provider_id)
    .bind(avatar_url)
    .fetch_one(pool)
    .await?;

    tracing::info!(user_id = %user.id, provider = provider, email = email, "Created new user via OAuth");
    Ok(user)
}
