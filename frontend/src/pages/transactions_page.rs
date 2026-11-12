use crate::components::logo_image_with_fallback;
use crate::components::ConvertedAmount;
use crate::components::page_header::PageHeader;
use crate::components::skeleton::TransactionRowSkeleton;
use crate::components::tooltip::Tooltip;
use crate::models::transaction::*;
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::preferences::PREFERENCES;
use crate::state::transactions::TRANSACTIONS;
use crate::state::create_intent::{CREATE_INTENT, CreateIntent};
use crate::storage::Storage;
use crate::utils::dates::format_date;
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;
use chrono::Datelike;
use dioxus::prelude::*;
use lucide_dioxus::{FileText, Pencil, Plus, Receipt, Trash2};
use rust_decimal::Decimal;
use uuid::Uuid;

mod dialog;
mod filter_sort_bar;
mod notes_dialog;

use dialog::TransactionDialog;
use filter_sort_bar::FilterSortBar;
use notes_dialog::NotesDialog;

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
    let sort_cfg = use_signal(|| SortConfig {
        field: "date".into(),
        direction: "desc".into(),
    });
    let mut dialog_open = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);

    // Notes dialog state
    let mut notes_dialog_open = use_signal(|| false);
    let mut notes_dialog_txn_id = use_signal(|| None::<Uuid>);

    // Load data
    use_future(move || async move {
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
        if !CATEGORIES().categories.is_empty() {
            return;
        }
        CATEGORIES.write().set_loading();
        match Storage::list_categories().await {
            Ok(data) => CATEGORIES.write().set_data(data),
            Err(e) => CATEGORIES.write().set_error(e.to_string()),
        }
    });

    use_future(move || async move {
        if !ACCOUNTS().accounts.is_empty() {
            return;
        }
        ACCOUNTS.write().set_loading();
        match Storage::list_accounts().await {
            Ok(data) => ACCOUNTS.write().set_data(data),
            Err(e) => ACCOUNTS.write().set_error(e.to_string()),
        }
    });

    let currency = AUTH().currency().to_string();
    let state = TRANSACTIONS();
    let accounts = ACCOUNTS();

    let groups = filter_sort_group(&state.transactions, &filters(), &sort_cfg());

    let handle_delete = move |id: Uuid| {
        spawn(async move {
            let _ = Storage::delete_transaction(&id).await;
            match Storage::list_transactions().await {
                Ok(data) => TRANSACTIONS.write().set_data(data),
                Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
            }
        });
    };

    use_effect(move || {
        if *CREATE_INTENT.read() == Some(CreateIntent::Transaction) {
            *CREATE_INTENT.write() = None;
            editing_id.set(None);
            dialog_open.set(true);
        }
    });

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
        div { class: "p-4 md:p-6 flex flex-col gap-6 animate-in fade-in duration-300",
            // Header
            div { class: "flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3",
                PageHeader { title: "Transactions", subtitle: "Track your one-time expenses" }
                button {
                    onclick: move |_| {
                        editing_id.set(None);
                        dialog_open.set(true);
                    },
                    class: "hidden md:inline-flex items-center justify-center gap-2 h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 w-full sm:w-auto",
                    Plus { size: 16 }
                    "Add Transaction"
                }
            }

            // Filter / Sort bar
            if !state.transactions.is_empty() {
                FilterSortBar { filters: filters, sort_cfg: sort_cfg }
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
                                dialog_open.set(true);
                            },
                            class: "hidden md:inline-flex items-center gap-2 h-10 px-4 py-2 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90",
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
                                                        {logo_image_with_fallback(&format!("txn-{}", txn.id), txn.logo_url.as_deref(), &txn.name, "h-9 w-9 sm:h-10 sm:w-10 object-contain p-1")}
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
                                                    // Account badge
                                                    {
                                                        let account = accounts.get_by_id(&txn.account_id);
                                                        let account_name = account.map(|a| a.name.as_str()).unwrap_or("No account");
                                                        let account_icon = account.and_then(|a| a.icon.clone()).unwrap_or_else(|| "Tag".into());
                                                        let account_color = account.and_then(|a| a.color.clone()).unwrap_or_else(|| "#6b7280".into());
                                                        let bg = format!("{}20", account_color);
                                                        rsx! {
                                                            span { class: "font-normal text-xs hidden sm:inline-flex items-center gap-1 px-2 py-0.5 rounded",
                                                                style: "color: {account_color}; background-color: {bg}",
                                                                {icon_display(&account_icon)}
                                                                {account_name.to_string()}
                                                            }
                                                        }
                                                    }
                                                    // Amount
                                                    p { class: "font-medium text-sm sm:text-base shrink-0",
                                                        ConvertedAmount { amount: txn.amount, currency: txn.currency.clone() }
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
                                                                move |_| {
                                                                    editing_id.set(Some(txn.id));
                                                                    dialog_open.set(true);
                                                                }
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

            // Create / Edit dialog
            if dialog_open() {
                TransactionDialog { open: dialog_open, editing_id: editing_id }
            }

            // Notes dialog
            if notes_dialog_open() {
                NotesDialog { open: notes_dialog_open, txn_id: notes_dialog_txn_id }
            }
        }
    }
}

#[cfg(test)]
mod tests;
