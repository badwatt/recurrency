use dioxus::prelude::*;
use lucide_dioxus::{CreditCard, FileText, Pause, Pencil, Play, Trash2, X, ArrowUpDown};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::api;
use crate::storage::Storage;
use crate::components::dialog::Dialog;
use crate::components::logo_image_with_fallback;
use crate::components::page_header::PageHeader;
use crate::components::skeleton::SubscriptionCardSkeleton;
use crate::components::tooltip::Tooltip;
use crate::components::logo_search::LogoSearch;
use crate::components::subscription_form::SubscriptionForm;
#[allow(unused_imports)]
use crate::components::entry_form::{ValidationResult, validate_subscription_form};
use crate::models::subscription::{
    CreateSubscriptionRequest, Subscription, UpdateSubscriptionRequest,
};
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::preferences::PREFERENCES;
use crate::state::subscriptions::SUBSCRIPTIONS;
use crate::utils::constants::{CURRENCIES, FREQUENCIES};
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;

mod logic;
pub use logic::*;

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
        let _ = prefs_state;
        spawn(async move {
            let state = PREFERENCES();
            if !state.initialized
                && let Ok(p) = api::preferences::get().await {
                    // Preserve theme from localStorage
                    let saved_theme = PREFERENCES().theme().to_string();
                    PREFERENCES.write().set_data(p);
                    PREFERENCES.write().set_theme(saved_theme);
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
                    {err.to_string()}                }
            }
        };
    }

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6",

            // ── Header ──
            div { class: "flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3 animate-in fade-in slide-in-from-bottom-4 duration-700",
                PageHeader { title: "Subscriptions", subtitle: "Track and manage recurring payments" }
                div { class: "flex items-center gap-2 shrink-0",
                    button {
                        class: "hidden sm:flex items-center gap-1.5 border border-border/60 rounded-lg px-3 py-2 hover:bg-muted text-sm transition-colors duration-200",
                        onclick: {
                            let vm = view_mode.clone();
                            move |_| set_view_mode(if vm.as_str() == "grid" { "list" } else { "grid" })
                        },
                        if view_mode.as_str() == "grid" { "≡ List" } else { "⊞ Grid" }
                    }
                    button {
                        class: "bg-primary text-primary-foreground font-semibold px-4 py-2.5 rounded-lg text-sm hover:bg-primary/90 active:scale-[0.97] transition-all duration-200",
                        onclick: move |_| handle_create.call(()),
                        "+ Add"
                    }
                }
            }

            // ── Content ──
            {if state.is_loading && state.subscriptions.is_empty() {
                rsx! {
                    div { class: "grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                        for _ in 0..8 {
                            SubscriptionCardSkeleton {}
                        }
                    }
                }
            } else if state.subscriptions.is_empty() {
                rsx! {
                    div { class: "rounded-xl border border-border/40 bg-card/80 p-6 flex h-64 flex-col items-center justify-center gap-4",
                        p { class: "text-lg font-medium", "No subscriptions yet" }
                        p { class: "text-sm text-muted-foreground", "Click the button above to add your first subscription" }
                        button {
                            class: "bg-primary text-primary-foreground font-semibold px-4 py-2 rounded-lg text-sm hover:bg-primary/90 active:scale-[0.97] transition-all duration-200",
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

                        // ── Grid view ──
                        {if view_mode == "grid" {
                            rsx! {
                                div { class: "grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                                    for (i, sub) in filtered.iter().enumerate() {
                                        {render_grid_card(sub, i, handle_edit, handle_delete, handle_toggle_active, handle_notes_click)}
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "space-y-3",
                                    for (_i, sub) in filtered.iter().enumerate() {
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
                Dialog {
                    open: true,
                    on_close: move |_| delete_confirm_id.set(None),
                    title: "Delete subscription?".to_string(),
                    footer: rsx! {
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
                                        let _ = Storage::delete_subscription(&id).await;
                                        match Storage::list_subscriptions().await {
                                            Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                            Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                        }
                                    });
                                    delete_confirm_id.set(None);
                                },
                                "Delete"
                            }
                        }
                    },
                    p { class: "text-sm text-muted-foreground", "This action cannot be undone." }
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
                            footer: rsx! {
                                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                                    button {
                                        class: "border border-border/60 px-4 py-2 rounded-lg text-sm hover:bg-muted transition-colors duration-200",
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
                                                let _ = Storage::update_subscription(&id, &req).await;
                                                match Storage::list_subscriptions().await {
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
                            },
                            textarea {
                                class: "w-full min-h-[120px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                                placeholder: "Add notes...",
                                value: notes_dialog_value.to_string(),
                                oninput: move |e: Event<FormData>| notes_dialog_value.set(e.value()),
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Grid card ────────────────────────────────────────────────────────

fn render_grid_card(
    sub: &Subscription,
    index: usize,
    on_edit: Callback<Subscription>,
    on_delete: Callback<Uuid>,
    on_toggle: Callback<Subscription>,
    on_notes: Callback<Subscription>,
) -> Element {
    let opacity = subscription_card_opacity(sub.is_active);
    let formatted = format_currency(sub.amount, &sub.currency);
    let next_text = subscription_next_text(sub.is_active, sub.next_billing_date, &PREFERENCES().date_format());

    let (color, cat_icon, cat_name) = {
        let cats = CATEGORIES();
        if let Some(c) = cats.get_by_id(&sub.category_id) {
            (
                c.color.clone().unwrap_or_else(|| "#6b7280".to_string()),
                c.icon.clone().unwrap_or_else(|| "Tag".to_string()),
                c.name.clone(),
            )
        } else {
            ("#6b7280".to_string(), "Tag".to_string(), "Uncategorized".to_string())
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

    let stagger_delay = index.saturating_mul(60);

    rsx! {
        div {
            class: "group relative overflow-hidden rounded-xl border border-border/40 bg-card/80 hover:bg-card hover:scale-[1.02] transition-all duration-300 {opacity}",
            style: "animation: fadeSlideIn 0.5s ease-out both; animation-delay: {stagger_delay}ms;",

            // Category color top accent strip
            div {
                class: "absolute top-0 left-0 right-0 h-[3px]",
                style: "background: {color};",
            }

            // Logo watermark background
            if let Some(ref url) = logo_url {
                div { class: "absolute inset-0 opacity-[0.06] flex items-center justify-center pointer-events-none select-none",
                    img {
                        src: url.to_string(),
                        alt: "",
                        class: "w-full h-full object-contain scale-[2.5]",
                    }
                }
            }

            // Content
            div { class: "relative z-10 flex flex-col p-5 pb-4 gap-3",

                // Top row: logo + status badge
                div { class: "flex items-start justify-between gap-2",
                    div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-muted overflow-hidden",
                        {logo_image_with_fallback(&format!("card-{}", sub.id), logo_url.as_deref(), &name, "h-full w-full object-cover")}
                        if logo_url.is_none() {
                            span { class: "text-muted-foreground", CreditCard { size: 22 } }
                        }
                    }

                    // Status badge — square rounded-md
                    span {
                        class: if is_active { "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" } else { "inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-medium bg-muted text-muted-foreground border border-border/40" },
                        if is_active { "Active" } else { "Inactive" }
                    }
                }

                // Name + next billing
                div { class: "min-w-0",
                    p { class: "font-semibold truncate text-sm leading-tight", {name.to_string()} }
                    p { class: "text-xs text-muted-foreground mt-1", {next_text.to_string()} }
                }

                // Amount
                p { class: "font-extrabold text-2xl tracking-tight", {formatted.to_string()} }

                // Category badge
                span {
                    class: "inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium w-fit",
                    style: "color: {color}; background-color: {color}15; border: 1px solid {color}25;",
                    {icon_display(&cat_icon)}
                    {cat_name.to_string()}                }

                // Action buttons
                div { class: "flex items-center gap-0.5 mt-auto pt-1",
                    Tooltip { content: sub.notes.clone().unwrap_or_default(),
                        button {
                            class: if has_notes { "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground transition-colors duration-200" } else { "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground opacity-40 transition-colors duration-200" },
                            onclick: move |_| on_notes.call(sub_for_notes.clone()),
                            FileText { size: 16 }
                        }
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground transition-colors duration-200",
                        onclick: move |_| on_toggle.call(sub_for_toggle.clone()),
                        if is_active { Pause { size: 14 } } else { Play { size: 14 } }
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-muted text-muted-foreground transition-colors duration-200",
                        onclick: move |_| on_edit.call(sub_for_edit.clone()),
                        Pencil { size: 14 }
                    }
                    button {
                        class: "h-8 w-8 flex items-center justify-center rounded-lg hover:bg-red-500/10 text-red-500 transition-colors duration-200",
                        onclick: move |_| on_delete.call(id_for_delete),
                        Trash2 { size: 14 }
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
    let next_text = subscription_next_text(sub.is_active, sub.next_billing_date, &PREFERENCES().date_format());

    let (color, cat_icon, cat_name) = {
        let cats = CATEGORIES();
        if let Some(c) = cats.get_by_id(&sub.category_id) {
            (
                c.color.clone().unwrap_or_else(|| "#6b7280".into()),
                c.icon.clone().unwrap_or_else(|| "Tag".into()),
                c.name.clone(),
            )
        } else {
            ("#6b7280".into(), "Tag".into(), "Uncategorized".into())
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
                            span { class: "text-muted-foreground", CreditCard { size: 20 } }
                        }
                    }

                    div { class: "flex-1 min-w-0",
                        p { class: "font-medium truncate text-sm sm:text-base", {name.to_string()} }
                        p { class: "text-xs text-muted-foreground truncate", {next_text.to_string()} }
                    }

                    div { class: "flex items-center gap-2 sm:gap-4 shrink-0",
                        span {
                            class: "hidden sm:inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-normal",
                            style: "color: {color}; background-color: {color}15;",
                            {icon_display(&cat_icon)}
                            {cat_name.to_string()}                        }
                        p { class: "font-medium text-sm sm:text-base", {formatted.to_string()} }

                        div { class: "flex items-center gap-0.5 sm:gap-1 shrink-0",
                            Tooltip { content: sub.notes.clone().unwrap_or_default(),
                                button {
                                    class: if has_notes { "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground" } else { "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-muted-foreground opacity-40" },
                                    onclick: move |_| on_notes.call(sub_for_notes.clone()),
                                    FileText { size: 16 }
                                }
                            }
                            button {
                                class: "h-7 w-7 sm:h-8 sm:w-8 flex items-center justify-center rounded-md hover:bg-muted",
                                onclick: move |_| on_toggle.call(sub_for_toggle.clone()),
                                if is_active { Pause { size: 14 } } else { Play { size: 14 } }
                            }
                            button {
                                class: "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted",
                                onclick: move |_| on_edit.call(sub_for_edit.clone()),
                                Pencil { size: 14 }
                            }
                            button {
                                class: "h-8 w-8 sm:h-9 sm:w-9 flex items-center justify-center rounded-md hover:bg-muted text-red-600",
                                onclick: move |_| on_delete.call(id_for_delete),
                                Trash2 { size: 14 }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Dialog ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
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
    let form_logo_url = form().logo_url;

    let name_err = name_error();
    let amount_err = amount_error();
    let currency_err = currency_error();
    let frequency_err = frequency_error();
    let start_date_err = start_date_error();
    let category_err = category_error();

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| dialog_open.set(false),
            title: title,
            footer: rsx! {
                div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
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
                                    let _ = Storage::update_subscription(&id, &req).await;
                                    match Storage::list_subscriptions().await {
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
                                    let _ = Storage::create_subscription(&req).await;
                                    match Storage::list_subscriptions().await {
                                        Ok(data) => SUBSCRIPTIONS.write().set_data(data),
                                        Err(e) => SUBSCRIPTIONS.write().set_error(e.to_string()),
                                    }
                                });
                            }

                            dialog_open.set(false);
                        },
                        {submit_label.to_string()}                    }
                }
            },
            p { class: "text-sm text-muted-foreground", {desc.to_string()} }
            div { class: "space-y-4",
                // Name
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Name" }
                    input {
                        class: if name_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                        placeholder: "Netflix, Spotify, etc.",
                        value: form_name.to_string(),
                        oninput: move |e: Event<FormData>| {
                            form.write().name = e.value();
                            name_error.set(String::new());
                        },
                    }
                    if !name_err.is_empty() {
                        p { class: "text-xs text-red-500", {name_err.to_string()} }
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
                            value: form_amount.to_string(),
                            oninput: move |e: Event<FormData>| {
                                form.write().amount = e.value();
                                amount_error.set(String::new());
                            },
                        }
                        if !amount_err.is_empty() {
                            p { class: "text-xs text-red-500", {amount_err.to_string()} }
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Currency" }
                        select {
                            class: if currency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                            value: form_currency.to_string(),
                            onchange: move |e: Event<FormData>| {
                                form.write().currency = e.value();
                                currency_error.set(String::new());
                            },
                            for (val, label) in CURRENCIES {
                                option { value: val.to_string(), {label.to_string()} }
                            }
                        }
                        if !currency_err.is_empty() {
                            p { class: "text-xs text-red-500", {currency_err.to_string()} }
                        }
                    }
                }

                // Frequency + Start date
                div { class: "grid grid-cols-2 gap-4",
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Frequency" }
                        select {
                            class: if frequency_err.is_empty() { "w-full h-9 rounded-md border bg-background px-2 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-2 text-sm" },
                            value: form_frequency.to_string(),
                            onchange: move |e: Event<FormData>| {
                                form.write().frequency = e.value();
                                frequency_error.set(String::new());
                            },
                            for (val, label) in FREQUENCIES {
                                option { value: val.to_string(), selected: form_frequency == *val, {label.to_string()} }
                            }
                        }
                        if !frequency_err.is_empty() {
                            p { class: "text-xs text-red-500", {frequency_err.to_string()} }
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Start Date" }
                        input {
                            r#type: "date",
                            class: if start_date_err.is_empty() { "w-full h-9 rounded-md border bg-background px-3 text-sm" } else { "w-full h-9 rounded-md border border-red-500 bg-background px-3 text-sm" },
                            value: form_start_date.to_string(),
                            oninput: move |e: Event<FormData>| {
                                form.write().start_date = e.value();
                                start_date_error.set(String::new());
                            },
                        }
                        if !start_date_err.is_empty() {
                            p { class: "text-xs text-red-500", {start_date_err.to_string()} }
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

                // Logo
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Logo (optional)" }
                    LogoSearch {
                        selected_url: form_logo_url.clone(),
                        default_query: form_name.clone(),
                        on_select: move |url: String| form.write().logo_url = url,
                    }
                }

                // Notes
                div { class: "space-y-1.5",
                    label { class: "text-sm font-medium", "Notes (optional)" }
                    textarea {
                        class: "w-full min-h-[80px] rounded-md border bg-background px-3 py-2 text-sm resize-none",
                        placeholder: "Additional notes...",
                        value: form_notes.to_string(),
                        oninput: move |e: Event<FormData>| form.write().notes = e.value(),
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
