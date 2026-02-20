---
description: How to add new backend API endpoints in the Rust/Axum backend
---

# Adding Backend API Endpoints

## File Structure

```
backend/src/
├── main.rs          # Router setup — register new routes here
├── handler.rs       # HTTP handler functions
├── models.rs        # Request/response structs (serde)
├── connection.rs    # ConnectionManager (live DB pools)
├── repository.rs    # App DB queries (sqlx) [planned]
├── crypto.rs        # AES encryption utils [planned]
└── datasource/
    ├── mod.rs        # DataSource trait
    └── postgres.rs   # PostgreSQL DataSource impl
```

## Steps to Add a New Endpoint

### 1. Define models in `models.rs`

```rust
#[derive(Debug, Deserialize)]
pub struct MyRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MyResponse {
    pub id: Uuid,
    pub name: String,
}
```

### 2. Add handler in `handler.rs`

```rust
pub async fn my_handler(
    State(state): State<AppState>,
    Json(req): Json<MyRequest>,
) -> impl IntoResponse {
    // Implementation
    (StatusCode::CREATED, Json(serde_json::json!(response))).into_response()
}
```

### 3. Register route in `main.rs`

```rust
let app = Router::new()
    .route("/api/my-endpoint", post(handler::my_handler))
    // ...existing routes
```

## AppState

Currently: `Arc<ConnectionManager>`

Planned: A struct with `ConnectionManager`, `PgPool` (app DB), and encryption key.

## Error Response Pattern

All handlers follow this error pattern:

```rust
Err(e) => {
    tracing::error!(error = %e, "Description");
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": e.to_string() })),
    ).into_response()
}
```

## DataSource Trait

To add operations on user-connected databases, extend the `DataSource` trait in `datasource/mod.rs`:

```rust
#[async_trait]
pub trait DataSource: Send + Sync {
    async fn my_new_operation(&self, ...) -> anyhow::Result<...>;
}
```

Then implement it in `datasource/postgres.rs`.
