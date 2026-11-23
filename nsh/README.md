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

```sh
nsh package nsh --target linux --out target/nsh-package \
  --name Recurrency --id com.badwatt.recurrency --app-version 0.56.0

nsh package nsh --target web --out target/nsh-web \
  --name Recurrency --id com.badwatt.recurrency --app-version 0.56.0
```

The web package is a static site; under a Content-Security-Policy it needs
`script-src 'self' 'wasm-unsafe-eval'` and `img-src 'self' blob: data:`.
Android packaging uses the separate `nsh-android` shell, which provides a
Rust toolchain with the Android targets, JDK 17, GTK/GStreamer host build
dependencies and the SDK/NDK environment. It requires SDK platform 35 and
Build Tools 35 (`sdkmanager "platforms;android-35" "build-tools;35.0.0"`).

```sh
nix develop .#nsh-android -c nsh package nsh --target android --abi arm64-v8a \
  --out target/nsh-android --name Recurrency --id com.badwatt.recurrency \
  --app-version 0.56.0 --build 5600 \
  --icon frontend/assets/icons/clock-1024.png
```

For a store/release APK add the project keystore:

```sh
nix develop .#nsh-android -c nsh package nsh --target android --abi arm64-v8a \
  --out target/nsh-android-release --name Recurrency \
  --id com.badwatt.recurrency --app-version 0.56.0 --build 5600 \
  --icon frontend/assets/icons/clock-1024.png \
  --keystore recurrency-release.jks --key-alias recurrency \
  --keystore-password "$ANDROID_KEYSTORE_PASSWORD"
```

Verify with `apksigner verify --print-certs target/nsh-android-release/Recurrency.apk`.

## Status

- [x] Design tokens: Mocha theme, Cabinet Grotesk / JetBrains Mono assets.
- [x] App frame: top navigation and dashboard shell.
- [x] Server URL, login and session restore.
- [x] Registration, email verification, logout and session restore.
- [x] Dashboard data, upcoming payments and the account/category ledger.
- [x] Responsive top/bottom bars, compact settings navigation and the quick-action FAB.
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
- [x] PDF export, full settings CRUD and the mobile calendar list.
- [x] Settings: 18 themes, currency/rates, dates, delivery channels, summaries, AI, voice, backups, server and admin.
- [ ] Desktop/mobile/web packaging and CI.

Reference captures of the Dioxus interface live in `design/native-shell/`.
The component tiers and reuse rules are documented in `COMPONENTS.md`.


## Components

Pages compose the shared library in `src/components`: avatars, badges, buttons,
cards, checkboxes, confirmations, date pickers, dialogs, empty states, fields,
inputs, the quick-action FAB, list toolbars, page headers, search fields,
segmented controls, select menus, skeletons, toggles, QR cards and typed string
selects. New page code must add a primitive there first and reuse it, matching
the Dioxus component boundaries.

- [x] Privacy mode masks every amount from the top-bar eye control.
- [x] In-app BYOK AI chat dialog with action results.
- [x] Date format and first day of week persisted and applied to lists, pickers and calendar.
- [x] Notes editors for subscriptions and transactions.
- [x] Bulk select/activate/pause/delete for subscriptions and bulk delete for transactions/debts.
- [x] Mobile floating add button on list pages; calendar entries jump to their list.
- [x] Reminder lead-time controls per notification channel.
- [x] Email verification code dialog with resend and automatic sign-in after verification.
- [x] Logo URLs in subscription/transaction forms and cards.
- [x] Searchable IANA timezone picker, dashboard summary schedule, bulk categories and admin delete confirmation.
- [x] 11-provider delivery-channel editor with per-provider fields, test and save.
- [x] Dashboard summary frequency, time and searchable IANA timezone.
- [x] Bulk category moves for transactions and debts.
- [x] Avatar upload, editable email and profile persistence.
- [x] Exchange-rate provider with encrypted Fixer key management.
- [x] Release banner with platform download and the Android APK QR card.
- [x] Per-device voice-engine preference and provider voice-note transcription.
- [x] Shared QR card, avatar badge, FAB menu and dynamic-row input.
- [x] Field-aware sort menus and transaction filters for category, price range and month.
- [x] Status filtering for subscriptions (all/active/inactive).
- [x] Shared filter-popover and sort-menu components reused by both list pages.
- [x] Browser microphone recording with MediaRecorder and provider transcription; file-based voice notes on native.
- [x] Cloud folder sync card with folder picker, manual backup sync, last-sync state and disconnect.
- [x] Shared color palette, entity icons and multiline textarea; account/category colors persist and render in the lists.
- [x] Shared privacy toggle, logo avatar, entity icon and color palette components.
- [x] Multiple annual billing dates with optional per-date amounts.
- [x] Device-local mode: accounts, categories, subscriptions, transactions, debts, calendar, reports, backups and local profile without a server.
