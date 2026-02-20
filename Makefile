.PHONY: help up down db db-reset backend frontend dev build clean lint

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# ---------------------------------------------------------------------------
# Docker / DB
# ---------------------------------------------------------------------------
up: ## Start PostgreSQL (docker compose up -d)
	docker compose up -d

down: ## Stop PostgreSQL (docker compose down)
	docker compose down

db: up ## Connect to PostgreSQL via psql
	docker compose exec postgres psql -U dbworks -d dbworks_dev

db-reset: down ## Reset database (destroy volume and re-create)
	docker volume rm dbworks_pgdata || true
	$(MAKE) up

# ---------------------------------------------------------------------------
# Backend (Rust / Axum)
# ---------------------------------------------------------------------------
backend: ## Run backend dev server (cargo run)
	cd backend && cargo run

backend-watch: ## Run backend with auto-reload (cargo watch)
	cd backend && cargo watch -x run

backend-build: ## Build backend in release mode
	cd backend && cargo build --release

backend-test: ## Run backend tests
	cd backend && cargo test

backend-lint: ## Run clippy on backend
	cd backend && cargo clippy -- -D warnings

backend-fmt: ## Format backend code
	cd backend && cargo fmt

# ---------------------------------------------------------------------------
# Frontend (React / Vite / TypeScript)
# ---------------------------------------------------------------------------
frontend: ## Run frontend dev server (vite)
	cd frontend && npm run dev

frontend-build: ## Build frontend for production
	cd frontend && npm run build

frontend-preview: ## Preview production build
	cd frontend && npm run preview

frontend-lint: ## Lint frontend code
	cd frontend && npm run lint

frontend-install: ## Install frontend dependencies
	cd frontend && npm install

# ---------------------------------------------------------------------------
# Combined
# ---------------------------------------------------------------------------
dev: up ## Start all services (DB + backend + frontend)
	@echo "Starting backend and frontend..."
	@trap 'kill 0' EXIT; \
		(cd backend && cargo run) & \
		(cd frontend && npm run dev) & \
		wait

build: backend-build frontend-build ## Build both backend and frontend

lint: backend-lint frontend-lint ## Lint both backend and frontend

clean: ## Clean build artifacts
	cd backend && cargo clean
	rm -rf frontend/dist
