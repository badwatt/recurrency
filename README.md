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

All common tasks are handled through the `./re` script:

| Command | Description |
|---|---|
| `./re up` | Start all services (PostgreSQL, backend, frontend) |
| `./re up --prod` | Start in production mode (disables demo) |
| `./re down` | Stop all services |
| `./re restart` | Restart everything |
| `./re status` | Show which services are running |
| `./re logs` | Tail backend logs |
| `./re setup` | Install dependencies & build |
| `./re test` | Run all tests (frontend + backend) |
| `./re typecheck` | TypeScript check |
| `./re build` | Production build |
| `./re clean` | Remove build artifacts |
| `./re reset` | Reset database (⚠️ deletes all data) |

**Dev mode** (`./re up`) enables demo by default. Use `./re up --prod` to disable it.

Services in dev mode:
- **Frontend:** http://localhost:1420 (Vite dev server with HMR)
- **Backend:** http://localhost:8080 (Axum with auto-rebuild)
- **PostgreSQL:** localhost:5432 (via Docker Compose)

---

## Production (Docker)

```bash
# Pull and run the all-in-one image
docker run -d --name recurrency \
  -p 3000:80 \
  -e JWT_SECRET=your-secret-key \
  -e ENABLE_DEMO=true \
  ghcr.io/alvaro17f/recurrency:latest

# Open in browser
open http://localhost:3000
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
| `JWT_SECRET` | `change-me-in-production` | Secret key for JWT tokens |
| `ENABLE_DEMO` | `true` (dev) / `false` (prod) | Show demo login button |
| `FRONTEND_DIST` | `/frontend/dist` | Path to frontend static files |
| `SMTP_HOST` | — | SMTP server for email verification |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_USER` | — | SMTP username |
| `SMTP_PASSWORD` | — | SMTP password |
| `SMTP_FROM` | — | Sender email address |
| `PGPASSWORD` | `recurrency` | PostgreSQL password |

---

## Stack

| Layer | Tech |
|---|---|
| **Frontend** | React + TypeScript + shadcn/ui + Tailwind + Zustand |
| **Backend** | Rust + Axum + SQLx + JWT + Argon2 |
| **Database** | PostgreSQL |

---

## Features

| | |
|---|---|
| 💳 **Subscriptions** | Full CRUD with billing cycles, next-date calculation, categories & notes |
| 🧾 **Transactions** | One-time expenses with categories & notes |
| 📅 **Calendar** | Monthly view of upcoming payments |
| 📊 **Reports** | Trend charts, category breakdown, CSV export |
| 🏷️ **Categories** | Custom icons, colors & inline creation |
| 📝 **Notes** | Hover to preview, click to edit — on any subscription or transaction |
| 🔔 **Notifications** | In-app, email, push |
| 🌙 **Dark Mode** | Light / dark / system — saved to DB |
| 📤 **Grid / List** | Adjustable subscription view |
| 💱 **Default Currency** | Set in settings, auto-selected on new items |
| 🔄 **Demo Reset** | Restore sample data (24h cooldown) |

---

## Development

```bash
# Start services with hot-reload
./re up

# Run tests
./re test

# Type check
./re typecheck

# Check services
./re status
```

---

## Architecture

```
backend/src/
├── auth/          ├── subscriptions/    ├── transactions/
├── categories/    ├── notifications/    ├── reports/
├── preferences/   ├── demo/             └── shared/

frontend/src/
├── pages/         ├── components/       ├── stores/
├── lib/api/       ├── lib/auth/         └── hooks/
```

---

## License

MIT