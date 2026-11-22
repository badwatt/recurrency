# Dioxus → Native Shell migration

## Goal

Replace the Dioxus WASM frontend with a **Rust Native Shell** app while keeping the
interface, design tokens and behavior of the current product.

The backend stays Axum/PostgreSQL and keeps the same REST contract.

## Branch

`migration/native-shell`

The Dioxus frontend is not removed until the native app has parity. Both trees are
kept side by side so screens can be compared against
`design/native-shell/*.png`.

## Layout

| Path | Purpose |
|---|---|
| `backend/` | unchanged Axum API |
| `frontend/` | Dioxus reference implementation |
| `nsh/` | Native Shell Rust app (independent Cargo workspace) |
| `design/native-shell/` | visual reference captures |

## Phases

1. **Foundation** — NSH crate, theme tokens, fonts, app frame, dashboard shell.
2. **Session** — server URL, login and session restore done; registration, logout and local-mode switch pending.
3. **Dashboard** — data, upcoming payments, ledger and month plan done; hover
   links and mobile layout pending.
4. **Lists and forms** — subscriptions, transactions and debts complete (list,
   search, create, edit, delete); debt payments/history and modal shell pending.
5. **Calendar, reports and settings** — charts, calendar, exports, themes, AI,
   notifications, backup/restore and admin.
6. **Platforms** — web, Linux/macOS/Windows, Android, update flow and CI artifacts.

## Parity contract

- Same type family, weights, sizes and spacing.
- Same color tokens for all 18 themes.
- Same responsive breakpoints and mobile navigation.
- Same copy and interaction states.
- Every phase is compared against the reference captures before the next one.

## Known constraint

Native Shell renders with each platform's native views. Typography metrics and
platform controls can differ by a point or a pixel from the HTML build; the design
tokens and layouts are matched, but cross-platform pixel identity is not a promise
of the framework.
