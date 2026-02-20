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
│   └── src/
│       ├── main.rs           # Entrypoint, router setup
│       ├── domain/           # Domain entities
│       ├── dto.rs            # Request/response DTOs
│       ├── infrastructure/   # Auth, crypto, database repos, datasource
│       └── presentation/     # Handlers, middleware, routes, state
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

### Environment Variables (Backend)

| Variable         | Description                    | Required                          |
| ---------------- | ------------------------------ | --------------------------------- |
| `DATABASE_URL`   | App database connection string | Yes (when persistence is enabled) |
| `ENCRYPTION_KEY` | Base64-encoded 32-byte AES key | Yes (when persistence is enabled) |

## Key Design Decisions

1. **DDD Architecture** — Domain, Infrastructure, Presentation layers for maintainability
2. **DataSource trait** (`infrastructure/datasource/mod.rs`) — Abstracts DB operations for future multi-DB support (MySQL, etc.)
3. **ConnectionManager** — Manages live connection pools; backed by DB persistence
4. **AES-GCM encryption** — Passwords encrypted at rest, key from environment variable
5. **Dual permission model** — Both user-level and group-level permissions supported; user permissions take priority over group permissions
