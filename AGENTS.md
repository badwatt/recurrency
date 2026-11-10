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

## Reglas Absolutas

- ⛔ **NUNCA** instalar herramientas en el sistema. Usar `flake.nix` para añadir dependencias en la shell.
- ⛔ **NUNCA** usar `unsafe`. Cero tolerancia.
- ⛔ **SIEMPRE** mantener dependencias lo más actualizadas posible.
- ⛔ **SIEMPRE** 100% coverage en tests. Sin excepciones.
- ⛔ **SIEMPRE** TDD. Red-Green-Refactor. Test primero, código después.
- ⛔ **NUNCA** errores ni warnings. NINGUNO. Incluye `cargo clippy`.
- ⛔ **SIEMPRE** usar la skill `rust-best-practices`.
- ⛔ **NUNCA** hacer bump de `VERSION` sin consultar al usuario. Dispara CI actions.
- ⛔ **SIEMPRE** usar **Semver** (`MAJOR.MINOR.PATCH`) para las versiones del proyecto: `MAJOR` para cambios incompatibles, `MINOR` para funcionalidad nueva compatible y `PATCH` para correcciones o refactors compatibles.
- ⛔ **SIEMPRE** usar `act` localmente para verificar CI actions antes de push.
- ⛔ **SIEMPRE** que sea posible elimina dependencias.
- ⛔ Trata de tener el **MÍNIMO** número de dependencias en el proyecto.
- ⛔ **NUNCA** valores hardcodeados. Usar constantes, configuración, o theme tokens. Colores, textos, URLs, dimensiones — todo configurables.
- ⛔ **SIEMPRE** extraer lógica repetitive a componentes reutilizables. Si se repite 2 veces, extraer. Componentes > copy-paste.

---

## TDD (Test-Driven Development) - OBLIGATORIO SIN EXCEPCIONES

**CADA commit debe tener tests.
CADA cambio empieza con test ROJO.
NUNCA código sin test.**

### Reglas de hierro

- ⛔ **SIEMPRE** escribir test ANTES de implementar. Sin excepciones.
- ⛔ **NUNCA** modificar código sin test que verifique el cambio.
- ⛔ **NUNCA** commit de código sin tests incluidos.
- ⛔ **SIEMPRE** test rojo primero. Luego código. Luego refactor.
- ⛔ **NUNCA** avanzar a siguiente feature si tests actuales no pasan.
- ⛔ **NUNCA** bajar cobertura global con cambios nuevos.

### Ciclo Red-Green-Refactor (obligatorio)

```
1. RED    → Escribir test. Ejecutar. Debe FALLAR.
2. GREEN  → Escribir MÍNIMO código para que pase.
3. REFACTOR → Limpiar código. Tests siguen verdes.
4. REPETIR  → Siguiente test.
```

### Cobertura: 100%

| Proyecto | Líneas | Estado |
|----------|--------|--------|
| Backend  | **86.88%** | ✅ 1294 tests |
| Frontend | **80.22%** | ✅ 1491 tests |

**Reglas:**
- **100%** líneas para cualquier módulo nuevo. Sin excepciones.
- **100%** para módulos críticos (auth, payments, seguridad)
- **100%** condicionales en handlers
- Si un cambio baja cobertura global → agregar más tests antes de commit

### Convenciones de tests

```rust
#[test]
fn validate_empty_name_fails() {
    let result = validate_subscription_form(&empty_sub);
    assert!(result.is_err());
}
```

- Nombre: `fn test_[subject]_[scenario]_[expected]()`
- Un concepto por test. Un assert por test.
- Tests deterministas (no aleatorios, no timestamps, no network)

### Pre-commit hook obligatorio

```bash
# Antes de CUALQUIER commit, ejecutar:
nix develop .. -c cargo test --lib

# Si falla → arreglar antes de commit.
# Si pasa → commit permitido.
```

### Qué es testeable / No testeable

| Testeable ✅ | No testeable ⛔ |
|-------------|----------------|
| Pure functions (validate, format, calc) | RSX macro-generated VDOM |
| Model operations (serde, clone, eq) | async DB operations sin mock |
| Trait implementations | WASM-specific (localStorage) |
| Mock services | Integration sin PostgreSQL |
| Handler validation (request/response) | — |

Cuando NO hay test, documentar:
```rust
// No test: async DB operation. Covered by integration tests.
// No test: RSX macro. Requires Dioxus WASM runtime.
```

### Ejecución

```bash
# Tests
nix develop .. -c cargo test --lib

# Coverage
nix develop .. -c cargo llvm-cov test --lib
```

---

## Architecture

### Backend Structure (Rust)

```
backend/
├── src/
│   ├── auth/            # JWT validation, handlers, service trait
│   ├── categories/     # Category CRUD, service trait
│   ├── demo/           # Demo mode
│   ├── email/          # Email service
│   ├── logos/          # Logo search (DuckDuckGo)
│   ├── notifications/  # Push/in-app notifications, service trait
│   ├── preferences/    # User preferences, service trait
│   ├── reports/        # Reports & statistics, service trait
│   ├── shared/        # Shared utilities, mocks, app_state
│   ├── subscriptions/ # Subscription CRUD, service trait
│   ├── transactions/  # Transaction CRUD, service trait
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
│   ├── pages/          # Route pages
│   ├── api/            # API client
│   ├── state/          # Signals/state
│   └── main.rs
├── Cargo.toml
└── dioxensus.toml
```

---

## Trait Injection (Obligatorio)

Cada servicio DEBE tener trait para testabilidad:

```rust
#[async_trait]
pub trait SubscriptionServiceTrait: Send + Sync {
    async fn create(&self, user_id: Uuid, req: CreateSubscriptionRequest) -> AppResult<SubscriptionResponse>;
    async fn get_all(&self, user_id: Uuid) -> AppResult<Vec<SubscriptionResponse>>;
    async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> AppResult<SubscriptionResponse>;
    async fn update(&self, user_id: Uuid, id: Uuid, req: UpdateSubscriptionRequest) -> AppResult<SubscriptionResponse>;
    async fn delete(&self, user_id: Uuid, id: Uuid) -> AppResult<()>;
}

pub struct SqlxSubscriptionService {
    pool: PgPool,
}

impl SqlxSubscriptionService {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl SubscriptionServiceTrait for SqlxSubscriptionService {
    async fn create(&self, user_id: Uuid, req: CreateSubscriptionRequest) -> AppResult<SubscriptionResponse> {
        // DB operations...
    }
    // ...
}
```

### AppState con traits

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub subscription_service: Arc<dyn SubscriptionServiceTrait>,
    pub transaction_service: Arc<dyn TransactionServiceTrait>,
    pub category_service: Arc<dyn CategoryServiceTrait>,
    pub notification_service: Arc<dyn NotificationServiceTrait>,
    pub report_service: Arc<dyn ReportServiceTrait>,
    pub preferences_service: Arc<dyn PreferencesServiceTrait>,
}
```

### Mock services para tests

```rust
pub struct MockSubscriptionService {
    pub should_fail: bool,
    pub mock_subscriptions: Mutex<Vec<Subscription>>,
}

#[async_trait]
impl SubscriptionServiceTrait for MockSubscriptionService {
    async fn create(&self, user_id: Uuid, req: CreateSubscriptionRequest) -> AppResult<SubscriptionResponse> {
        if self.should_fail {
            return Err(AppError::Internal("mock error".into()));
        }
        let sub = Subscription { /* ... */ };
        self.mock_subscriptions.lock().unwrap().push(sub.clone());
        Ok(sub.into())
    }
    // ...
}
```

Mock services viven en `src/shared/mocks.rs`.

---

## Integration Tests

### Requisitos

- Docker Compose con PostgreSQL para tests de integración
- Tests en `tests/integration/` o módulo `integration_tests`

### Docker Compose (tests)

```yaml
# docker/docker-compose.test.yml
version: '3.8'
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: test
      POSTGRES_PASSWORD: test
      POSTGRES_DB: recurrency_test
    ports:
      - "5433:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U test"]
      interval: 5s
      timeout: 5s
      retries: 5
```

### Ejecución

```bash
# Iniciar DB de test
docker compose -f docker/docker-compose.test.yml up -d

# Variable de entorno
export DATABASE_URL="postgres://test:test@localhost:5433/recurrency_test"

# Ejecutar migraciones
nix develop .. -c sqlx migrate run

# Tests de integración
nix develop .. -c cargo test --test integration_tests
```

### Coverage con integración

```bash
# Unit tests solos (rápido)
nix develop .. -c cargo llvm-cov test --lib

# Unit + integration (completo)
nix develop .. -c cargo llvm-cov test
```

---

## CI/CD (GitHub Actions)

### `.github/workflows/test.yml`

```yaml
name: Test & Coverage
on: [push, pull_request]
jobs:
  backend:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v25
      - run: nix develop .. -c cargo test --lib
      - run: nix develop .. -c cargo llvm-cov test --lib
      - name: Verify coverage
        run: |
          nix develop .. -c cargo llvm-cov report | grep "TOTAL" | awk '{if($4 < 80) exit 1}'
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v25
      - run: cd frontend && nix develop .. -c cargo test
      - run: cd frontend && nix develop .. -c cargo llvm-cov test
      - name: Verify coverage
        run: |
          cd frontend && nix develop .. -c cargo llvm-cov report | grep "TOTAL" | awk '{if($4 < 80) exit 1}'
```

---

## Git & Commits

### Commit messages

- **Auto-commit tests**: SÍ permitido si tests pasan y coverage ≥ 80%
- **Push**: PREGUNTAR siempre antes de `git push`
- ⛔ **NUNCA** bump de `VERSION` sin consultar al usuario
- ⛔ **SIEMPRE** usar `act` localmente antes de push para verificar CI
- Format: `type(scope): description`
- Types: `feat`, `fix`, `test`, `refactor`, `docs`, `chore`

### Workflow

```bash
# 1. Escribir test (rojo)
nix develop .. -c cargo test --lib  # falla

# 2. Implementar (verde)
nix develop .. -c cargo test --lib  # pasa

# 3. Commit (auto-permitido)
git add -A && git commit -m "feat(scope): description"

# 4. Push → PREGUNTAR usuario
# git push  (no sin permiso)
```

---

## Code Quality

- **Dead code elimination**: Eliminar siempre código no usado
- No dejar código comentado u obsoleto
- ⛔ **NINGÚN error ni warning**. Incluye `cargo clippy`. Cero tolerancia.
- `cargo fmt` antes de commit
- ⛔ **NUNCA** `unsafe`. Refactorizar para evitarlo.
- ⛔ **NUNCA** instalar herramientas en el sistema. Usar `flake.nix`.
- ⛔ **NUNCA** bump de `VERSION` sin consultar al usuario.
- ⛔ **SIEMPRE** usar `act` localmente para verificar CI actions.
- ⛔ **SIEMPRE** usar skill `rust-best-practices`.
- ⛔ **SIEMPRE** mantener dependencias actualizadas.
- ⛔ **SIEMPRE** eliminar dependencias innecesarias.
- ⛔ Mínimo número de dependencias posible en el proyecto.

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
- [x] Docker Compose setup for PostgreSQL tests
- [x] GitHub Actions CI/CD
- [x] Complete trait injection in all service modules

---

## Notes

- Multi-currency: Display only, no conversion rates API
- Export: Frontend generates CSV/PDF client-side
- Self-hosted: User runs their own PostgreSQL instance
- Notifications: Resend for email, OneSignal for push
- Coverage: `cargo llvm-cov` via `nix develop`
