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
4. **Lists and forms** — subscriptions, transactions and debts complete; calendar,
   reports, CSV export, auth and settings done. Money/Admin read-only lists, compact navigation, mobile calendar, chart canvases and the shared modal shell are in; their CRUD, PDF and 18-theme runtime are in; native OS packaging remains pending.
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


## Component migration

The native port follows the Dioxus component boundaries instead of rebuilding
whole pages. Each shared primitive is translated once and page code composes
it; a page must not paint its own button, field, card, badge, empty state or
toolbar.

| Dioxus (`frontend/src/components`) | Native Shell (`nsh/src/components`) | Status |
| --- | --- | --- |
| `button.rs` | `button.rs` (`ButtonProps`, variants, sizes, radius, icon) | Ported |
| `badge.rs` | `badge.rs` | Ported |
| `card.rs` | `card.rs` | Ported |
| `field.rs` | `field.rs` | Ported |
| `input.rs` | `input.rs` | Ported |
| `search_input.rs` | `search_input.rs` | Ported |
| `empty_state.rs` | `empty_state.rs` | Ported |
| `page_header.rs` | `page_header.rs` | Ported |
| `list_toolbar.rs` | `list_toolbar.rs` | Ported |
| `toggle.rs` | `toggle.rs` | Ported |
| `dialog.rs` / `select.rs` | `dialog.rs` (`DialogHost`) | Ported |
| `select_menu.rs` | `select_menu.rs` | Ported inline menu |
| `date_picker.rs` | `date_picker.rs` | Ported custom calendar |
| `checkbox.rs`, `segmented.rs`, `skeleton.rs` | same names | Ported |
| `theme_picker.rs` | `theme.rs` catalogue + Appearance grid | Ported + server sync |
| `web_server_qr.rs` | `components/web_server_qr.rs` | Ported (canvas QR) |
| `backup_restore_card.rs` | Settings data card | Ported |
| `chart.rs`, `toast.rs`, `tooltip.rs` | page-local | Pending |
