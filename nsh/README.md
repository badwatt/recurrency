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
- [ ] Modal shell, transactions, debts, calendar, reports.
- [ ] Settings, themes, AI, backup and admin.
- [ ] Desktop/mobile/web packaging and CI.

Reference captures of the Dioxus interface live in `design/native-shell/`.
