# Component architecture

The Native Shell port mirrors `frontend/src/components` and follows atomic
design. Pages compose modules from `src/components`; they do not paint their
own buttons, inputs, cards, dialogs or selects.

```
components/
├── atoms/       smallest styled primitives
├── molecules/   atoms composed into one interaction
└── organisms/   complete surfaces and flows
```

## Atoms — `components/atoms/`

| Component | Purpose |
| --- | --- |
| `avatar` | circular user image or initials |
| `badge` | neutral/primary/success/destructive status label |
| `button` | every press target: variants, sizes, alignment and radius |
| `checkbox` | labelled binary control |
| `color_picker` | account/category palette and hex value |
| `input` | single-line and embedded inputs |
| `logo_avatar` | remote brand tile with fallback |
| `segmented` | two-or-more exclusive options |
| `skeleton` | loading placeholder |
| `textarea` | multiline notes surface |
| `toggle` | switch control |

## Molecules — `components/molecules/`

| Component | Purpose |
| --- | --- |
| `card` | shared rounded bordered surface |
| `charge_days` | month-day grid with Select all/Clear and Last day |
| `compact_select` | compact toolbar select |
| `converted_amount` | display-currency amount with conversion/privacy |
| `date_picker` | themed calendar popup |
| `empty_state` | icon, title, description and action |
| `entity_icon` | stored icon name + color, dot fallback |
| `field` | label + control + hint/error |
| `list_toolbar` | search, actions and grid/list switch |
| `optional_date` | first payment / end date hidden behind an add action |
| `privacy_toggle` | eye control that masks every amount |
| `search_input` | search field with icon |
| `select` | the one select: trigger, fixed panel, optional search, optional create |
| `select_menu` | UUID adapter over `select`; accounts/categories |
| `selection_bar` | count, select all/clear and bulk actions |
| `string_select` | string adapter over `select`; currencies/timezones |
| `timezone_select` | IANA catalogue adapter |
| `toast` | transient success/error notice |

## Organisms — `components/organisms/`

| Component | Purpose |
| --- | --- |
| `billing_schedule` | interval/unit, weekly, monthly, yearly and day schedule; `Monthly` mode for debts |
| `confirm` | destructive confirmation row and dialog |
| `dialog` + `ModalHost` | one shell for every form and destructive flow |
| `dropdown_menu` | labelled action menu |
| `fab_menu` | mobile quick-action menu |
| `filter_menu` | filter trigger, active count and popover |
| `logo_search` | logo URL, search and thumbnail results |
| `page_header` | title, subtitle and page actions |
| `sort_menu` | sort field and direction |
| `theme_picker` | theme catalogue and full-screen mobile picker |
| `web_server_qr` | server pairing and APK QR card |

## Rules

- One visual decision, one owner. Pages never duplicate button, input, card or
  dialog chrome.
- Every select goes through `components::select`. `string_select`,
  `compact_select` and `select_menu` are adapters, not separate designs.
- `searchable(true)` shows the search field and focuses it when the panel opens.
- Add/edit/notes/bulk forms use `dialog`; destructive confirmations use
  `confirm_row`/`confirm_dialog`.
- Money lists enter selection mode in place; `selection_bar` owns the count,
  select-all, clear and bulk actions.
- Amounts go through `converted_amount` or `format_currency` so privacy mode is
  applied everywhere.
- Colors come from the active theme or `color_picker`; no literal colors in
  pages.
