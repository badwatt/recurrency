# Recurrency

> Every expense is a subscription.

A self-hosted subscription and expense tracker. Because every purchase is really a question:
*Would I subscribe to this today?*

![Recurrency Dashboard](assets/dashboard.png)

---

## Quick Start

```bash
git clone https://github.com/alvaro17f/recurrency.git
cd recurrency

# Start all services (frontend, backend, database)
./re up

# Open in browser
open http://localhost:1420
```

**Demo:** Click "Demo account" on the login page.

---

## The `./re` Script

| Command | Description |
|---|---|
| `./re up` | Start all services |
| `./re up --prod` | Production mode (disables demo) |
| `./re down` | Stop all services |
| `./re restart` | Restart everything |
| `./re status` | Show running services |
| `./re logs` | Tail backend logs |
| `./re setup` | Install dependencies & build |
| `./re test` | Run all tests |
| `./re typecheck` | Type check |
| `./re build` | Production build |
| `./re clean` | Remove build artifacts |
| `./re reset` | Reset database (⚠️ deletes all data) |

**Services:**
- **Frontend:** http://localhost:1420 (Dioxus WASM)
- **Backend:** http://localhost:8080 (Axum)
- **PostgreSQL:** localhost:5432

---

## Production (Docker)

```bash
docker run -d --name recurrency \
  -p 3000:80 \
  -e JWT_SECRET=your-secret-key \
  -e ENABLE_DEMO=true \
  ghcr.io/alvaro17f/recurrency:latest
```

Or with `docker compose`:

```bash
docker compose pull
docker compose up -d
```

---

## Configuration

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | PostgreSQL connection |
| `JWT_SECRET` | `change-me-in-production` | Secret for JWT tokens |
| `ENABLE_DEMO` | `true` | Show demo login button |
| `FRONTEND_DIST` | `/frontend/dist` | Path to frontend static files |
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
| 💳 **Subscriptions** | CRUD with billing cycles, next-date calc, categories, notes |
| 🧾 **Transactions** | One-time expenses with categories |
| 📅 **Calendar** | Monthly view of upcoming payments |
| 📊 **Reports** | Trend charts, category breakdown, CSV/PDF export |
| 🏷️ **Categories** | Custom icons, colors, inline creation |
| 🔔 **Notifications** | In-app, email, push (OneSignal) |
| 🌙 **Dark Mode** | Light / dark / system |
| 📤 **Grid / List** | Adjustable view |
| 💱 **Currencies** | Multi-currency display |
| 🔄 **Demo Reset** | Restore sample data |
| 📈 **Statistics** | Spending trends, category distribution |
| 🔔 **Reminders** | Configurable notification days |

---

## Test Coverage

| Project | Lines | Functions |
|---|---|---|
| **Backend** | **82%** | **79%** |
| **Frontend** | **78%** | **64%** |

```bash
# Backend tests
cd backend && cargo test

# Frontend tests
cd frontend && cargo test

# Coverage
cd backend && nix-shell -p pkg-config openssl --run "cargo llvm-cov test"
```

---

## Development

```bash
# Start services
./re up

# Run tests
./re test

# Check coverage
cd backend && nix-shell -p pkg-config openssl --run "cargo llvm-cov report"
```

---

## Architecture

```
backend/src/
├── auth/          # JWT, registration, login
├── subscriptions/ # Subscription CRUD
├── transactions/  # Transaction CRUD
├── categories/     # Category CRUD
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

## License

MIT