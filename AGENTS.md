# Recurrency - Agente Specification

## Overview

**Recurrency** is a subscription and expense tracker that treats every expense like a subscription. It helps users budget by viewing recurring costs as ongoing commitments, prompting them to ask: *"Would I subscribe to this today?"*

## Tech Stack

### Frontend
- **Framework:** Dioxus (Rust WASM)
- **UI Library:** dioxus-ui components
- **State Management:** Dioxus signals + use_shared_state
- **Charts:** Recharts
- **Auth:** JWT (handled by Rust backend)
- **Styling:** CSS/Tailwind equivalent via Dioxus

### Backend
- **Language:** Rust (Axum)
- **Database:** PostgreSQL (self-hosted)
- **API:** REST (Axum)

### External Services
- **Email:** Resend
- **Push Notifications:** OneSignal

---

## TDD (Test-Driven Development) - OBLIGATORIO

**Este proyecto requiere TDD para TODOS los cambios.**

### Ciclo Red-Green-Refactor

```
1. RED    → Escribir el test PRIMERO. El test debe fallar.
2. GREEN  → Implementar/alterar código mínimo para que el test pase.
3. REFACTOR → Mejorar código manteniendo tests verdes.
```

### Reglas

- [ ] **SIEMPRE** escribir tests ANTES de implementar features
- [ ] **SIEMPRE** escribir tests ANTES de corregir bugs
- [ ] **SIEMPRE** actualizar tests cuando se modifica funcionalidad existente
- [ ] **SIEMPRE** mantener tests actualizados (no dejar tests rotos o outdated)
- [ ] Coverage mínimo: **70%** en ambos proyectos
- [ ] Tests deben ser deterministas (no flaky)
- [ ] Nombrar tests descriptivamente: `fn test_[subject]_[scenario]_[expected]`

### Ejecución de Tests

```bash
# Backend
cd backend && nix-shell -p pkg-config openssl --run "cargo test"

# Frontend
cd frontend && cargo test

# Coverage (NixOS)
cd backend && nix shell github:NixOS/nixpkgs/nixos-unstable#llvmPackages_22.llvm -c \
  bash -c 'export LLVM_COV=$(which llvm-cov); export LLVM_PROFDATA=$(which llvm-profdata); cargo llvm-cov test'
```

---

## Development

### Workflow

- **TDD (obligatorio)**: Red-Green-Refactor para cada cambio
- **Conventional Commits**: `type(scope): description`
  - Types: feat, fix, docs, style, refactor, test, chore

### Git & Commits

- ⚠️ **IMPORTANT**: Preguntar al usuario antes de hacer commit
- Commit message format: `type(scope): description` (Conventional Commits)
- Types: feat, fix, docs, style, refactor, test, chore

### Code Quality

- **Dead code elimination**: Eliminar siempre código no usado, imports, variables
- No dejar código comentado u obsoleto
- Verificar con `cargo check` / `cargo clippy` antes de terminar

---

## Features

### Core Features

- [x] **CRUD Subscriptions** - Create, edit, delete subscriptions with:
  - Amount and currency (display only)
  - Billing frequency (weekly, monthly, quarterly, yearly)
  - Start date and next billing date
  - Category assignment
  - Active/inactive status
  - Notes

- [x] **CRUD Transactions** - One-time expenses with same metadata

- [x] **Calendar View** - See upcoming payments in a monthly calendar

- [x] **Dashboard** - Monthly summary with:
  - Total recurring costs
  - Total one-time expenses
  - Breakdown by category
  - Comparison with previous month

- [x] **Custom Categories** - User-defined categories with:
  - Name and icon
  - Color

- [x] **Notifications/Reminders**:
  - [x] In-app notifications (toasts, badges)
  - [x] System notifications (OS native)
  - [x] Email reminders (via Resend API)
  - [x] Push notifications (via OneSignal)

- [x] **Statistics & Charts**:
  - Monthly/yearly spending trends
  - Category distribution
  - Historical comparison

- [x] **Data Export**:
  - [x] CSV export
  - [x] PDF report

- [x] **Multi-currency display** - Show amounts in user's chosen currency (no conversion)

- [ ] **Unusual spending alerts** - Flag expenses that exceed normal patterns

### User Management

- [x] User registration and authentication
- [x] JWT token validation in Rust backend
- [x] Session management

### Settings

- [x] Currency preference
- [x] Notification preferences (per channel)
- [x] Category management

---

## Architecture

### Backend Structure (Rust)

```
backend/
├── src/
│   ├── auth/            # JWT validation, handlers
│   ├── categories/     # Category CRUD
│   ├── demo/           # Demo mode
│   ├── email/          # Email service
│   ├── logos/          # Logo search (DuckDuckGo)
│   ├── notifications/  # Push/in-app notifications
│   ├── preferences/    # User preferences
│   ├── reports/        # Reports & statistics
│   ├── shared/        # Shared utilities
│   ├── subscriptions/ # Subscription CRUD
│   ├── transactions/  # Transaction CRUD
│   └── main.rs
├── migrations/          # SQL migrations
└── Cargo.toml
```

### Frontend Structure (Dioxus)

```
frontend/
├── src/
│   ├── auth/            # Auth pages & state
│   ├── components/     # Shared UI components
│   ├── pages/          # Route pages (Dashboard, Calendar, etc.)
│   ├── api/            # API client
│   ├── hooks/          # Shared hooks
│   └── main.rs
├── Cargo.toml
└── dioxensus.toml
```

---

## Database Schema (PostgreSQL)

```sql
-- users
users (
  id UUID PRIMARY KEY,
  email VARCHAR UNIQUE NOT NULL,
  password_hash VARCHAR NOT NULL,
  name VARCHAR,
  currency VARCHAR DEFAULT 'USD',
  verified BOOLEAN DEFAULT false,
  verification_code VARCHAR,
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
  is_system BOOLEAN DEFAULT false,
  is_default BOOLEAN DEFAULT false,
  created_at TIMESTAMP,
  updated_at TIMESTAMP
)

-- subscriptions
subscriptions (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  category_id UUID REFERENCES categories(id),
  name VARCHAR NOT NULL,
  amount DECIMAL NOT NULL,
  currency VARCHAR NOT NULL,
  frequency VARCHAR NOT NULL,
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
  channel VARCHAR NOT NULL,
  enabled BOOLEAN DEFAULT true,
  reminder_days_before INTEGER DEFAULT 3
)

-- notifications
notifications (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  type VARCHAR NOT NULL,
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
- `GET /api/auth/verify?token=` - Verify email
- `POST /api/auth/resend-verification` - Resend verification
- `POST /api/auth/logout` - Logout
- `GET /api/auth/settings?token=` - Get settings
- `PUT /api/auth/settings` - Update settings

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
- `PUT /api/categories/:id/default` - Set as default

### Notifications
- `GET /api/notifications` - List notifications
- `PUT /api/notifications/:id/read` - Mark as read
- `PUT /api/notifications/read-all` - Mark all as read

### Reports
- `GET /api/reports/summary` - Monthly/yearly summary
- `GET /api/reports/trends` - Spending trends
- `GET /api/reports/categories` - Category breakdown

### Demo
- `GET /api/demo/init` - Initialize demo
- `POST /api/demo/reset` - Reset demo data

### Logos
- `GET /api/logos/search?q=` - Search logos

---

## Dependencies

### Backend (Cargo.toml)
- axum (web framework)
- sqlx (PostgreSQL)
- jsonwebtoken
- argon2 (password hashing)
- serde, serde_json
- tokio
- tracing

### Frontend (Cargo.toml)
- dioxus (UI framework)
- reqwest (HTTP client)
- serde, serde_json
- chrono (date handling)

---

## TODO

- [x] Migrate from Tauri to Dioxus
- [x] Setup Dioxus frontend
- [x] Implement auth (registration, login, JWT)
- [x] Implement subscriptions CRUD
- [x] Implement transactions CRUD
- [x] Implement categories CRUD
- [x] Setup notifications (in-app, system, email, push)
- [x] Build dashboard
- [x] Build calendar view
- [x] Implement reports and export
- [x] Write unit tests (TDD)
- [x] Demo mode
- [ ] Add unusual spending alerts
- [ ] Docker Compose setup for PostgreSQL

---

## Notes

- Multi-currency: Display only, no conversion rates API
- Export: Frontend generates CSV/PDF client-side
- Self-hosted: User runs their own PostgreSQL instance
- Notifications: Resend for email, OneSignal for push
