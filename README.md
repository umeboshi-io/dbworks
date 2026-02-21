# 🗄️ DBWorks

> A schema-driven database manager — connect to any PostgreSQL database, introspect its schema, and perform CRUD operations through a modern web UI.

[![CI](https://github.com/umeboshi-io/dbworks/actions/workflows/ci.yml/badge.svg)](https://github.com/umeboshi-io/dbworks/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Backend-Rust%20%2F%20Axum-orange?logo=rust)](backend/)
[![React](https://img.shields.io/badge/Frontend-React%20%2F%20TypeScript-blue?logo=react)](frontend/)
[![PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL%2018-336791?logo=postgresql)](docker-compose.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

---

## ✨ Features

- **Connect** to PostgreSQL databases with encrypted credentials (AES-256-GCM)
- **Introspect** schemas — browse tables, columns, and data types
- **CRUD operations** — create, read, update, and delete rows through a clean UI
- **Multi-tenant permissions** — organization, user, and group-level access control
- **OAuth login** — Google & GitHub authentication support
- **DDD architecture** — clean separation of Domain, Usecase, Infrastructure, and Presentation layers

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────┐
│                   Frontend                       │
│            React + TypeScript + Vite              │
│                  (port 5173)                     │
└──────────────────┬───────────────────────────────┘
                   │ REST API
┌──────────────────▼───────────────────────────────┐
│                   Backend                        │
│              Rust + Axum (port 3001)              │
│                                                  │
│  ┌─────────┐ ┌─────────┐ ┌──────────────────┐   │
│  │ Domain  │ │ Usecase  │ │  Presentation    │   │
│  │         │ │         │  │  (Handlers, MW)  │   │
│  └─────────┘ └─────────┘ └──────────────────┘   │
│  ┌──────────────────────────────────────────┐    │
│  │          Infrastructure                   │    │
│  │  Auth · Crypto · Repos · DataSource       │    │
│  └──────────────────────────────────────────┘    │
└──────────┬─────────────────────┬─────────────────┘
           │ App DB              │ User DBs
    ┌──────▼──────┐       ┌──────▼──────┐
    │  PostgreSQL  │       │  Any PG DB  │
    │  (App Data)  │       │ (Connected) │
    └─────────────┘       └─────────────┘
```

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [Docker](https://www.docker.com/) & Docker Compose

### 1. Clone and start

```bash
git clone https://github.com/umeboshi-io/dbworks.git
cd dbworks
make dev
```

This starts PostgreSQL, the backend (`:3001`), and the frontend (`:5173`).

### 2. Open the app

Visit [http://localhost:5173](http://localhost:5173) in your browser.

## 📖 Usage

### Available Commands

| Command                 | Description                                  |
| ----------------------- | -------------------------------------------- |
| `make dev`              | Start all services (DB + backend + frontend) |
| `make up` / `make down` | Start / stop PostgreSQL                      |
| `make backend`          | Run backend only                             |
| `make frontend`         | Run frontend only                            |
| `make build`            | Build both for production                    |
| `make lint`             | Lint both backend and frontend               |
| `make backend-test`     | Run backend tests                            |
| `make db`               | Connect to PostgreSQL via psql               |
| `make db-reset`         | Reset database (destroy volume)              |
| `make clean`            | Clean build artifacts                        |

### Environment Variables

| Variable               | Description                    | Default                                                 |
| ---------------------- | ------------------------------ | ------------------------------------------------------- |
| `DATABASE_URL`         | App database connection string | `postgres://dbworks:dbworks@localhost:5432/dbworks_dev` |
| `ENCRYPTION_KEY`       | Base64-encoded 32-byte AES key | _(optional for dev)_                                    |
| `JWT_SECRET`           | Secret for JWT token signing   | `dbworks-dev-secret-change-me`                          |
| `GOOGLE_CLIENT_ID`     | Google OAuth client ID         | _(optional)_                                            |
| `GOOGLE_CLIENT_SECRET` | Google OAuth client secret     | _(optional)_                                            |
| `GITHUB_CLIENT_ID`     | GitHub OAuth client ID         | _(optional)_                                            |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth client secret     | _(optional)_                                            |

## 🔒 Permission Model

DBWorks uses a multi-tenant permission system:

```
Organization
 ├── Users (super_admin | member)
 ├── Groups (collections of users)
 └── Connections (saved database connections)
      ├── User Permissions (connection-level & table-level)
      └── Group Permissions (connection-level & table-level)
```

**Resolution order:**

1. **SuperAdmin** → full access
2. **User-level** permission → apply (`none` = explicit deny)
3. **Group-level** permission → apply max across groups
4. **No permission** → deny

## 🧪 Testing

```bash
# All tests
make backend-test

# Specific test suites
cd backend
cargo test --test presentation_tests   # Handler integration tests (25 tests)
cargo test --test usecase_tests        # Usecase integration tests (39 tests)
cargo test --lib                       # Unit tests
```

> **Note:** Integration tests require a running PostgreSQL instance. Set `TEST_DATABASE_URL` or use the default (`dbworks_test` database).

## 📁 Project Structure

```
dbworks/
├── backend/                # Rust / Axum API server
│   ├── migrations/         # SQL migrations (sqlx)
│   └── src/
│       ├── domain/         # Entities + repository traits
│       ├── usecase/        # Business logic (1 function per file)
│       ├── infrastructure/ # Auth, crypto, DB repos, datasource
│       └── presentation/   # Handlers, middleware, routes
├── frontend/               # React + Vite + TypeScript
│   └── src/
│       ├── api/            # API client
│       ├── pages/          # ConnectionPage, TablePage, LoginPage
│       └── components/     # DataTable, DynamicForm
├── Makefile                # Dev commands
└── docker-compose.yml      # PostgreSQL for development
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## � Contributors

<a href="https://github.com/umeboshi-io/dbworks/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=umeboshi-io/dbworks" />
</a>

## �📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
