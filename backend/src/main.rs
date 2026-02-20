mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use infrastructure::auth::oauth::OAuthClients;
use infrastructure::crypto::Encryptor;
use presentation::routes::create_router;
use presentation::state::{AppState, AppStateInner, ConnectionManager};

#[tokio::main]
async fn main() {
    // Load .env if present
    let _ = dotenvy::dotenv();

    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting DBWorks backend...");

    // Connect to app database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dbworks:dbworks@localhost:5432/dbworks_dev".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to app database");
    tracing::info!("Connected to app database");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    tracing::info!("Database migrations applied");

    // Initialize encryptor
    let encryptor = match Encryptor::from_env() {
        Ok(e) => {
            tracing::info!("Encryption key loaded");
            Some(e)
        }
        Err(e) => {
            tracing::warn!(
                "Encryption not configured: {}. Connections will not be persisted.",
                e
            );
            None
        }
    };

    // Create connection manager with DB persistence
    let connection_manager = ConnectionManager::new(Some(pool.clone()), encryptor);

    // Load saved connections from DB
    if let Err(e) = connection_manager.load_saved_connections().await {
        tracing::error!(error = %e, "Failed to load saved connections");
    }

    // Initialize OAuth clients
    let oauth_clients = OAuthClients::from_env();
    if oauth_clients.google.is_some() {
        tracing::info!("Google OAuth configured");
    } else {
        tracing::warn!(
            "Google OAuth not configured (GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET missing)"
        );
    }
    if oauth_clients.github.is_some() {
        tracing::info!("GitHub OAuth configured");
    } else {
        tracing::warn!(
            "GitHub OAuth not configured (GITHUB_CLIENT_ID / GITHUB_CLIENT_SECRET missing)"
        );
    }

    // JWT secret
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET not set, using default (insecure for production!)");
        "dbworks-dev-secret-change-me".to_string()
    });

    let state: AppState = Arc::new(AppStateInner {
        connection_manager,
        pool,
        oauth_clients,
        jwt_secret,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router().layer(cors).with_state(state);

    tracing::info!("🚀 DBWorks backend listening on http://localhost:3001");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
