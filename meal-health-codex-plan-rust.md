# Meal Logger: Backend-Only MCP Implementation Plan

## 1. Objective

Build a personal meal logger that ChatGPT can use directly through a remote MCP tool.

```text
meal photo in ChatGPT
  -> ChatGPT estimates nutrition and asks for confirmation
  -> user explicitly confirms
  -> ChatGPT calls log_meal over MCP
  -> Rust backend validates and stores the meal in PostgreSQL
```

The MVP ends at the database. It does **not** include an Android application, Health Connect, Google Health, Firebase Cloud Messaging, WorkManager, or phone synchronization.

ChatGPT performs visual analysis. The backend never receives or stores meal photographs; it accepts only confirmed structured nutrition data.

## 2. Definition of done

The backend MVP is complete when:

1. A user attaches a meal image in ChatGPT.
2. ChatGPT estimates the meal and receives explicit user confirmation.
3. ChatGPT invokes the remote MCP `log_meal` tool.
4. The backend validates and persists the confirmed meal in PostgreSQL.
5. The response returns a stable meal ID and `logged` status.
6. Replaying the same request does not create another meal.
7. The user can retrieve their own logged meals through an authenticated API.

Do not claim real ChatGPT connectivity or deployment until it has actually been configured and tested.

## 3. Fixed architecture

### Backend

- Rust using the current stable toolchain.
- Axum for HTTP routing and middleware, Tokio for async execution.
- Official Model Context Protocol Rust SDK (`rmcp`).
- Streamable HTTP MCP endpoint at `/mcp`.
- PostgreSQL locally; a managed PostgreSQL provider may be selected later for deployment.
- SQLx with committed SQL migrations and explicit SQL; no heavy ORM.
- Production crates use `#![forbid(unsafe_code)]`.
- Structured logs, correlation IDs, health endpoints, input validation, and idempotency.

### Authentication

- Start local milestones with an isolated development identity adapter or test tokens.
- The development bypass must be impossible to enable in a production build; test this.
- Add Auth0 OAuth 2.1/OIDC protection before connecting a real public remote MCP endpoint.
- Do not implement an authorization server or OAuth cryptography from scratch.

### Explicitly deferred

- Android, Kotlin, Compose, Health Connect, Google Health.
- Firebase Cloud Messaging, WorkManager, devices, outbox delivery, and sync acknowledgements.
- Google Cloud Run, Cloud SQL, Terraform, Cloud Scheduler, and any paid cloud resources.
- Web dashboard, image storage, food-detection models, Fitbit integration, social features, Kubernetes, and Kafka.

## 4. Repository layout

Create a Rust-focused repository unless existing files dictate otherwise:

```text
meal-to-health/
├── AGENTS.md
├── README.md
├── Makefile
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── docker-compose.yml
├── .gitignore
├── docs/
│   ├── architecture.md
│   ├── threat-model.md
│   ├── runbook.md
│   ├── memory-notes.md
│   └── adr/
├── backend/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── auth/
│   │   ├── config/
│   │   ├── http_api/
│   │   ├── mcp_server/
│   │   ├── meals/
│   │   ├── postgres/
│   │   └── telemetry/
│   ├── migrations/
│   ├── tests/
│   ├── Dockerfile
│   └── Cargo.toml
└── labs/memory/
```

Never commit credentials, database volumes, generated binaries, IDE state, or local environment files.

## 5. Core domain model

Generate UUIDs in the backend. Prefer UUIDv7 if the selected library supports it well; otherwise use UUIDv4.

### `users`

- `id`
- `auth_subject`, unique
- `created_at`

### `meals`

- `id`
- `user_id`
- `name`
- `energy_kcal`
- optional `protein_g`, `carbohydrate_g`, `fat_g`, `fiber_g`
- optional `estimate_low_kcal`, `estimate_high_kcal`, `confidence`
- `meal_type`: `breakfast`, `lunch`, `dinner`, `snack`, or `unknown`
- `eaten_at` as a timezone-aware timestamp
- `idempotency_key`, unique per user
- `created_at`, `updated_at`

### `meal_events`

Append-only audit events with `id`, `meal_id`, `event_type`, JSON payload, and `created_at`. Never put access tokens, authorization headers, or photographs into events.

Create a meal and its audit event in one PostgreSQL transaction.

## 6. Public contracts

### MCP tool: `log_meal`

Mark and describe this write tool as requiring explicit user confirmation. It must not be called merely because an image was attached.

Input schema:

```json
{
  "name": "Chicken, rice and salad",
  "energy_kcal": 730,
  "eaten_at": "2026-08-21T20:15:00+02:00",
  "meal_type": "dinner",
  "protein_g": 48,
  "carbohydrate_g": 76,
  "fat_g": 23,
  "fiber_g": 7,
  "estimate_low_kcal": 650,
  "estimate_high_kcal": 820,
  "confidence": 0.72,
  "idempotency_key": "client-generated-stable-key"
}
```

Only `name`, `energy_kcal`, `eaten_at`, and `idempotency_key` are required. Reject invalid values rather than coercing them. Start with 1–20,000 kcal and document any later change.

Output schema:

```json
{
  "meal_id": "uuid",
  "status": "logged",
  "message": "730 kcal logged"
}
```

The same authenticated user and idempotency key always return the original meal.

### REST endpoints

- `GET /v1/meals?cursor=<opaque-cursor>&limit=<n>` lists the current user's meals in a documented stable order.
- `GET /v1/meals/{meal_id}` retrieves one of the current user's meals.
- `GET /health/live` checks process liveness only.
- `GET /health/ready` verifies dependencies without leaking details.

Use opaque cursors for pagination. Do not add update or delete endpoints until the basic logging flow is proven.

## 7. Security and privacy

- Never upload or persist meal photographs.
- Treat nutrition entries and identifiers as sensitive data.
- Validate JWT issuer, audience/resource, expiry, signature, and required scope (`meals:write`) once OAuth is enabled.
- Enforce user isolation on every query; users may access only their own meals.
- Redact tokens, authorization headers, and meal details from high-cardinality logs.
- Use TLS and private database access in any future deployment.
- Document retention and deletion behavior before exposing the service beyond personal use.
- Threat-model replayed MCP requests, malicious tool arguments, leaked tokens, SQL injection, and cross-user access.

## 8. Rust memory-management learning track

This remains a learning objective but must not delay the basic backend unnecessarily.

Production notes should explain owned request DTOs at the boundary, internal borrowing where simple, `Arc<AppState>` for shared dependencies, async state held across `.await`, and why synchronous mutex guards are never held across `.await`.

Keep exercises under `labs/memory` separate from deployed crates:

1. Ownership and borrowing with moved `String` and `Vec` values.
2. `Box` and `Drop` using a boxed linked structure.
3. `Rc<RefCell<T>>` compared with `Arc<Mutex<T>>` across Tokio tasks.
4. Optional bump-arena and allocation-profiling exercises, with any `unsafe` tightly scoped to the lab and checked with Miri where supported.

For exercises, scaffold tests, diagrams, and hints before supplying solutions unless the user requests autonomous solutions. Maintain concise notes in `docs/memory-notes.md`.

## 9. Milestones

### Milestone 0: Repository bootstrap

Deliver a Cargo workspace, README, `.gitignore`, Make targets, Docker Compose PostgreSQL, configuration loader, CI skeleton, architecture/security ADRs, and memory-lab scaffolding.

Accept when a new developer can start PostgreSQL and run formatting, linting, and tests from documented commands.

### Milestone 1: Meals and PostgreSQL

Deliver migrations for users, meals, and events; validation; a transactional `CreateMeal` service; idempotency; health endpoints; and read endpoints.

Test invalid data, user isolation, transaction rollback, pagination, and concurrent duplicate requests. Prefer integration tests against real PostgreSQL.

### Milestone 2: Local MCP server

Deliver `rmcp` with streamable HTTP at `/mcp`, the `log_meal` tool, strict schemas, an explicit-confirmation description, and a development identity adapter.

Accept when MCP Inspector discovers the tool, creates a meal, and a repeated call with the same key returns the same ID.

### Milestone 3: Secure remote MCP readiness

Deliver Auth0 development-tenant documentation, protected-resource metadata, OAuth/OIDC discovery, JWT validation, scope enforcement, and a test proving the development bypass cannot run in production.

Accept when missing, expired, wrong-issuer, wrong-audience, and insufficient-scope tokens are rejected, and two users cannot read each other's entries.

### Milestone 4: Deploy only with authorization

After user authorization, select and provision hosting, secrets, managed PostgreSQL, TLS, monitoring, and a deployment runbook. Cloud Run and Cloud SQL are a recommended option, not part of the current MVP.

### Future phase: Health Connect

If Google Health visibility becomes required, add a native Android companion app. It will authenticate to this backend and use the Android Health Connect SDK—with user-granted permission—to write `NutritionRecord`s. Reintroduce durable sync, versioning, and optional push notifications only then.

## 10. Testing and local workflow

Run unit tests for validation and service behavior; PostgreSQL integration tests for transactions, idempotency, concurrency, and authorization; HTTP contract tests; and MCP Inspector smoke tests.

Provide copy-pasteable equivalents for:

```text
make bootstrap
make db-up
make migrate
make test
make lint
make backend-run
make mcp-inspect
make memory-labs
```

Default local tests must use fakes or local services and never require production credentials.

## 11. Execution rules

Before coding, read repository instructions and documentation, preserve unrelated changes, verify current dependency and SDK versions against primary documentation, and keep one implementation step in progress at a time.

Do not create cloud resources, paid accounts, production Auth0 configuration, or a real ChatGPT remote connection without explicit user authorization. Make reversible local choices, record material decisions, and ask only when an external-state, security, or architecture choice cannot be made safely.

At the end of each milestone, report the outcome, changed files, commands/tests and results, limitations, and the next milestone.
