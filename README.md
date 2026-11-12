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

**Demo:** Local `cargo up` enables it by default; in Docker set `ENABLE_DEMO=true`. Click "Demo account" on the login page.

---

## Development

All commands run inside `nix develop`.

| Task | Command |
|---|---|
| Start all services | `cargo up` |
| Stop all services | `cargo down` |
| Start PostgreSQL only | `cargo db` |
| Reset database | `cargo reset` |
| Backend (port 8080) | `cargo backend` |
| Frontend (port 1420) | `cargo frontend` |
| List project commands | `cargo dev-help` |
| Tests | `cargo test-all` |
| Format | `cargo fmt-all` |
| Clippy | `cargo clippy-all` |
| Coverage | `cargo coverage` |
| Tailwind CSS | `cargo css` |
| Local debug Android APK | `cargo build-android` |
| Install Android SDK/NDK | `cargo setup-android` |
| Patch an existing Android build | `cargo patch-android` |

Build artifacts live in `target/bin/` (APK, desktop bundle, `recurrency-aio`).

The aliases are in `.cargo/config.toml`; the raw equivalents are `cargo test --workspace --lib`, `cargo fmt --all`, `cargo clippy --workspace --lib -- -D warnings`, and `cargo run -p recurrency_tools -- <command>`.

`cargo build` remains Cargo's normal debug build; `cargo build-all` builds the release artifacts.

**Services:**
- **Frontend:** http://localhost:1420 (Dioxus WASM)
- **Backend:** http://localhost:8080 (Axum)
- **PostgreSQL:** localhost:5432

---

## Production (Docker)

One all-in-one image contains PostgreSQL, the backend and the web frontend
(nginx). The same image can run **headless / API-only**. Pin a version in
production instead of `latest`.

### Web UI + API

```bash
docker run -d \
  --name recurrency \
  -p 80:80 -p 8080:8080 \
  -v recurrency_db:/var/lib/postgresql/15/main \
  -e JWT_SECRET="$(openssl rand -hex 32)" \
  ghcr.io/badwatt/recurrency:0.32.0
```

- **Web:** http://localhost
- **API through nginx:** http://localhost/api
- **API directly:** http://localhost:8080/api
- The named volume keeps PostgreSQL data across container recreation.
- Reuse the same `JWT_SECRET` across restarts; changing it invalidates sessions.

### Headless / API-only

```bash
docker run -d \
  --name recurrency \
  -p 8080:8080 \
  -v recurrency_db:/var/lib/postgresql/15/main \
  -e JWT_SECRET="$(openssl rand -hex 32)" \
  -e ENABLE_WEB=false \
  ghcr.io/badwatt/recurrency:0.32.0
```

- nginx is not started; only PostgreSQL and the backend run.
- **API:** http://localhost:8080/api
- `ENABLE_WEB=false` is a runtime toggle; no rebuild is needed.
- The image still contains the web assets and nginx; there is no separate
  smaller headless image.
- For native clients, enter the server URL without `/api`
  (`http://host:8080`); the app adds the API paths.
- Terminate TLS with Caddy, Traefik, nginx or another reverse proxy in front of
  port 8080.

### Docker Compose

```bash
curl -O https://raw.githubusercontent.com/badwatt/recurrency/main/docker/docker-compose.yml
JWT_SECRET="$(openssl rand -hex 32)" docker compose up -d recurrency

# Headless
ENABLE_WEB=false JWT_SECRET="$(openssl rand -hex 32)" docker compose up -d recurrency
```

The compose file publishes both `80` and `8080`. In headless mode port 80 is
published but nothing listens on it; remove `80:80` from the compose file if
you do not want to expose it.

---

## Configuration

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | set by entrypoint in Docker | PostgreSQL connection |
| `DATABASE_MAX_CONNECTIONS` | `5` | PostgreSQL pool size |
| `JWT_SECRET` | **required** | Strong random secret for JWT tokens; startup fails if missing/placeholder |
| `APP_PORT` | `8080` | Backend listen port |
| `APP_ENV` | `development` | Set `production` to lower the default log level |
| `RUST_LOG` | `APP_ENV` default | Tracing filter (for example `info` or `debug`) |
| `CORS_ALLOWED_ORIGIN` | `*` | Restrict browser requests to one allowed origin |
| `ENABLE_DEMO` | `false` | Show the demo login button (opt-in) |
| `ENABLE_WEB` | `true` | Docker only: serve the web UI; `false` for headless |
| `PGPASSWORD` | `recurrency` | PostgreSQL password inside the Docker image |
| `RECURRENCY_API_URL` | compiled default | Default server URL for native builds; overridden in the app |
| `FIXER_API_KEY` | — | Optional Fixer API key; otherwise the free Frankfurter provider is used |
| `SMTP_HOST` | — | SMTP server |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_USER` | — | SMTP username |
| `SMTP_PASSWORD` | — | SMTP password |
| `SMTP_FROM` | — | Sender email |
| `SMTP_ENCRYPTION` | auto (`ssl` for 465, otherwise `tls`) | SMTP transport security |
| `NOTIFICATION_ALLOW_PRIVATE_HOSTS` | `false` | Allow ntfy/Gotify/webhook targets on private networks |

---

## Stack

| Layer | Tech |
|---|---|
| **Frontend** | Dioxus (Rust WASM) + Tailwind |
| **Backend** | Rust + Axum + SQLx + JWT + Argon2 |
| **Database** | PostgreSQL |
| **Notifications** | Per-user SMTP + 11 native channels + summary scheduler |
| **Currency** | Automatic conversion via Fixer (optional key) or Frankfurter (no key) |
| **Deployment** | Single all-in-one Docker image; web or headless API-only |

---

## Features

| | |
|---|---|
| 💳 **Accounts** | Budgeting buckets required on every entry, with inline search/create |
| 📊 **Monthly plan** | Fixed monthly budget and pending-from-today per account |
| 🔁 **Subscriptions** | CRUD with a unified "every N days/weeks/months/years" recurrence: multiple weekdays, multiple monthly days, unlimited annual dates and optional per-date amounts |
| 🧾 **Transactions** | One-time expenses with account and optional category |
| 📅 **Calendar** | Upcoming and completed charges, with an inclusive end date and finalized state |
| 📈 **Reports** | Trend charts, category breakdown, CSV/PDF export |
| 🏷️ **Categories** | Optional classification with custom icons, colors and inline creation |
| 🔔 **Delivery channels** | Email, ntfy, Telegram, Discord, Gotify, Slack, Mattermost, Pushover, PushPlus, ServerChan and webhooks |
| 🗓️ **Summary schedule** | Daily, weekly or monthly dashboard summary in your timezone, for the current or next month |
| 👤 **Profile** | Change email, display name and avatar; delete the account with password confirmation |
| ⚙️ **Settings** | Profile, Preferences, Money (accounts/categories) and Notifications |
| 📋 **Duplicate** | Duplicate a subscription with all fields copied; the form opens with a `(copia)` suffix |
| 🏁 **Finalized** | Active, inactive and finalized sections; finalized cards show the end date and no future charges |
| 🏷️ **Category badges** | Categories appear next to accounts; the system Uncategorized fallback is never shown |
| ⬆️ **Update notice** | Dismissible banner when a newer GitHub release is available |
| 📱 **Native app** | Android and desktop clients with configurable server URL at login and in Settings |
| 💳 **Debts** | Manual debts with payments that reduce the balance, plus automatic debts paid month by month and backed by a subscription |
| 💱 **Currency conversion** | Totals and reports convert into the main currency; Fixer is optional and Frankfurter works without a key |
| 📱 **Mobile views** | Grid/list toggle available on phones and tablets |
| 🐳 **Deployment** | One all-in-one image; web UI or headless API-only |
| 🌙 **Dark Mode** | Light / dark / system |
| 📤 **Grid / List** | Adjustable view |
| 💱 **Currencies** | Multi-currency display |
| 🔄 **Demo Reset** | Restore sample data |

---

## Test Coverage

| Project | Lines | Functions | Regions | Tests |
|---|---|---|---|---|
| **Backend** | — | — | — | 1197 lib + 47 integration |
| **Frontend** | — | — | — | 1658 |
| **Tools** | — | — | — | 8 |

Integration tests use a disposable PostgreSQL instance:

```bash
docker compose -f docker/docker-compose.test.yml up -d
cd backend
DATABASE_URL='postgres://recurrency:recurrency_test_password@localhost:5433/recurrency_test' \
  JWT_SECRET=test-jwt-secret-key \
  cargo test --features integration-tests
docker compose -f docker/docker-compose.test.yml down -v
```

CI runs the same suite on every push and pull request via
`.github/workflows/test.yml`.

Full coverage report (Nix dev shell):

```bash
# Backend tests + coverage
cd backend && nix develop .. -c cargo llvm-cov test --lib

# Frontend tests + coverage
cd frontend && nix develop .. -c cargo llvm-cov test
```

### Coverage Notes

Remaining backend gaps are mostly async database handlers covered by
PostgreSQL integration tests (services use trait injection for testability).

Remaining frontend gaps are mostly RSX-generated VDOM that needs a Dioxus WASM
test runtime. Both projects include model, utility and mock-service tests.

---

## Architecture

```
backend/src/
├── auth/           # JWT, registration, login, settings
├── accounts/       # Mandatory budgeting accounts
├── subscriptions/  # Recurrence schedules and subscription CRUD
├── transactions/   # Transaction CRUD
├── categories/     # Optional category CRUD
├── currency/       # Exchange-rate providers and conversion
├── debts/          # Manual and automatic debt tracking
├── notifications/  # Delivery channels, encryption, summary scheduler
├── email/          # SMTP delivery
├── reports/        # Statistics, trends and exports
├── preferences/    # User preferences
├── demo/           # Demo mode
├── logos/          # Logo search
└── shared/         # DB, auth extractor, app state, utils

frontend/src/
├── pages/          # Route pages
├── components/     # Reusable UI
├── api/            # HTTP client
├── state/          # Signals/state
├── layout/         # App and settings shells
├── storage/        # Local/offline storage
├── theme/          # Theme handling
└── auth/           # Auth pages
```

---

## Releasing

1. Bump `version` in the root `Cargo.toml` (the single source of truth).
2. Commit and push to `main`.
3. The Release workflow only runs its release jobs when that version increases; then it runs the tests, publishes the all-in-one Docker image, builds the APK and creates the GitHub release/tag.

A manual `workflow_dispatch` always releases the current version.

## License

[MIT](LICENSE)
