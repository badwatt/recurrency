# Component architecture

The Native Shell port mirrors `frontend/src/components` and follows atomic
design. Pages never paint new chrome; they compose the modules in `src/components`.

## Atoms

| Component | Purpose |
| --- | --- |
| `avatar` | circular user image or initials |
| `badge` | neutral/primary/success/destructive status label |
| `button` | every press target, variants/sizes/alignment/radius |
| `checkbox` | labelled binary control |
| `checkbox_mark` | compact box for card/row selection |
| `color_picker` | account/category palette and hex value |
| `entity_icon` | stored icon name + color, dot fallback |
| `input` | single-line field and dynamic-row value input |
| `logo_avatar` | remote brand tile with fallback |
| `privacy_toggle` | eye control that masks every amount |
| `skeleton` | loading placeholder |
| `textarea` | multiline notes surface |
| `toggle` | switch control |

## Molecules

| Component | Purpose |
| --- | --- |
| `compact_select` | overlay select for page toolbars |
| `confirm` | destructive confirmation row and dialog |
| `date_picker` | themed calendar popup |
| `dropdown_menu` | labelled action menu |
| `empty_state` | icon, title, description and action |
| `field` | label + control + hint/error |
| `filter_menu` | filter trigger, active count and popover |
| `input` | search and form fields |
| `list_toolbar` | search, actions and grid/list switch |
| `logo_search` | logo URL, search and thumbnail results |
| `page_header` | title, subtitle and page actions |
| `search_input` | search field with icon |
| `segmented` | two-or-more exclusive options |
| `select_menu` | searchable entity picker |
| `sort_menu` | sort field and direction |
| `string_select` | searchable catalogue picker |
| `theme_picker` | 18-theme swatch grid |
| `timezone_select` | searchable IANA catalogue |

## Organisms

| Component | Purpose |
| --- | --- |
| `dialog` + `ModalHost` | one shell for every form and destructive flow |
| `fab_menu` | mobile quick-action menu |
| `selection_bar` | count, select all/clear and bulk actions |
| `web_server_qr::qr_card` | server pairing and APK QR card |

## Rules

- New page chrome starts as a component; only one instance is allowed to own a visual decision.
- Money lists enter selection mode in place and use `selection_bar` plus `checkbox_mark`; the bulk dialog reuses the same selected ids.
- Add/edit/notes/bulk forms use `dialog`; destructive confirmations use `confirm_row`.
- Catalogues use `string_select`; entity pickers use `select_menu`.
- Amounts go through `format_currency`, which honours privacy mode.
- Colors come from the active theme or `color_picker`; no literal colors in pages.
