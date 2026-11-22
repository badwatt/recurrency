use chrono::Datelike;
use dioxus::prelude::*;
use lucide_dioxus::{ArrowUpDown, Copy, CreditCard, FileText, Pause, Pencil, Play, Plus, Trash, X};
use rust_decimal::Decimal;
use std::collections::HashSet;
use uuid::Uuid;

use crate::api;
use crate::components::ConvertedAmount;
use crate::components::button::{Button, ButtonAlign, ButtonRadius, ButtonSize, ButtonVariant};
use crate::components::checkbox::Checkbox;
use crate::components::field::Field;
use crate::components::popover::{Popover, PopoverAlign};
use crate::components::input::{Input, InputSize};
use crate::components::ListToolbar;
use crate::components::dialog::Dialog;
#[allow(unused_imports)]
use crate::components::entry_form::{ValidationResult, validate_subscription_form};
use crate::components::logo_image_with_fallback;
use crate::components::logo_search::LogoSearch;
use crate::components::page_header::PageHeader;
use crate::components::select_menu::{SelectMenu, SelectOption};
use crate::components::skeleton::{SubscriptionCardSkeleton, SubscriptionListSkeleton};
use crate::components::subscription_form::SubscriptionForm;
use crate::components::tooltip::Tooltip;
use crate::models::subscription::{
    CreateSubscriptionRequest, Recurrence, Subscription, UpdateSubscriptionRequest,
};
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::create_intent::{CREATE_INTENT, CreateIntent};
use crate::state::debts::DEBTS;
use crate::state::preferences::PREFERENCES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::storage::Storage;
use crate::utils::constants::{CURRENCIES, RECURRENCE_UNITS};
use crate::utils::icons::icon_display;

pub(crate) mod delete_dialog;
mod logic;
pub(crate) mod notes_dialog;
use delete_dialog::{BulkDeleteSubscriptionsDialog, DeleteSubscriptionDialog};
pub use logic::*;
use notes_dialog::NotesSubscriptionDialog;
pub(crate) mod cards;
pub(crate) mod dialog;
use cards::{render_grid_card, render_list_card};
use dialog::render_dialog;

// Filter/sort types + pure logic live in logic.rs

// Type alias for backward compatibility within this file
type DialogForm = SubscriptionForm;

// ── Page component ───────────────────────────────────────────────────

#[component]
pub fn SubscriptionsPage() -> Element {
    let mut filters: Signal<FilterConfig> = use_signal(FilterConfig::default);
    let mut sort: Signal<SortConfig> = use_signal(SortConfig::default);
    let mut dialog_open: Signal<bool> = use_signal(|| false);
    let mut editing_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut form: Signal<DialogForm> = use_signal(DialogForm::default);
    let mut recurrence: Signal<Recurrence> = use_signal(Recurrence::default);
    let mut form_active: Signal<bool> = use_signal(|| true);
    let mut filters_popover: Signal<bool> = use_signal(|| false);
    let mut sort_popover: Signal<bool> = use_signal(|| false);
    let mut search: Signal<String> = use_signal(String::new);
    let mut selected: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
    let mut selection_mode: Signal<bool> = use_signal(|| false);
    let mut bulk_delete_ids: Signal<Vec<Uuid>> = use_signal(Vec::new);
    let mut bulk_busy: Signal<bool> = use_signal(|| false);
    let mut bulk_category: Signal<String> = use_signal(String::new);
    let mut delete_confirm_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut notes_dialog_open: Signal<bool> = use_signal(|| false);
    let mut notes_dialog_sub_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut notes_dialog_value: Signal<String> = use_signal(String::new);
    let notes_saving: Signal<bool> = use_signal(|| false);

    // Validation error signals
    let mut name_error: Signal<String> = use_signal(String::new);
    let mut amount_error: Signal<String> = use_signal(String::new);
    let mut currency_error: Signal<String> = use_signal(String::new);
    let mut frequency_error: Signal<String> = use_signal(String::new);
    let mut start_date_error: Signal<String> = use_signal(String::new);
    let mut end_date_error: Signal<String> = use_signal(String::new);
    let mut account_error: Signal<String> = use_signal(String::new);
    let mut category_error: Signal<String> = use_signal(String::new);
    let mut form_account_id: Signal<String> = use_signal(String::new);

    let subs_state = SUBSCRIPTIONS();
    let cats_state = CATEGORIES();
    let prefs_state = PREFERENCES();
    let auth_state = AUTH();

    let view_mode = prefs_state.preferences.subscriptions_view.clone();
    // In server mode the view mode is known only after the preferences load;
    // render a neutral placeholder until then so a stored "list" never flashes
    // as the grid first. Offline mode and local views are authoritative.
    let prefs_ready = prefs_state.initialized
        || prefs_state.has_local_view()
        || crate::api::client::is_offline_mode();
    let default_currency = auth_state.currency().to_string();

    // Fetch on mount (force re-fetch to ensure fresh data)
    let mut has_fetched = use_signal(|| false);
    use_effect(move || {
        let _ = subs_state;
        if !*has_fetched.peek() {
            has_fetched.set(true);
            spawn(async move {
                SUBSCRIPTIONS.write().set_loading();
                match Storage::list_subscriptions().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
            });
        }
    });

    use_effect(move || {
        let _ = cats_state;
        spawn(async move {
            let state = CATEGORIES();
            if !state.is_loading && state.categories.is_empty() && state.error.is_none() {
                CATEGORIES.write().set_loading();
                match Storage::list_categories().await {
                    Ok(data) => CATEGORIES.write().set_data(data),
                    Err(e) => CATEGORIES.write().set_error(e.to_string()),
                }
            }
        });
    });

    use_effect(move || {
        if DEBTS().debts.is_empty() {
            spawn(async move {
                if let Ok(debts) = crate::storage::Storage::list_debts().await {
                    let ids = debts
                        .iter()
                        .filter_map(|debt| debt.subscription_id)
                        .collect();
                    DEBTS.write().set_data(debts);
                    filters.write().debt_subscription_ids = ids;
                }
            });
        }
    });

    use_effect(move || {
        let state = ACCOUNTS();
        if !state.is_loading && state.accounts.is_empty() && state.error.is_none() {
            spawn(async move {
                ACCOUNTS.write().set_loading();
                match Storage::list_accounts().await {
                    Ok(data) => ACCOUNTS.write().set_data(data),
                    Err(e) => ACCOUNTS.write().set_error(e.to_string()),
                }
                if form_account_id().is_empty()
                    && let Some(default) = ACCOUNTS().default_account()
                {
                    form_account_id.set(default.id.to_string());
                }
            });
        }
    });

    use_effect(move || {
        let _ = prefs_state;
        spawn(async move {
            if PREFERENCES().initialized {
                return;
            }
            match api::preferences::get().await {
                Ok(p) => {
                    // Theme: an explicit server choice wins on every device.
                    // If the server has no theme yet, migrate the local choice.
                    let saved_view = PREFERENCES().subscriptions_view().to_string();
                    let local_view = PREFERENCES().has_local_view();
                    let theme_to_push = PREFERENCES.write().apply_server_preferences(p);
                    if let Some(theme) = theme_to_push {
                        let req = crate::models::preferences::UpdatePreferencesRequest {
                            theme: Some(theme),
                            subscriptions_view: None,
                            first_day_of_week: None,
                            date_format: None,
                        };
                        if api::preferences::update(&req).await.is_ok() {
                            PREFERENCES.write().mark_theme_synced();
                        }
                    }
                    // Adopt the server view when nothing was stored locally, and
                    // persist the effective value so the next launch hydrates it
                    // before the first render (no grid flash).
                    let effective_view = if local_view {
                        saved_view
                    } else {
                        PREFERENCES().subscriptions_view().to_string()
                    };
                    PREFERENCES.write().set_subscriptions_view(effective_view);
                }
                // Server preferences unavailable: unblock the page with the
                // local defaults instead of keeping the loading state forever.
                Err(_) => PREFERENCES.write().initialized = true,
            }
        });

        // Safety net: never stall the page on a hanging preferences request.
        spawn(async move {
            if PREFERENCES().initialized || PREFERENCES().has_local_view() {
                return;
            }
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(3000).await;
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            if !PREFERENCES().initialized {
                PREFERENCES.write().initialized = true;
            }
        });
    });

    // Callbacks
    let handle_edit: Callback<Subscription> = use_callback(move |sub: Subscription| {
        editing_id.set(Some(sub.id));
        form_account_id.set(sub.account_id.to_string());
        form.set(DialogForm {
            name: sub.name,
            amount: sub.amount.to_string(),
            currency: sub.currency,
            frequency: sub.frequency,
            schedule_day: sub.schedule_day,
            schedule_month: sub.schedule_month,
            start_date: sub.start_date.to_string(),
            end_date: sub.end_date.map(|d| d.to_string()).unwrap_or_default(),
            category_id: sub.category_id.to_string(),
            notes: sub.notes.unwrap_or_default(),
            logo_url: sub.logo_url.unwrap_or_default(),
        });
        recurrence.set(sub.recurrence.clone());
        form_active.set(sub.is_active);
        name_error.set(String::new());
        amount_error.set(String::new());
        currency_error.set(String::new());
        frequency_error.set(String::new());
        start_date_error.set(String::new());
        end_date_error.set(String::new());
        account_error.set(String::new());
        category_error.set(String::new());
        dialog_open.set(true);
    });

    let handle_delete: Callback<Uuid> = use_callback(move |id: Uuid| {
        delete_confirm_id.set(Some(id));
    });

    let handle_toggle_active: Callback<Subscription> = use_callback(move |sub: Subscription| {
        let id = sub.id;
        let new_active = !sub.is_active;
        spawn(async move {
            let req = UpdateSubscriptionRequest {
                account_id: None,
                is_active: Some(new_active),
                ..Default::default()
            };
            let _ = Storage::update_subscription(&id, &req).await;
            match Storage::list_subscriptions().await {
                Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
            }
        });
    });

    let handle_create: Callback<()> = use_callback(move |_| {
        editing_id.set(None);
        let default_cat = CATEGORIES()
            .default_category()
            .map(|c| c.id.to_string())
            .unwrap_or_default();
        let default_account = ACCOUNTS()
            .default_account()
            .map(|a| a.id.to_string())
            .unwrap_or_default();
        form_account_id.set(default_account);
        form.set(DialogForm {
            name: String::new(),
            amount: String::new(),
            currency: default_currency.clone(),
            frequency: "monthly".into(),
            schedule_day: None,
            schedule_month: None,
            start_date: String::new(),
            end_date: String::new(),
            category_id: default_cat,
            notes: String::new(),
            logo_url: String::new(),
        });
        recurrence.set(Recurrence {
            days: vec![chrono::Utc::now().date_naive().day() as u8],
            ..Default::default()
        });
        form_active.set(true);
        name_error.set(String::new());
        amount_error.set(String::new());
        currency_error.set(String::new());
        frequency_error.set(String::new());
        start_date_error.set(String::new());
        end_date_error.set(String::new());
        account_error.set(String::new());
        category_error.set(String::new());
        dialog_open.set(true);
    });

    use_effect(move || {
        if *CREATE_INTENT.read() == Some(CreateIntent::Subscription) {
            *CREATE_INTENT.write() = None;
            handle_create.call(());
        }
    });

    let handle_duplicate: Callback<Subscription> = use_callback(move |sub: Subscription| {
        editing_id.set(None);
        form_account_id.set(sub.account_id.to_string());
        form.set(DialogForm {
            name: format!("{} (copia)", sub.name),
            amount: sub.amount.to_string(),
            currency: sub.currency,
            frequency: sub.frequency,
            schedule_day: sub.schedule_day,
            schedule_month: sub.schedule_month,
            start_date: sub.start_date.to_string(),
            end_date: sub.end_date.map(|d| d.to_string()).unwrap_or_default(),
            category_id: sub.category_id.to_string(),
            notes: sub.notes.unwrap_or_default(),
            logo_url: sub.logo_url.unwrap_or_default(),
        });
        recurrence.set(sub.recurrence.clone());
        form_active.set(sub.is_active);
        name_error.set(String::new());
        amount_error.set(String::new());
        currency_error.set(String::new());
        frequency_error.set(String::new());
        start_date_error.set(String::new());
        end_date_error.set(String::new());
        account_error.set(String::new());
        category_error.set(String::new());
        dialog_open.set(true);
    });

    let handle_notes_click: Callback<Subscription> = use_callback(move |sub: Subscription| {
        notes_dialog_sub_id.set(Some(sub.id));
        notes_dialog_value.set(sub.notes.unwrap_or_default());
        notes_dialog_open.set(true);
    });

    let clear_filters: Callback<()> = use_callback(move |_| {
        let debt_ids = filters().debt_subscription_ids;
        filters.set(FilterConfig {
            debt_subscription_ids: debt_ids,
            ..FilterConfig::default()
        });
    });

    let toggle_sort_direction: Callback<()> = use_callback(move |_| {
        let dir = sort().direction;
        sort.write().direction = if dir == "asc" {
            "desc".into()
        } else {
            "asc".into()
        };
    });

    let toggle_selected: Callback<Uuid> = use_callback(move |id: Uuid| {
        if selected.read().contains(&id) {
            selected.write().remove(&id);
        } else {
            selected.write().insert(id);
        }
    });

    let set_view_mode: Callback<String> = use_callback(move |mode: String| {
        PREFERENCES.write().set_subscriptions_view(mode.clone());
        spawn(async move {
            let state = PREFERENCES();
            drop(state);
            let req = crate::models::preferences::UpdatePreferencesRequest {
                theme: None,
                subscriptions_view: Some(mode),
                first_day_of_week: None,
                date_format: None,
            };
            let _ = api::preferences::update(&req).await;
        });
    });

    // Filtered + sorted
    let filters_val = filters();
    let sort_val = sort();
    let state = SUBSCRIPTIONS();
    let filtered = filter_subscriptions_by_search(
        apply_filters_and_sort(&state.subscriptions, &filters_val, &sort_val),
        &search(),
    );
    let today = chrono::Utc::now().date_naive();
    let (active_subs, inactive_subs, finalized_subs) = subscription_sections(&filtered, today);
    let has_active = !active_subs.is_empty();
    let has_inactive = !inactive_subs.is_empty();
    let has_finalized = !finalized_subs.is_empty();

    let active_filter_count = count_active_filters(&filters_val);
    let sort_arrow = if sort_val.direction == "asc" {
        "↑"
    } else {
        "↓"
    };
    let filtered_count = filtered.len();
    let plural = if filtered_count != 1 { "s" } else { "" };
    let selected_ids = selected();
    let selected_count = selected_ids.len();
    let all_filtered_selected =
        !filtered.is_empty() && filtered.iter().all(|sub| selected_ids.contains(&sub.id));
    let selected_ids_vec = selected_ids.iter().copied().collect::<Vec<_>>();

    // Category options for selects (owned)
    let category_options: Vec<(String, String)> = CATEGORIES()
        .categories
        .iter()
        .map(|c| (c.id.to_string(), c.name.clone()))
        .collect();

    let bulk_set_active: Callback<bool> = use_callback(move |active: bool| {
        if *bulk_busy.peek() || selected().is_empty() {
            return;
        }
        let ids: Vec<Uuid> = selected().iter().copied().collect();
        bulk_busy.set(true);
        spawn(async move {
            for id in ids {
                let req = UpdateSubscriptionRequest {
                    account_id: None,
                    is_active: Some(active),
                    ..Default::default()
                };
                let _ = Storage::update_subscription(&id, &req).await;
            }
            match Storage::list_subscriptions().await {
                Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
            }
            selected.write().clear();
            bulk_busy.set(false);
        });
    });

    let filtered_ids: Vec<Uuid> = filtered.iter().map(|sub| sub.id).collect();
    let toggle_all_selected = move |_| {
        if all_filtered_selected {
            selected.write().clear();
        } else {
            for id in &filtered_ids {
                selected.write().insert(*id);
            }
        }
    };

    let toolbar_actions = rsx! {
        div { class: "static sm:relative",
            Button {
                variant: ButtonVariant::Unstyled,
                size: ButtonSize::IconLg,
                radius: ButtonRadius::Lg,
                align: ButtonAlign::Center,
                aria_label: Some("Filters".to_string()),
                title: Some("Filters".to_string()),
                aria_expanded: Some((if filters_popover() { "true" } else { "false" }).to_string()),
                class: (if active_filter_count > 0 { "relative inline-flex items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary hover:bg-primary/15" } else { "relative inline-flex items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" }).to_string(),
                onclick: move |_| filters_popover.set(!filters_popover()),
                svg {
                    view_box: "0 0 24 24",
                    width: "16",
                    height: "16",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    path { d: "M4 5h16l-6 7v5l-4 2v-7L4 5z" }
                }
                if active_filter_count > 0 {
                    span { class: "absolute -right-1 -top-1 min-w-4 rounded-full bg-primary px-1 text-[10px] leading-4 text-primary-foreground", "{active_filter_count}" }
                }
            }
            Popover {
                open: filters_popover(),
                on_close: move |_| filters_popover.set(false),
                align: PopoverAlign::CenterSm,
                class: "w-[min(20rem,calc(100vw-2rem))] space-y-3 p-3".to_string(),
                    div { class: "flex items-center justify-between",
                        h4 { class: "text-sm font-medium", "Filters" }
                        if active_filter_count > 0 {
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Xs,
                                class: "text-muted-foreground",
                                onclick: move |_| clear_filters.call(()),
                                "Clear all"
                            }
                        }
                    }
                    Field { label: "Status", label_class: "text-xs text-muted-foreground".to_string(),
                        SelectMenu {
                            value: filters_val.status.clone().unwrap_or_else(|| "all".to_string()),
                            options: STATUS_OPTIONS.iter().map(|(value, label)| SelectOption::new(*value, *label)).collect::<Vec<_>>(),
                            on_change: move |value: String| {
                                filters.write().status = if value == "all" { None } else { Some(value) };
                            },
                            searchable: false,
                        }
                    }
                    Field { label: "Repeats every", label_class: "text-xs text-muted-foreground".to_string(),
                        SelectMenu {
                            value: filters_val.frequency.clone().unwrap_or_else(|| "all".to_string()),
                            options: {
                                let mut options = vec![SelectOption::new("all", "All units")];
                                options.extend(RECURRENCE_UNITS.iter().map(|(value, label)| SelectOption::new(*value, *label)));
                                options
                            },
                            on_change: move |value: String| {
                                filters.write().frequency = if value == "all" { None } else { Some(value) };
                            },
                            searchable: false,
                        }
                    }
                    Field { label: "Category", label_class: "text-xs text-muted-foreground".to_string(),
                        SelectMenu {
                            value: filters_val.category_id.clone().unwrap_or_else(|| "all".to_string()),
                            options: {
                                let mut options = vec![SelectOption::new("all", "All categories")];
                                options.extend(category_options.iter().map(|(id, name)| SelectOption::new(id.as_str(), name.as_str())));
                                options
                            },
                            on_change: move |value: String| {
                                filters.write().category_id = if value == "all" { None } else { Some(value) };
                            },
                            searchable: true,
                        }
                    }
                    div { class: "flex items-center gap-2",
                        Checkbox {
                            id: "filter-debts-only",
                            checked: filters_val.debts_only,
                            on_change: move |checked| filters.write().debts_only = checked,
                        }
                        label { r#for: "filter-debts-only", class: "text-xs font-medium text-muted-foreground", "Debt subscriptions only" }
                    }
                    Field { label: "Price range", label_class: "text-xs text-muted-foreground".to_string(),
                        div { class: "flex items-center gap-2",
                            Input {
                                r#type: "number",
                                size: InputSize::Xs,
                                placeholder: "Min",
                                value: filters_val.price_min.as_deref().unwrap_or(""),
                                on_input: move |value: String| {
                                    filters.write().price_min = if value.is_empty() { None } else { Some(value) };
                                }
                            }
                            span { class: "text-xs text-muted-foreground", "–" }
                            Input {
                                r#type: "number",
                                size: InputSize::Xs,
                                placeholder: "Max",
                                value: filters_val.price_max.as_deref().unwrap_or(""),
                                on_input: move |value: String| {
                                    filters.write().price_max = if value.is_empty() { None } else { Some(value) };
                                }
                            }
                        }
                    }
            }
        }
        div { class: "static sm:relative",
            Button {
                variant: ButtonVariant::Unstyled,
                size: ButtonSize::IconLg,
                radius: ButtonRadius::Lg,
                align: ButtonAlign::Center,
                aria_label: Some("Sort".to_string()),
                title: Some("Sort".to_string()),
                aria_expanded: Some((if sort_popover() { "true" } else { "false" }).to_string()),
                class: "inline-flex items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground".to_string(),
                onclick: move |_| sort_popover.set(!sort_popover()),
                ArrowUpDown { size: 16 }
            }
            Popover {
                open: sort_popover(),
                on_close: move |_| sort_popover.set(false),
                align: PopoverAlign::CenterSm,
                class: "w-[min(13rem,calc(100vw-2rem))] space-y-2 p-3".to_string(),
                    label { class: "text-xs font-medium text-muted-foreground", "Sort by" }
                    SelectMenu {
                        value: sort_val.field.clone(),
                        options: SORT_OPTIONS.iter().map(|(value, label)| SelectOption::new(*value, *label)).collect::<Vec<_>>(),
                        on_change: move |value: String| sort.write().field = value,
                        searchable: false,
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        full_width: true,
                        align: ButtonAlign::Between,
                        onclick: move |_| toggle_sort_direction.call(()),
                        span { if sort_val.direction == "asc" { "Ascending" } else { "Descending" } }
                        span { {sort_arrow.to_string()} }
                    }
            }
        }
        Button {
            variant: ButtonVariant::Unstyled,
            size: ButtonSize::IconLg,
            radius: ButtonRadius::Lg,
            align: ButtonAlign::Center,
            aria_label: Some((if selection_mode() { "Exit selection mode" } else { "Enter selection mode" }).to_string()),
            title: Some((if selection_mode() { "Exit selection mode" } else { "Select subscriptions" }).to_string()),
            aria_pressed: Some((if selection_mode() { "true" } else { "false" }).to_string()),
            class: (if selection_mode() { "inline-flex items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary hover:bg-primary/15" } else { "inline-flex items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" }).to_string(),
            onclick: move |_| {
                if selection_mode() {
                    selection_mode.set(false);
                    selected.write().clear();
                } else {
                    selection_mode.set(true);
                }
            },
            svg {
                view_box: "0 0 24 24",
                width: "16",
                height: "16",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                rect { x: "3", y: "3", width: "18", height: "18", rx: "3" }
                if selection_mode() {
                    path { d: "m7 12 3 3 7-7" }
                }
            }
        }
    };

    // ── Error ──
    if let Some(ref err) = state.error {
        return rsx! {
            div { class: "p-4 md:p-6",
                div { class: "rounded-lg border border-red-500 bg-red-500/10 p-4 text-red-600",
                    {err.to_string()}                }
            }
        };
    }

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6 page-enter",

            // ── Header ──
            PageHeader {
                title: "Subscriptions",
                subtitle: "Track and manage recurring payments",
                actions: rsx! {
                    Button {
                        class: "hidden md:inline-flex",
                        onclick: move |_| handle_create.call(()),
                        Plus { size: 16 }
                        "Add Subscription"
                    }
                },
            }

            if !state.subscriptions.is_empty() {
                ListToolbar {
                    value: search(),
                    oninput: move |value| search.set(value),
                    view: view_mode.clone(),
                    onview: move |mode| set_view_mode.call(mode),
                    show_view: true,
                    actions: toolbar_actions,
                }
            }

            if selection_mode() {
                div {
                    role: "status",
                    aria_live: "polite",
                    class: "flex flex-wrap items-center gap-2 rounded-lg border border-primary/30 bg-primary/5 px-3 py-2",
                    span { class: "text-sm font-medium",
                        if selected_count == 0 { "Select subscriptions" } else { "{selected_count} selected" }
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: toggle_all_selected,
                        if all_filtered_selected { "Clear selection" } else { "Select all" }
                    }
                    SelectMenu {
                        value: bulk_category(),
                        options: {
                            let mut options = vec![SelectOption::new("", "Set category…")];
                            options.extend(CATEGORIES().categories.iter().filter(|category| !category.is_system).map(|category| SelectOption::new(category.id.to_string(), category.name.clone())));
                            options
                        },
                        on_change: move |value: String| bulk_category.set(value),
                        searchable: true,
                        placeholder: "Set category…",
                        class: "w-44",
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: bulk_busy() || selected_count == 0 || bulk_category().is_empty(),
                        onclick: move |_| {
                            if bulk_busy() || bulk_category().is_empty() { return; }
                            let ids: Vec<Uuid> = selected().iter().copied().collect();
                            let Some(category_id) = Uuid::parse_str(&bulk_category()).ok() else { return; };
                            bulk_busy.set(true);
                            spawn(async move {
                                for id in ids {
                                    let req = UpdateSubscriptionRequest { category_id: Some(category_id), ..Default::default() };
                                    let _ = Storage::update_subscription(&id, &req).await;
                                }
                                match Storage::list_subscriptions().await {
                                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                    Err(error) => SUBSCRIPTIONS.write().set_error(error.to_string()),
                                }
                                selected.write().clear();
                                bulk_category.set(String::new());
                                bulk_busy.set(false);
                            });
                        },
                        "Apply category"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: bulk_busy() || selected_count == 0,
                        onclick: move |_| bulk_set_active.call(true),
                        Play { size: 13 }
                        "Activate"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        disabled: bulk_busy() || selected_count == 0,
                        onclick: move |_| bulk_set_active.call(false),
                        Pause { size: 13 }
                        "Pause"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        class: "border-destructive/40 bg-destructive/10 text-destructive hover:bg-destructive/15",
                        disabled: bulk_busy() || selected_count == 0,
                        onclick: move |_| {
                            let ids = selected_ids_vec.clone();
                            if !ids.is_empty() {
                                bulk_delete_ids.set(ids);
                            }
                        },
                        Trash { size: 13 }
                        "Delete"
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "ml-auto text-muted-foreground",
                        onclick: move |_| {
                            selected.write().clear();
                            selection_mode.set(false);
                        },
                        X { size: 13 }
                        "Clear"
                    }
                }
            }

            // ── Content ──
            {if !prefs_ready {
                rsx! {
                    div { class: "flex h-64 items-center justify-center rounded-xl border border-border/40 bg-card/80",
                        p { class: "text-sm text-muted-foreground", "Loading subscriptions…" }
                    }
                }
            } else if state.is_loading && state.subscriptions.is_empty() {
                rsx! {
                    if view_mode == "list" {
                        div { class: "space-y-3",
                            for _ in 0..6 {
                                SubscriptionListSkeleton {}
                            }
                        }
                    } else {
                        div { class: "grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                            for _ in 0..8 {
                                SubscriptionCardSkeleton {}
                            }
                        }
                    }
                }
            } else if state.subscriptions.is_empty() {
                rsx! {
                    div { class: "rounded-lg border border-border bg-card p-8 flex flex-col items-center justify-center gap-4",
                        span { class: "text-muted-foreground", CreditCard { size: 40 } }
                        div { class: "text-center",
                            p { class: "text-lg font-medium", "No subscriptions yet" }
                            p { class: "text-sm text-muted-foreground", "Add your first recurring expense" }
                        }
                        Button {
                            class: "hidden md:inline-flex",
                            onclick: move |_| handle_create.call(()),
                            Plus { size: 16 }
                            "Add Subscription"
                        }
                    }
                }
            } else if filtered.is_empty() {
                rsx! {
                    div { class: "rounded-xl border border-border/40 bg-card/80 p-6 flex h-40 flex-col items-center justify-center gap-2",
                        p { class: "text-sm text-muted-foreground", "No subscriptions match your search or filters" }
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| {
                                search.set(String::new());
                                clear_filters.call(());
                            },
                            "Clear search and filters"
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "space-y-4",

                        // Count
                        p { class: "text-sm text-muted-foreground",
                            "{filtered_count} subscription{plural}"
                        }

                        // ── Active / Inactive / Finalized sections ──
                        if has_active {
                            {render_subscription_section(
                                "Active".to_string(),
                                &active_subs,
                                view_mode.clone(),
                                &selected_ids,
                                selection_mode(),
                                toggle_selected,
                                handle_edit,
                                handle_delete,
                                handle_toggle_active,
                                handle_notes_click,
                                handle_duplicate,
                            )}
                        }
                        if has_inactive {
                            {render_subscription_section(
                                "Inactive".to_string(),
                                &inactive_subs,
                                view_mode.clone(),
                                &selected_ids,
                                selection_mode(),
                                toggle_selected,
                                handle_edit,
                                handle_delete,
                                handle_toggle_active,
                                handle_notes_click,
                                handle_duplicate,
                            )}
                        }
                        if has_finalized {
                            {render_subscription_section(
                                "Finalized".to_string(),
                                &finalized_subs,
                                view_mode,
                                &selected_ids,
                                selection_mode(),
                                toggle_selected,
                                handle_edit,
                                handle_delete,
                                handle_toggle_active,
                                handle_notes_click,
                                handle_duplicate,
                            )}
                        }
                    }
                }
            }}

            // ── Create/Edit Dialog ──
            if dialog_open() {
                {render_dialog(form, recurrence, form_active, editing_id, dialog_open, name_error, amount_error, currency_error, frequency_error, start_date_error, end_date_error, account_error, category_error, form_account_id, None)}
            }

            // ── Bulk delete confirmation ──
            if !bulk_delete_ids().is_empty() {
                BulkDeleteSubscriptionsDialog {
                    ids: bulk_delete_ids,
                    on_done: move |_| selected.write().clear(),
                }
            }

            // ── Delete confirmation ──
            if delete_confirm_id().is_some() {
                DeleteSubscriptionDialog { confirm_id: delete_confirm_id }
            }

            // ── Notes dialog ──
            if notes_dialog_open() {
                NotesSubscriptionDialog {
                    open: notes_dialog_open,
                    sub_id: notes_dialog_sub_id,
                    value: notes_dialog_value,
                    saving: notes_saving,
                }
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn render_subscription_section(
    title: String,
    subs: &[Subscription],
    view_mode: String,
    selected: &HashSet<Uuid>,
    selection_mode: bool,
    on_select: Callback<Uuid>,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
    on_duplicate: Callback<Subscription>,
) -> Element {
    rsx! {
        section { class: "space-y-3",
            div { class: "flex items-center gap-2 pt-2",
                h2 { class: "text-sm font-semibold uppercase tracking-wider text-muted-foreground", {title} }
                span { class: "rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground", {subs.len().to_string()} }
            }
            if view_mode == "grid" {
                div { class: "grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                    for (i, sub) in subs.iter().enumerate() {
                        {render_grid_card(sub, i, selection_mode, selected.contains(&sub.id), on_select, on_edit, on_delete, on_toggle, on_notes, on_duplicate)}
                    }
                }
            } else {
                div { class: "space-y-3",
                    for (_i, sub) in subs.iter().enumerate() {
                        {render_list_card(sub, selection_mode, selected.contains(&sub.id), on_select, on_edit, on_delete, on_toggle, on_notes, on_duplicate)}
                    }
                }
            }
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

// ── DialogForm tests ──────────────────────────────────────────────────
#[cfg(test)]
mod dialog_form_tests;
