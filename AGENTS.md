# DBWorks

A database schema-driven CRUD manager. Connect to any PostgreSQL database, introspect its schema, and perform CRUD operations through a modern web UI.

## Architecture

- **Backend**: Rust (Axum) — `backend/`
- **Frontend**: React + TypeScript (Vite) — `frontend/`
- **Database**: PostgreSQL 16 via Docker Compose
- **Build**: Makefile at project root

## Project Structure

```
dbworks/
├── Makefile              # Dev commands (make dev, make build, etc.)
├── docker-compose.yml    # PostgreSQL for dev
├── db/init.sql           # Schema + seed data
├── backend/              # Rust / Axum API server (port 3001)
│   ├── migrations/       # sqlx migrations
│   └── src/
│       ├── main.rs
│       ├── domain/           # Entities + repository traits
│       │   ├── organization.rs, user.rs, group.rs, connection.rs, permission.rs
│       │   └── repository/   # Trait definitions (1 trait per file)
│       ├── usecase/          # Business logic (1 function per file)
│       │   ├── mod.rs, error.rs
│       │   ├── organization/ # create_organization.rs, list_organizations.rs
│       │   ├── user/         # create_user.rs, list_users.rs
│       │   ├── group/        # create_group.rs, list_groups.rs, add_group_member.rs, ...
│       │   ├── connection/   # create_connection.rs, list_connections.rs, delete_connection.rs
│       │   ├── permission/   # grant_*/revoke_*/list_* (12 files)
│       │   └── data/         # list_tables.rs, get_table_schema.rs, list_rows.rs, ... (7 files + helpers)
│       ├── infrastructure/   # Auth (OAuth, JWT), crypto, database repos, datasource
│       │   ├── auth/         # oauth.rs, jwt.rs
│       │   ├── crypto.rs     # AES-256-GCM encryption
│       │   ├── database/     # Pg*Repository implementations
│       │   └── datasource/   # DataSource trait + PostgresDataSource
│       └── presentation/     # HTTP layer
│           ├── handler/      # 1 file per domain (organization, user, group, connection, permission, data)
│           ├── middleware.rs  # Auth middleware (JWT + X-User-Id fallback)
│           ├── request.rs    # Request DTOs
│           ├── routes.rs     # Route registration
│           └── state.rs      # AppState + ConnectionManager
├── backend/tests/            # Integration tests
│   ├── common/mod.rs         # Test DB setup (setup_test_db)
│   ├── usecase/              # Usecase integration tests (39 tests)
│   └── presentation/         # Handler integration tests (25 tests, axum oneshot)
└── frontend/             # React + Vite + TypeScript (port 5173)
    └── src/
        ├── App.tsx            # Main app with sidebar nav
        ├── types.ts           # Shared TypeScript types
        ├── api/client.ts      # API client
        ├── pages/             # ConnectionPage, TablePage, LoginPage
        └── components/        # DataTable, DynamicForm
```

## Data Model

The application uses a multi-tenant permission model:

- **Organization** — Top-level tenant, owns connections
- **User** — Member of an organization, role: `super_admin` or `member`
- **Group** — Organizational unit within an org, users can belong to multiple groups
- **Connection** — Saved database connection (password AES-256-GCM encrypted)
- **Permissions** — Granted at both user and group level (Connection + Table granularity)

### Permission Resolution

```
1. SuperAdmin → full access (always highest priority)
2. User-level permission exists → apply it
   - "none" → explicit deny (overrides group permissions)
   - "read"/"write"/"admin" → apply as-is
3. No user-level permission → apply max of group permissions
4. Neither exists → deny
```

## Development

```bash
make dev       # Start DB + backend + frontend
make up        # Start PostgreSQL only
make backend   # Run backend only
make frontend  # Run frontend only
make build     # Build both for production
make lint      # Lint both
make clean     # Clean build artifacts
```

### Testing

```bash
cargo test                          # All tests (unit + integration)
cargo test --test usecase_tests     # Usecase integration tests only
cargo test --test presentation_tests # Handler integration tests only
```

### Environment Variables (Backend)

| Variable         | Description                    | Required                          |
| ---------------- | ------------------------------ | --------------------------------- |
| `DATABASE_URL`   | App database connection string | Yes (when persistence is enabled) |
| `ENCRYPTION_KEY` | Base64-encoded 32-byte AES key | Yes (when persistence is enabled) |

## Key Design Decisions

1. **DDD Architecture** — Domain, Usecase, Infrastructure, Presentation layers
2. **1-Function-1-File usecase** — Each public function in its own file, re-exported via `mod.rs`
3. **DataSource trait** (`infrastructure/datasource/mod.rs`) — Abstracts DB operations for future multi-DB support
4. **ConnectionManager** — Manages live connection pools; backed by DB persistence
5. **AES-GCM encryption** — Passwords encrypted at rest, key from environment variable
6. **Dual permission model** — Both user-level and group-level permissions; user permissions take priority
7. **Handler integration tests** — Use `axum::Router::oneshot()` with real test DB, `X-User-Id` header for auth
