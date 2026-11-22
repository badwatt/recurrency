# Recurrency — Native Shell

Rust port of the Recurrency interface on [Native Shell](https://nsh.dev/), with no
JavaScript runtime.

This directory is an independent Cargo workspace; the existing `frontend/` stays
in place until every screen reaches visual parity.

## Run

```sh
nsh check nsh
nsh run nsh
```

Linux needs GTK 4.14+, WebKitGTK 6.0 and GStreamer; the repository dev shell now
provides their development packages. The `nsh` CLI itself is installed separately
from <https://nsh.dev/docs/installation>.

## Status

- [x] Design tokens: Mocha theme, Cabinet Grotesk / JetBrains Mono assets.
- [x] App frame: top navigation and dashboard shell.
- [x] Server URL, login and session restore.
- [ ] Registration, logout and local mode.
- [x] Dashboard data, upcoming payments and the account/category ledger.
- [ ] Dashboard hover links and responsive/mobile layout.
- [x] Subscriptions list, search and active/inactive groups.
- [x] Inline create/edit form, pause/resume and delete confirmation.
- [x] Transactions list, monthly groups, search, create, edit and delete.
- [x] Debts list, search, create/edit/delete, payments and history.
- [x] Calendar month grid with scheduled and one-time payments.
- [x] Reports summary, category split, spending trends and CSV export.
- [x] Settings account, appearance, general, notifications, AI, backup, server and about.
- [x] Registration, login, top-bar account access, logout, CSV export and server backups.
- [x] Accounts and categories CRUD, plus admin role, user deletion and registration toggles in Settings.
- [x] Compact top bar, bottom navigation and wrapping page grids.
- [x] Shared modal shell for create/edit/payment/history forms.
- [ ] PDF export, full settings CRUD and calendar mobile view.
- [ ] Settings, themes, AI, backup and admin.
- [ ] Desktop/mobile/web packaging and CI.

Reference captures of the Dioxus interface live in `design/native-shell/`.


## Components

Pages compose the shared library in `src/components`: buttons, badges, cards,
fields, inputs, search fields, empty states, page headers, list toolbars,
toggles and the dialog host. New page code should add a primitive there first
and reuse it, matching the Dioxus component boundaries.

- [x] Privacy mode masks every amount from the top-bar eye control.
- [x] In-app BYOK AI chat dialog with action results.
- [x] Date format and first day of week persisted and applied to lists, pickers and calendar.
- [x] Notes editors for subscriptions and transactions.
- [x] Bulk select/activate/pause/delete for subscriptions and bulk delete for transactions/debts.
- [x] Mobile floating add button on list pages; calendar entries jump to their list.
- [x] Reminder lead-time controls per notification channel.
- [x] Email verification code dialog with resend and automatic sign-in after verification.
- [x] Logo URLs in subscription/transaction forms and cards.
