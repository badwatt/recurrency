<h1 align="center">Recurrency</h1>

<p align="center">
  <strong>Every expense is a subscription.</strong><br>
  A self-hosted subscription, expense and debt tracker for people who want to see
  the real cost of what they keep paying for.
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.98+-orange?logo=rust&logoColor=white">
  <img alt="Dioxus" src="https://img.shields.io/badge/UI-Dioxus-6f42c1">
  <img alt="PostgreSQL" src="https://img.shields.io/badge/Database-PostgreSQL-336791?logo=postgresql&logoColor=white">
  <img alt="Docker" src="https://img.shields.io/badge/Deploy-Docker-2496ed?logo=docker&logoColor=white">
  <img alt="License" src="https://img.shields.io/badge/License-MIT-blue">
</p>

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#screenshots">Screenshots</a> ·
  <a href="#features">Features</a> ·
  <a href="#configuration">Configuration</a> ·
  <a href="#development">Development</a>
</p>

![Recurrency dashboard](assets/dashboard.png)

Recurrency treats recurring payments the way your brain actually experiences
them: as ongoing commitments. It answers three questions without spreadsheets:

- **What do I need to transfer this month?**
- **What is still pending from today?**
- **Would I subscribe to this again today?**

The stack is deliberately boring to operate: a Rust/Axum backend, PostgreSQL and
a Dioxus web frontend, shipped as one all-in-one Docker image with an optional
headless API mode.

---

## Quick start

### Docker

```bash
docker run -d \
  --name recurrency \
  -p 80:80 -p 8080:80 \
  -v recurrency_db:/var/lib/postgresql/18/main \
  ghcr.io/badwatt/recurrency:latest
```

Open **http://localhost**. The binary serves the web UI and the API on the
same port; the API is available at `http://localhost/api` and directly at
`http://localhost:8080/api`.

The image includes PostgreSQL **18**. If `DATABASE_URL` is set, the entrypoint
does not start the bundled PostgreSQL at all and connects to that external
server instead.

The named volume persists PostgreSQL across container recreation. On first boot
the application generates a random JWT signing secret and stores it in the
database, so no configuration is required. Set `JWT_SECRET` to override it;
changing the effective secret invalidates existing sessions.

> PostgreSQL 18 uses a new data directory (`/var/lib/postgresql/18/main`).
> Volumes created by the old PostgreSQL 15 image need a `pg_dump`/restore before
> upgrading. Keep the old volume until the restored instance has been verified.

> [!IMPORTANT]
> The container fails fast if the application cannot start (failed migration or
> database error). It exits instead of serving an empty frontend, so Docker can
> restart it or surface the failure.

> [!TIP]
> Pin a release tag instead of `latest` in production, for example
> `ghcr.io/badwatt/recurrency:0.43.0`.

### From source

Nix is the only prerequisite for development.

```bash
git clone https://github.com/badwatt/recurrency.git
cd recurrency
nix develop
cargo up
```

`cargo up` starts PostgreSQL, the Axum backend and the Dioxus dev server.

| Service | URL |
|---|---|
| Web UI | http://localhost:1420 |
| API | http://localhost:8080/api |
| PostgreSQL | localhost:5432 |

Press `Ctrl+C` to stop all three.

> [!NOTE]
> **Demo mode is optional.** The production image ships with it disabled
> (`ENABLE_DEMO=false`). `cargo up` enables it for local development; set
> `ENABLE_DEMO=false` to hide the demo login button. There is no demo data unless
> you explicitly enable it.

---

## Screenshots

| Dashboard | Subscriptions |
|---|---|
| ![Dashboard](assets/dashboard.png) | ![Subscriptions](assets/subscriptions.png) |

| Calendar | Reports |
|---|---|
| ![Calendar](assets/calendar.png) | ![Reports](assets/reports.png) |

The dashboard is a monthly transfer ledger: what each account needs, what is
already covered and what is still pending. Subscriptions keep their account,
category, logo and next charge visible; the calendar puts every charge on a
timeline; reports turn the same data into trends and category breakdowns.

---

## Features

### Know what this month costs

- **Monthly plan** — fixed budget and pending-from-today, per account and in total.
- **Personal date format** — choose DD/MM/YYYY, MM/DD/YYYY or YYYY/MM/DD in Settings.
- **Accounts as budgeting buckets** — every subscription and transaction belongs to one.
- **Real charges, not averages** — summaries use the charges that actually fall in the month.
- **Current or next month** — switch the dashboard summary period from Settings.
- **Calendar plan** — the same transfer/pending totals sit above the calendar on desktop and mobile.
- **Upcoming and manual payments** — see what is coming and what is still one-off.

### Track everything that repeats

- **Flexible recurrence** — every N days, weeks, months or years.
- **Multiple weekdays** — one subscription can charge on several days of the week.
- **Multiple monthly days** — including `Last day` for short months.
- **Custom annual dates** — unlimited dates per year, each with an optional amount.
- **Payment progress** — multi-charge schedules show `Payment 1/3`, `2/3`, `3/3`.
- **Inclusive end dates** — decide exactly when a subscription stops.
- **Finalized / inactive** — finished schedules stay out of upcoming charges without losing history.
- **Duplicate** — copy a subscription and edit only what changed.
- **Notes** — attach context, clear it when it is no longer needed.
- **Search and bulk actions** — search every list, switch grid/list views and pause, activate or delete several subscriptions at once.

### Expenses, not only subscriptions

- **Transactions** — one-time expenses with the same account and optional category model.
- **Calendar** — charges by day, including completed and overdue payments.
- **Debts** — manual debts with payment history, or automatic debts paid month by month.
- **Installments** — automatic debts show progress such as `Payment 2/6`.
- **Bulk actions** — select transactions or debts to recategorize or delete them together; subscriptions can be paused, activated or deleted in bulk.

### Currency without surprises

- **One main currency** — set it once in Settings.
- **Original currency preserved** — converted values show the source amount in parentheses.
- **Automatic rates** — Frankfurter works without a key; choose Fixer and save an encrypted per-user API key from Settings.
- **Graceful fallback** — unknown rates keep the original amount instead of breaking reports.
- **Totals and reports convert** — no mixed-currency arithmetic in summaries.

### Notifications that reach you

- **11 delivery channels** — Email, ntfy, Telegram, Discord, Gotify, Slack, Mattermost, Pushover, PushPlus, ServerChan and webhooks.
- **Per-user SMTP** — your own mail server, not a shared vendor.
- **Encrypted channel secrets** — credentials are stored encrypted at rest.
- **Summary scheduler** — daily, weekly or monthly, in your timezone, for the current or next month.
- **SSRF guard** — private network targets require an explicit opt-in.

### Built for real devices

- **Web** — responsive layouts, grid/list toggle and mobile floating actions.
- **Android and desktop** — native clients with a configurable server URL.
- **Dark, light or system** — follows your preference.
- **Update notice** — dismissible banner when a newer GitHub release exists; can be turned off in Settings.
- **Custom icons and colors** — searchable icon picker for accounts and categories.

### Your data and the project

- **Backup & restore** — download your accounts, categories, subscriptions, transactions, debts and payments as JSON, or restore a backup by click or drag-and-drop.
- **About** — installed version, current and latest changelog, and a quiet reminder to star the project on GitHub.
- **Console welcome** — a small developer thank-you and the running version in the browser console.

---

## How it fits together

```text
                    ┌─────────────────────────────┐
                    │  Dioxus web / Android /     │
                    │  desktop clients            │
                    └──────────────┬──────────────┘
                                   │ REST + JWT
                    ┌──────────────▼──────────────┐
                    │  Axum backend               │
                    │  auth · subscriptions ·      │
                    │  transactions · debts ·      │
                    │  reports · notifications    │
                    └───────┬──────────────┬──────┘
                            │              │
                 ┌──────────▼────┐   ┌────▼─────────────┐
                 │ PostgreSQL    │   │ Email, push,     │
                 │ users, money, │   │ ntfy, webhooks…  │
                 │ schedules     │   └──────────────────┘
                 └───────────────┘
```

**Core stack**

| Layer | Technology |
|---|---|
| Frontend | Dioxus (Rust WASM) + Tailwind |
| Backend | Rust + Axum + SQLx |
| Database | PostgreSQL |
| Auth | JWT + Argon2 |
| Currency | Frankfurter (default) or Fixer |
| Notifications | SMTP + 11 delivery providers |
| Packaging | Nix · Cargo · Docker |

---

## Configuration

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | bundled local PostgreSQL | PostgreSQL connection string; when set, Docker does not start its bundled database |
| `DATABASE_MAX_CONNECTIONS` | `5` | PostgreSQL pool size |
| `JWT_SECRET` | generated in `app_secrets` | Optional JWT signing secret; when set it overrides the database-stored secret |
| `APP_PORT` | `8080` (`80` in the Docker image) | Application listen port |
| `APP_ENV` | `development` | Set `production` to lower the default log level |
| `RUST_LOG` | `APP_ENV` default | Tracing filter, for example `info` or `debug` |
| `CORS_ALLOWED_ORIGIN` | `*` | Restrict browser requests to one origin |
| `ENABLE_DEMO` | `false` | Enable the optional demo login |
| `ENABLE_WEB` | `true` | Serve the embedded web UI; `false` for headless |
| `RECURRENCY_DATA_DIR` | `recurrency-data` | Standalone binary only: local database directory when `DATABASE_URL` is unset |
| `PGPASSWORD` | `recurrency` | Password for the bundled PostgreSQL; ignored with an external `DATABASE_URL` |
| `RECURRENCY_API_URL` | compiled default | Default server URL for native builds |
| `FIXER_API_KEY` | — | Optional server-wide Fixer key; users can also save their own key in Settings |
| `SMTP_HOST` | — | SMTP server |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_USER` | — | SMTP username |
| `SMTP_PASSWORD` | — | SMTP password |
| `SMTP_FROM` | — | Sender address |
| `SMTP_ENCRYPTION` | auto (`ssl` on 465, otherwise `tls`) | SMTP transport security |
| `NOTIFICATION_ALLOW_PRIVATE_HOSTS` | `false` | Allow ntfy/Gotify/webhook targets on private networks |

### Standalone all-in-one binary

The application can also be built as one native binary with the web frontend
embedded and served by Axum:

```bash
cd recurrency
nix develop
cargo build-app
./target/bin/recurrency --help

# API + embedded web UI
DATABASE_URL="postgres://recurrency:recurrency@localhost:5432/recurrency" \
  ./target/bin/recurrency serve

# API only, same binary
ENABLE_WEB=false ./target/bin/recurrency serve

# Migrations and configuration check
./target/bin/recurrency migrate --database-url "$DATABASE_URL"
./target/bin/recurrency doctor
```

The binary embeds both the built frontend and a local PostgreSQL 18.7 runtime.
If `DATABASE_URL` is omitted, it starts a private database under
`./recurrency-data` (override with `RECURRENCY_DATA_DIR`), creates the schema on
first run, and stops the local database on `Ctrl-C`/`SIGTERM`. Set
`DATABASE_URL` to use an external PostgreSQL instead.

Existing local clusters created by older releases (PostgreSQL 17) are upgraded
automatically on startup with `pg_dump`/restore. The old data directory is kept
as `data.pg17` for rollback.

Embedded PostgreSQL is used rather than SQLite because the application relies
on PostgreSQL types and SQL semantics (UUID, JSONB, decimal money, intervals,
partial indexes). A SQLite backend would require a second repository for every
domain module and would risk changing financial behaviour.

`--no-web` is equivalent to `ENABLE_WEB=false`. `cargo build-api` builds the
API-only binary without embedded assets or the local database.

### Headless API

The same image can run without the web UI:

```bash
docker run -d \
  --name recurrency \
  -p 8080:80 \
  -v recurrency_db:/var/lib/postgresql/18/main \
  -e ENABLE_WEB=false \
  ghcr.io/badwatt/recurrency:latest
```

- API: `http://localhost:8080/api`.
- The web UI is not mounted; PostgreSQL and the binary still run.
- `ENABLE_WEB=false` is a runtime toggle; no rebuild is needed.
- Put Caddy, Traefik, nginx or another reverse proxy in front for TLS.
- Native clients enter the server URL **without** `/api`; the app adds API paths.

### Docker Compose

```bash
curl -O https://raw.githubusercontent.com/badwatt/recurrency/main/docker/docker-compose.yml
docker compose up -d recurrency
```

Headless:

```bash
ENABLE_WEB=false docker compose up -d recurrency
```

The compose file publishes `80` and `8080` (both map to the binary's port
`80`). In headless mode nothing listens on port 80; remove `80:8080` if you do
not want to expose it.

---

## Development

All commands run inside `nix develop`.

| Task | Command |
|---|---|
| Start all services | `cargo up` |
| Stop all services | `cargo down` |
| PostgreSQL only | `cargo db` |
| Reset database | `cargo reset` |
| Backend only | `cargo backend` |
| Frontend only | `cargo frontend` |
| List project commands | `cargo dev-help` |
| Tests | `cargo test-all` |
| Format | `cargo fmt-all` |
| Clippy | `cargo clippy-all` |
| Coverage | `cargo coverage` |
| Tailwind CSS | `cargo css` |
| Local debug APK | `cargo build-android` |
| Install Android SDK/NDK | `cargo setup-android` |
| Patch an existing Android build | `cargo patch-android` |

Artifacts land in `target/bin/` (APK, desktop bundle and `recurrency`).

The aliases live in `.cargo/config.toml`. Their raw equivalents are
`cargo test --workspace --lib`, `cargo fmt --all`,
`cargo clippy --workspace --lib -- -D warnings` and
`cargo run -p recurrency_tools -- <command>`.

### Project layout

```text
backend/src/
├── auth/           # JWT, registration, login, account settings
├── accounts/       # Mandatory budgeting accounts
├── subscriptions/  # Recurrence engine and subscription CRUD
├── transactions/   # One-time expenses
├── categories/     # Optional categories and icons
├── currency/       # Exchange-rate providers and conversion
├── debts/          # Manual and automatic debt tracking
├── notifications/  # Delivery channels, encryption, scheduler
├── email/          # SMTP delivery
├── reports/        # Trends, breakdowns and exports
├── preferences/    # User preferences
├── demo/           # Optional demo mode
├── logos/          # Logo search
└── shared/         # DB, auth extractor, app state, test helpers

frontend/src/
├── pages/          # Route pages
├── components/     # Reusable UI
├── api/            # HTTP client
├── state/          # Signals and shared state
├── layout/         # App and settings shells
├── storage/        # Local/offline storage
├── theme/          # Theme handling
└── auth/           # Auth pages
```

### Tests

```bash
# Unit tests
cargo test-all

# Integration tests (disposable PostgreSQL)
docker compose -f docker/docker-compose.test.yml up -d
cd backend
DATABASE_URL='postgres://recurrency:recurrency_test_password@localhost:5433/recurrency_test' \
  JWT_SECRET=test-jwt-secret-key \
  cargo test --features integration-tests
docker compose -f docker/docker-compose.test.yml down -v

# Coverage
cd backend && nix develop .. -c cargo llvm-cov test --lib
cd frontend && nix develop .. -c cargo llvm-cov test
```

Current suite: **1213 backend unit tests** (plus 2 embedded-db migration
tests), **47 backend integration tests**, **1707 frontend tests** and
**8 tooling tests**.

### Releasing

1. Bump `version` in the root `Cargo.toml` (the single source of truth).
2. Commit and push to `main`.
3. The Release workflow runs only when that version increases; it publishes the
   all-in-one image, builds and signs the APK, and creates the GitHub release.

A manual `workflow_dispatch` releases the current version.

---

## Security

- Argon2 password hashing and JWT authentication.
- The JWT signing secret is generated once and stored in `app_secrets` when
  `JWT_SECRET` is not set. Full PostgreSQL backups include it; set `JWT_SECRET`
  from a secret manager in production if the signing key must stay outside the
  database.
- Notification credentials encrypted at rest with AES-256-GCM.
- Private-network notification targets blocked unless explicitly allowed.
- No `unsafe` Rust.
- Fail-closed configuration: missing or placeholder secrets stop startup.

Found a security issue? Please open a private report instead of a public issue.
