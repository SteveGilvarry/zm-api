# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**zm_api** is a Rust-based REST API server (v3.0.0) for ZoneMinder surveillance system management. Built with Axum 0.8 web framework, SeaORM for database access, and supports WebRTC/MSE streaming.

## Build and Run Commands

```bash
# Build
cargo build                    # Development build
cargo build --release          # Release build

# Run server
cargo run                      # Run with default profile (dev)
APP_PROFILE=prod cargo run     # Run with specific profile

# Code quality
cargo fmt                      # Format code
cargo clippy --all-targets --all-features -- -D warnings   # Lint
```

## Testing

```bash
# Unit tests (no database required)
cargo test

# Run a single test
cargo test test_name -- --nocapture

# Integration tests (requires test database)
./scripts/test-db.sh start                    # Start MariaDB container on port 3307
./scripts/test-db.sh migrate                  # Run migrations
APP_PROFILE=test-db cargo test --test '*' -- --include-ignored
./scripts/test-db.sh stop                     # Stop database
```

### Test Database Commands
```bash
./scripts/test-db.sh start     # Start test database
./scripts/test-db.sh stop      # Stop test database
./scripts/test-db.sh reset     # Reset database (drops all data, reruns migrations)
./scripts/test-db.sh status    # Show database status
./scripts/test-db.sh cleanup   # Stop and remove all volumes
```

## Architecture

**Layered Architecture:**
```
Routes (src/routes/) → Handlers (src/handlers/) → Services (src/service/) → Repositories (src/repo/) → Entities (src/entity/)
```

**Key Directories:**
- `src/routes/` - Axum route definitions (43 modules)
- `src/handlers/` - HTTP request handlers
- `src/service/` - Business logic layer
- `src/repo/` - Database query abstraction
- `src/entity/` - SeaORM database models (auto-generated from ZoneMinder schema)
- `src/dto/request/` and `src/dto/response/` - Request/response types
- `src/configure/` - Configuration loading (base.toml + profile overrides)
- `src/migration/` - SeaORM migrations for custom tables
- `src/client/` - External service clients (database, email, HTTP, WebRTC)

**Shared State (`AppState`):**
- Database connection, email client, HTTP client, WebRTC signaling client, MSE stream manager
- Injected into handlers via Axum's state extraction

## Configuration

Configuration loads in order: `settings/base.toml` → `settings/{APP_PROFILE}.toml` → environment variables (prefix `APP_`).

**Profiles:** `dev`, `test`, `test-db`, `prod`

## Database

- **ORM:** SeaORM 1.0 with MySQL and PostgreSQL support
- **Entity Generation:** Entities are auto-generated from ZoneMinder's existing database schema using `sea-orm-cli`
- **Custom Tables:** Managed via migrations in `src/migration/`
- **Test Database:** MariaDB 11.4 on port 3307 (separate from production 3306)

## Error Handling

All errors use the `AppError` enum (`src/error/mod.rs`) which maps to HTTP responses. Use `AppResult<T>` as the return type for handlers.

## API Documentation

- Swagger UI: `/swagger-ui`
- OpenAPI spec: `/api-docs/openapi.json`
- DTOs use `utoipa::ToSchema` for automatic OpenAPI generation

## Adding New Endpoints

1. Define DTOs in `src/dto/request/` and `src/dto/response/`
2. Create handler in `src/handlers/`
3. Create service in `src/service/` (if business logic needed)
4. Create repository in `src/repo/` (if database access needed)
5. Create route in `src/routes/` and merge into `src/routes/mod.rs`
6. Add tests
