---
description: How to add new backend API endpoints in the Rust/Axum backend
---

# Adding Backend API Endpoints

## Layer Overview

The backend follows DDD with 4 layers:

```
domain/          → Entities + repository traits (pure, no deps)
usecase/         → Business logic (1 public function per file)
infrastructure/  → DB repos, auth, crypto, datasource implementations
presentation/    → HTTP handlers, middleware, routes, state
```

## Steps to Add a New Endpoint

### 1. Define domain entity (if new)

Create `domain/my_entity.rs` with serde + sqlx derives:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MyEntity {
    pub id: Uuid,
    pub name: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

### 2. Define repository trait (if new)

Create `domain/repository/my_entity.rs`:

```rust
#[async_trait]
pub trait MyEntityRepository: Send + Sync {
    async fn create(&self, name: &str) -> anyhow::Result<MyEntity>;
    async fn list(&self) -> anyhow::Result<Vec<MyEntity>>;
}
```

Re-export in `domain/repository/mod.rs`.

### 3. Add usecase function

Create `usecase/my_domain/my_action.rs` (1 function per file):

```rust
use crate::domain::repository::MyEntityRepository;
use crate::usecase::UsecaseError;

pub async fn create_my_entity(
    repo: &dyn MyEntityRepository,
    name: &str,
) -> Result<MyEntity, UsecaseError> {
    repo.create(name)
        .await
        .map_err(|e| UsecaseError::BadRequest(e.to_string()))
}
```

Re-export in `usecase/my_domain/mod.rs` and register in `usecase/mod.rs`.

### 4. Add handler

Create or update `presentation/handler/my_domain.rs`:

```rust
pub async fn create_my_entity(
    State(state): State<AppState>,
    Json(req): Json<CreateMyEntityRequest>,
) -> impl IntoResponse {
    match usecase::my_domain::create_my_entity(&*state.my_entity_repo, &req.name).await {
        Ok(entity) => (StatusCode::CREATED, Json(serde_json::json!(entity))).into_response(),
        Err(e) => into_response(e),
    }
}
```

### 5. Add request DTO

Add to `presentation/request.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct CreateMyEntityRequest {
    pub name: String,
}
```

### 6. Register route

In `presentation/routes.rs`:

```rust
.route("/api/my-entities", post(my_domain::create_my_entity))
```

### 7. Add handler integration test

In `tests/presentation/my_domain_handler_test.rs`:

```rust
#[tokio::test]
#[serial]
async fn create_my_entity_returns_201() {
    let pool = common::setup_test_db().await;
    let app = build_test_app(pool);

    let body = serde_json::json!({ "name": "Test" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/my-entities")
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);
}
```

## Error Handling

Usecase functions return `Result<T, UsecaseError>`. The `into_response` function in `handler/mod.rs` maps:

| UsecaseError | HTTP Status |
| ------------ | ----------- |
| Unauthorized | 401         |
| Forbidden    | 403         |
| NotFound     | 404         |
| BadRequest   | 400         |
| Internal     | 500         |

## Auth Pattern

Handlers that require authentication use:

```rust
let caller = match get_current_user(&*state.user_repo, &state.jwt_secret, &headers).await {
    Ok(u) => u,
    Err(status) => {
        return (status, Json(serde_json::json!({ "error": "Unauthorized" }))).into_response();
    }
};
```

For super_admin-only operations, the usecase calls `require_super_admin(&caller)?`.

## Testing

- **Unit tests**: Inline `#[cfg(test)]` modules in each file
- **Usecase integration tests**: `tests/usecase/` — direct usecase function calls with real DB
- **Handler integration tests**: `tests/presentation/` — `axum::Router::oneshot()` with real DB
- **Test DB setup**: `tests/common/mod.rs` → `setup_test_db()` (connects, migrates, truncates)
- **Auth in tests**: Use `X-User-Id` header (middleware fallback), no JWT needed
