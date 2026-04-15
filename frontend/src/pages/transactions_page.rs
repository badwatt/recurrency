use dioxus::prelude::*;
use crate::state::transactions::TRANSACTIONS;
use crate::state::categories::CATEGORIES;
use crate::state::auth::AUTH;
use crate::api::transactions;
use crate::models::transaction::*;
use crate::utils::format::format_currency;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

// ─── Lucide-style SVG icon components ───────────────────────────────────────

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
        if sort.direction == "asc" { cmp } else { cmp.reverse() }
    });

    // Group by month descending
    let mut groups: std::collections::BTreeMap<String, Vec<Transaction>> = std::collections::BTreeMap::new();
    for t in result {
        let key = format!("{}-{:02}", t.date.year(), t.date.month());
        groups.entry(key).or_default().push(t);
    }

    let mut out: Vec<_> = groups.into_iter().collect();
    out.sort_by(|a, b| b.0.cmp(&a.0));

    out.into_iter().map(|(key, txns)| {
        let (year, month) = key.split_once('-').unwrap();
        let y: i32 = year.parse().unwrap();
        let m: u32 = month.parse().unwrap();
        let label = format_month(y, m);
        let total: Decimal = txns.iter().map(|t| t.amount).sum();
        MonthGroup { key, label, transactions: txns, total }
    }).collect()
}

fn format_month(year: i32, month: u32) -> String {
    let names = [
        "", "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"
    ];
    format!("{} {}", names.get(month as usize).unwrap_or(&""), year)
}

#[component]
pub fn TransactionsPage() -> Element {
    let mut filters = use_signal(FilterConfig::default);
    let mut sort_cfg = use_signal(|| SortConfig { field: "date".into(), direction: "desc".into() });
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

    // Load data
    use_future(move || async move {
        if TRANSACTIONS().is_loading { return; }
        TRANSACTIONS.write().set_loading();
        match transactions::list().await {
            Ok(data) => TRANSACTIONS.write().set_data(data),
            Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
        }
    });

    use_future(move || async move {
        if !CATEGORIES().categories.is_empty() { return; }
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
        dialog_open.set(true);
    };

    let handle_save = move |_| {
        let name = form_name();
        let amount_str = form_amount();
        let amount = match Decimal::from_str(&amount_str) {
            Ok(d) => d,
            Err(_) => return,
        };
        let date = match NaiveDate::parse_from_str(&form_date(), "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return,
        };
        let editing = editing_id();
        let cat_id = form_category_id();

        form_saving.set(true);
        spawn(async move {
            let result = if let Some(id) = editing {
                transactions::update(&id, &UpdateTransactionRequest {
                    name: Some(name),
                    amount: Some(amount),
                    currency: Some(form_currency()),
                    date: Some(date),
                    category_id: cat_id,
                    notes: if form_notes().is_empty() { None } else { Some(form_notes()) },
                    logo_url: None,
                }).await
            } else {
                transactions::create(&CreateTransactionRequest {
                    name,
                    amount,
                    currency: form_currency(),
                    date,
                    category_id: cat_id,
                    notes: if form_notes().is_empty() { None } else { Some(form_notes()) },
                    logo_url: None,
                }).await
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
        div { class: "p-4 md:p-6 space-y-6",
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
                                                        if let Some(url) = &txn.logo_url {
                                                            img { src: "{url}", alt: "{txn.name}", class: "h-full w-full object-contain p-1" }
                                                        } else {
                                                            span { class: "text-muted-foreground",
                                                                {ReceiptIcon()}
                                                            }
                                                        }
                                                    }
                                                    // Name + date
                                                    div { class: "flex-1 min-w-0",
                                                        p { class: "font-medium truncate text-sm sm:text-base", "{txn.name}" }
                                                        p { class: "text-xs text-muted-foreground",
                                                            {txn.date.format("%b %e, %Y").to_string()}
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
                div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
                    div { class: "w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg mx-4",
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
                                    oninput: move |e| form_name.set(e.value()),
                                    class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
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
                                        oninput: move |e| form_amount.set(e.value()),
                                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "text-sm font-medium", "Currency" }
                                    select {
                                        value: "{form_currency}",
                                        onchange: move |e| form_currency.set(e.value()),
                                        class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                        for (val, label) in crate::utils::constants::CURRENCIES {
                                            option { value: "{val}", "{label}" }
                                        }
                                    }
                                }
                            }
                            // Date
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", "Date" }
                                input {
                                    r#type: "date",
                                    value: "{form_date}",
                                    oninput: move |e| form_date.set(e.value()),
                                    class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                }
                            }
                            // Category
                            div { class: "space-y-2",
                                label { class: "text-sm font-medium", "Category" }
                                select {
                                    value: form_category_id().map(|id| id.to_string()).unwrap_or_default(),
                                    onchange: move |e| {
                                        let v = e.value();
                                        form_category_id.set(if v.is_empty() { None } else { Uuid::parse_str(&v).ok() });
                                    },
                                    class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                    option { value: "", "Select category" }
                                    for cat in &cats.categories {
                                        option { value: "{cat.id}", "{cat.name}" }
                                    }
                                }
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
                            div { class: "flex justify-end gap-2 pt-2",
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

    #[test]
    fn filter_by_category() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let txns = vec![
            make_txn("A", dec!(10), NaiveDate::from_ymd_opt(2024,1,1).unwrap(), c1),
            make_txn("B", dec!(20), NaiveDate::from_ymd_opt(2024,1,2).unwrap(), c2),
        ];
        let filters = FilterConfig { category_id: Some(c1), ..Default::default() };
        let groups = filter_sort_group(&txns, &filters, &SortConfig::default());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].transactions.len(), 1);
        assert_eq!(groups[0].transactions[0].name, "A");
    }

    #[test]
    fn filter_by_price_range() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn("Cheap", dec!(5), NaiveDate::from_ymd_opt(2024,1,1).unwrap(), c),
            make_txn("Mid", dec!(15), NaiveDate::from_ymd_opt(2024,1,2).unwrap(), c),
            make_txn("Expensive", dec!(50), NaiveDate::from_ymd_opt(2024,1,3).unwrap(), c),
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
            make_txn("A", dec!(10), NaiveDate::from_ymd_opt(2024,1,1).unwrap(), c),
            make_txn("B", dec!(30), NaiveDate::from_ymd_opt(2024,1,2).unwrap(), c),
            make_txn("C", dec!(20), NaiveDate::from_ymd_opt(2024,1,3).unwrap(), c),
        ];
        let sort = SortConfig { field: "amount".into(), direction: "desc".into() };
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &sort);
        assert_eq!(groups[0].transactions[0].name, "B");
        assert_eq!(groups[0].transactions[1].name, "C");
        assert_eq!(groups[0].transactions[2].name, "A");
    }

    #[test]
    fn group_by_month_descending() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn("Jan", dec!(10), NaiveDate::from_ymd_opt(2024,1,15).unwrap(), c),
            make_txn("Mar", dec!(20), NaiveDate::from_ymd_opt(2024,3,1).unwrap(), c),
            make_txn("Feb", dec!(15), NaiveDate::from_ymd_opt(2024,2,10).unwrap(), c),
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
            make_txn("A", dec!(10), NaiveDate::from_ymd_opt(2024,1,1).unwrap(), c),
            make_txn("B", dec!(20), NaiveDate::from_ymd_opt(2024,1,15).unwrap(), c),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].total, dec!(30));
    }

    #[test]
    fn empty_filters_return_all() {
        let c = Uuid::new_v4();
        let txns = vec![
            make_txn("X", dec!(10), NaiveDate::from_ymd_opt(2024,1,1).unwrap(), c),
        ];
        let groups = filter_sort_group(&txns, &FilterConfig::default(), &SortConfig::default());
        assert_eq!(groups[0].transactions.len(), 1);
    }

    #[test]
    fn format_month_labels() {
        assert_eq!(format_month(2024, 1), "January 2024");
        assert_eq!(format_month(2024, 6), "June 2024");
        assert_eq!(format_month(2024, 12), "December 2024");
    }
}