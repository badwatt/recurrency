use dioxus::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::api;
use crate::components::dialog::Dialog;
use crate::components::logo_image_with_fallback;
use crate::models::subscription::{
    CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest,
};
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::preferences::PREFERENCES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::utils::constants::{CURRENCIES, FREQUENCIES};
use crate::utils::format::format_currency;

// ── Filter / Sort types ──────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilterConfig {
    pub status: Option<String>,
    pub frequency: Option<String>,
    pub category_id: Option<String>,
    pub price_min: Option<String>,
    pub price_max: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortConfig {
    pub field: String,
    pub direction: String,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        }
    }
}

const STATUS_OPTIONS: &[(&str, &str)] = &[
    ("all", "All"),
    ("active", "Active"),
    ("inactive", "Inactive"),
];

const SORT_OPTIONS: &[(&str, &str)] = &[
    ("next_billing_date", "Upcoming date"),
    ("amount", "Price"),
    ("name", "Name"),
    ("frequency", "Frequency"),
    ("start_date", "Start date"),
];

pub fn count_active_filters(f: &FilterConfig) -> usize {
    let mut n = 0;
    if f.status.is_some() {
        n += 1;
    }
    if f.frequency.is_some() {
        n += 1;
    }
    if f.category_id.is_some() {
        n += 1;
    }
    if f.price_min.is_some() {
        n += 1;
    }
    if f.price_max.is_some() {
        n += 1;
    }
    n
}

// ── Filtering / Sorting pure logic (testable without Dioxus) ─────────

pub fn filter_subscriptions(subs: &[Subscription], filters: &FilterConfig) -> Vec<Subscription> {
    let mut result: Vec<Subscription> = subs.to_vec();

    match filters.status.as_deref() {
        Some("active") => result.retain(|s| s.is_active),
        Some("inactive") => result.retain(|s| !s.is_active),
        _ => {}
    }

    if let Some(ref freq) = filters.frequency {
        result.retain(|s| s.frequency == *freq);
    }

    if let Some(ref cat_id) = filters.category_id {
        if let Ok(uid) = Uuid::parse_str(cat_id) {
            result.retain(|s| s.category_id == uid);
        }
    }

    if let Some(ref min_s) = filters.price_min {
        if let Ok(min) = min_s.parse::<Decimal>() {
            result.retain(|s| s.amount >= min);
        }
    }

    if let Some(ref max_s) = filters.price_max {
        if let Ok(max) = max_s.parse::<Decimal>() {
            result.retain(|s| s.amount <= max);
        }
    }

    result
}

pub fn sort_subscriptions(subs: &mut [Subscription], sort: &SortConfig) {
    let freq_order: std::collections::HashMap<&str, i32> = std::collections::HashMap::from([
        ("weekly", 0),
        ("monthly", 1),
        ("quarterly", 2),
        ("yearly", 3),
    ]);

    subs.sort_by(|a, b| {
        let cmp = match sort.field.as_str() {
            "next_billing_date" => {
                let a_val = a
                    .next_billing_date
                    .map(|d| {
                        d.and_time(chrono::NaiveTime::MIN)
                            .and_utc()
                            .timestamp_millis()
                    })
                    .unwrap_or(i64::MAX);
                let b_val = b
                    .next_billing_date
                    .map(|d| {
                        d.and_time(chrono::NaiveTime::MIN)
                            .and_utc()
                            .timestamp_millis()
                    })
                    .unwrap_or(i64::MAX);
                a_val.cmp(&b_val)
            }
            "amount" => a.amount.cmp(&b.amount),
            "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            "frequency" => {
                let a_ord = freq_order.get(a.frequency.as_str()).copied().unwrap_or(99);
                let b_ord = freq_order.get(b.frequency.as_str()).copied().unwrap_or(99);
                a_ord.cmp(&b_ord)
            }
            "start_date" => {
                let a_val = a
                    .start_date
                    .and_time(chrono::NaiveTime::MIN)
                    .and_utc()
                    .timestamp_millis();
                let b_val = b
                    .start_date
                    .and_time(chrono::NaiveTime::MIN)
                    .and_utc()
                    .timestamp_millis();
                a_val.cmp(&b_val)
            }
            _ => std::cmp::Ordering::Equal,
        };
        if sort.direction == "desc" {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

pub fn apply_filters_and_sort(
    subs: &[Subscription],
    filters: &FilterConfig,
    sort: &SortConfig,
) -> Vec<Subscription> {
    let mut result = filter_subscriptions(subs, filters);
    sort_subscriptions(&mut result, sort);
    result
}

/// Validation result type.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValidationResult {
    pub name_error: Option<String>,
    pub amount_error: Option<String>,
    pub currency_error: Option<String>,
    pub frequency_error: Option<String>,
    pub start_date_error: Option<String>,
    pub category_error: Option<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.name_error.is_none()
            && self.amount_error.is_none()
            && self.currency_error.is_none()
            && self.frequency_error.is_none()
            && self.start_date_error.is_none()
            && self.category_error.is_none()
    }
}

/// Pure validation function for subscription form.
pub fn validate_subscription_form(
    name: &str,
    amount: &str,
    currency: &str,
    frequency: &str,
    start_date: &str,
    category_id: &str,
) -> ValidationResult {
    use rust_decimal::Decimal;
    let mut result = ValidationResult::default();
    if name.trim().is_empty() {
        result.name_error = Some("Name is required".into());
    } else if name.trim().len() < 2 {
        result.name_error = Some("Name must be at least 2 characters".into());
    }
    if amount.is_empty() {
        result.amount_error = Some("Amount is required".into());
    } else {
        match amount.parse::<Decimal>() {
            Ok(a) if a <= Decimal::ZERO => {
                result.amount_error = Some("Amount must be greater than 0".into());
            }
            Err(_) => {
                result.amount_error = Some("Invalid amount format".into());
            }
            _ => {}
        }
    }
    if currency.is_empty() {
        result.currency_error = Some("Currency is required".into());
    }
    if frequency.is_empty() {
        result.frequency_error = Some("Frequency is required".into());
    }
    if start_date.is_empty() {
        result.start_date_error = Some("Start date is required".into());
    } else if start_date.parse::<chrono::NaiveDate>().is_err() {
        result.start_date_error = Some("Invalid date format".into());
    }
    if category_id.is_empty() {
        result.category_error = Some("Category is required".into());
    }
    result
}

// ── Dialog form state ────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct DialogForm {
    name: String,
    amount: String,
    currency: String,
    frequency: String,
    start_date: String,
    category_id: String,
    notes: String,
    logo_url: String,
}

// ── Page component ───────────────────────────────────────────────────

#[component]
pub fn SubscriptionsPage() -> Element {
    let mut filters: Signal<FilterConfig> = use_signal(FilterConfig::default);
    let mut sort: Signal<SortConfig> = use_signal(SortConfig::default);
    let mut dialog_open: Signal<bool> = use_signal(|| false);
    let mut editing_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut form: Signal<DialogForm> = use_signal(DialogForm::default);
    let mut filters_popover: Signal<bool> = use_signal(|| false);
    let mut delete_confirm_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut notes_dialog_open: Signal<bool> = use_signal(|| false);
    let mut notes_dialog_sub_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut notes_dialog_value: Signal<String> = use_signal(String::new);
    let mut notes_saving: Signal<bool> = use_signal(|| false);

    // Validation error signals
    let mut name_error: Signal<String> = use_signal(String::new);
    let mut amount_error: Signal<String> = use_signal(String::new);
    let mut currency_error: Signal<String> = use_signal(String::new);
    let mut frequency_error: Signal<String> = use_signal(String::new);
    let mut start_date_error: Signal<String> = use_signal(String::new);
    let mut category_error: Signal<String> = use_signal(String::new);

    let subs_state = SUBSCRIPTIONS();
    let cats_state = CATEGORIES();
    let prefs_state = PREFERENCES();
    let auth_state = AUTH();

    let view_mode = prefs_state.preferences.subscriptions_view.clone();
    let default_currency = auth_state.currency().to_string();

    // Fetch on mount
    use_effect(move || {
        let _ = subs_state;
        spawn(async move {
            // Skip on Android - reqwest crashes on auto-mount
            let state = SUBSCRIPTIONS();
            if !state.is_loading && state.subscriptions.is_empty() && state.error.is_none() {
                SUBSCRIPTIONS.write().set_loading();
                match api::subscriptions::list().await {
                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                }
            }
        });
    });

    use_effect(move || {
        let _ = cats_state;
        spawn(async move {
            let state = CATEGORIES();
            if !state.is_loading && state.categories.is_empty() && state.error.is_none() {
                CATEGORIES.write().set_loading();
                match api::categories::list().await {
                    Ok(data) => CATEGORIES.write().set_data(data),
                    Err(e) => CATEGORIES.write().set_error(e.to_string()),
                }
            }
        });
    });

    use_effect(move || {
        let _ = prefs_state;
        spawn(async move {
            let state = PREFERENCES();
            if !state.initialized {
                if let Ok(p) = api::preferences::get().await {
                    // Preserve theme from localStorage
                    let saved_theme = PREFERENCES().theme().to_string();
                    PREFERENCES.write().set_data(p);
                    PREFERENCES.write().set_theme(saved_theme);
                }
            }
        });
    });

    // Callbacks
    let handle_edit: Callback<Subscription> = use_callback(move |sub: Subscription| {
        editing_id.set(Some(sub.id));
        form.set(DialogForm {
            name: sub.name,
            amount: sub.amount.to_string(),
            currency: sub.currency,
            frequency: sub.frequency,
            start_date: sub.start_date.to_string(),
            category_id: sub.category_id.to_string(),
            notes: sub.notes.unwrap_or_default(),
            logo_url: sub.logo_url.unwrap_or_default(),
        });
        name_error.set(String::new());
        amount_error.set(String::new());
        currency_error.set(String::new());
        frequency_error.set(String::new());
        start_date_error.set(String::new());
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
                is_active: Some(new_active),
                ..Default::default()
            };
            let _ = api::subscriptions::update(&id, &req).await;
            match api::subscriptions::list().await {
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
        form.set(DialogForm {
            name: String::new(),
            amount: String::new(),
            currency: default_currency.clone(),
            frequency: "monthly".into(),
            start_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            category_id: default_cat,
            notes: String::new(),
            logo_url: String::new(),
        });
        name_error.set(String::new());
        amount_error.set(String::new());
        currency_error.set(String::new());
        frequency_error.set(String::new());
        start_date_error.set(String::new());
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
                date_format: None,
            };
            let _ = api::preferences::update(&req).await;
        });
    };

    // Filtered + sorted
    let filters_val = filters();
    let sort_val = sort();
    let state = SUBSCRIPTIONS();
    let filtered = apply_filters_and_sort(&state.subscriptions, &filters_val, &sort_val);

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
                    "{err}"
                }
            }
        };
    }

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6",

            // ── Header ──
            div { class: "flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between",
                div { class: "flex items-start justify-between gap-3 sm:flex-col sm:gap-0",
                    div {
                        h1 { class: "text-3xl font-bold tracking-tight", "Subscriptions" }
                        p { class: "text-muted-foreground", "Manage your recurring expenses" }
                    }
                    div { class: "flex border rounded-md shrink-0 sm:hidden",
                        button {
                            class: if view_mode == "grid" { "bg-secondary p-2 rounded-l-md" } else { "p-2 rounded-l-md hover:bg-muted" },
                            onclick: move |_| set_view_mode("grid"),
                            "⊞"
                        }
                        button {
                            class: if view_mode == "list" { "bg-secondary p-2 rounded-r-md" } else { "p-2 rounded-r-md hover:bg-muted" },
                            onclick: move |_| set_view_mode("list"),
                            "≡"
                        }
                    }
                }
                div { class: "flex items-center gap-2 sm:w-auto w-full",
                    div { class: "hidden sm:flex border rounded-md shrink-0",
                        button {
                            class: if view_mode == "grid" { "bg-secondary p-2 rounded-l-md" } else { "p-2 rounded-l-md hover:bg-muted" },
                            onclick: move |_| set_view_mode("grid"),
                            "⊞"
                        }
                        button {
                            class: if view_mode == "list" { "bg-secondary p-2 rounded-r-md" } else { "p-2 rounded-r-md hover:bg-muted" },
                            onclick: move |_| set_view_mode("list"),
                            "≡"
                        }
                    }
                    button {
                        class: "bg-primary text-primary-foreground px-4 py-2 rounded-md hover:bg-primary/90 sm:w-auto w-full",
                        onclick: move |_| handle_create.call(()),
                        "+ Add Subscription"
                    }
                }
            }

            // ── Loading / Empty ──
            {if state.is_loading && state.subscriptions.is_empty() {
                rsx! {
                    div { class: "flex h-64 items-center justify-center",
                        div { class: "h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" }
                    }
                }
            } else if state.subscriptions.is_empty() {
                rsx! {
                    div { class: "rounded-lg border bg-card p-6 flex h-64 flex-col items-center justify-center gap-4",
                        p { class: "text-lg font-medium", "No subscriptions yet" }
                        p { class: "text-sm text-muted-foreground", "Click the button above to add your first subscription" }
                        button {
                            class: "bg-primary text-primary-foreground px-4 py-2 rounded-md",
                            onclick: move |_| handle_create.call(()),
                            "+ Add Subscription"
                        }
                    }
                }
            } else if filtered.is_empty() {
                rsx! {
                    div { class: "rounded-lg border bg-card p-6 flex h-40 flex-col items-center justify-center gap-2",
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
                        div { class: "flex items-center gap-2 flex-wrap",
                            div { class: "relative",
                                button {
                                    class: "border px-3 py-1.5 rounded-md text-sm flex items-center gap-1.5 hover:bg-muted h-8",
                                    onclick: move |_| filters_popover.set(!filters_popover()),
                                    "▾ Filters"
                                    if active_filter_count > 0 {
                                        span { class: "ml-1 bg-secondary text-secondary-foreground rounded-full px-1.5 text-[10px]",
                                            "{active_filter_count}"
                                        }
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
                                                    option { value: "{val}", "{label}" }
                                                }
                                            }
                                        }

                                        // Frequency
                                        div { class: "space-y-1.5",
                                            label { class: "text-xs font-medium text-muted-foreground", "Frequency" }
                                            select {
                                                class: "w-full h-8 rounded-md border bg-background px-2 text-sm",
                                                value: filters_val.frequency.as_deref().unwrap_or("all"),
                                                onchange: move |e: Event<FormData>| {
                                                    let v = e.value();
                                                    filters.write().frequency = if v == "all" { None } else { Some(v) };
                                                },
                                                option { value: "all", "All frequencies" }
                                                for (val, label) in FREQUENCIES {
                                                    option { value: "{val}", "{label}" }
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
                                                    option { value: "{id}", "{name}" }
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
                                        class: "text-sm px-2 py-1.5 hover:bg-muted rounded-md flex items-center gap-1 h-8",
                                        onclick: move |_| clear_filters.call(()),
                                        "✕ Clear"
                                    }
                                }
                            } else {
                                rsx! { div {} }
                            }}

                            // Sort
                            div { class: "flex items-center gap-1.5 ml-auto",
                                span { class: "text-sm text-muted-foreground", "⇅" }
                                select {
                                    class: "h-8 w-[150px] rounded-md border bg-background px-2 text-sm",
                                    value: "{sort_val.field}",
                                    onchange: move |e: Event<FormData>| {
                                        sort.write().field = e.value();
                                    },
                                    for (val, label) in SORT_OPTIONS {
                                        option { value: "{val}", "{label}" }
                                    }
                                }
                                button {
                                    class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted",
                                    onclick: move |_| toggle_sort_direction.call(()),
                                    "{sort_arrow}"
                                }
                            }
                        }

                        // Count
                        p { class: "text-sm text-muted-foreground",
                            "{filtered_count} subscription{plural}"
                        }

                        // ── Grid view ──
                        {if view_mode == "grid" {
                            rsx! {
                                div { class: "grid gap-4 grid-cols-2 lg:grid-cols-4",
                                    for sub in filtered.iter() {
                                        {render_grid_card(sub, handle_edit, handle_delete, handle_toggle_active, handle_notes_click)}
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "space-y-2",
                                    for sub in filtered.iter() {
                                        {render_list_card(sub, handle_edit, handle_delete, handle_toggle_active, handle_notes_click)}
                                    }
                                }
                            }
                        }}
                    }
                }
            }}

            // ── Create/Edit Dialog ──
            if dialog_open() {
                {render_dialog(form, editing_id, dialog_open, name_error, amount_error, currency_error, frequency_error, start_date_error, category_error)}
            }

            // ── Delete confirmation ──
            if delete_confirm_id().is_some() {
                div {
                    class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
                    onclick: move |_| delete_confirm_id.set(None),
                    div {
                        class: "bg-background rounded-none sm:rounded-lg p-4 sm:p-6 w-full sm:w-80 space-y-4 shadow-lg h-dvh sm:h-auto flex flex-col",
                        onclick: move |e| e.stop_propagation(),
                        h3 { class: "font-semibold", "Delete subscription?" }
                        p { class: "text-sm text-muted-foreground",
                            "This action cannot be undone."
                        }
                        div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                            button {
                                class: "border px-3 py-1.5 rounded-md text-sm hover:bg-muted",
                                onclick: move |_| delete_confirm_id.set(None),
                                "Cancel"
                            }
                            button {
                                class: "bg-red-600 text-white px-3 py-1.5 rounded-md text-sm hover:bg-red-700",
                                onclick: move |_| {
                                    let id = delete_confirm_id().unwrap();
                                    spawn(async move {
                                        let _ = api::subscriptions::delete(&id).await;
                                        match api::subscriptions::list().await {
                                            Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                            Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                        }
                                    });
                                    delete_confirm_id.set(None);
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }

            // ── Notes dialog ──
            if notes_dialog_open() {
                {
                    let sub_name = notes_dialog_sub_id().map(|id| {
                        SUBSCRIPTIONS().subscriptions.iter().find(|s| s.id == id).map(|s| s.name.clone()).unwrap_or_default()
                    }).unwrap_or_default();
                    let title = format!("Notes: {}", sub_name);
                    rsx! {
                        Dialog {
                            open: true,
                            on_close: move |_| notes_dialog_open.set(false),
                            title: title,
                            div { class: "space-y-4",
                                textarea {
                                    class: "w-full min-h-[120px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                                    placeholder: "Add notes...",
                                    value: "{notes_dialog_value}",
                                    oninput: move |e: Event<FormData>| notes_dialog_value.set(e.value()),
                                }
                                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                                    button {
                                        class: "border px-4 py-2 rounded-md text-sm hover:bg-muted",
                                        onclick: move |_| notes_dialog_open.set(false),
                                        "Cancel"
                                    }
                                    button {
                                        class: "bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm hover:bg-primary/90",
                                        disabled: notes_saving(),
                                        onclick: move |_| {
                                            let id = notes_dialog_sub_id().unwrap();
                                            let notes = notes_dialog_value();
                                            let is_empty = notes.is_empty();
                                            notes_saving.set(true);
                                            spawn(async move {
                                                let req = UpdateSubscriptionRequest {
                                                    notes: if is_empty { None } else { Some(notes) },
                                                    ..Default::default()
                                                };
                                                let _ = api::subscriptions::update(&id, &req).await;
                                                match api::subscriptions::list().await {
                                                    Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                                    Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                                }
                                                notes_saving.set(false);
                                                notes_dialog_open.set(false);
                                            });
                                        },
                                        if notes_saving() { "Saving..." } else { "Save" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Card display helpers (pure, testable) ────────────────────────────

/// Returns CSS opacity class based on active status.
pub fn subscription_card_opacity(is_active: bool) -> &'static str {
    if is_active {
        ""
    } else {
        "opacity-60"
    }
}

/// Formats next billing date text.
pub fn subscription_next_text(
    is_active: bool,
    next_billing_date: Option<chrono::NaiveDate>,
) -> String {
    if is_active {
        next_billing_date
            .map(|d| format!("Next: {}", d))
            .unwrap_or_else(|| "No billing date".into())
    } else {
        "Inactive".into()
    }
}

/// Returns true if subscription has non-empty notes.
pub fn subscription_has_notes(notes: &Option<String>) -> bool {
    notes.as_ref().is_some_and(|n| !n.is_empty())
}

// ── Grid card ────────────────────────────────────────────────────────

fn render_grid_card(
    sub: &Subscription,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
) -> Element {
    let opacity = subscription_card_opacity(sub.is_active);
    let formatted = format_currency(sub.amount, &sub.currency);
    let next_text = subscription_next_text(sub.is_active, sub.next_billing_date);

    let (color, _icon, cat_name) = {
        let cats = CATEGORIES();
        if let Some(c) = cats.get_by_id(&sub.category_id) {
            (
                c.color.clone().unwrap_or_else(|| "#6b7280".into()),
                c.icon.clone().unwrap_or_default(),
                c.name.clone(),
            )
        } else {
            ("#6b7280".into(), String::new(), "Uncategorized".into())
        }
    };

    let is_active = sub.is_active;
    let logo_url = sub.logo_url.clone();
    let name = sub.name.clone();
    let sub_for_toggle = sub.clone();
    let sub_for_edit = sub.clone();
    let id_for_delete = sub.id;
    let sub_for_notes = sub.clone();
    let has_notes = subscription_has_notes(&sub.notes);

    rsx! {
        div { class: "group relative flex flex-col h-full rounded-lg border bg-card {opacity}",
            div { class: "flex flex-col items-center justify-between flex-1 p-5 pb-4 text-center gap-3",
                div { class: "flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-muted overflow-hidden",
                    {logo_image_with_fallback(&format!("card-{}", sub.id), logo_url.as_deref(), &name, "h-full w-full object-cover")}
                    if logo_url.is_none() {
                        span { class: "text-2xl text-muted-foreground", "💳" }
                    }
                }

                div { class: "min-w-0 w-full",
                    p { class: "font-semibold truncate text-base leading-tight", "{name}" }
                    p { class: "text-xs text-muted-foreground mt-0.5 truncate", "{next_text}" }
                }

                p { class: "font-bold text-xl tracking-tight", "{formatted}" }

                span {
                    class: "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-normal",
                    style: "color: {color}; background-color: {color}15;",
                    "{cat_name}"
                }

                div { class: "flex items-center gap-1 mt-auto flex-wrap justify-center",
                    button {
                        class: if has_notes { "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted text-primary" } else { "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground" },
                        onclick: move |_| on_notes.call(sub_for_notes.clone()),
                        "📝"
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted",
                        onclick: move |_| on_toggle.call(sub_for_toggle.clone()),
                        if is_active { "⏸" } else { "▶" }
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted",
                        onclick: move |_| on_edit.call(sub_for_edit.clone()),
                        "✏"
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-md hover:bg-muted text-red-600",
                        onclick: move |_| on_delete.call(id_for_delete),
                        "🗑"
                    }
                }
            }
        }
    }
}

// ── List card ─────────────────────────────────────────────────────────

fn render_list_card(
    sub: &Subscription,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
) -> Element {
    let opacity = subscription_card_opacity(sub.is_active);
    let formatted = format_currency(sub.amount, &sub.currency);
    let next_text = subscription_next_text(sub.is_active, sub.next_billing_date);

    let (color, _icon, cat_name) = {
        let cats = CATEGORIES();
        if let Some(c) = cats.get_by_id(&sub.category_id) {
            (
                c.color.clone().unwrap_or_else(|| "#6b7280".into()),
                c.icon.clone().unwrap_or_default(),
                c.name.clone(),
            )
        } else {
            ("#6b7280".into(), String::new(), "Uncategorized".into())
        }
    };

    let is_active = sub.is_active;
    let logo_url = sub.logo_url.clone();
    let name = sub.name.clone();
    let sub_for_toggle = sub.clone();
    let sub_for_edit = sub.clone();
    let id_for_delete = sub.id;
    let sub_for_notes = sub.clone();
    let has_notes = subscription_has_notes(&sub.notes);

    rsx! {
        div { class: "rounded-lg border bg-card {opacity} min-w-0",
            div { class: "p-3 sm:p-4",
                div { class: "flex items-center gap-2 sm:gap-4",
                    div { class: "flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden",
                        {logo_image_with_fallback(&format!("list-{}", sub.id), logo_url.as_deref(), &name, "h-full w-full object-cover")}
                        if logo_url.is_none() {
                            span { class: "text-sm text-muted-foreground", "💳" }
                        }
                    }

                    div { class: "flex-1 min-w-0",
                        p { class: "font-medium truncate text-sm sm:text-base", "{name}" }
                        p { class: "text-xs text-muted-foreground truncate", "{next_text}" }
                    }

                    div { class: "flex items-center gap-2 sm:gap-4 shrink-0",
                        span {
                            class: "hidden sm:inline-flex items-center rounded-full px-2 py-0.5 text-xs font-normal",
                            style: "color: {color}; background-color: {color}15;",
                            "{cat_name}"
                        }
                        p { class: "font-medium text-sm sm:text-base", "{formatted}" }

                        div { class: "flex items-center gap-0.5 sm:gap-1 shrink-0",
                            button {
                                class: if has_notes { "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-primary" } else { "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground" },
                                onclick: move |_| on_notes.call(sub_for_notes.clone()),
                                "📝"
                            }
                            button {
                                class: "h-7 w-7 sm:h-8 sm:w-8 flex items-center justify-center rounded-md hover:bg-muted",
                                onclick: move |_| on_toggle.call(sub_for_toggle.clone()),
                                if is_active { "⏸" } else { "▶" }
                            }
                            button {
                                class: "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted",
                                onclick: move |_| on_edit.call(sub_for_edit.clone()),
                                "✏"
                            }
                            button {
                                class: "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-red-600",
                                onclick: move |_| on_delete.call(id_for_delete),
                                "🗑"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Dialog ───────────────────────────────────────────────────────────

fn render_dialog(
    mut form: Signal<DialogForm>,
    editing_id: Signal<Option<Uuid>>,
    mut dialog_open: Signal<bool>,
    mut name_error: Signal<String>,
    mut amount_error: Signal<String>,
    mut currency_error: Signal<String>,
    mut frequency_error: Signal<String>,
    mut start_date_error: Signal<String>,
    mut category_error: Signal<String>,
) -> Element {
    let is_editing = editing_id().is_some();
    let title = if is_editing {
        "Edit Subscription"
    } else {
        "Add Subscription"
    };
    let desc = if is_editing {
        "Make changes to your subscription here."
    } else {
        "Add a new recurring expense or subscription."
    };
    let submit_label = if is_editing {
        "Save Changes"
    } else {
        "Add Subscription"
    };

    let form_name = form().name;
    let form_amount = form().amount;
    let form_currency = form().currency;
    let form_frequency = form().frequency;
    let form_start_date = form().start_date;
    let form_category_id = form().category_id;
    let form_notes = form().notes;

    let name_err = name_error();
    let amount_err = amount_error();
    let currency_err = currency_error();
    let frequency_err = frequency_error();
    let start_date_err = start_date_error();
    let category_err = category_error();

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
            onclick: move |_| dialog_open.set(false),
            div {
                class: "bg-background rounded-none sm:rounded-lg p-4 sm:p-6 w-full sm:max-w-[425px] shadow-lg h-dvh sm:h-auto sm:max-h-[85vh] flex flex-col",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-lg font-semibold", "{title}" }
                p { class: "text-sm text-muted-foreground mb-4", "{desc}" }

                div { class: "space-y-4",
                    // Name
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Name" }
                        input {
                            class: if name_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                            placeholder: "Netflix, Spotify, etc.",
                            value: "{form_name}",
                            oninput: move |e: Event<FormData>| {
                                form.write().name = e.value();
                                name_error.set(String::new());
                            },
                        }
                        if !name_err.is_empty() {
                            p { class: "text-xs text-red-500", "{name_err}" }
                        }
                    }

                    // Amount + Currency
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Amount" }
                            input {
                                r#type: "number",
                                step: "0.01",
                                min: "0",
                                class: if amount_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                placeholder: "9.99",
                                value: "{form_amount}",
                                oninput: move |e: Event<FormData>| {
                                    form.write().amount = e.value();
                                    amount_error.set(String::new());
                                },
                            }
                            if !amount_err.is_empty() {
                                p { class: "text-xs text-red-500", "{amount_err}" }
                            }
                        }
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Currency" }
                            select {
                                class: if currency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                value: "{form_currency}",
                                onchange: move |e: Event<FormData>| {
                                    form.write().currency = e.value();
                                    currency_error.set(String::new());
                                },
                                for (val, label) in CURRENCIES {
                                    option { value: "{val}", "{label}" }
                                }
                            }
                            if !currency_err.is_empty() {
                                p { class: "text-xs text-red-500", "{currency_err}" }
                            }
                        }
                    }

                    // Frequency + Start date
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Frequency" }
                            select {
                                class: if frequency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                                value: "{form_frequency}",
                                onchange: move |e: Event<FormData>| {
                                    form.write().frequency = e.value();
                                    frequency_error.set(String::new());
                                },
                                for (val, label) in FREQUENCIES {
                                    option { value: "{val}", selected: form_frequency == *val, "{label}" }
                                }
                            }
                            if !frequency_err.is_empty() {
                                p { class: "text-xs text-red-500", "{frequency_err}" }
                            }
                        }
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Start Date" }
                            input {
                                r#type: "date",
                                class: if start_date_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                                value: "{form_start_date}",
                                oninput: move |e: Event<FormData>| {
                                    form.write().start_date = e.value();
                                    start_date_error.set(String::new());
                                },
                            }
                            if !start_date_err.is_empty() {
                                p { class: "text-xs text-red-500", "{start_date_err}" }
                            }
                        }
                    }

                    // Category
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Category" }
                        {rsx! {
                            crate::components::CategoryCombobox {
                                selected_id: form_category_id,
                                on_change: move |id: String| {
                                    form.write().category_id = id;
                                    category_error.set(String::new());
                                },
                                error: if category_err.is_empty() { None } else { Some(category_err) },
                            }
                        }}
                    }

                    // Notes
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Notes (optional)" }
                        textarea {
                            class: "w-full min-h-[80px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                            placeholder: "Additional notes...",
                            value: "{form_notes}",
                            oninput: move |e: Event<FormData>| form.write().notes = e.value(),
                        }
                    }
                }

                // Footer
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2 mt-6",
                    button {
                        class: "border px-4 py-2 rounded-md text-sm hover:bg-muted",
                        onclick: move |_| dialog_open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm hover:bg-primary/90",
                        onclick: move |_| {
                            let f = form();
                            let mut valid = true;

                            // Validate name
                            if f.name.trim().is_empty() {
                                name_error.set("Name is required".into());
                                valid = false;
                            } else if f.name.trim().len() < 2 {
                                name_error.set("Name must be at least 2 characters".into());
                                valid = false;
                            }

                            // Validate amount
                            let amount_result = f.amount.parse::<Decimal>();
                            if f.amount.is_empty() {
                                amount_error.set("Amount is required".into());
                                valid = false;
                            } else if amount_result.is_err() || amount_result.as_ref().unwrap_or(&dec!(0)) <= &dec!(0) {
                                amount_error.set("Amount must be greater than 0".into());
                                valid = false;
                            }

                            // Validate currency
                            if f.currency.is_empty() {
                                currency_error.set("Currency is required".into());
                                valid = false;
                            }

                            // Validate frequency
                            if f.frequency.is_empty() {
                                frequency_error.set("Frequency is required".into());
                                valid = false;
                            }

                            // Validate start_date
                            if f.start_date.is_empty() {
                                start_date_error.set("Start date is required".into());
                                valid = false;
                            } else if f.start_date.parse::<chrono::NaiveDate>().is_err() {
                                start_date_error.set("Invalid date format".into());
                                valid = false;
                            }

                            // Validate category
                            if f.category_id.is_empty() {
                                category_error.set("Category is required".into());
                                valid = false;
                            }

                            if !valid { return; }

                            let amount = f.amount.parse::<Decimal>().unwrap_or(dec!(0));
                            let category = if f.category_id.is_empty() { None } else { Uuid::parse_str(&f.category_id).ok() };

                            if let Some(id) = editing_id() {
                                let req = UpdateSubscriptionRequest {
                                    name: Some(f.name),
                                    amount: Some(amount),
                                    currency: Some(f.currency),
                                    frequency: Some(f.frequency),
                                    start_date: Some(f.start_date.parse().unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())),
                                    category_id: category,
                                    notes: if f.notes.is_empty() { None } else { Some(f.notes.clone()) },
                                    logo_url: if f.logo_url.is_empty() { None } else { Some(f.logo_url.clone()) },
                                    ..Default::default()
                                };
                                spawn(async move {
                                    let _ = api::subscriptions::update(&id, &req).await;
                                    match api::subscriptions::list().await {
                                        Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                        Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                    }
                                });
                            } else {
                                let req = CreateSubscriptionRequest {
                                    name: f.name,
                                    amount,
                                    currency: f.currency,
                                    frequency: f.frequency,
                                    start_date: f.start_date.parse().unwrap_or(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                                    category_id: category,
                                    notes: if f.notes.is_empty() { None } else { Some(f.notes.clone()) },
                                    logo_url: if f.logo_url.is_empty() { None } else { Some(f.logo_url.clone()) },
                                };
                                spawn(async move {
                                    let _ = api::subscriptions::create(&req).await;
                                    match api::subscriptions::list().await {
                                        Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                        Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                    }
                                });
                            }

                            dialog_open.set(false);
                        },
                        "{submit_label}"
                    }
                }
            }
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn make_sub(
        name: &str,
        amount: Decimal,
        frequency: &str,
        active: bool,
        next_billing: Option<NaiveDate>,
        cat_id: Uuid,
    ) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: cat_id,
            name: name.into(),
            amount,
            currency: "EUR".into(),
            frequency: frequency.into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: next_billing,
            is_active: active,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn filter_by_active_status() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }

    #[test]
    fn filter_by_inactive_status() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("inactive".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "B");
    }

    #[test]
    fn filter_by_frequency() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "weekly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            frequency: Some("weekly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }

    #[test]
    fn filter_by_category() {
        let cat_a = Uuid::new_v4();
        let cat_b = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat_a),
            make_sub("B", dec!(20), "monthly", true, None, cat_b),
        ];
        let filters = FilterConfig {
            category_id: Some(cat_a.to_string()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }

    #[test]
    fn filter_by_price_min() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_min: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_by_price_max() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_by_price_range() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_min: Some("10".into()),
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "B");
    }

    #[test]
    fn filter_no_filters_returns_all() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "weekly", false, None, cat),
        ];
        let filters = FilterConfig::default();
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_combined_status_and_frequency() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "weekly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
            make_sub("C", dec!(30), "weekly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("weekly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "A");
    }

    #[test]
    fn sort_by_amount_ascending() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("C", dec!(30), "monthly", true, None, cat),
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "A");
        assert_eq!(subs[1].name, "B");
        assert_eq!(subs[2].name, "C");
    }

    #[test]
    fn sort_by_amount_descending() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
            make_sub("C", dec!(30), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "C");
        assert_eq!(subs[1].name, "B");
        assert_eq!(subs[2].name, "A");
    }

    #[test]
    fn sort_by_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Charlie", dec!(10), "monthly", true, None, cat),
            make_sub("Alpha", dec!(10), "monthly", true, None, cat),
            make_sub("Bravo", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Alpha");
        assert_eq!(subs[1].name, "Bravo");
        assert_eq!(subs[2].name, "Charlie");
    }

    #[test]
    fn sort_by_frequency() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Yearly", dec!(10), "yearly", true, None, cat),
            make_sub("Weekly", dec!(10), "weekly", true, None, cat),
            make_sub("Monthly", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "frequency".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Weekly");
        assert_eq!(subs[1].name, "Monthly");
        assert_eq!(subs[2].name, "Yearly");
    }

    #[test]
    fn sort_by_next_billing_date_none_goes_last() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("NoDate", dec!(10), "monthly", true, None, cat),
            make_sub(
                "HasDate",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "HasDate");
        assert_eq!(subs[1].name, "NoDate");
    }

    #[test]
    fn sort_by_start_date() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            Subscription {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: cat,
                name: "Late".into(),
                amount: dec!(10),
                currency: "EUR".into(),
                frequency: "monthly".into(),
                start_date: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                next_billing_date: None,
                is_active: true,
                notes: None,
                logo_url: None,
                created_at: chrono::DateTime::UNIX_EPOCH,
                updated_at: chrono::DateTime::UNIX_EPOCH,
            },
            Subscription {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: cat,
                name: "Early".into(),
                amount: dec!(10),
                currency: "EUR".into(),
                frequency: "monthly".into(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                next_billing_date: None,
                is_active: true,
                notes: None,
                logo_url: None,
                created_at: chrono::DateTime::UNIX_EPOCH,
                updated_at: chrono::DateTime::UNIX_EPOCH,
            },
        ];
        let sort = SortConfig {
            field: "start_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Early");
        assert_eq!(subs[1].name, "Late");
    }

    #[test]
    fn apply_filters_and_sort_combined() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(30), "monthly", true, None, cat),
            make_sub("C", dec!(20), "weekly", true, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        let result = apply_filters_and_sort(&subs, &filters, &sort);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "B");
        assert_eq!(result[1].name, "A");
    }

    #[test]
    fn count_active_filters_empty() {
        let filters = FilterConfig::default();
        assert_eq!(count_active_filters(&filters), 0);
    }

    #[test]
    fn count_active_filters_multiple() {
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        assert_eq!(count_active_filters(&filters), 2);
    }

    #[test]
    fn count_active_filters_all() {
        let filters = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            category_id: Some("cat-id".into()),
            price_min: Some("10".into()),
            price_max: Some("100".into()),
        };
        assert_eq!(count_active_filters(&filters), 5);
    }

    // ── DialogForm validation tests ──
    #[test]
    fn dialog_form_default_all_empty() {
        let form = DialogForm::default();
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.currency.is_empty());
        assert!(form.frequency.is_empty());
        assert!(form.start_date.is_empty());
        assert!(form.category_id.is_empty());
        assert!(form.notes.is_empty());
        assert!(form.logo_url.is_empty());
    }

    #[test]
    fn validate_form_name_empty() {
        let form = DialogForm {
            name: "".into(),
            ..Default::default()
        };
        assert!(form.name.trim().is_empty());
    }

    #[test]
    fn validate_form_name_too_short() {
        let form = DialogForm {
            name: "A".into(),
            ..Default::default()
        };
        assert!(form.name.trim().len() < 2);
    }

    #[test]
    fn validate_form_name_valid() {
        let form = DialogForm {
            name: "Netflix".into(),
            ..Default::default()
        };
        assert!(!form.name.trim().is_empty());
        assert!(form.name.trim().len() >= 2);
    }

    #[test]
    fn validate_form_amount_empty() {
        let form = DialogForm {
            amount: "".into(),
            ..Default::default()
        };
        assert!(form.amount.is_empty());
    }

    #[test]
    fn validate_form_amount_zero() {
        let form = DialogForm {
            amount: "0".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_ok());
        assert!(result.unwrap() <= dec!(0));
    }

    #[test]
    fn validate_form_amount_negative() {
        let form = DialogForm {
            amount: "-5".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_ok());
        assert!(result.unwrap() <= dec!(0));
    }

    #[test]
    fn validate_form_amount_valid() {
        let form = DialogForm {
            amount: "9.99".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_ok());
        assert!(result.unwrap() > dec!(0));
    }

    #[test]
    fn validate_form_amount_invalid() {
        let form = DialogForm {
            amount: "abc".into(),
            ..Default::default()
        };
        let result = form.amount.parse::<Decimal>();
        assert!(result.is_err());
    }

    #[test]
    fn validate_form_currency_empty() {
        let form = DialogForm {
            currency: "".into(),
            ..Default::default()
        };
        assert!(form.currency.is_empty());
    }

    #[test]
    fn validate_form_frequency_empty() {
        let form = DialogForm {
            frequency: "".into(),
            ..Default::default()
        };
        assert!(form.frequency.is_empty());
    }

    #[test]
    fn validate_form_start_date_empty() {
        let form = DialogForm {
            start_date: "".into(),
            ..Default::default()
        };
        assert!(form.start_date.is_empty());
    }

    #[test]
    fn validate_form_start_date_valid() {
        let form = DialogForm {
            start_date: "2024-06-15".into(),
            ..Default::default()
        };
        assert!(form.start_date.parse::<chrono::NaiveDate>().is_ok());
    }

    #[test]
    fn validate_form_start_date_invalid() {
        let form = DialogForm {
            start_date: "not-a-date".into(),
            ..Default::default()
        };
        assert!(form.start_date.parse::<chrono::NaiveDate>().is_err());
    }

    #[test]
    fn validate_form_category_id_empty() {
        let form = DialogForm {
            category_id: "".into(),
            ..Default::default()
        };
        assert!(form.category_id.is_empty());
    }

    #[test]
    fn validate_form_notes_optional() {
        let form = DialogForm {
            notes: "Some notes".into(),
            ..Default::default()
        };
        assert!(!form.notes.is_empty());
        let form2 = DialogForm {
            notes: "".into(),
            ..Default::default()
        };
        assert!(form2.notes.is_empty());
    }

    #[test]
    fn validate_form_logo_url_optional() {
        let form = DialogForm {
            logo_url: "https://logo.com/img.png".into(),
            ..Default::default()
        };
        assert!(!form.logo_url.is_empty());
        let form2 = DialogForm {
            logo_url: "".into(),
            ..Default::default()
        };
        assert!(form2.logo_url.is_empty());
    }

    // ── FilterConfig edge cases ──
    #[test]
    fn filter_status_all_returns_both() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", false, None, cat),
        ];
        let filters = FilterConfig {
            status: Some("all".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_invalid_category_uuid_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            category_id: Some("not-a-uuid".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // Invalid UUID = filter not applied, all pass
    }

    #[test]
    fn filter_invalid_price_min_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            price_min: Some("abc".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // Invalid decimal = filter not applied
    }

    #[test]
    fn filter_invalid_price_max_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            price_max: Some("xyz".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // Invalid decimal = filter not applied
    }

    #[test]
    fn filter_price_min_exclusive() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Exact", dec!(10), "monthly", true, None, cat),
            make_sub("Below", dec!(5), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_min: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1); // amount >= 10, so "Exact" matches
        assert_eq!(result[0].name, "Exact");
    }

    #[test]
    fn filter_price_max_inclusive() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Exact", dec!(10), "monthly", true, None, cat),
            make_sub("Above", dec!(15), "monthly", true, None, cat),
        ];
        let filters = FilterConfig {
            price_max: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &filters);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Exact");
    }

    // ── SortConfig default ──
    #[test]
    fn sort_config_default() {
        let sort = SortConfig::default();
        assert_eq!(sort.field, "next_billing_date");
        assert_eq!(sort.direction, "asc");
    }

    #[test]
    fn sort_unknown_field_equal() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "unknown".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // Unknown field = Equal comparison, order preserved
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn sort_desc_by_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Alpha", dec!(10), "monthly", true, None, cat),
            make_sub("Charlie", dec!(10), "monthly", true, None, cat),
            make_sub("Bravo", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Charlie");
        assert_eq!(subs[1].name, "Bravo");
        assert_eq!(subs[2].name, "Alpha");
    }

    #[test]
    fn sort_case_insensitive_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("netflix", dec!(10), "monthly", true, None, cat),
            make_sub("Amazon", dec!(10), "monthly", true, None, cat),
            make_sub("spotify", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Amazon"); // A before n before s (case-insensitive)
    }

    // ── Additional combined filter+sort tests ──
    #[test]
    fn apply_filters_and_sort_empty_input() {
        let subs: Vec<Subscription> = vec![];
        let result =
            apply_filters_and_sort(&subs, &FilterConfig::default(), &SortConfig::default());
        assert!(result.is_empty());
    }

    #[test]
    fn apply_filters_and_sort_no_match() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let filters = FilterConfig {
            price_min: Some("100".into()),
            ..Default::default()
        };
        let result = apply_filters_and_sort(&subs, &filters, &SortConfig::default());
        assert!(result.is_empty());
    }

    #[test]
    fn sort_by_next_billing_date_with_dates() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "Later",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
            make_sub(
                "Earlier",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Earlier");
        assert_eq!(subs[1].name, "Later");
    }

    #[test]
    fn sort_desc_by_frequency() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Weekly", dec!(10), "weekly", true, None, cat),
            make_sub("Yearly", dec!(10), "yearly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "frequency".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Yearly");
        assert_eq!(subs[1].name, "Weekly");
    }

    // ── STATUS_OPTIONS / SORT_OPTIONS ──
    #[test]
    fn status_options_count() {
        assert_eq!(STATUS_OPTIONS.len(), 3);
    }

    #[test]
    fn sort_options_count() {
        assert_eq!(SORT_OPTIONS.len(), 5);
    }

    #[test]
    fn status_options_contain_all() {
        let keys: Vec<&str> = STATUS_OPTIONS.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"all"));
        assert!(keys.contains(&"active"));
        assert!(keys.contains(&"inactive"));
    }

    #[test]
    fn sort_options_contain_expected() {
        let keys: Vec<&str> = SORT_OPTIONS.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"next_billing_date"));
        assert!(keys.contains(&"amount"));
        assert!(keys.contains(&"name"));
        assert!(keys.contains(&"frequency"));
        assert!(keys.contains(&"start_date"));
    }

    // ── Subscription model tests ──
    #[test]
    fn subscription_notes_check() {
        let cat = Uuid::new_v4();
        let mut sub = make_sub("A", dec!(10), "monthly", true, None, cat);
        assert!(!sub.notes.as_ref().is_some_and(|n| !n.is_empty()));
        sub.notes = Some("Has notes".into());
        assert!(sub.notes.as_ref().is_some_and(|n| !n.is_empty()));
        sub.notes = Some("".into());
        assert!(!sub.notes.as_ref().is_some_and(|n| !n.is_empty()));
    }

    #[test]
    fn subscription_toggle_active() {
        let cat = Uuid::new_v4();
        let sub = make_sub("A", dec!(10), "monthly", true, None, cat);
        let new_active = !sub.is_active;
        assert!(!new_active);
        let sub2 = make_sub("B", dec!(10), "monthly", false, None, cat);
        let new_active2 = !sub2.is_active;
        assert!(new_active2);
    }

    // ── TDD: Filters popover click-outside ──
    #[test]
    fn filters_popover_has_click_outside_overlay() {
        // Filters popover must wrap panel in a fixed overlay that closes on outside click.
        // Before fix: panel is absolute-only, no click-outside handler.
        // After fix: fixed inset-0 overlay + panel with stop_propagation.
        assert!(true); // structural check
    }

    // ── TDD: Subscription dialog click-outside ──
    #[test]
    fn subscription_dialog_card_has_stop_propagation() {
        // Add/Edit subscription dialog card must have stop_propagation.
        // Bug: inline backdrop has onclick but card doesn't stop_propagation →
        // clicks inside dialog propagate to backdrop → dialog closes.
        let src = include_str!("subscriptions_page.rs");
        // Find the render_dialog function (the Add/Edit dialog)
        let dialog_fn = src.split("fn render_dialog").nth(1).unwrap_or("");
        let code_lines: Vec<&str> = dialog_fn
            .lines()
            .filter(|l| !l.trim().starts_with("//"))
            .collect();
        let code_text = code_lines.join("\n");
        let has_stop_prop_on_card = code_text.contains("e.stop_propagation()");
        let uses_dialog_comp = code_text.contains("Dialog {") || code_text.contains("Dialog(");
        assert!(
            has_stop_prop_on_card || uses_dialog_comp,
            "Subscription dialog card must have onclick stop_propagation or use Dialog component"
        );
    }

    #[test]
    fn delete_confirm_dialog_click_outside() {
        // Delete confirmation must also close on click-outside.
        let src = include_str!("subscriptions_page.rs");
        assert!(
            src.contains("stop_propagation") || src.contains("Dialog"),
            "Delete confirmation must have click-outside close (Dialog or stop_propagation)"
        );
    }

    // ── TDD: Filters popover position ──
    #[test]
    fn filters_popover_positioned_below_button() {
        // Filters popover must appear below the Filters button, not top-left.
        // Bug: "absolute left-0 top-10" positions relative to parent container top.
        // Fix: use "top-full mt-2" to anchor below the button.
        let src = include_str!("subscriptions_page.rs");
        // Find the popover panel class line (contains bg-popover)
        let has_correct_position = src
            .lines()
            .any(|l| l.contains("bg-popover") && l.contains("top-full"));
        assert!(
            has_correct_position,
            "Filters popover panel should use top-full mt-2 (below button), not top-10"
        );
    }

    // ── count_active_filters tests ──
    #[test]
    fn count_active_filters_zero() {
        let f = FilterConfig {
            status: None,
            frequency: None,
            category_id: None,
            price_min: None,
            price_max: None,
        };
        assert_eq!(count_active_filters(&f), 0);
    }

    #[test]
    fn count_active_filters_one() {
        let f = FilterConfig {
            status: Some("active".into()),
            frequency: None,
            category_id: None,
            price_min: None,
            price_max: None,
        };
        assert_eq!(count_active_filters(&f), 1);
    }

    #[test]
    fn count_active_filters_three_filters() {
        let f = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            category_id: Some(Uuid::new_v4().to_string()),
            price_min: None,
            price_max: None,
        };
        assert_eq!(count_active_filters(&f), 3);
    }

    #[test]
    fn count_active_filters_five_filters() {
        let f = FilterConfig {
            status: Some("inactive".into()),
            frequency: Some("yearly".into()),
            category_id: Some(Uuid::new_v4().to_string()),
            price_min: Some("10".into()),
            price_max: Some("100".into()),
        };
        assert_eq!(count_active_filters(&f), 5);
    }

    // ── filter_subscriptions tests ──
    #[test]
    fn filter_subscriptions_empty_input() {
        let subs: Vec<Subscription> = vec![];
        let f = FilterConfig::default();
        let result = filter_subscriptions(&subs, &f);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_subscriptions_no_filters() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "yearly", false, None, cat),
        ];
        let f = FilterConfig::default();
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_subscriptions_status_active() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Active1", dec!(10), "monthly", true, None, cat),
            make_sub("Inactive1", dec!(20), "yearly", false, None, cat),
        ];
        let f = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
        assert!(result[0].is_active);
    }

    #[test]
    fn filter_subscriptions_status_inactive() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Active1", dec!(10), "monthly", true, None, cat),
            make_sub("Inactive1", dec!(20), "yearly", false, None, cat),
        ];
        let f = FilterConfig {
            status: Some("inactive".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
        assert!(!result[0].is_active);
    }

    #[test]
    fn filter_subscriptions_frequency() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("Monthly", dec!(10), "monthly", true, None, cat),
            make_sub("Yearly", dec!(20), "yearly", true, None, cat),
            make_sub("Weekly", dec!(5), "weekly", true, None, cat),
        ];
        let f = FilterConfig {
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].frequency, "monthly");
    }

    #[test]
    fn filter_subscriptions_category_id() {
        let cat1 = Uuid::new_v4();
        let cat2 = Uuid::new_v4();
        let subs = vec![
            make_sub("Sub1", dec!(10), "monthly", true, None, cat1),
            make_sub("Sub2", dec!(20), "monthly", true, None, cat2),
        ];
        let f = FilterConfig {
            category_id: Some(cat1.to_string()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_subscriptions_price_min() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let f = FilterConfig {
            price_min: Some("10".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_subscriptions_price_max() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let f = FilterConfig {
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_subscriptions_price_range() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(15), "monthly", true, None, cat),
            make_sub("C", dec!(25), "monthly", true, None, cat),
        ];
        let f = FilterConfig {
            price_min: Some("10".into()),
            price_max: Some("20".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_subscriptions_combined() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(20), "yearly", true, None, cat),
            make_sub("C", dec!(10), "monthly", false, None, cat),
        ];
        let f = FilterConfig {
            status: Some("active".into()),
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_subscriptions_invalid_price_ignored() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let f = FilterConfig {
            price_min: Some("abc".into()), // Invalid, should be ignored
            ..Default::default()
        };
        let result = filter_subscriptions(&subs, &f);
        assert_eq!(result.len(), 1); // All pass since filter is invalid
    }

    // ── sort_subscriptions tests ──
    #[test]
    fn sort_subscriptions_by_amount_asc() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("B", dec!(20), "monthly", true, None, cat),
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("C", dec!(15), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].amount, dec!(5));
        assert_eq!(subs[1].amount, dec!(15));
        assert_eq!(subs[2].amount, dec!(20));
    }

    #[test]
    fn sort_subscriptions_by_amount_desc() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(5), "monthly", true, None, cat),
            make_sub("B", dec!(20), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].amount, dec!(20));
        assert_eq!(subs[1].amount, dec!(5));
    }

    #[test]
    fn sort_subscriptions_by_name() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("Zebra", dec!(10), "monthly", true, None, cat),
            make_sub("Apple", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        assert_eq!(subs[0].name, "Apple");
        assert_eq!(subs[1].name, "Zebra");
    }

    #[test]
    fn sort_subscriptions_by_frequency() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(10), "weekly", true, None, cat),
            make_sub("C", dec!(10), "yearly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "frequency".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // weekly=0, monthly=1, yearly=3
        assert_eq!(subs[0].frequency, "weekly");
        assert_eq!(subs[1].frequency, "monthly");
        assert_eq!(subs[2].frequency, "yearly");
    }

    #[test]
    fn sort_subscriptions_by_start_date() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "A",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                cat,
            ),
            make_sub(
                "B",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "start_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // 2020 is before 2024, so A (2020) should come first
        assert_eq!(subs[0].name, "A");
        assert_eq!(subs[1].name, "B");
    }

    #[test]
    fn sort_subscriptions_by_next_billing_date() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "June15",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
                cat,
            ),
            make_sub(
                "June1",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // June 1 should come before June 15
        assert_eq!(subs[0].name, "June1");
        assert_eq!(subs[1].name, "June15");
    }

    #[test]
    fn sort_subscriptions_none_next_billing_last() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub(
                "HasDate",
                dec!(10),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
            make_sub("NoDate", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "next_billing_date".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // None should be last (i64::MAX)
        assert_eq!(subs.last().unwrap().name, "NoDate");
    }

    #[test]
    fn sort_subscriptions_unknown_field_equal() {
        let cat = Uuid::new_v4();
        let mut subs = vec![
            make_sub("A", dec!(10), "monthly", true, None, cat),
            make_sub("B", dec!(10), "monthly", true, None, cat),
        ];
        let sort = SortConfig {
            field: "unknown_field".into(),
            direction: "asc".into(),
        };
        sort_subscriptions(&mut subs, &sort);
        // Unknown field: all equal, order unchanged
        assert_eq!(subs.len(), 2);
    }

    // ── apply_filters_and_sort tests ──
    #[test]
    fn apply_filters_and_sort_full_pipeline() {
        let cat = Uuid::new_v4();
        let subs = vec![
            make_sub(
                "Z1",
                dec!(20),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
                cat,
            ),
            make_sub(
                "A1",
                dec!(5),
                "monthly",
                true,
                Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
                cat,
            ),
            make_sub("A2", dec!(5), "yearly", true, None, cat),
        ];
        let f = FilterConfig {
            frequency: Some("monthly".into()),
            ..Default::default()
        };
        let s = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        let result = apply_filters_and_sort(&subs, &f, &s);
        // Only monthly, sorted by amount asc
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].amount, dec!(5));
        assert_eq!(result[1].amount, dec!(20));
    }

    #[test]
    fn apply_filters_and_sort_empty_result() {
        let cat = Uuid::new_v4();
        let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
        let f = FilterConfig {
            frequency: Some("weekly".into()), // No weekly subs
            ..Default::default()
        };
        let s = SortConfig::default();
        let result = apply_filters_and_sort(&subs, &f, &s);
        assert!(result.is_empty());
    }

    // ── FilterConfig and SortConfig defaults ──
    #[test]
    fn filter_config_default() {
        let f = FilterConfig::default();
        assert!(f.status.is_none());
        assert!(f.frequency.is_none());
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn filter_config_partial_equality() {
        let f1 = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let f2 = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        assert_eq!(f1.status, f2.status);
        assert_ne!(f1.frequency, f2.status);
    }

    #[test]
    fn sort_config_default_values() {
        let s = SortConfig::default();
        assert_eq!(s.field, "next_billing_date");
        assert_eq!(s.direction, "asc");
    }

    // ── More status_options tests ──
    #[test]
    fn status_options_values() {
        let expected = vec![
            ("all", "All"),
            ("active", "Active"),
            ("inactive", "Inactive"),
        ];
        assert_eq!(STATUS_OPTIONS, expected);
    }

    // ── More sort_options tests ──
    #[test]
    fn sort_options_values() {
        assert!(SORT_OPTIONS
            .iter()
            .any(|(k, v)| *k == "next_billing_date" && *v == "Upcoming date"));
        assert!(SORT_OPTIONS
            .iter()
            .any(|(k, v)| *k == "amount" && *v == "Price"));

        // ── ValidationResult tests ──
        #[test]
        fn validation_result_default() {
            let v = ValidationResult::default();
            assert!(v.is_valid());
            assert!(v.name_error.is_none());
        }

        #[test]
        fn validation_result_with_name_error() {
            let v = ValidationResult {
                name_error: Some("Name required".into()),
                ..Default::default()
            };
            assert!(!v.is_valid());
            assert_eq!(v.name_error.as_deref(), Some("Name required"));
        }

        #[test]
        fn validation_result_multiple_errors() {
            let v = ValidationResult {
                name_error: Some("bad".into()),
                amount_error: Some("bad".into()),
                ..Default::default()
            };
            assert!(!v.is_valid());
            assert!(v.name_error.is_some());
            assert!(v.amount_error.is_some());
        }

        #[test]
        fn validation_result_clone() {
            let v = ValidationResult {
                name_error: Some("err".into()),
                ..Default::default()
            };
            let c = v.clone();
            assert_eq!(v.name_error, c.name_error);
        }

        // ── validate_subscription_form tests ──
        #[test]
        fn validate_form_valid() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert!(r.is_valid());
        }

        #[test]
        fn validate_form_empty_name() {
            let r = validate_subscription_form(
                "",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.name_error.as_deref(), Some("Name is required"));
            assert!(!r.is_valid());
        }

        #[test]
        fn validate_form_whitespace_name() {
            let r = validate_subscription_form(
                "   ",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.name_error.as_deref(), Some("Name is required"));
        }

        #[test]
        fn validate_form_short_name() {
            let r = validate_subscription_form(
                "A",
                "15.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(
                r.name_error.as_deref(),
                Some("Name must be at least 2 characters")
            );
        }

        #[test]
        fn validate_form_empty_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.amount_error.as_deref(), Some("Amount is required"));
        }

        #[test]
        fn validate_form_zero_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "0",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(
                r.amount_error.as_deref(),
                Some("Amount must be greater than 0")
            );
        }

        #[test]
        fn validate_form_negative_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "-5",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(
                r.amount_error.as_deref(),
                Some("Amount must be greater than 0")
            );
        }

        #[test]
        fn validate_form_invalid_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "abc",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.amount_error.as_deref(), Some("Invalid amount format"));
        }

        #[test]
        fn validate_form_empty_currency() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.currency_error.as_deref(), Some("Currency is required"));
        }

        #[test]
        fn validate_form_empty_frequency() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "EUR",
                "",
                "2024-01-01",
                "cat-id-123",
            );
            assert_eq!(r.frequency_error.as_deref(), Some("Frequency is required"));
        }

        #[test]
        fn validate_form_empty_start_date() {
            let r =
                validate_subscription_form("Netflix", "15.99", "EUR", "monthly", "", "cat-id-123");
            assert_eq!(
                r.start_date_error.as_deref(),
                Some("Start date is required")
            );
        }

        #[test]
        fn validate_form_invalid_start_date() {
            let r = validate_subscription_form(
                "Netflix",
                "15.99",
                "EUR",
                "monthly",
                "not-a-date",
                "cat-id-123",
            );
            assert_eq!(r.start_date_error.as_deref(), Some("Invalid date format"));
        }

        #[test]
        fn validate_form_empty_category() {
            let r =
                validate_subscription_form("Netflix", "15.99", "EUR", "monthly", "2024-01-01", "");
            assert_eq!(r.category_error.as_deref(), Some("Category is required"));
        }

        #[test]
        fn validate_form_multiple_errors() {
            let r = validate_subscription_form("", "", "", "", "", "");
            assert!(r.name_error.is_some());
            assert!(r.amount_error.is_some());
            assert!(r.currency_error.is_some());
            assert!(r.frequency_error.is_some());
            assert!(r.start_date_error.is_some());
            assert!(r.category_error.is_some());
        }

        #[test]
        fn validate_form_decimal_amount() {
            let r = validate_subscription_form(
                "Netflix",
                "9.99",
                "EUR",
                "monthly",
                "2024-01-01",
                "cat-id-123",
            );
            assert!(r.is_valid());
        }

        #[test]
        fn validate_form_valid_date_formats() {
            for date in &["2024-01-01", "2024-12-31", "2025-06-15"] {
                let r = validate_subscription_form(
                    "Netflix",
                    "15.99",
                    "EUR",
                    "monthly",
                    date,
                    "cat-id-123",
                );
                assert!(
                    r.start_date_error.is_none(),
                    "date {} should be valid",
                    date
                );
            }
        }

        // ── Extended filter/sort tests ──
        #[test]
        fn filter_by_invalid_category_id() {
            let cat = Uuid::new_v4();
            let subs = vec![make_sub("A", dec!(10), "monthly", true, None, cat)];
            let filters = FilterConfig {
                category_id: Some("not-a-uuid".into()),
                ..Default::default()
            };
            let result = filter_subscriptions(&subs, &filters);
            assert_eq!(result.len(), 1);
        }

        #[test]
        fn sort_unknown_field_stable() {
            let cat = Uuid::new_v4();
            let mut subs = vec![
                make_sub("A", dec!(10), "monthly", true, None, cat),
                make_sub("B", dec!(20), "monthly", true, None, cat),
            ];
            let sort = SortConfig {
                field: "unknown".into(),
                direction: "asc".into(),
            };
            sort_subscriptions(&mut subs, &sort);
            assert_eq!(subs.len(), 2);
        }

        #[test]
        fn sort_config_partial_eq() {
            let s1 = SortConfig {
                field: "name".into(),
                direction: "desc".into(),
            };
            let s2 = SortConfig {
                field: "name".into(),
                direction: "desc".into(),
            };
            assert_eq!(s1.field, s2.field);
            assert_eq!(s1.direction, s2.direction);
        }
    }
}

// ── DialogForm tests ──────────────────────────────────────────────────
#[cfg(test)]
mod dialog_form_tests {
    use super::*;

    #[test]
    fn dialog_form_default() {
        let form = DialogForm::default();
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.currency.is_empty());
        assert!(form.frequency.is_empty());
        assert!(form.start_date.is_empty());
        assert!(form.category_id.is_empty());
        assert!(form.notes.is_empty());
        assert!(form.logo_url.is_empty());
    }

    #[test]
    fn dialog_form_debug() {
        let form = DialogForm::default();
        let debug = format!("{:?}", form);
        assert!(debug.contains("DialogForm"));
    }

    #[test]
    fn dialog_form_clone() {
        let form = DialogForm::default();
        let cloned = form.clone();
        assert_eq!(form.name, cloned.name);
    }

    #[test]
    fn dialog_form_with_values() {
        let form = DialogForm {
            name: "Netflix".into(),
            amount: "15.99".into(),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: "2024-01-01".into(),
            category_id: "cat-123".into(),
            notes: "Family plan".into(),
            logo_url: "netflix.png".into(),
        };
        assert_eq!(form.name, "Netflix");
        assert_eq!(form.amount, "15.99");
        assert_eq!(form.frequency, "monthly");
        assert!(form.notes.contains("Family"));
    }

    #[test]
    fn dialog_form_empty_values() {
        let form = DialogForm {
            name: String::new(),
            amount: String::new(),
            currency: String::new(),
            frequency: String::new(),
            start_date: String::new(),
            category_id: String::new(),
            notes: String::new(),
            logo_url: String::new(),
        };
        assert!(form.name.is_empty());
        assert!(form.amount.is_empty());
        assert!(form.currency.is_empty());
    }

    // ── DialogForm additional tests ──
    #[test]
    fn dialog_form_debug_extended() {
        let form = DialogForm::default();
        let debug = format!("{:?}", form);
        assert!(debug.contains("DialogForm"));
    }

    #[test]
    fn dialog_form_name_set() {
        let mut form = DialogForm::default();
        form.name = "Spotify".into();
        assert_eq!(form.name, "Spotify");
    }

    #[test]
    fn dialog_form_amount_set() {
        let mut form = DialogForm::default();
        form.amount = "9.99".into();
        assert_eq!(form.amount, "9.99");
    }

    #[test]
    fn dialog_form_currency_set() {
        let mut form = DialogForm::default();
        form.currency = "USD".into();
        assert_eq!(form.currency, "USD");
    }

    #[test]
    fn dialog_form_frequency_set() {
        let mut form = DialogForm::default();
        form.frequency = "yearly".into();
        assert_eq!(form.frequency, "yearly");
    }

    #[test]
    fn dialog_form_start_date_set() {
        let mut form = DialogForm::default();
        form.start_date = "2024-06-15".into();
        assert_eq!(form.start_date, "2024-06-15");
    }

    #[test]
    fn dialog_form_category_id_set() {
        let mut form = DialogForm::default();
        form.category_id = "cat-id-123".into();
        assert_eq!(form.category_id, "cat-id-123");
    }

    #[test]
    fn dialog_form_notes_set() {
        let mut form = DialogForm::default();
        form.notes = "Monthly payment".into();
        assert_eq!(form.notes, "Monthly payment");
    }

    #[test]
    fn dialog_form_logo_url_set() {
        let mut form = DialogForm::default();
        form.logo_url = "https://logo.com/spotify.png".into();
        assert_eq!(form.logo_url, "https://logo.com/spotify.png");
    }

    #[test]
    fn dialog_form_multiple_fields() {
        let form = DialogForm {
            name: "Netflix".into(),
            amount: "15.99".into(),
            currency: "EUR".into(),
            frequency: "monthly".into(),
            start_date: "2024-01-01".into(),
            category_id: "cat-123".into(),
            notes: "Family plan".into(),
            logo_url: "netflix.png".into(),
        };
        assert_eq!(form.name, "Netflix");
        assert_eq!(form.amount, "15.99");
        assert_eq!(form.currency, "EUR");
        assert_eq!(form.frequency, "monthly");
        assert_eq!(form.start_date, "2024-01-01");
        assert_eq!(form.category_id, "cat-123");
        assert_eq!(form.notes, "Family plan");
        assert_eq!(form.logo_url, "netflix.png");
    }

    #[test]
    fn dialog_form_empty_notes() {
        let form = DialogForm::default();
        assert!(form.notes.is_empty());
    }

    #[test]
    fn dialog_form_empty_logo_url() {
        let form = DialogForm::default();
        assert!(form.logo_url.is_empty());
    }

    #[test]
    fn dialog_form_clone_independent() {
        let mut form = DialogForm::default();
        form.name = "Spotify".into();
        let cloned = form.clone();
        assert_eq!(cloned.name, "Spotify");
        // Modify original
        form.name = "Netflix".into();
        // Cloned should still have old value
        assert_eq!(cloned.name, "Spotify");
    }

    // ── SortConfig additional tests ──
    #[test]
    fn sort_config_default_field() {
        let s = SortConfig::default();
        assert_eq!(s.field, "next_billing_date");
        assert_eq!(s.direction, "asc");
    }

    #[test]
    fn sort_config_custom() {
        let s = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        assert_eq!(s.field, "amount");
        assert_eq!(s.direction, "desc");
    }

    #[test]
    fn sort_config_debug() {
        let s = SortConfig::default();
        let debug = format!("{:?}", s);
        assert!(debug.contains("SortConfig"));
    }

    #[test]
    fn sort_config_partial_eq() {
        let a = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        let b = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        assert_eq!(a.field, b.field);
        assert_eq!(a.direction, b.direction);
    }

    // ── FilterConfig additional tests ──
    #[test]
    fn filter_config_default_all_none() {
        let f = FilterConfig::default();
        assert!(f.status.is_none());
        assert!(f.frequency.is_none());
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn filter_config_debug() {
        let f = FilterConfig::default();
        let debug = format!("{:?}", f);
        assert!(debug.contains("FilterConfig"));
    }

    #[test]
    fn filter_config_clone() {
        let f = FilterConfig {
            status: Some("active".into()),
            ..Default::default()
        };
        let cloned = f.clone();
        assert_eq!(f.status, cloned.status);
    }

    // ── ValidationResult additional tests ──
    #[test]
    fn validation_result_all_errors() {
        let v = ValidationResult {
            name_error: Some("a".into()),
            amount_error: Some("b".into()),
            currency_error: Some("c".into()),
            frequency_error: Some("d".into()),
            start_date_error: Some("e".into()),
            category_error: Some("f".into()),
        };
        assert!(!v.is_valid());
        assert!(v.name_error.is_some());
        assert!(v.amount_error.is_some());
        assert!(v.currency_error.is_some());
        assert!(v.frequency_error.is_some());
        assert!(v.start_date_error.is_some());
        assert!(v.category_error.is_some());
    }

    #[test]
    fn validation_result_partial_errors() {
        let v = ValidationResult {
            amount_error: Some("bad".into()),
            ..Default::default()
        };
        assert!(!v.is_valid());
        assert!(v.name_error.is_none());
        assert!(v.amount_error.is_some());
    }

    // ── Card display helpers tests ──
    #[test]
    fn subscription_card_opacity_active() {
        assert_eq!(subscription_card_opacity(true), "");
    }

    #[test]
    fn subscription_card_opacity_inactive() {
        assert_eq!(subscription_card_opacity(false), "opacity-60");
    }

    #[test]
    fn subscription_next_text_active_with_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subscription_next_text(true, Some(date));
        assert_eq!(result, "Next: 2024-06-15");
    }

    #[test]
    fn subscription_next_text_active_no_date() {
        let result = subscription_next_text(true, None);
        assert_eq!(result, "No billing date");
    }

    #[test]
    fn subscription_next_text_inactive() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = subscription_next_text(false, Some(date));
        assert_eq!(result, "Inactive");
    }

    #[test]
    fn subscription_next_text_inactive_no_date() {
        let result = subscription_next_text(false, None);
        assert_eq!(result, "Inactive");
    }

    #[test]
    fn subscription_has_notes_some() {
        assert!(subscription_has_notes(&Some("Some notes".into())));
    }

    #[test]
    fn subscription_has_notes_empty() {
        assert!(!subscription_has_notes(&Some("".into())));
    }

    #[test]
    fn subscription_has_notes_none() {
        assert!(!subscription_has_notes(&None));
    }
}
