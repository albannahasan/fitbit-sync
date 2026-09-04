# Meal Logger MCP Backend

A Rust backend for logging user-confirmed meal nutrition through a future remote Model Context Protocol (MCP) tool.

The backend accepts structured nutrition data only. It does not receive or store meal photographs.

## Current status

The repository is in its bootstrap stage. The Cargo workspace and backend crate exist, but PostgreSQL persistence, HTTP endpoints, MCP tooling, and authentication have not yet been implemented.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (current stable toolchain)
- Cargo (installed with Rust)

PostgreSQL and Docker Compose will be documented here once the local database setup is added.

## Run the backend

From the repository root:

```sh
cargo run -p fitbit-mcp-backend
```

The current scaffold prints a placeholder message.

## Development checks

Run these commands from the repository root:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Intended API

The planned service will provide:

- `POST /mcp` — streamable HTTP MCP endpoint with a `log_meal` tool
- `GET /v1/meals` — list the authenticated user's meals
- `GET /v1/meals/{meal_id}` — retrieve one authenticated user's meal
- `GET /health/live` — process liveness
- `GET /health/ready` — dependency readiness

`log_meal` will require explicit user confirmation before a meal is persisted. Requests will use a per-user idempotency key so retries do not create duplicate meals.

## Scope

This MVP ends at PostgreSQL. Android, Health Connect, image storage, cloud deployment, and Fitbit integration are intentionally deferred.

## Future deployment: Shuttle

When this backend is ready to be deployed, [Shuttle](https://www.shuttle.dev/) is the recommended Rust-native option. It supports Axum applications and can provision a PostgreSQL database as part of its deployment workflow.

Deployment is deliberately deferred until authentication, authorization, and the local MCP flow have been implemented and tested. Do not deploy or create a Shuttle project yet.

## Project plan

See [meal-health-codex-plan-rust.md](meal-health-codex-plan-rust.md) for the architecture, security constraints, milestones, and acceptance criteria.
