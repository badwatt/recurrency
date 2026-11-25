<p align="center">
  <img src="frontend/assets/icons/clock-1024.png" width="96" alt="Recurrency">
</p>

<h1 align="center">Recurrency</h1>

<p align="center">
  <strong>Every expense is a subscription.</strong><br>
  A calm, self-hosted tracker for recurring payments, one-time expenses and debts.
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-orange?logo=rust&logoColor=white">
  <img alt="Dioxus" src="https://img.shields.io/badge/UI-Dioxus-6f42c1">
  <img alt="PostgreSQL" src="https://img.shields.io/badge/Database-PostgreSQL-336791?logo=postgresql&logoColor=white">
  <img alt="Docker" src="https://img.shields.io/badge/Deploy-Docker-2496ed?logo=docker&logoColor=white">
</p>

<p align="center">
  <a href="#quick-start">Quick start</a> ·
  <a href="#screenshots">Screenshots</a> ·
  <a href="#features">Features</a> ·
  <a href="#configuration">Configuration</a> ·
  <a href="#development">Development</a>
</p>

---

Recurrency shows what your commitments really cost. Instead of treating a
subscription as a single purchase, it keeps the next charge, the account it
leaves from and the months ahead in one place.

It answers three questions without a spreadsheet:

- **What do I need to transfer this month?**
- **What is still pending from today?**
- **Would I subscribe to this again today?**

The stack is intentionally boring to run: **Rust + Axum**, **PostgreSQL** and a
**Dioxus** web frontend, shipped as one all-in-one Docker image with an optional
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

Open **http://localhost**. The same binary serves the web UI and the API:

| Service | URL |
|---|---|
| Web UI | `http://localhost` |
| API | `http://localhost/api` |
| API (direct) | `http://localhost:8080/api` |

The image bundles **PostgreSQL 18**. Set `DATABASE_URL` to use an external
PostgreSQL instead; the entrypoint then skips the bundled database.

> [!IMPORTANT]
> The container fails fast if migrations or the database fail. It exits instead
> of serving an empty frontend, so Docker can restart it or show the error.

> [!TIP]
> Pin a release instead of `latest` in production:
> `ghcr.io/badwatt/recurrency:X.Y.Z`.

### From source

Nix is the only prerequisite.

```bash
git clone https://github.com/badwatt/recurrency.git
cd recurrency
nix develop
cargo up
```

`cargo up` starts PostgreSQL, the Axum backend and the Dioxus dev server.

| Service | URL |
|---|---|
| Web UI | `http://localhost:1420` |
| API | `http://localhost:8080/api` |
| PostgreSQL | `localhost:5432` |

Press `Ctrl+C` to stop everything.

> [!NOTE]
> Demo mode is optional. The production image ships with it disabled
> (`ENABLE_DEMO=false`); `cargo up` enables it locally.

---

## Screenshots

| Dashboard | Subscriptions |
|:---:|:---:|
| ![Dashboard](assets/dashboard.png) | ![Subscriptions](assets/subscriptions.png) |

| Calendar | Reports |
|:---:|:---:|
| ![Calendar](assets/calendar.png) | ![Reports](assets/reports.png) |

The dashboard is a monthly transfer ledger: what each account needs, what is
covered and what is still pending. Subscriptions keep the account, category,
logo and next charge visible; the calendar puts every charge on a timeline;
reports turn the same data into trends and category breakdowns.

---

## Features

- **Monthly plan** — transfer total, pending amount and budget progress per
  account.
- **Recurring payments** — daily, weekly, monthly, yearly or custom intervals,
  including “last day of the month”.
- **One-time expenses** — transactions that share the same accounts, categories
  and reports as subscriptions.
- **Debts** — manual balances or automatic month-by-month payments.
- **Calendar** — every charge and transaction, with a day popup and filters.
- **Reports** — spending trends, category breakdown and CSV/PDF export.
- **Multi-currency** — all active ISO 4217 currencies with daily exchange rates.
- **Notifications** — email, ntfy, Gotify, Telegram, Discord, Slack, webhooks
  and more, plus a scheduled dashboard summary.
- **AI assistant** — bring your own OpenAI-compatible key; keys stay encrypted
  on the server.
- **Privacy mode** — mask every amount with one click.
- **Real devices** — responsive web UI, desktop bundles and an Android APK.
- **Local mode** — opt-in, device-local storage without a server.
- **Backups** — automatic PostgreSQL dumps, validated restores and JSON export.

---

## How it fits together

```text
┌──────────────────────────────┐
│  Browser / desktop / Android │
│  Dioxus UI · local mode      │
└──────────────┬───────────────┘
               │ HTTP / JSON
┌──────────────▼───────────────┐
│  Axum API                    │
│  auth · features · reports   │
└──────────────┬───────────────┘
               │ SQL
┌──────────────▼───────────────┐
│  PostgreSQL 18               │
│  users · money · schedules   │
└──────────────────────────────┘
```

The backend is organized by feature (`auth`, `accounts`, `subscriptions`,
`transactions`, `debts`, `notifications`, `reports`), with shared database and
auth helpers in `shared`.

---

## Configuration

The image works with no configuration: the first boot generates a JWT secret,
creates the schema and stores the secret in the database.

| Variable | Default | Purpose |
|---|---|---|
| `DATABASE_URL` | bundled PostgreSQL | External PostgreSQL connection |
| `APP_PORT` | `80` in Docker, `8080` locally | Listen port |
| `JWT_SECRET` | generated in DB | Override the signing secret |
| `ENABLE_DEMO` | `false` | Show the demo login |
| `ENABLE_WEB` | `true` | Serve the embedded UI; `false` for headless |
| `BACKUP_ENABLED` | `true` | Daily logical backups |
| `BACKUP_DIR` | `/var/lib/recurrency/backups` | Backup directory |
| `BACKUP_RETENTION_DAYS` | `7` | Days to keep dumps |
| `CORS_ALLOWED_ORIGIN` | `*` | Restrict browser requests |
| `RECURRENCY_DATA_DIR` | `recurrency-data` | Standalone binary database directory |

> [!WARNING]
> PostgreSQL 18 uses `/var/lib/postgresql/18/main`. A volume created by the old
> PostgreSQL 15 image needs a `pg_dump`/restore before upgrading. Keep the old
> volume until the restored instance is verified.

---

## Standalone binary

The same application can run as one native binary with the frontend embedded:

```bash
nix develop
cargo build-app
./target/bin/recurrency --help
```

```bash
# API + web UI
DATABASE_URL="postgres://recurrency:recurrency@localhost:5432/recurrency" \
  ./target/bin/recurrency serve

# API only
ENABLE_WEB=false ./target/bin/recurrency serve
```

Without `DATABASE_URL`, the binary starts a private PostgreSQL under
`./recurrency-data` (override with `RECURRENCY_DATA_DIR`), creates the schema on
first run and stops the database on `Ctrl+C`/`SIGTERM`.

Useful commands:

```bash
./target/bin/recurrency migrate --database-url "$DATABASE_URL"
./target/bin/recurrency backup --database-url "$DATABASE_URL"
./target/bin/recurrency doctor
./target/bin/recurrency admin list
./target/bin/recurrency admin create --email admin@example.com --password "$PASSWORD" --name Admin --admin
```

`recurrency backup` creates a validated logical dump and applies the retention
window. The container runs it daily after `/api/ready` succeeds.

---

## Development

All commands run inside the Nix shell.

| Task | Command |
|---|---|
| Start everything | `cargo up` |
| Stop everything | `cargo down` |
| Database only | `cargo db` |
| Reset database | `cargo reset` |
| Backend only | `cargo backend` |
| Frontend only | `cargo frontend` |
| Tests | `cargo test-all` |
| Format | `cargo fmt-all` |
| Clippy | `cargo clippy-all` |
| Coverage | `cargo coverage` |
| Tailwind CSS | `cargo css` |
| Android APK | `cargo build-android` |
| Desktop bundle | `cargo build-desktop` |

Artifacts land in `target/bin/`.

### Project layout

```text
backend/src/
├── auth/           JWT, registration, login
├── accounts/       mandatory budgeting accounts
├── subscriptions/  recurrence engine and CRUD
├── transactions/   one-time expenses
├── categories/     optional categories and icons
├── currency/       exchange rates and conversion
├── debts/          manual and automatic debts
├── notifications/  delivery channels and scheduler
├── reports/        trends, breakdowns and exports
└── shared/         database, auth extractor, app state

frontend/src/
├── pages/          route pages
├── components/     reusable UI
├── api/            HTTP client
├── state/          signals and shared state
├── layout/         app and settings shells
├── storage/        local/offline storage
└── theme/          themes and palette tokens
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
```

CI enforces **85% backend** and **76% frontend** line coverage.

### macOS and Gatekeeper

Release DMGs are ad-hoc signed, not notarized. If macOS says the app is damaged
after installing, remove the quarantine flag:

```bash
xattr -dr com.apple.quarantine /Applications/Recurrency.app
```

### Releasing

A release starts when `version` changes in the workspace `Cargo.toml`. The
workflow derives the `v<version>` tag, verifies the project and publishes the
artifacts. Re-running it is idempotent.

---

## Security

- Passwords are hashed; sessions use JWT.
- AI and delivery secrets are encrypted at rest.
- Demo credentials are never administrators and are excluded from backups.
- Put Recurrency behind HTTPS when exposing it outside your network.
- Keep database and backup volumes private; they contain financial data.
