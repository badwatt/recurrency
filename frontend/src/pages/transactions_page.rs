use crate::storage::Storage;
use crate::components::dialog::Dialog;
use crate::components::page_header::PageHeader;
use crate::components::skeleton::TransactionRowSkeleton;
use crate::components::tooltip::Tooltip;
use crate::components::logo_image_with_fallback;
use crate::components::logo_search::LogoSearch;
use crate::models::transaction::*;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::preferences::PREFERENCES;
use crate::state::transactions::TRANSACTIONS;
use crate::utils::dates::format_date;
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;
use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;
use lucide_dioxus::{FileText, Pencil, Plus, Receipt, Trash2};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilterConfig {
    pub category_id: Option<Uuid>,
    pub price_min: Option<Decimal>,
    pub price_max: Option<Decimal>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SortConfig {
    pub field: String,
    pub direction: String, // "asc" | "desc"
}

/// A month group with label and total.
#[derive(Debug, Clone)]
pub struct MonthGroup {
    pub key: String,
    pub label: String,
    pub transactions: Vec<Transaction>,
    pub total: Decimal,
}

/// Filter + sort transactions and group by month (descending).
// TransactionValidationResult and validate_transaction_form moved to components/entry_form.rs
#[allow(unused_imports)]
use crate::components::entry_form::{TransactionValidationResult, validate_transaction_form};

pub fn filter_sort_group(
    txns: &[Transaction],
    filters: &FilterConfig,
    sort: &SortConfig,
) -> Vec<MonthGroup> {
    let mut result: Vec<Transaction> = txns.to_vec();

    // Filter
    if let Some(cid) = filters.category_id {
        result.retain(|t| t.category_id == cid);
    }
    if let Some(min) = filters.price_min {
        result.retain(|t| t.amount >= min);
    }
    if let Some(max) = filters.price_max {
        result.retain(|t| t.amount <= max);
    }

    // Sort
    result.sort_by(|a, b| {
        let cmp = match sort.field.as_str() {
            "date" => a.date.cmp(&b.date),
            "amount" => a.amount.cmp(&b.amount),
            "name" => a.name.cmp(&b.name),
            _ => a.date.cmp(&b.date),
        };
        if sort.direction == "asc" {
            cmp
        } else {
            cmp.reverse()
        }
    });

    // Group by month descending
    let mut groups: std::collections::BTreeMap<String, Vec<Transaction>> =
        std::collections::BTreeMap::new();
    for t in result {
        let key = format!("{}-{:02}", t.date.year(), t.date.month());
        groups.entry(key).or_default().push(t);
    }

    let mut out: Vec<_> = groups.into_iter().collect();
    out.sort_by(|a, b| b.0.cmp(&a.0));

    out.into_iter()
        .map(|(key, txns)| {
            let (year, month) = key.split_once('-').unwrap();
            let y: i32 = year.parse().unwrap();
            let m: u32 = month.parse().unwrap();
            let label = format_month(y, m);
            let total: Decimal = txns.iter().map(|t| t.amount).sum();
            MonthGroup {
                key,
                label,
                transactions: txns,
                total,
            }
        })
        .collect()
}

fn format_month(year: i32, month: u32) -> String {
    let names = [
        "",
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    format!("{} {}", names.get(month as usize).unwrap_or(&""), year)
}

#[component]
pub fn TransactionsPage() -> Element {
    let mut filters = use_signal(FilterConfig::default);
    let mut sort_cfg = use_signal(|| SortConfig {
        field: "date".into(),
        direction: "desc".into(),
    });
    let mut dialog_open = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut filters_popover = use_signal(|| false);
    let _show_filters = use_signal(|| false);

    // Local form state for create/edit dialog
    let mut form_name = use_signal(String::new);
    let mut form_amount = use_signal(String::new);
    let mut form_currency = use_signal(|| "EUR".to_string());
    let mut form_date = use_signal(String::new);
    let mut form_category_id = use_signal(|| None::<Uuid>);
    let mut form_notes = use_signal(String::new);
    let mut form_logo_url = use_signal(String::new);
    let mut form_saving = use_signal(|| false);

    // Notes dialog state
    let mut notes_dialog_open = use_signal(|| false);
    let mut notes_dialog_txn_id = use_signal(|| None::<Uuid>);
    let mut notes_dialog_value = use_signal(String::new);
    let mut notes_saving = use_signal(|| false);

    // Validation error signals
    let mut name_error = use_signal(String::new);
    let mut amount_error = use_signal(String::new);
    let mut currency_error = use_signal(String::new);
    let mut date_error = use_signal(String::new);
    let mut category_error = use_signal(String::new);

    // Load data
    use_future(move || async move {
        // android guard removed
        if TRANSACTIONS().is_loading {
            return;
        }
        TRANSACTIONS.write().set_loading();
        match Storage::list_transactions().await {
            Ok(data) => TRANSACTIONS.write().set_data(data),
            Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
        }
    });

    use_future(move || async move {
        // android guard removed
        if !CATEGORIES().categories.is_empty() {
            return;
        }
        CATEGORIES.write().set_loading();
        match Storage::list_categories().await {
            Ok(data) => CATEGORIES.write().set_data(data),
            Err(e) => CATEGORIES.write().set_error(e.to_string()),
        }
    });

    let currency = AUTH().currency().to_string();
    let state = TRANSACTIONS();
    let cats = CATEGORIES();

    let groups = filter_sort_group(&state.transactions, &filters(), &sort_cfg());

    let mut open_edit = move |txn: &Transaction| {
        editing_id.set(Some(txn.id));
        form_name.set(txn.name.clone());
        form_amount.set(txn.amount.to_string());
        form_currency.set(txn.currency.clone());
        form_date.set(txn.date.to_string());
        form_category_id.set(Some(txn.category_id));
        form_notes.set(txn.notes.clone().unwrap_or_default());
        form_logo_url.set(txn.logo_url.clone().unwrap_or_default());
        name_error.set(String::new());
        amount_error.set(String::new());
        currency_error.set(String::new());
        date_error.set(String::new());
        category_error.set(String::new());
        dialog_open.set(true);
    };

    let handle_save = move |_| {
        // Validate
        let mut valid = true;
        let name_val = form_name();
        let amount_str = form_amount();
        let currency_val = form_currency();
        let date_str = form_date();
        let cat_id_val = form_category_id();

        if name_val.trim().is_empty() {
            name_error.set("Name is required".into());
            valid = false;
        } else if name_val.trim().len() < 2 {
            name_error.set("Name must be at least 2 characters".into());
            valid = false;
        }

        if amount_str.is_empty() {
            amount_error.set("Amount is required".into());
            valid = false;
        } else {
            match Decimal::from_str(&amount_str) {
                Ok(d) if d > Decimal::ZERO => {}
                Ok(_) => {
                    amount_error.set("Amount must be greater than 0".into());
                    valid = false;
                }
                Err(_) => {
                    amount_error.set("Invalid amount".into());
                    valid = false;
                }
            }
        }

        if currency_val.is_empty() {
            currency_error.set("Currency is required".into());
            valid = false;
        }

        if date_str.is_empty() {
            date_error.set("Date is required".into());
            valid = false;
        } else if NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").is_err() {
            date_error.set("Invalid date format".into());
            valid = false;
        }

        if cat_id_val.is_none() {
            category_error.set("Category is required".into());
            valid = false;
        }

        if !valid {
            return;
        }

        let amount = match Decimal::from_str(&amount_str) {
            Ok(d) => d,
            Err(_) => return,
        };
        let date = match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return,
        };
        let editing = editing_id();
        let cat_id = cat_id_val;

        form_saving.set(true);
        spawn(async move {
            let result = if let Some(id) = editing {
                Storage::update_transaction(
                    &id,
                    &UpdateTransactionRequest {
                        name: Some(name_val),
                        amount: Some(amount),
                        currency: Some(form_currency()),
                        date: Some(date),
                        category_id: cat_id,
                        notes: if form_notes().is_empty() {
                            None
                        } else {
                            Some(form_notes())
                        },
                        logo_url: if form_logo_url().is_empty() {
                            None
                        } else {
                            Some(form_logo_url())
                        },
                    },
                )
                .await
            } else {
                Storage::create_transaction(&CreateTransactionRequest {
                    name: name_val,
                    amount,
                    currency: form_currency(),
                    date,
                    category_id: cat_id,
                    notes: if form_notes().is_empty() {
                        None
                    } else {
                        Some(form_notes())
                    },
                    logo_url: if form_logo_url().is_empty() {
                        None
                    } else {
                        Some(form_logo_url())
                    },
                })
                .await
            };

            match result {
                Ok(_) => {
                    dialog_open.set(false);
                    // Reload
                    match Storage::list_transactions().await {
                        Ok(data) => TRANSACTIONS.write().set_data(data),
                        Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
                    }
                }
                Err(e) => log::error!("Save failed: {}", e),
            }
            form_saving.set(false);
        });
    };

    let handle_delete = move |id: Uuid| {
        spawn(async move {
            let _ = Storage::delete_transaction(&id).await;
            match Storage::list_transactions().await {
                Ok(data) => TRANSACTIONS.write().set_data(data),
                Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
            }
        });
    };

    if let Some(err) = &state.error {
        return rsx! {
            div { class: "p-4 md:p-6",
                div { class: "rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive",
                    {err.clone()}
                }
            }
        };
    }

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6",
            // Header
            div { class: "flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3 animate-in fade-in slide-in-from-bottom-4 duration-700",
                PageHeader { title: "Transactions", subtitle: "Track your one-time expenses" }
                button {
                    onclick: move |_| {
                        editing_id.set(None);
                        form_name.set(String::new());
                        form_amount.set(String::new());
                        form_currency.set(AUTH().currency().to_string());
                        form_date.set(chrono::Local::now().format("%Y-%m-%d").to_string());
                        form_category_id.set(CATEGORIES().default_category().map(|c| c.id));
                        form_notes.set(String::new());
                        form_logo_url.set(String::new());
                        name_error.set(String::new());
                        amount_error.set(String::new());
                        currency_error.set(String::new());
                        date_error.set(String::new());
                        category_error.set(String::new());
                        dialog_open.set(true);
                    },
                    class: "inline-flex items-center justify-center gap-2 h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 w-full sm:w-auto",
                    Plus { size: 16 }
                    "Add Transaction"
                }
            }

            // Filter / Sort bar
            if !state.transactions.is_empty() {
                div { class: "flex flex-col sm:flex-row items-start sm:items-center gap-2",
                    // Filters button + popover
                    div { class: "relative",
                        button {
                            class: "border px-3 py-1.5 rounded-md text-sm flex items-center gap-1.5 hover:bg-muted h-8",
                            onclick: move |_| filters_popover.set(!filters_popover()),
                            "▾ Filters"
                            if filters().category_id.is_some() || filters().price_min.is_some() || filters().price_max.is_some() {
                                span { class: "ml-1 bg-secondary text-secondary-foreground rounded-full px-1.5 text-[10px]", "1" }
                            }
                        }
                        if filters_popover() {
                            div { class: "absolute left-0 top-full mt-2 z-50 w-72 rounded-md border bg-popover p-3 shadow-md space-y-3",
                                onclick: move |e| e.stop_propagation(),
                                div { class: "flex items-center justify-between",
                                    label { class: "text-sm font-medium", "Filters" }
                                    if filters().category_id.is_some() || filters().price_min.is_some() || filters().price_max.is_some() {
                                        button {
                                            class: "text-xs text-muted-foreground hover:text-foreground px-2 py-0.5",
                                            onclick: move |_| filters.set(FilterConfig::default()),
                                            "Clear"
                                        }
                                    }
                                }
                                // Category filter
                                div { class: "space-y-1.5",
                                    label { class: "text-xs font-medium text-muted-foreground", "Category" }
                                    select {
                                        class: "w-full h-8 rounded-md border bg-background px-2 text-sm",
                                        value: filters().category_id.map(|id| id.to_string()).unwrap_or_default(),
                                        onchange: move |e| {
                                            let val = e.value();
                                            let id = if val.is_empty() { None } else { Uuid::parse_str(&val).ok() };
                                            let mut f = filters();
                                            f.category_id = id;
                                            filters.set(f);
                                        },
                                        option { value: "", "All categories" }
                                        for cat in &cats.categories {
                                            option { value: cat.id.to_string(), {cat.name.to_string()} }
                                        }
                                    }
                                }
                                // Min price
                                div { class: "space-y-1.5",
                                    label { class: "text-xs font-medium text-muted-foreground", "Min price" }
                                    input {
                                        r#type: "number",
                                        step: "0.01",
                                        placeholder: "0.00",
                                        value: filters().price_min.map(|d| d.to_string()).unwrap_or_default(),
                                        oninput: move |e| {
                                            let mut f = filters();
                                            f.price_min = Decimal::from_str(&e.value()).ok();
                                            filters.set(f);
                                        },
                                        class: "w-full h-8 rounded-md border bg-background px-3 py-1 text-sm",
                                    }
                                }
                                // Max price
                                div { class: "space-y-1.5",
                                    label { class: "text-xs font-medium text-muted-foreground", "Max price" }
                                    input {
                                        r#type: "number",
                                        step: "0.01",
                                        placeholder: "999.99",
                                        value: filters().price_max.map(|d| d.to_string()).unwrap_or_default(),
                                        oninput: move |e| {
                                            let mut f = filters();
                                            f.price_max = Decimal::from_str(&e.value()).ok();
                                            filters.set(f);
                                        },
                                        class: "w-full h-8 rounded-md border bg-background px-3 py-1 text-sm",
                                    }
                                }
                            }
                        }
                    }

                    // Sort section
                    div { class: "flex items-center gap-1.5 w-full sm:w-auto sm:ml-auto",
                        select {
                            class: "h-8 w-full sm:w-[120px] rounded-md border bg-background px-2 text-sm",
                            value: "{sort_cfg().field}",
                            onchange: move |e| {
                                let mut s = sort_cfg();
                                s.field = e.value();
                                sort_cfg.set(s);
                            },
                            option { value: "date", "Date" }
                            option { value: "amount", "Price" }
                            option { value: "name", "Name" }
                        }
                        button {
                            class: "h-8 w-8 flex items-center justify-center rounded-md border bg-background hover:bg-muted",
                            onclick: move |_| {
                                let mut s = sort_cfg();
                                s.direction = if s.direction == "desc" { "asc".to_string() } else { "desc".to_string() };
                                sort_cfg.set(s);
                            },
                            if sort_cfg().direction == "desc" { "↓" } else { "↑" }
                        }
                    }
                }
            }

            // Content
            {if state.is_loading {
                rsx! {
                    div { class: "space-y-3",
                        for _ in 0..5 {
                            TransactionRowSkeleton {}
                        }
                    }
                }
            } else if state.transactions.is_empty() {
                rsx! {
                    div { class: "rounded-lg border border-border bg-card p-8 flex flex-col items-center justify-center gap-4",
                        span { class: "text-muted-foreground",
                            Receipt { size: 40 }
                        }
                        div { class: "text-center",
                            p { class: "text-lg font-medium", "No transactions yet" }
                            p { class: "text-sm text-muted-foreground", "Add your first one-time expense" }
                        }
                        button {
                            onclick: move |_| {
                                editing_id.set(None);
                                form_name.set(String::new());
                                form_amount.set(String::new());
                                form_currency.set(AUTH().currency().to_string());
                                form_date.set(chrono::Local::now().format("%Y-%m-%d").to_string());
                                form_category_id.set(CATEGORIES().default_category().map(|c| c.id));
                                form_notes.set(String::new());
                                name_error.set(String::new());
                                amount_error.set(String::new());
                                currency_error.set(String::new());
                                date_error.set(String::new());
                                category_error.set(String::new());
                                dialog_open.set(true);
                            },
                            class: "inline-flex items-center gap-2 h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90",
                            Plus { size: 16 }
                            "Add Transaction"
                        }
                    }
                }
            } else if groups.is_empty() {
                rsx! {
                    div { class: "rounded-lg border border-border bg-card p-8 flex flex-col items-center justify-center gap-2",
                        p { class: "text-sm text-muted-foreground", "No transactions match your filters" }
                        button {
                            onclick: move |_| filters.set(FilterConfig::default()),
                            class: "text-sm text-primary hover:underline",
                            "Clear filters"
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "space-y-4",
                        for group in &groups {
                            {rsx! {
                                div { class: "space-y-2",
                                    // Month header
                                    div { class: "flex items-center justify-between rounded-lg bg-muted/50 px-4 py-2",
                                        span { class: "text-sm font-semibold text-muted-foreground",
                                            "{group.label} · {group.transactions.len()} transactions"
                                        }
                                        span { class: "text-sm font-bold",
                                            {format_currency(group.total, &currency)}
                                        }
                                    }
                                    // Transaction list
                                    for txn in &group.transactions {
                                        {rsx! {
                                            div { key: "{txn.id}", class: "rounded-lg border border-border bg-card p-3 sm:p-4 hover:bg-muted/50 transition-colors",
                                                div { class: "flex items-center gap-2 sm:gap-4",
                                                    // Icon
                                                    div { class: "flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden",
                                                        {logo_image_with_fallback(&format!("txn-{}", txn.id), txn.logo_url.as_deref(), &txn.name, "h-full w-full object-contain p-1")}
                                                        if txn.logo_url.is_none() {
                                                            span { class: "text-muted-foreground",
                                                                Receipt { size: 16 }
                                                            }
                                                        }
                                                    }
                                                    // Name + date
                                                    div { class: "flex-1 min-w-0",
                                                        p { class: "font-medium truncate text-sm sm:text-base", {txn.name.to_string()} }
                                                        p { class: "text-xs text-muted-foreground",
                                                            {format_date(&txn.date, &PREFERENCES().date_format())}
                                                        }
                                                    }
                                                    // Category badge
                                                    {
                                                        let cat = cats.get_by_id(&txn.category_id);
                                                        let cat_name = cat.map(|c| c.name.as_str()).unwrap_or("Uncategorized");
                                                        let cat_icon = cat.and_then(|c| c.icon.clone()).unwrap_or_else(|| "Tag".into());
                                                        let cat_color = cat.and_then(|c| c.color.clone()).unwrap_or_else(|| "#6b7280".into());
                                                        let bg = format!("{}20", cat_color);
                                                        rsx! {
                                                            span { class: "font-normal text-xs hidden sm:inline-flex items-center gap-1 px-2 py-0.5 rounded",
                                                                style: "color: {cat_color}; background-color: {bg}",
                                                                {icon_display(&cat_icon)}
                                                                {cat_name.to_string()}                                                            }
                                                        }
                                                    }
                                                    // Amount
                                                    p { class: "font-medium text-sm sm:text-base shrink-0",
                                                        {format_currency(txn.amount, &txn.currency)}
                                                    }
                                                    // Actions
                                                    div { class: "flex items-center gap-0.5 sm:gap-1 shrink-0",
                                                        Tooltip { content: txn.notes.clone().unwrap_or_default(),
                                                            button {
                                                                class: if txn.notes.as_ref().is_some_and(|n| !n.is_empty()) { "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-muted-foreground" } else { "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-muted-foreground opacity-40" },
                                                                onclick: {
                                                                    let txn = txn.clone();
                                                                    move |_| {
                                                                        notes_dialog_txn_id.set(Some(txn.id));
                                                                        notes_dialog_value.set(txn.notes.clone().unwrap_or_default());
                                                                        notes_dialog_open.set(true);
                                                                    }
                                                                },
                                                                FileText { size: 16 }
                                                            }
                                                        }
                                                        button {
                                                            class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent",
                                                            onclick: {
                                                                let txn = txn.clone();
                                                                move |_| open_edit(&txn)
                                                            },
                                                            Pencil { size: 16 }
                                                        }
                                                        button {
                                                            class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-destructive",
                                                            onclick: {
                                                                let id = txn.id;
                                                                move |_| handle_delete(id)
                                                            },
                                                            Trash2 { size: 16 }
                                                        }
                                                    }
                                                }
                                            }
                                        }}
                                    }
                                }
                            }}
                        }
                    }
                }
            }}

            // Dialog overlay
            if dialog_open() {
                Dialog {
                    open: true,
                    on_close: move |_| dialog_open.set(false),
                    title: if editing_id().is_some() { "Edit Transaction" } else { "Add Transaction" },
                    footer: rsx! {
                        div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2",
                            button {
                                onclick: move |_| dialog_open.set(false),
                                class: "h-10 px-4 py-2 rounded-md border border-border text-sm font-medium hover:bg-accent",
                                "Cancel"
                            }
                            button {
                                onclick: handle_save,
                                disabled: form_saving(),
                                class: "h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50",
                                if form_saving() { "Saving..." } else { "Save" }
                            }
                        }
                    },
                div { class: "space-y-4",
                    // Name
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Name" }
                        input {
                            r#type: "text",
                            value: form_name.to_string(),
                            oninput: move |e| {
                                form_name.set(e.value());
                                name_error.set(String::new());
                            },
                            class: if name_error().is_empty() { "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" } else { "flex h-10 w-full rounded-md border border-red-500 bg-background px-3 py-2 text-sm" },
                        }
                        if !name_error().is_empty() {
                            p { class: "text-xs text-red-500", "{name_error()}" }
                        }
                    }
                    // Amount
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Amount" }
                            input {
                                r#type: "number",
                                step: "0.01",
                                value: form_amount.to_string(),
                                oninput: move |e| {
                                    form_amount.set(e.value());
                                    amount_error.set(String::new());
                                },
                                class: if amount_error().is_empty() { "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" } else { "flex h-10 w-full rounded-md border border-red-500 bg-background px-3 py-2 text-sm" },
                            }
                            if !amount_error().is_empty() {
                                p { class: "text-xs text-red-500", "{amount_error()}" }
                            }
                        }
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Currency" }
                            select {
                                value: form_currency.to_string(),
                                onchange: move |e| {
                                    form_currency.set(e.value());
                                    currency_error.set(String::new());
                                },
                                class: if currency_error().is_empty() { "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" } else { "flex h-10 w-full rounded-md border border-red-500 bg-background px-3 py-2 text-sm" },
                                for (val, label) in crate::utils::constants::CURRENCIES {
                                    option { value: val.to_string(), {label.to_string()} }
                                }
                            }
                            if !currency_error().is_empty() {
                                p { class: "text-xs text-red-500", "{currency_error()}" }
                            }
                        }
                    }
                    // Date
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Date" }
                        input {
                            r#type: "date",
                            value: form_date.to_string(),
                            oninput: move |e| {
                                form_date.set(e.value());
                                date_error.set(String::new());
                            },
                            class: if date_error().is_empty() { "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" } else { "flex h-10 w-full rounded-md border border-red-500 bg-background px-3 py-2 text-sm" },
                        }
                        if !date_error().is_empty() {
                            p { class: "text-xs text-red-500", "{date_error()}" }
                        }
                    }
                    // Category
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Category" }
                        {rsx! {
                            crate::components::CategoryCombobox {
                                selected_id: form_category_id().map(|id| id.to_string()).unwrap_or_default(),
                                on_change: move |id: String| {
                                    form_category_id.set(if id.is_empty() { None } else { Uuid::parse_str(&id).ok() });
                                    category_error.set(String::new());
                                },
                                error: if category_error().is_empty() { None } else { Some(category_error()) },
                            }
                        }}
                    }
                    // Notes
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Notes" }
                        textarea {
                            value: form_notes.to_string(),
                            oninput: move |e| form_notes.set(e.value()),
                            class: "flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm min-h-[80px]",
                        }
                    }
                    // Logo
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Logo (optional)" }
                        LogoSearch {
                            selected_url: form_logo_url(),
                            default_query: form_name(),
                            on_select: move |url: String| form_logo_url.set(url),
                        }
                    }
                    }
                }
            }


            // ── Notes dialog ──
            if notes_dialog_open() {
                {
                    let txn_name = notes_dialog_txn_id().map(|id| {
                        TRANSACTIONS().transactions.iter().find(|t| t.id == id).map(|t| t.name.clone()).unwrap_or_default()
                    }).unwrap_or_default();
                    let title = format!("Notes: {}", txn_name);
                    rsx! {
                        Dialog {
                            open: true,
                            on_close: move |_| notes_dialog_open.set(false),
                            title: title,
                            footer: rsx! {
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
                                            let id = notes_dialog_txn_id().unwrap();
                                            let notes = notes_dialog_value();
                                            let is_empty = notes.is_empty();
                                            notes_saving.set(true);
                                            spawn(async move {
                                                let req = UpdateTransactionRequest {
                                                    notes: if is_empty { None } else { Some(notes) },
                                                    ..Default::default()
                                                };
                                                let _ = Storage::update_transaction(&id, &req).await;
                                                match Storage::list_transactions().await {
                                                    Ok(data) => TRANSACTIONS.write().set_data(data),
                                                    Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
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

#[cfg(test)]
mod tests;
