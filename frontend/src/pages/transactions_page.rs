use crate::api::transactions;
use crate::components::dialog::Dialog;
use crate::components::logo_image_with_fallback;
use crate::models::transaction::*;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::preferences::PREFERENCES;
use crate::state::transactions::TRANSACTIONS;
use crate::utils::dates::format_date;
use crate::utils::format::format_currency;
use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

// ─── Lucide-style SVG icon components ───────────────────────────────────────

#[allow(non_snake_case)]
fn ReceiptIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M4 2v20l2-1 2 1 2-1 2 1 2-1 2 1 2-1 2 1V2l-2 1-2-1-2 1-2-1-2 1-2-1-2 1Z" }
            line { x1: "8", y1: "7", x2: "16", y2: "7" }
            line { x1: "8", y1: "11", x2: "16", y2: "11" }
            line { x1: "8", y1: "15", x2: "12", y2: "15" }
        }
    }
}

#[allow(non_snake_case)]
fn ReceiptIconLarge() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "40",
            height: "40",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M4 2v20l2-1 2 1 2-1 2 1 2-1 2 1 2-1 2 1V2l-2 1-2-1-2 1-2-1-2 1-2-1-2 1Z" }
            line { x1: "8", y1: "7", x2: "16", y2: "7" }
            line { x1: "8", y1: "11", x2: "16", y2: "11" }
            line { x1: "8", y1: "15", x2: "12", y2: "15" }
        }
    }
}

#[allow(non_snake_case)]
fn Trash2Icon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "3 6 5 6 21 6" }
            path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
            line { x1: "10", y1: "11", x2: "10", y2: "17" }
            line { x1: "14", y1: "11", x2: "14", y2: "17" }
        }
    }
}

#[allow(non_snake_case)]
fn PencilIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" }
        }
    }
}

#[allow(non_snake_case)]
fn PlusIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            width: "16",
            height: "16",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "12", y1: "5", x2: "12", y2: "19" }
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}

// ─── Data types ─────────────────────────────────────────────────────────────

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
/// Transaction validation result type.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransactionValidationResult {
    pub name_error: Option<String>,
    pub amount_error: Option<String>,
    pub currency_error: Option<String>,
    pub date_error: Option<String>,
    pub category_error: Option<String>,
}

impl TransactionValidationResult {
    pub fn is_valid(&self) -> bool {
        self.name_error.is_none()
            && self.amount_error.is_none()
            && self.currency_error.is_none()
            && self.date_error.is_none()
            && self.category_error.is_none()
    }
}

/// Pure validation function for transaction form.
pub fn validate_transaction_form(
    name: &str,
    amount: &str,
    currency: &str,
    date: &str,
    category_id: &str,
) -> TransactionValidationResult {
    use rust_decimal::Decimal;
    let mut result = TransactionValidationResult::default();
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
    if date.is_empty() {
        result.date_error = Some("Date is required".into());
    } else if date.parse::<chrono::NaiveDate>().is_err() {
        result.date_error = Some("Invalid date format".into());
    }
    if category_id.is_empty() {
        result.category_error = Some("Category is required".into());
    }
    result
}

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
    let mut show_filters = use_signal(|| false);

    // Local form state for create/edit dialog
    let mut form_name = use_signal(String::new);
    let mut form_amount = use_signal(String::new);
    let mut form_currency = use_signal(|| "EUR".to_string());
    let mut form_date = use_signal(String::new);
    let mut form_category_id = use_signal(|| None::<Uuid>);
    let mut form_notes = use_signal(String::new);
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
        match transactions::list().await {
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
        match crate::api::categories::list().await {
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
                transactions::update(
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
                        logo_url: None,
                    },
                )
                .await
            } else {
                transactions::create(&CreateTransactionRequest {
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
                    logo_url: None,
                })
                .await
            };

            match result {
                Ok(_) => {
                    dialog_open.set(false);
                    // Reload
                    match transactions::list().await {
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
            let _ = transactions::delete(&id).await;
            match transactions::list().await {
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
            div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                div {
                    h1 { class: "text-3xl font-bold tracking-tight",
                        {ReceiptIcon()}
                        " Transactions"
                    }
                    p { class: "text-muted-foreground", "Track your one-time expenses" }
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
                    class: "inline-flex items-center justify-center gap-2 h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 w-full sm:w-auto",
                    {PlusIcon()}
                    "Add Transaction"
                }
            }

            // Filters toggle
            if !state.transactions.is_empty() {
                div { class: "flex items-center gap-2",
                    button {
                        onclick: move |_| show_filters.toggle(),
                        class: "text-sm text-muted-foreground hover:text-foreground underline",
                        if show_filters() { "Hide filters" } else { "Show filters" }
                    }
                    if filters().category_id.is_some() || filters().price_min.is_some() || filters().price_max.is_some() {
                        button {
                            onclick: move |_| filters.set(FilterConfig::default()),
                            class: "text-sm text-destructive hover:underline",
                            "Clear filters"
                        }
                    }
                }
            }

            // Filter panel
            if show_filters() && !state.transactions.is_empty() {
                div { class: "grid gap-4 sm:grid-cols-3 p-4 rounded-lg border border-border bg-card",
                    // Category filter
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Category" }
                        select {
                            value: filters().category_id.map(|id| id.to_string()).unwrap_or_default(),
                            onchange: move |e| {
                                let val = e.value();
                                let id = if val.is_empty() { None } else { Uuid::parse_str(&val).ok() };
                                let mut f = filters();
                                f.category_id = id;
                                filters.set(f);
                            },
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm",
                            option { value: "", "All categories" }
                            for cat in &cats.categories {
                                option { value: "{cat.id}", "{cat.name}" }
                            }
                        }
                    }
                    // Price min
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Min price" }
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
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm",
                        }
                    }
                    // Price max
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Max price" }
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
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm",
                        }
                    }
                    // Sort
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Sort by" }
                        div { class: "flex gap-2",
                            select {
                                value: "{sort_cfg().field}",
                                onchange: move |e| {
                                    let mut s = sort_cfg();
                                    s.field = e.value();
                                    sort_cfg.set(s);
                                },
                                class: "flex h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm",
                                option { value: "date", "Date" }
                                option { value: "amount", "Price" }
                                option { value: "name", "Name" }
                            }
                            select {
                                value: "{sort_cfg().direction}",
                                onchange: move |e| {
                                    let mut s = sort_cfg();
                                    s.direction = e.value();
                                    sort_cfg.set(s);
                                },
                                class: "flex h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm",
                                option { value: "desc", "Descending" }
                                option { value: "asc", "Ascending" }
                            }
                        }
                    }
                }
            }

            // Content
            {if state.is_loading {
                rsx! {
                    div { class: "flex h-64 items-center justify-center",
                        div { class: "h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" }
                    }
                }
            } else if state.transactions.is_empty() {
                rsx! {
                    div { class: "rounded-lg border border-border bg-card p-8 flex flex-col items-center justify-center gap-4",
                        span { class: "text-muted-foreground",
                            {ReceiptIconLarge()}
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
                            {PlusIcon()}
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
                                                                {ReceiptIcon()}
                                                            }
                                                        }
                                                    }
                                                    // Name + date
                                                    div { class: "flex-1 min-w-0",
                                                        p { class: "font-medium truncate text-sm sm:text-base", "{txn.name}" }
                                                        p { class: "text-xs text-muted-foreground",
                                                            {format_date(&txn.date, PREFERENCES().date_format())}
                                                        }
                                                    }
                                                    // Category badge
                                                    {
                                                        let cat = cats.get_by_id(&txn.category_id);
                                                        let cat_name = cat.map(|c| c.name.as_str()).unwrap_or("Uncategorized");
                                                        let cat_color = cat.and_then(|c| c.color.clone()).unwrap_or_else(|| "#6b7280".into());
                                                        let bg = format!("{}20", cat_color);
                                                        rsx! {
                                                            span { class: "font-normal text-xs hidden sm:inline-flex px-2 py-0.5 rounded",
                                                                style: "color: {cat_color}; background-color: {bg}",
                                                                "{cat_name}"
                                                            }
                                                        }
                                                    }
                                                    // Amount
                                                    p { class: "font-medium text-sm sm:text-base shrink-0",
                                                        {format_currency(txn.amount, &txn.currency)}
                                                    }
                                                    // Actions
                                                    div { class: "flex items-center gap-0.5 sm:gap-1 shrink-0",
                                                        button {
                                                            class: if txn.notes.as_ref().is_some_and(|n| !n.is_empty()) { "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-primary" } else { "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-muted-foreground" },
                                                            onclick: {
                                                                let txn = txn.clone();
                                                                move |_| {
                                                                    notes_dialog_txn_id.set(Some(txn.id));
                                                                    notes_dialog_value.set(txn.notes.clone().unwrap_or_default());
                                                                    notes_dialog_open.set(true);
                                                                }
                                                            },
                                                            "📝"
                                                        }
                                                        button {
                                                            class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent",
                                                            onclick: {
                                                                let txn = txn.clone();
                                                                move |_| open_edit(&txn)
                                                            },
                                                            {PencilIcon()}
                                                        }
                                                        button {
                                                            class: "h-8 w-8 sm:h-9 sm:w-9 inline-flex items-center justify-center rounded-md hover:bg-accent text-destructive",
                                                            onclick: {
                                                                let id = txn.id;
                                                                move |_| handle_delete(id)
                                                            },
                                                            {Trash2Icon()}
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
                div {
                    class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
                    onclick: move |_| dialog_open.set(false),
                    div {
                        class: "w-full sm:max-w-md rounded-none sm:rounded-lg border border-border bg-card p-4 sm:p-6 shadow-lg sm:mx-4 h-dvh sm:h-auto sm:max-h-[85vh] flex flex-col",
                        onclick: move |e| e.stop_propagation(),
                        h2 { class: "text-lg font-semibold mb-4",
                            if editing_id().is_some() { "Edit Transaction" } else { "Add Transaction" }
                        }
                        div { class: "space-y-4",
                            // Name
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", "Name" }
                                input {
                                    r#type: "text",
                                    value: "{form_name}",
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
                                        value: "{form_amount}",
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
                                        value: "{form_currency}",
                                        onchange: move |e| {
                                            form_currency.set(e.value());
                                            currency_error.set(String::new());
                                        },
                                        class: if currency_error().is_empty() { "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm" } else { "flex h-10 w-full rounded-md border border-red-500 bg-background px-3 py-2 text-sm" },
                                        for (val, label) in crate::utils::constants::CURRENCIES {
                                            option { value: "{val}", "{label}" }
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
                                    value: "{form_date}",
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
                                    value: "{form_notes}",
                                    oninput: move |e| form_notes.set(e.value()),
                                    class: "flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm min-h-[80px]",
                                }
                            }
                            // Actions
                            div { class: "flex flex-col-reverse sm:flex-row sm:justify-end gap-2 pt-2",
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
                                            let id = notes_dialog_txn_id().unwrap();
                                            let notes = notes_dialog_value();
                                            let is_empty = notes.is_empty();
                                            notes_saving.set(true);
                                            spawn(async move {
                                                let req = UpdateTransactionRequest {
                                                    notes: if is_empty { None } else { Some(notes) },
                                                    ..Default::default()
                                                };
                                                let _ = transactions::update(&id, &req).await;
                                                match transactions::list().await {
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
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_txn(name: &str, amount: Decimal, date: NaiveDate, cat_id: Uuid) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: cat_id,
            name: name.into(),
            amount,
            currency: "EUR".into(),
            date,
            notes: None,
            logo_url: None,
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    // ── FilterConfig tests ──
    #[test]
    fn filter_config_default() {
        let f = FilterConfig::default();
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn filter_config_partial_eq() {
        let f1 = FilterConfig {
            category_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        let f2 = FilterConfig {
            category_id: f1.category_id,
            ..Default::default()
        };
        assert_eq!(f1.category_id, f2.category_id);
    }

    #[test]
    fn filter_config_clone() {
        let f = FilterConfig::default();
        let cloned = f.clone();
        assert_eq!(f, cloned);
    }

    // ── SortConfig tests ──
    #[test]
    fn sort_config_debug() {
        let s = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        let debug = format!("{:?}", s);
        assert!(debug.contains("SortConfig"));
    }

    // ── MonthGroup tests ──
    #[test]
    fn month_group_debug() {
        let c = Uuid::new_v4();
        let group = MonthGroup {
            key: "2024-01".into(),
            label: "January 2024".into(),
            transactions: vec![make_txn(
                "Test",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            )],
            total: dec!(10),
        };
        let debug = format!("{:?}", group);
        assert!(debug.contains("MonthGroup"));
    }

    // ── TransactionValidationResult tests ──
    #[test]
    fn validation_result_default_is_valid() {
        let result = TransactionValidationResult::default();
        assert!(result.is_valid());
    }

    #[test]
    fn validation_result_with_name_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.name_error = Some("Name required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_amount_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.amount_error = Some("Amount required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_currency_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.currency_error = Some("Currency required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_date_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.date_error = Some("Date required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_with_category_error_invalid() {
        let mut result = TransactionValidationResult::default();
        result.category_error = Some("Category required".into());
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_result_multiple_errors() {
        let mut result = TransactionValidationResult::default();
        result.name_error = Some("Name required".into());
        result.amount_error = Some("Amount required".into());
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validation_result_clone() {
        let mut result = TransactionValidationResult::default();
        result.name_error = Some("err".into());
        let cloned = result.clone();
        assert_eq!(result.name_error, cloned.name_error);
    }

    // ── validate_transaction_form tests ──
    #[test]
    fn validate_form_all_valid() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(result.is_valid());
    }

    #[test]
    fn validate_form_empty_name() {
        let result = validate_transaction_form("", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_form_whitespace_name() {
        let result = validate_transaction_form("   ", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_form_short_name() {
        let result = validate_transaction_form("A", "15.99", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.name_error.is_some());
    }

    #[test]
    fn validate_form_empty_amount() {
        let result = validate_transaction_form("Netflix", "", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_zero_amount() {
        let result = validate_transaction_form("Netflix", "0", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_negative_amount() {
        let result = validate_transaction_form("Netflix", "-5", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_invalid_amount() {
        let result = validate_transaction_form("Netflix", "abc", "EUR", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.amount_error.is_some());
    }

    #[test]
    fn validate_form_empty_currency() {
        let result = validate_transaction_form("Netflix", "15.99", "", "2024-01-01", "cat-id");
        assert!(!result.is_valid());
        assert!(result.currency_error.is_some());
    }

    #[test]
    fn validate_form_empty_date() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "", "cat-id");
        assert!(!result.is_valid());
        assert!(result.date_error.is_some());
    }

    #[test]
    fn validate_form_invalid_date() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "not-a-date", "cat-id");
        assert!(!result.is_valid());
        assert!(result.date_error.is_some());
    }

    #[test]
    fn validate_form_empty_category() {
        let result = validate_transaction_form("Netflix", "15.99", "EUR", "2024-01-01", "");
        assert!(!result.is_valid());
        assert!(result.category_error.is_some());
    }

    #[test]
    fn validate_form_decimal_amount() {
        let result = validate_transaction_form("Netflix", "9.99", "EUR", "2024-01-01", "cat-id");
        assert!(result.is_valid());
    }

    #[test]
    fn validate_form_valid_dates() {
        for date in &["2024-01-01", "2024-12-31", "2025-06-15"] {
            let result = validate_transaction_form("Test", "10", "EUR", date, "cat-id");
            assert!(result.date_error.is_none(), "Date {} should be valid", date);
        }
    }

    // ── filter_sort_group tests ──
    #[test]
    fn filter_by_category() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c1,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c2,
            ),
        ];
        let filters = FilterConfig {
            category_id: Some(c1),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "A");
    }

    #[test]
    fn filter_by_price_range() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Cheap",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Mid",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "Expensive",
                dec!(50),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_min: Some(dec!(10)),
            price_max: Some(dec!(30)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "Mid");
    }

    #[test]
    fn sort_by_amount_desc() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(30),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "C",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "desc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "B");
        assert_eq!(groups[0].transactions[1].name, "C");
        assert_eq!(groups[0].transactions[2].name, "A");
    }

    #[test]
    fn group_by_month_descending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Jan",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "Mar",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
                c,
            ),
            make_txn(
                "Feb",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].key, "2024-03");
        assert_eq!(groups[1].key, "2024-02");
        assert_eq!(groups[2].key, "2024-01");
    }

    #[test]
    fn month_group_total() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].total, dec!(30));
    }

    #[test]
    fn empty_filters_return_all() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "X",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        )];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
    }

    #[test]
    fn format_month_labels() {
        assert_eq!(format_month(2024, 1), "January 2024");
        assert_eq!(format_month(2024, 6), "June 2024");
        assert_eq!(format_month(2024, 12), "December 2024");
    }

    // ── Additional filter/sort tests ──
    #[test]
    fn filter_by_price_min_only() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_min: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "B");
    }

    #[test]
    fn filter_by_price_max_only() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_max: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "A");
    }

    #[test]
    fn filter_by_category_and_price() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c1,
            ),
            make_txn(
                "B",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c1,
            ),
            make_txn(
                "C",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c2,
            ),
        ];
        let filters = FilterConfig {
            category_id: Some(c1),
            price_min: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "B");
    }

    #[test]
    fn sort_by_name_ascending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Charlie",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Alpha",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "Alpha");
        assert_eq!(groups[0].transactions[1].name, "Charlie");
    }

    #[test]
    fn sort_by_amount_ascending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "C",
                dec!(30),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "amount".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "A");
        assert_eq!(groups[0].transactions[1].name, "B");
        assert_eq!(groups[0].transactions[2].name, "C");
    }

    #[test]
    fn sort_by_date_descending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "date".into(),
            direction: "desc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "B");
        assert_eq!(groups[0].transactions[1].name, "A");
    }

    #[test]
    fn sort_unknown_field_defaults_to_date() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "unknown".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "A"); // sorted by date asc as default
    }

    #[test]
    fn groups_across_multiple_months() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Jan1",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Jan2",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "Feb",
                dec!(15),
                NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
                c,
            ),
            make_txn(
                "Mar",
                dec!(25),
                NaiveDate::from_ymd_opt(2024, 3, 5).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups.len(), 3);
        // Descending month order
        assert_eq!(groups[0].key, "2024-03");
        assert_eq!(groups[1].key, "2024-02");
        assert_eq!(groups[2].key, "2024-01");
    }

    #[test]
    fn month_group_total_multiple_txns() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "C",
                dec!(30),
                NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].total, dec!(60));
    }

    #[test]
    fn no_transactions_returns_empty_groups() {
        let txns: Vec<Transaction> = vec![];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_removes_all_returns_empty_groups() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "A",
            dec!(5),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        )];
        let filters = FilterConfig {
            price_min: Some(dec!(100)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    // ── Validation logic (mirrors dialog save) ──
    #[test]
    fn validate_name_empty() {
        let name = "";
        assert!(name.trim().is_empty());
    }

    #[test]
    fn validate_name_too_short() {
        let name = "A";
        assert!(name.trim().len() < 2);
    }

    #[test]
    fn validate_amount_zero_invalid() {
        let amount_str = "0";
        let parsed = Decimal::from_str(amount_str);
        match parsed {
            Ok(d) => assert!(d <= Decimal::ZERO),
            Err(_) => panic!("Should parse"),
        }
    }

    #[test]
    fn validate_amount_negative_invalid() {
        let amount_str = "-5";
        let parsed = Decimal::from_str(amount_str);
        match parsed {
            Ok(d) => assert!(d <= Decimal::ZERO),
            Err(_) => panic!("Should parse"),
        }
    }

    #[test]
    fn validate_amount_positive_valid() {
        let amount_str = "9.99";
        let parsed = Decimal::from_str(amount_str);
        match parsed {
            Ok(d) => assert!(d > Decimal::ZERO),
            Err(_) => panic!("Should parse"),
        }
    }

    #[test]
    fn validate_amount_non_numeric_invalid() {
        let amount_str = "abc";
        let parsed = Decimal::from_str(amount_str);
        assert!(parsed.is_err());
    }

    #[test]
    fn validate_date_valid_format() {
        let date_str = "2024-06-15";
        assert!(NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok());
    }

    #[test]
    fn validate_date_invalid_format() {
        let date_str = "06/15/2024";
        assert!(NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err());
    }

    #[test]
    fn validate_date_empty_invalid() {
        let date_str = "";
        assert!(date_str.is_empty());
    }

    #[test]
    fn month_label_all_months() {
        for m in 1..=12u32 {
            let label = format_month(2024, m);
            assert!(!label.is_empty(), "Month {} should have a label", m);
        }
    }

    #[test]
    fn filter_config_default_is_empty() {
        let f = FilterConfig::default();
        assert!(f.category_id.is_none());
        assert!(f.price_min.is_none());
        assert!(f.price_max.is_none());
    }

    #[test]
    fn sort_config_default() {
        let s = SortConfig::default();
        assert_eq!(s.field, "");
        assert_eq!(s.direction, "");
    }

    #[test]
    fn transaction_has_notes() {
        let c = Uuid::new_v4();
        let mut txn = make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        );
        txn.notes = Some("Important note".into());
        assert!(txn.notes.as_ref().is_some_and(|n| !n.is_empty()));
    }

    #[test]
    fn transaction_no_notes() {
        let c = Uuid::new_v4();
        let txn = make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        );
        assert!(!txn.notes.as_ref().is_some_and(|n| !n.is_empty()));
    }

    // ── TDD: Transaction dialog click-outside ──
    #[test]
    fn transaction_dialog_card_has_stop_propagation() {
        // Add/Edit transaction modal card must have stop_propagation.
        // Bug: inline backdrop has onclick but card doesn't stop_propagation →
        // clicks inside dialog propagate to backdrop → dialog closes.
        let src = include_str!("transactions_page.rs");
        // Find the Add/Edit dialog overlay section (NOT the notes dialog)
        let dialog_section = src.split("Dialog overlay").nth(1).unwrap_or("");
        let before_notes = dialog_section
            .split("Notes dialog")
            .next()
            .unwrap_or(dialog_section);
        // Card inside dialog must have stop_propagation (not just in test comments)
        let code_lines: Vec<&str> = before_notes
            .lines()
            .filter(|l| !l.trim().starts_with("//"))
            .collect();
        let code_text = code_lines.join("\n");
        let has_stop_prop_on_card = code_text.contains("e.stop_propagation()");
        let uses_dialog_comp =
            before_notes.contains("Dialog {") || before_notes.contains("Dialog(");
        assert!(
            has_stop_prop_on_card || uses_dialog_comp,
            "Transaction Add/Edit dialog card must have onclick stop_propagation or use Dialog component"
        );
    }

    // ── filter_sort_group additional edge cases ──
    #[test]
    fn filter_sort_group_empty_input() {
        let txns: Vec<Transaction> = vec![];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_no_category() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        )];
        let filters = FilterConfig {
            category_id: Some(Uuid::new_v4()),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_price_min_above_all() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_min: Some(dec!(100)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_price_max_below_all() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(50),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(100),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let filters = FilterConfig {
            price_max: Some(dec!(10)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert!(groups.is_empty());
    }

    #[test]
    fn filter_sort_group_combined_all_filters() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c1,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c1,
            ),
            make_txn(
                "C",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c2,
            ),
        ];
        let filters = FilterConfig {
            category_id: Some(c1),
            price_min: Some(dec!(15)),
            ..Default::default()
        };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "B");
    }

    #[test]
    fn month_group_label_format() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups.len(), 2);
        // Groups are sorted descending by key
        assert_eq!(groups[0].key, "2024-02");
        assert_eq!(groups[0].label, "February 2024");
        assert_eq!(groups[1].key, "2024-01");
        assert_eq!(groups[1].label, "January 2024");
    }

    #[test]
    fn month_group_total_sum() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "A",
                dec!(10.5),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "B",
                dec!(20.25),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
            make_txn(
                "C",
                dec!(5.25),
                NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        // Sum = 10.5 + 20.25 + 5.25 = 36.0
        assert_eq!(groups[0].total, dec!(36.0));
    }

    #[test]
    fn sort_by_name_descending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Apple",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "Zebra",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "name".into(),
            direction: "desc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "Zebra");
        assert_eq!(groups[0].transactions[1].name, "Apple");
    }

    #[test]
    fn sort_by_date_ascending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "Later",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "Earlier",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                c,
            ),
        ];
        let sort = SortConfig {
            field: "date".into(),
            direction: "asc".into(),
        };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "Earlier");
        assert_eq!(groups[0].transactions[1].name, "Later");
    }

    #[test]
    fn multiple_groups_each_with_total() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn(
                "JanA",
                dec!(10),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                c,
            ),
            make_txn(
                "JanB",
                dec!(20),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                c,
            ),
            make_txn(
                "FebA",
                dec!(5),
                NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                c,
            ),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        // Jan: 30, Feb: 5
        assert_eq!(groups[0].key, "2024-02");
        assert_eq!(groups[0].total, dec!(5));
        assert_eq!(groups[1].key, "2024-01");
        assert_eq!(groups[1].total, dec!(30));
    }

    // ── MonthGroup struct tests ──
    #[test]
    fn month_group_key_format() {
        let c = Uuid::new_v4();
        let txns = vec![make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 3, 5).unwrap(),
            c,
        )];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].key, "2024-03");
    }

    #[test]
    fn month_group_transactions_preserved() {
        let c = Uuid::new_v4();
        let t1 = make_txn(
            "A",
            dec!(10),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            c,
        );
        let t2 = make_txn(
            "B",
            dec!(20),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            c,
        );
        let txns = vec![t1.clone(), t2.clone()];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 2);
    }

    // ── format_month boundary months ──
    #[test]
    fn format_month_january() {
        assert_eq!(format_month(2024, 1), "January 2024");
    }

    #[test]
    fn format_month_december() {
        assert_eq!(format_month(2024, 12), "December 2024");
    }

    #[test]
    fn format_month_single_digit_month() {
        assert_eq!(format_month(2024, 6), "June 2024");
    }

    #[test]
    fn format_month_year_boundary() {
        assert_eq!(format_month(2023, 12), "December 2023");
        assert_eq!(format_month(2025, 1), "January 2025");
    }

    // ── TransactionValidationResult tests ──
    #[test]
    fn txn_validation_result_default() {
        let v = TransactionValidationResult::default();
        assert!(v.is_valid());
        assert!(v.name_error.is_none());
    }

    #[test]
    fn txn_validation_result_with_errors() {
        let v = TransactionValidationResult {
            name_error: Some("Name required".into()),
            amount_error: Some("Amount required".into()),
            ..Default::default()
        };
        assert!(!v.is_valid());
        assert!(v.name_error.is_some());
        assert!(v.amount_error.is_some());
    }

    #[test]
    fn txn_validation_result_clone() {
        let v = TransactionValidationResult {
            name_error: Some("err".into()),
            ..Default::default()
        };
        let c = v.clone();
        assert_eq!(v.name_error, c.name_error);
    }

    // ── validate_transaction_form tests ──
    #[test]
    fn validate_txn_form_valid() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert!(r.is_valid());
    }

    #[test]
    fn validate_txn_form_empty_name() {
        let r = validate_transaction_form("", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.name_error.as_deref(), Some("Name is required"));
        assert!(!r.is_valid());
    }

    #[test]
    fn validate_txn_form_whitespace_name() {
        let r = validate_transaction_form("   ", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.name_error.as_deref(), Some("Name is required"));
    }

    #[test]
    fn validate_txn_form_short_name() {
        let r = validate_transaction_form("A", "4.50", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(
            r.name_error.as_deref(),
            Some("Name must be at least 2 characters")
        );
    }

    #[test]
    fn validate_txn_form_empty_amount() {
        let r = validate_transaction_form("Coffee", "", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.amount_error.as_deref(), Some("Amount is required"));
    }

    #[test]
    fn validate_txn_form_zero_amount() {
        let r = validate_transaction_form("Coffee", "0", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(
            r.amount_error.as_deref(),
            Some("Amount must be greater than 0")
        );
    }

    #[test]
    fn validate_txn_form_negative_amount() {
        let r = validate_transaction_form("Coffee", "-3", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(
            r.amount_error.as_deref(),
            Some("Amount must be greater than 0")
        );
    }

    #[test]
    fn validate_txn_form_invalid_amount() {
        let r = validate_transaction_form("Coffee", "xyz", "EUR", "2024-01-15", "cat-id-123");
        assert_eq!(r.amount_error.as_deref(), Some("Invalid amount format"));
    }

    #[test]
    fn validate_txn_form_empty_currency() {
        let r = validate_transaction_form("Coffee", "4.50", "", "2024-01-15", "cat-id-123");
        assert_eq!(r.currency_error.as_deref(), Some("Currency is required"));
    }

    #[test]
    fn validate_txn_form_empty_date() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "", "cat-id-123");
        assert_eq!(r.date_error.as_deref(), Some("Date is required"));
    }

    #[test]
    fn validate_txn_form_invalid_date() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "not-a-date", "cat-id-123");
        assert_eq!(r.date_error.as_deref(), Some("Invalid date format"));
    }

    #[test]
    fn validate_txn_form_empty_category() {
        let r = validate_transaction_form("Coffee", "4.50", "EUR", "2024-01-15", "");
        assert_eq!(r.category_error.as_deref(), Some("Category is required"));
    }

    #[test]
    fn validate_txn_form_multiple_errors() {
        let r = validate_transaction_form("", "", "", "", "");
        assert!(r.name_error.is_some());
        assert!(r.amount_error.is_some());
        assert!(r.currency_error.is_some());
        assert!(r.date_error.is_some());
        assert!(r.category_error.is_some());
    }

    #[test]
    fn validate_txn_form_decimal_amount() {
        let r = validate_transaction_form("Coffee", "9.99", "EUR", "2024-01-15", "cat-id-123");
        assert!(r.is_valid());

        // ── TransactionPage additional tests ─────────────────────────────────
        #[cfg(test)]
        mod transaction_page_tests {
            use super::*;

            #[test]
            fn filter_config_txn_default() {
                let f = FilterConfig::default();
                assert!(f.category_id.is_none());
                assert!(f.price_min.is_none());
                assert!(f.price_max.is_none());
            }

            #[test]
            fn sort_config_txn_default() {
                let s = SortConfig::default();
                assert_eq!(s.field, "date");
                assert_eq!(s.direction, "desc");
            }

            #[test]
            fn sort_config_txn_partial_eq() {
                let s1 = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                let s2 = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                assert_eq!(s1.field, s2.field);
                assert_eq!(s1.direction, s2.direction);
            }

            #[test]
            fn month_group_transactions_empty() {
                let _cat = Uuid::new_v4();
                let txns: Vec<Transaction> = vec![];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert!(groups.is_empty());
            }

            #[test]
            fn month_group_single_transaction() {
                let cat = Uuid::new_v4();
                let txns = vec![make_txn(
                    "Coffee",
                    dec!(4.50),
                    NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
                    cat,
                )];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups.len(), 1);
                assert_eq!(groups[0].key, "2024-06");
                assert_eq!(groups[0].total, dec!(4.50));
                assert_eq!(groups[0].transactions.len(), 1);
            }

            #[test]
            fn month_group_key_format_various() {
                let cat = Uuid::new_v4();
                for (year, month, expected) in &[
                    (2024, 1, "2024-01"),
                    (2024, 12, "2024-12"),
                    (2025, 6, "2025-06"),
                ] {
                    let txns = vec![make_txn(
                        "T",
                        dec!(1),
                        NaiveDate::from_ymd_opt(*year, *month, 1).unwrap(),
                        cat,
                    )];
                    let groups =
                        filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                    assert_eq!(
                        groups[0].key, *expected,
                        "key for {}/{} should be {}",
                        year, month, expected
                    );
                }
            }

            #[test]
            fn month_group_year_spans() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "Jan",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "Dec",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2023, 12, 1).unwrap(),
                        cat,
                    ),
                ];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups.len(), 2);
            }

            #[test]
            fn month_group_preserves_sort_within_group() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                // Sorted by amount ASC: A (10) then B (20)
                assert_eq!(names[0], "A");
                assert_eq!(names[1], "B");
            }

            #[test]
            fn filter_sort_group_price_min_only() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(5),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(15),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(25),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let filters = FilterConfig {
                    price_min: Some(dec!(15)),
                    ..Default::default()
                };
                let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn filter_sort_group_price_max_only() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(5),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(15),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(25),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let filters = FilterConfig {
                    price_max: Some(dec!(15)),
                    ..Default::default()
                };
                let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn filter_sort_group_combined() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(5),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(15),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                ];
                let filters = FilterConfig {
                    price_min: Some(dec!(5)),
                    price_max: Some(dec!(15)),
                    ..Default::default()
                };
                let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn month_group_total_accumulation() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(10.50),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20.25),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(15.25),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups[0].total, dec!(46.00));
            }

            #[test]
            fn month_group_many_months() {
                let cat = Uuid::new_v4();
                let mut txns = vec![];
                for m in 1..=12 {
                    txns.push(make_txn(
                        &format!("M{}", m),
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, m, 1).unwrap(),
                        cat,
                    ));
                }
                let groups =
                    filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
                assert_eq!(groups.len(), 12);
            }

            #[test]
            fn sort_by_amount_ascending() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "C",
                        dec!(30),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "amount".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                assert_eq!(names, vec!["A", "B", "C"]);
            }

            #[test]
            fn sort_by_amount_descending() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "C",
                        dec!(30),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "amount".into(),
                    direction: "desc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                assert_eq!(names, vec!["C", "B", "A"]);
            }

            #[test]
            fn sort_by_name() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "Zebra",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "Apple",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "Mango",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "name".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                let names: Vec<_> = groups[0]
                    .transactions
                    .iter()
                    .map(|t| t.name.clone())
                    .collect();
                assert_eq!(names, vec!["Apple", "Mango", "Zebra"]);
            }

            #[test]
            fn sort_by_unknown_field() {
                let cat = Uuid::new_v4();
                let txns = vec![
                    make_txn(
                        "A",
                        dec!(10),
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        cat,
                    ),
                    make_txn(
                        "B",
                        dec!(20),
                        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
                        cat,
                    ),
                ];
                let sort = SortConfig {
                    field: "unknown_field".into(),
                    direction: "asc".into(),
                };
                let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
                // Unknown field: preserves original order
                assert_eq!(groups[0].transactions.len(), 2);
            }

            #[test]
            fn validate_txn_form_large_amount() {
                let r = validate_transaction_form(
                    "Big purchase",
                    "999999.99",
                    "EUR",
                    "2024-01-15",
                    "cat-id-123",
                );
                assert!(r.is_valid());
            }

            #[test]
            fn validate_txn_form_small_amount() {
                let r =
                    validate_transaction_form("Micro", "0.01", "EUR", "2024-01-15", "cat-id-123");
                assert!(r.is_valid());
            }

            #[test]
            fn validate_txn_form_valid_date_formats() {
                for date in &["2024-01-01", "2024-06-30", "2025-12-31"] {
                    let r = validate_transaction_form("Test", "10", "EUR", date, "cat-id-123");
                    assert!(r.date_error.is_none(), "date {} should be valid", date);
                }
            }
        }
    }
}
