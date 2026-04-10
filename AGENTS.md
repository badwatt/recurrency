# Recurrency - Agente Specification

## Overview

**Recurrency** is a subscription and expense tracker that treats every expense like a subscription. It helps users budget by viewing recurring costs as ongoing commitments, prompting them to ask: *"Would I subscribe to this today?"*

## Tech Stack

### Frontend
- **Framework:** Tauri + React (TypeScript)
- **UI Library:** shadcn/ui
- **State Management:** Zustand
- **Charts:** Tremor
- **Auth:** Better Auth (JWT validation in Rust backend)
- **Styling:** Tailwind CSS

### Backend
- **Language:** Rust
- **Framework:** TBD (Axum, Actix, etc.)
- **Database:** PostgreSQL (Docker Compose, self-hosted)
- **API:** REST

### External Services
- **Email:** Resend
- **Push Notifications:** OneSignal

## Project Type

Self-hosted application for personal use.

---

## Features

### Core Features

- [ ] **CRUD Subscriptions** - Create, edit, delete subscriptions with:
  - Amount and currency (display only)
  - Billing frequency (weekly, monthly, quarterly, yearly)
  - Start date and next billing date
  - Category assignment
  - Active/inactive status
  - Notes

- [ ] **CRUD Transactions** - One-time expenses with same metadata

- [ ] **Calendar View** - See upcoming payments in a monthly calendar

- [ ] **Dashboard** - Monthly summary with:
  - Total recurring costs
  - Total one-time expenses
  - Breakdown by category
  - Comparison with previous month

- [ ] **Custom Categories** - User-defined categories with:
  - Name and icon
  - Color

- [ ] **Notifications/Reminders**:
  - [ ] In-app notifications (toasts, badges)
  - [ ] System notifications (OS native via Tauri)
  - [ ] Email reminders (via Resend API)
  - [ ] Push notifications (via OneSignal)

- [ ] **Statistics & Charts**:
  - Monthly/yearly spending trends
  - Category distribution
  - Historical comparison

- [ ] **Data Export**:
  - [ ] CSV export (generated in frontend)
  - [ ] PDF report (generated in frontend)

- [ ] **Multi-currency display** - Show amounts in user's chosen currency (no conversion)

- [ ] **Unusual spending alerts** - Flag expenses that exceed normal patterns

### User Management

- [ ] User registration and authentication
- [ ] JWT token validation in Rust backend
- [ ] Session management

### Settings

- [ ] Currency preference
- [ ] Notification preferences (per channel)
- [ ] Category management

---

## Architecture

### Domain-Driven Design (DDD)

```
src/
├── auth/                 # Auth bounded context
│   ├── domain/          # Entities, value objects
│   ├── application/     # Use cases, services
│   └── infrastructure/  # DB adapters, external services
│
├── subscriptions/        # Subscriptions bounded context
│   ├── domain/
│   ├── application/
│   └── infrastructure/
│
├── transactions/         # Transactions bounded context
│   ├── domain/
│   ├── application/
│   └── infrastructure/
│
├── categories/          # Categories bounded context
│   ├── domain/
│   ├── application/
│   └── infrastructure/
│
├── notifications/       # Notifications bounded context
│   ├── domain/
│   ├── application/
│   └── infrastructure/
│
├── reports/             # Reports bounded context
│   ├── domain/
│   ├── application/
│   └── infrastructure/
│
└── shared/              # Shared kernel
    ├── domain/          # Common entities, value objects
    └── application/     # Common services
```

### Backend Structure (Rust)

```
backend/
├── src/
│   ├── auth/            # JWT validation, middleware
│   ├── subscriptions/
│   ├── transactions/
│   ├── categories/
│   ├── notifications/
│   ├── reports/
│   └── shared/          # Common utilities
│
├── migrations/          # SQL migrations
└── Cargo.toml
```

---

## Development

### Workflow

- **TDD (Test-Driven Development)**
  - Write tests first (unit tests)
  - Use red-green-refactor cycle
  - Target high coverage in domain logic

- **Conventional Commits**
  - Format: `type(scope): description`
  - Types: feat, fix, docs, style, refactor, test, chore

- **Test Requirement**
  - ⚠️ **IMPORTANT**: After every change, run tests and verify they pass:
    - Backend: `cd backend && cargo test`
    - Frontend: `cd frontend && bun test`
  - Run `cargo check` / `tsc --noEmit` to verify no compilation errors before finishing

- **Git & Commits**
  - ⚠️ **IMPORTANT**: Always ask the user before committing changes
  - Never push to remote (user pushes manually if desired)
  - Commit message format: `type(scope): description` (Conventional Commits)
  - Types: feat, fix, docs, style, refactor, test, chore

### Code Quality

- Linting and formatting (ESLint, Prettier for frontend; Clippy, fmt for Rust)
- Type safety (strict TypeScript, Rust types)
- Pull request reviews required
- **Dead code elimination**: Always remove unused code, functions, imports, and variables when refactoring or replacing functionality. Do not leave commented-out code or obsolete logic.

---

## Database Schema (PostgreSQL)

### Tables

```sql
-- users
users (
  id UUID PRIMARY KEY,
  email VARCHAR UNIQUE NOT NULL,
  password_hash VARCHAR NOT NULL,
  currency VARCHAR DEFAULT 'USD',
  created_at TIMESTAMP,
  updated_at TIMESTAMP
)

-- categories
categories (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  name VARCHAR NOT NULL,
  icon VARCHAR,
  color VARCHAR,
  created_at TIMESTAMP
)

-- subscriptions
subscriptions (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  category_id UUID REFERENCES categories(id),
  name VARCHAR NOT NULL,
  amount DECIMAL NOT NULL,
  currency VARCHAR NOT NULL,
  frequency VARCHAR NOT NULL, -- weekly, monthly, quarterly, yearly
  start_date DATE NOT NULL,
  next_billing_date DATE,
  is_active BOOLEAN DEFAULT true,
  notes TEXT,
  created_at TIMESTAMP,
  updated_at TIMESTAMP
)

-- transactions
transactions (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  category_id UUID REFERENCES categories(id),
  name VARCHAR NOT NULL,
  amount DECIMAL NOT NULL,
  currency VARCHAR NOT NULL,
  date DATE NOT NULL,
  notes TEXT,
  created_at TIMESTAMP
)

-- notification_preferences
notification_preferences (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  channel VARCHAR NOT NULL, -- in_app, system, email, push
  enabled BOOLEAN DEFAULT true,
  reminder_days_before INTEGER DEFAULT 3
)

-- notifications
notifications (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  type VARCHAR NOT NULL, -- upcoming_payment, unusual_spending, etc.
  title VARCHAR NOT NULL,
  message TEXT,
  is_read BOOLEAN DEFAULT false,
  created_at TIMESTAMP
)
```

---

## API Endpoints

### Auth
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login
- `POST /api/auth/verify` - Verify JWT token
- `POST /api/auth/logout` - Logout

### Subscriptions
- `GET /api/subscriptions` - List subscriptions
- `POST /api/subscriptions` - Create subscription
- `PUT /api/subscriptions/:id` - Update subscription
- `DELETE /api/subscriptions/:id` - Delete subscription

### Transactions
- `GET /api/transactions` - List transactions
- `POST /api/transactions` - Create transaction
- `PUT /api/transactions/:id` - Update transaction
- `DELETE /api/transactions/:id` - Delete transaction

### Categories
- `GET /api/categories` - List categories
- `POST /api/categories` - Create category
- `PUT /api/categories/:id` - Update category
- `DELETE /api/categories/:id` - Delete category

### Notifications
- `GET /api/notifications` - List notifications
- `PUT /api/notifications/:id/read` - Mark as read
- `PUT /api/notifications/read-all` - Mark all as read

### Settings
- `GET /api/settings` - Get user settings
- `PUT /api/settings` - Update settings
- `GET /api/settings/notifications` - Get notification preferences
- `PUT /api/settings/notifications` - Update notification preferences

### Reports
- `GET /api/reports/summary` - Monthly/yearly summary
- `GET /api/reports/trends` - Spending trends
- `GET /api/reports/categories` - Category breakdown

---

## Dependencies

### Frontend (package.json)
- @tauri-apps/api
- react, react-dom
- better-auth
- zustand
- @tremor/react
- shadcn/ui components
- tailwindcss
- date-fns
- recharts (backup if tremor insufficient)

### Backend (Cargo.toml)
- axum or actix-web
- sqlx (with PostgreSQL)
- jsonwebtoken
- argon2 (password hashing)
- serde, serde_json
- tokio
- tracing

---

## TODO

- [x] Initialize Tauri project
- [x] Setup React frontend with shadcn/ui
- [x] Configure Zustand stores
- [x] Setup Rust backend with PostgreSQL
- [x] Implement Better Auth + JWT validation (Rust backend)
- [x] Implement Better Auth + JWT validation (frontend connected to Rust backend)
- [x] Create DDD folder structure
- [x] Write domain entities
- [x] Implement subscriptions CRUD
- [x] Implement transactions CRUD
- [x] Implement categories CRUD
- [x] Setup notifications (in-app, system, email, push)
- [ ] Build dashboard with Tremor charts
- [x] Build calendar view
- [x] Implement reports and export
- [ ] Add unusual spending alerts
- [x] Write unit tests (TDD)
- [ ] Docker Compose setup for PostgreSQL

---

## Notes

- Multi-currency: Display only, no conversion rates API
- Export: Frontend generates CSV/PDF client-side
- Self-hosted: User runs their own PostgreSQL instance
- Notifications: Resend for email, OneSignal for push, Tauri for system
