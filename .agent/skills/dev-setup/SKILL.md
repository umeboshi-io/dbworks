---
description: How to set up and run the DBWorks development environment
---

# Development Environment Setup

## Prerequisites

- Rust (latest stable)
- Node.js 20.19+ or 22.12+
- Docker & Docker Compose

## Quick Start

// turbo-all

1. Start all services (PostgreSQL + backend + frontend):

```bash
make dev
```

2. Or start services individually:

```bash
make up          # PostgreSQL only
make backend     # Backend on port 3001
make frontend    # Frontend on port 5173
```

## Database Reset

When `db/init.sql` changes, the Docker volume must be recreated:

```bash
make down
docker volume rm dbworks_pgdata
make up
```

## Environment Variables

For connection persistence (when enabled), start the backend with:

```bash
ENCRYPTION_KEY=$(openssl rand -base64 32) \
DATABASE_URL=postgres://dbworks:dbworks@localhost:5432/dbworks_dev \
cargo run
```

## Build & Lint

```bash
make build    # Build backend (release) + frontend (production)
make lint     # Run clippy + eslint
make clean    # Remove build artifacts
```
