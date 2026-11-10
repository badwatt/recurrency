# Recurrency

> Every expense is a subscription.

A self-hosted subscription and expense tracker. Because every purchase is really a question:
*Would I subscribe to this today?*

![Recurrency Dashboard](assets/dashboard.png)

---

## Quick Start

```bash
git clone https://github.com/badwatt/recurrency.git
cd recurrency
nix develop
cargo up
```

Open http://localhost:1420. `cargo up` starts PostgreSQL, the backend, and the
frontend; `Ctrl+C` stops all three.

**Demo:** Click "Demo account" on the login page.

---

## Development

All commands run inside `nix develop`.

| Task | Command |
|---|---|
| Start all services | `cargo up` |
| Start PostgreSQL | `docker compose -f docker/docker-compose.dev.yml up -d` |
| Stop PostgreSQL | `docker compose -f docker/docker-compose.dev.yml down` |
| Reset database | `docker compose -f docker/docker-compose.dev.yml down -v && docker compose -f docker/docker-compose.dev.yml up -d` |
| Backend (port 8080) | `cargo run -p recurrency_backend` |
| Frontend (port 1420) | `cd frontend && dx serve --platform web --port 1420` |
| List project commands | `cargo dev-help` |
| Tests | `cargo test-all` |
| Format | `cargo fmt-all` |
| Clippy | `cargo clippy-all` |
| Tailwind CSS | `cd frontend && npm run build:css` |
| Backend coverage | `cargo llvm-cov -p recurrency_backend test --lib` |
| Frontend coverage | `cargo llvm-cov -p recurrency_frontend test --lib` |
| Local debug Android APK | `cargo build-android` |
| Install Android SDK/NDK | `cargo setup-android` |
| Patch an existing Android build | `cargo patch-android` |

Build artifacts live in `target/bin/` (APK, desktop bundle, `recurrency-aio`).

The aliases are in `.cargo/config.toml`; the raw equivalents are `cargo test --workspace --lib`, `cargo fmt --all`, and `cargo clippy --workspace --lib -- -D warnings`.


### Migrating from `./taskfile`

| Old | New |
|---|---|
| `./taskfile up` | `cargo up` |
| `./taskfile down` / `stop` | `cargo down` / `cargo stop` |
| `./taskfile test` | `cargo test` or `cargo test-all` |
| `./taskfile coverage` | `cargo coverage` |
| `./taskfile reset` | `cargo reset` |
| `./taskfile backend` | `cargo backend` |
| `./taskfile frontend` | `cargo frontend` |
| `./taskfile db` | `cargo db` |
| `./taskfile css` | `cargo css` |
| `./taskfile build` | `cargo build-all` |
| `./taskfile build:app` | `cargo build-app` |
| `./taskfile build:api` | `cargo build-api` |
| `./taskfile build:web` | `cargo build-web` |
| `./taskfile build:android` | `cargo build-android` |
| `./taskfile build:desktop` | `cargo build-desktop` |
| `./taskfile clear` | `cargo clear` |
| `./taskfile clear:app` | `cargo clear-app` |
| `./taskfile clear:api` | `cargo clear-api` |
| `./taskfile clear:web` | `cargo clear-web` |
| `./taskfile clear:android` | `cargo clear-android` |
| `./taskfile clear:desktop` | `cargo clear-desktop` |
| `./taskfile clear:all` | `cargo clear-all` |
| `./taskfile help` | `cargo dev-help` |

`cargo build` remains Cargo's normal debug build; `cargo build-all` is the old release build.

**Services:**
- **Frontend:** http://localhost:1420 (Dioxus WASM)
- **Backend:** http://localhost:8080 (Axum)
- **PostgreSQL:** localhost:5432

---

## Production (Docker)

Single container with PostgreSQL + backend + frontend:

```bash
# Pull and run
docker run -d -p 80:80 -e JWT_SECRET="$(openssl rand -hex 32)" ghcr.io/badwatt/recurrency:latest

# Or use the compose file
curl -O https://raw.githubusercontent.com/badwatt/recurrency/main/docker/docker-compose.yml
docker compose up -d
```

- **App:** http://localhost:80 (nginx proxies `/api` → backend)
- **API:** http://localhost:80/api

Custom env:

```bash
JWT_SECRET="$(openssl rand -hex 32)" docker compose up -d
```

---

## Configuration

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | PostgreSQL connection |
| `JWT_SECRET` | **required** | Strong random secret for JWT tokens; startup fails if missing/placeholder |
| `ENABLE_DEMO` | `false` | Show demo login button (opt-in) |
| `PGPASSWORD` | `recurrency` | PostgreSQL password |
| `SMTP_HOST` | — | SMTP server |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_USER` | — | SMTP username |
| `SMTP_PASSWORD` | — | SMTP password |
| `SMTP_FROM` | — | Sender email |

---

## Stack

| Layer | Tech |
|---|---|
| **Frontend** | Dioxus (Rust WASM) + Tailwind |
| **Backend** | Rust + Axum + SQLx + JWT + Argon2 |
| **Database** | PostgreSQL |
| **Email** | Resend API |
| **Push** | OneSignal |

---

## Features

| | |
|---|---|
| 💳 **Accounts** | Budgeting buckets required on every entry, with inline search/create |
| 📊 **Monthly plan** | Fixed monthly budget and pending-from-today per account |
| 💳 **Subscriptions** | CRUD with billing cycles, next-date calc, account, optional category, notes |
| 🧾 **Transactions** | One-time expenses with account and optional category |
| 📅 **Calendar** | Monthly view of upcoming payments |
| 📊 **Reports** | Trend charts, category breakdown, CSV/PDF export |
| 🏷️ **Categories** | Optional classification with custom icons, colors, inline creation |
| 🔔 **Notifications** | In-app, email, push (OneSignal) |
| 🌙 **Dark Mode** | Light / dark / system |
| 📤 **Grid / List** | Adjustable view |
| 💱 **Currencies** | Multi-currency display |
| 🔄 **Demo Reset** | Restore sample data |
| 📈 **Statistics** | Spending trends, category distribution |
| 🔔 **Reminders** | Configurable notification days |

---

## Test Coverage

| Project | Lines | Functions | Regions | Tests |
|---|---|---|---|---|
| **Backend** | — | — | — | 1131 |
| **Frontend** | — | — | — | 1639 |

Full coverage report (Nix dev shell):

```bash
# Backend tests + coverage
cd backend && nix develop .. -c cargo llvm-cov test --lib

# Frontend tests + coverage
cd frontend && nix develop .. -c cargo llvm-cov test
```

### Coverage Notes

Backend uncovered ~13% = async DB handlers requiring:
- PostgreSQL integration tests (trait injection added for testability)

Frontend uncovered ~20% = RSX macro-generated VDOM requiring:
- Dioxus WASM test runtime

Both projects include comprehensive model/utility tests + mock services.

---

## Architecture

```
backend/src/
├── auth/          # JWT, registration, login
├── subscriptions/ # Subscription CRUD
├── transactions/  # Transaction CRUD
├── accounts/       # Budgeting account CRUD
├── categories/     # Optional category CRUD
├── notifications/  # In-app, email, push
├── reports/       # Statistics, trends
├── preferences/    # User settings
├── demo/          # Demo mode
├── logos/         # Logo search
└── shared/        # DB, auth extractor, utils

frontend/src/
├── pages/          # Route pages
├── components/     # Reusable UI
├── api/            # HTTP client
├── state/          # Signals/state
└── auth/           # Auth pages
```

---

## Releasing

1. Bump `version` in the root `Cargo.toml` (the single source of truth).
2. Commit and push to `main`.
3. The Release workflow only runs its release jobs when that version increases; then it runs the tests, publishes the Docker images, builds the APK and creates the GitHub release/tag.

A manual `workflow_dispatch` always releases the current version.

## License

MIT