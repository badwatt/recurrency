use chrono::Datelike;
use dioxus::prelude::*;
use lucide_dioxus::{ArrowUpDown, Copy, CreditCard, FileText, Pause, Pencil, Play, Trash2, X};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::api;
use crate::storage::Storage;
use crate::components::dialog::Dialog;
use crate::components::logo_image_with_fallback;
use crate::components::page_header::PageHeader;
use crate::components::skeleton::{SubscriptionCardSkeleton, SubscriptionListSkeleton};
use crate::components::tooltip::Tooltip;
use crate::components::logo_search::LogoSearch;
use crate::components::subscription_form::SubscriptionForm;
#[allow(unused_imports)]
use crate::components::entry_form::{ValidationResult, validate_subscription_form};
use crate::models::subscription::{
    CreateSubscriptionRequest, Recurrence, Subscription, UpdateSubscriptionRequest,
};
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::create_intent::{CREATE_INTENT, CreateIntent};
use crate::state::preferences::PREFERENCES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::utils::constants::{CURRENCIES, RECURRENCE_UNITS};
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;

mod delete_dialog;
mod logic;
mod notes_dialog;
pub use logic::*;
use delete_dialog::DeleteSubscriptionDialog;
use notes_dialog::NotesSubscriptionDialog;
mod cards;
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
                    // Preserve local UI preferences; a view mode persisted on
                    // this device must win over the server value.
                    let saved_theme = PREFERENCES().theme().to_string();
                    let saved_view = PREFERENCES().subscriptions_view().to_string();
                    let local_view = PREFERENCES().has_local_view();
                    PREFERENCES.write().set_data(p);
                    PREFERENCES.write().set_theme(saved_theme);
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
        filters.set(FilterConfig::default());
    });

    let toggle_sort_direction: Callback<()> = use_callback(move |_| {
        let dir = sort().direction.clone();
        sort.write().direction = if dir == "asc" {
            "desc".into()
        } else {
            "asc".into()
        };
    });

    let set_view_mode = move |mode: &str| {
        PREFERENCES.write().set_subscriptions_view(mode.to_string());
        let mode = mode.to_string();
        spawn(async move {
            let state = PREFERENCES();
            drop(state);
            let req = crate::models::preferences::UpdatePreferencesRequest {
                theme: None,
                subscriptions_view: Some(mode),
                first_day_of_week: None,
            };
            let _ = api::preferences::update(&req).await;
        });
    };

    // Filtered + sorted
    let filters_val = filters();
    let sort_val = sort();
    let state = SUBSCRIPTIONS();
    let filtered = apply_filters_and_sort(&state.subscriptions, &filters_val, &sort_val);
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

    // Category options for selects (owned)
    let category_options: Vec<(String, String)> = CATEGORIES()
        .categories
        .iter()
        .map(|c| (c.id.to_string(), c.name.clone()))
        .collect();

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
        div { class: "p-4 md:p-6 flex flex-col gap-6 animate-in fade-in duration-300",

            // ── Header ──
            div { class: "flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3",
                PageHeader { title: "Subscriptions", subtitle: "Track and manage recurring payments" }
                div { class: "flex items-center gap-2 shrink-0",
                    button {
                        class: "inline-flex items-center gap-1.5 border border-border/60 rounded-lg px-3 py-2 hover:bg-muted text-sm transition-colors duration-200",
                        onclick: {
                            let vm = view_mode.clone();
                            move |_| set_view_mode(if vm.as_str() == "grid" { "list" } else { "grid" })
                        },
                        if view_mode.as_str() == "grid" { "≡ List" } else { "⊞ Grid" }
                    }
                    button {
                        class: "hidden md:inline-flex bg-primary text-primary-foreground font-semibold px-4 py-2.5 rounded-lg text-sm hover:bg-primary/90 active:scale-[0.97] transition-all duration-200",
                        onclick: move |_| handle_create.call(()),
                        "+ Add"
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
                    div { class: "rounded-xl border border-border/40 bg-card/80 p-6 flex h-64 flex-col items-center justify-center gap-4",
                        p { class: "text-lg font-medium", "No subscriptions yet" }
                        p { class: "text-sm text-muted-foreground", "Click the button above to add your first subscription" }
                        button {
                            class: "hidden md:inline-flex bg-primary text-primary-foreground font-semibold px-4 py-2 rounded-lg text-sm hover:bg-primary/90 active:scale-[0.97] transition-all duration-200",
                            onclick: move |_| handle_create.call(()),
                            "+ Add Subscription"
                        }
                    }
                }
            } else if filtered.is_empty() {
                rsx! {
                    div { class: "rounded-xl border border-border/40 bg-card/80 p-6 flex h-40 flex-col items-center justify-center gap-2",
                        p { class: "text-sm text-muted-foreground", "No subscriptions match your filters" }
                        button {
                            class: "border px-3 py-1.5 rounded-md text-sm hover:bg-muted",
                            onclick: move |_| clear_filters.call(()),
                            "Clear filters"
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "space-y-4",

                        // ── Filter / Sort bar ──
                        div { class: "flex flex-row items-center gap-2 sm:flex-row sm:items-center",
                            div { class: "shrink-0 relative",
                                button {
                                    class: "border border-border/60 px-3 py-1.5 rounded-lg text-sm flex items-center gap-1.5 hover:bg-muted h-8 transition-colors duration-200",
                                    onclick: move |_| filters_popover.set(!filters_popover()),
                                    "▾ Filters"
                                    if active_filter_count > 0 {
                                        span { class: "ml-1 bg-secondary text-secondary-foreground rounded-full px-1.5 text-[10px]",
                                            {active_filter_count.to_string()}                                        }
                                    }
                                }
                                if filters_popover() {
                                    // Panel positioned relative to button container (parent has .relative)
                                    div {
                                        class: "absolute left-0 top-full mt-2 z-50 w-80 rounded-md border bg-popover p-3 shadow-md space-y-3",
                                        onclick: move |e| e.stop_propagation(),
                                    div { class: "flex items-center justify-between",
                                            h4 { class: "font-medium text-sm", "Filters" }
                                            if active_filter_count > 0 {
                                                button {
                                                    class: "text-xs text-muted-foreground hover:text-foreground px-2 py-0.5",
                                                    onclick: move |_| clear_filters.call(()),
                                                    "Clear all"
                                                }
                                            }
                                        }

                                        // Status
                                        div { class: "space-y-1.5",
                                            label { class: "text-xs font-medium text-muted-foreground", "Status" }
                                            select {
                                                class: "w-full h-8 rounded-md border bg-background px-2 text-sm",
                                                value: filters_val.status.as_deref().unwrap_or("all"),
                                                onchange: move |e: Event<FormData>| {
                                                    let v = e.value();
                                                    filters.write().status = if v == "all" { None } else { Some(v) };
                                                },
                                                for (val, label) in STATUS_OPTIONS {
                                                    option { value: val.to_string(), {label.to_string()} }
                                                }
                                            }
                                        }

                                        // Frequency
                                        div { class: "space-y-1.5",
                                            label { class: "text-xs font-medium text-muted-foreground", "Repeats every" }
                                            select {
                                                class: "w-full h-8 rounded-md border bg-background px-2 text-sm",
                                                value: filters_val.frequency.as_deref().unwrap_or("all"),
                                                onchange: move |e: Event<FormData>| {
                                                    let v = e.value();
                                                    filters.write().frequency = if v == "all" { None } else { Some(v) };
                                                },
                                                option { value: "all", "All units" }
                                                for (val, label) in RECURRENCE_UNITS {
                                                    option { value: val.to_string(), {label.to_string()} }
                                                }
                                            }
                                        }

                                        // Category
                                        div { class: "space-y-1.5",
                                            label { class: "text-xs font-medium text-muted-foreground", "Category" }
                                            select {
                                                class: "w-full h-8 rounded-md border bg-background px-2 text-sm",
                                                value: filters_val.category_id.as_deref().unwrap_or("all"),
                                                onchange: move |e: Event<FormData>| {
                                                    let v = e.value();
                                                    filters.write().category_id = if v == "all" { None } else { Some(v) };
                                                },
                                                option { value: "all", "All categories" }
                                                for (id, name) in category_options.iter() {
                                                    option { value: id.to_string(), {name.to_string()} }
                                                }
                                            }
                                        }

                                        // Price range
                                        div { class: "space-y-1.5",
                                            label { class: "text-xs font-medium text-muted-foreground", "Price range" }
                                            div { class: "flex items-center gap-2",
                                                input {
                                                    r#type: "number",
                                                    class: "h-8 text-sm rounded-md border bg-background px-2 w-full",
                                                    placeholder: "Min",
                                                    value: filters_val.price_min.as_deref().unwrap_or(""),
                                                    oninput: move |e: Event<FormData>| {
                                                        let v = e.value();
                                                        filters.write().price_min = if v.is_empty() { None } else { Some(v) };
                                                    }
                                                }
                                                span { class: "text-muted-foreground text-xs", "–" }
                                                input {
                                                    r#type: "number",
                                                    class: "h-8 text-sm rounded-md border bg-background px-2 w-full",
                                                    placeholder: "Max",
                                                    value: filters_val.price_max.as_deref().unwrap_or(""),
                                                    oninput: move |e: Event<FormData>| {
                                                        let v = e.value();
                                                        filters.write().price_max = if v.is_empty() { None } else { Some(v) };
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    // Click-outside overlay (after panel so panel gets events first)
                                    div {
                                        class: "fixed inset-0 z-40",
                                        onclick: move |_| filters_popover.set(false),
                                    }
                                }
                            }

                            // Clear button
                            {if active_filter_count > 0 {
                                rsx! {
                                    button {
                                        class: "text-sm px-2 py-1.5 hover:bg-muted rounded-lg flex items-center gap-1 h-8 transition-colors duration-200",
                                        onclick: move |_| clear_filters.call(()),
                                        X { size: 14 }
                                        "Clear"
                                    }
                                }
                            } else {
                                rsx! { div {} }
                            }}

                            // Sort
                            div { class: "flex items-center gap-1.5 ml-auto",
                                span { class: "text-sm text-muted-foreground", ArrowUpDown { size: 16 } }
                                select {
                                    class: "h-8 w-[130px] sm:w-[150px] rounded-md border bg-background px-2 text-sm",
                                    value: sort_val.field.to_string(),
                                    onchange: move |e: Event<FormData>| {
                                        sort.write().field = e.value();
                                    },
                                    for (val, label) in SORT_OPTIONS {
                                        option { value: val.to_string(), {label.to_string()} }
                                    }
                                }
                                button {
                                    class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted transition-colors duration-200",
                                    onclick: move |_| toggle_sort_direction.call(()),
                                    {sort_arrow.to_string()}                                }
                            }
                        }

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
                                view_mode.clone(),
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
                        {render_grid_card(sub, i, on_edit, on_delete, on_toggle, on_notes, on_duplicate)}
                    }
                }
            } else {
                div { class: "space-y-3",
                    for (_i, sub) in subs.iter().enumerate() {
                        {render_list_card(sub, on_edit, on_delete, on_toggle, on_notes, on_duplicate)}
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
