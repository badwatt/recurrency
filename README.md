# Recur

> Every expense is a subscription.

A self-hosted subscription and expense tracker. Because every purchase is really a question:
*Would I subscribe to this today?*

---

## Quick Start

```bash
git clone https://github.com/alvaro17f/recur.git
cd recur

# Start all services (frontend, backend, database)
docker compose up -d

# Open in browser
open http://localhost:3000
```

**Demo:** Click "Demo account" on the login page.

---

## Configuration

| Variable | Default | Description |
|---|---|---|
| `JWT_SECRET` | `change-me-in-production` | Secret key for JWT tokens |
| `ENABLE_DEMO` | `true` | Show demo login button |
| `PGPASSWORD` | `recur` | PostgreSQL password |

```bash
docker compose up -d -e JWT_SECRET=your-secret-key
```

---

## Self-Hosting

Pull the latest image:

```bash
docker compose pull
docker compose up -d
```

---

## Stack

**Frontend:** Tauri + React + TypeScript + shadcn/ui + Tailwind + Zustand + Tremor

**Backend:** Rust + Axum + SQLx + JWT + Argon2

**Database:** PostgreSQL

---

## Features

| | |
|---|---|
| 💳 **Subscriptions** | Full CRUD with billing cycles & next-date calculation |
| 🧾 **Transactions** | One-time expenses with categories |
| 📅 **Calendar** | Monthly view of upcoming payments |
| 📊 **Reports** | Trend charts, category breakdown, CSV export |
| 🏷️ **Categories** | Custom with icons, colors & inline creation |
| 🔔 **Notifications** | In-app, email, push |
| 🌙 **Dark Mode** | Light / dark / system — saved to DB |
| 📤 **Grid / List** | Adjustable subscription view |
| 🔄 **Demo Reset** | Restore sample data (24h cooldown) |

---

## Development

```bash
# Start services with hot-reload
./re up

# Run tests
cd backend && cargo test
cd frontend && bun test

# Type check
cd frontend && bun run typecheck
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
