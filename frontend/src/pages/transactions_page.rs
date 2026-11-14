use crate::components::ConvertedAmount;
use crate::components::logo_image_with_fallback;
use crate::components::page_header::PageHeader;
use crate::components::skeleton::TransactionRowSkeleton;
use crate::components::tooltip::Tooltip;
use crate::models::transaction::*;
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::create_intent::{CREATE_INTENT, CreateIntent};
use crate::state::preferences::PREFERENCES;
use crate::state::transactions::TRANSACTIONS;
use crate::storage::Storage;
use crate::utils::dates::format_date;
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;
use chrono::Datelike;
use dioxus::prelude::*;
use lucide_dioxus::{FileText, Pencil, Plus, Receipt, Trash};
use rust_decimal::Decimal;
use uuid::Uuid;

mod cards;
mod dialog;
mod filter_sort_bar;
mod notes_dialog;

use cards::{TransactionCard, TransactionGridCard};
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

fn filter_groups_by_query(groups: Vec<MonthGroup>, query: &str) -> Vec<MonthGroup> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return groups;
    }
    groups
        .into_iter()
        .filter_map(|mut group| {
            group.transactions.retain(|txn| {
                txn.name.to_lowercase().contains(&query)
                    || txn
                        .notes
                        .as_deref()
                        .is_some_and(|notes| notes.to_lowercase().contains(&query))
            });
            if group.transactions.is_empty() {
                None
            } else {
                group.total = group.transactions.iter().map(|txn| txn.amount).sum();
                Some(group)
            }
        })
        .collect()
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
    let mut query = use_signal(String::new);
    let view_mode = use_signal(|| PREFERENCES().transactions_view().to_string());
    let mut selection_mode = use_signal(|| false);
    let mut selected = use_signal(std::collections::HashSet::<Uuid>::new);
    let mut bulk_busy = use_signal(|| false);
    let mut bulk_category = use_signal(String::new);

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

    let groups = filter_groups_by_query(
        filter_sort_group(&state.transactions, &filters(), &sort_cfg()),
        &query(),
    );

    let visible_txn_ids: Vec<Uuid> = groups
        .iter()
        .flat_map(|group| group.transactions.iter().map(|txn| txn.id))
        .collect();

    let handle_delete: Callback<Uuid> = use_callback(move |id: Uuid| {
        spawn(async move {
            let _ = Storage::delete_transaction(&id).await;
            match Storage::list_transactions().await {
                Ok(data) => TRANSACTIONS.write().set_data(data),
                Err(e) => TRANSACTIONS.write().set_error(e.to_string()),
            }
        });
    });

    let handle_edit: Callback<Uuid> = use_callback(move |id: Uuid| {
        editing_id.set(Some(id));
        dialog_open.set(true);
    });

    let handle_notes: Callback<Uuid> = use_callback(move |id: Uuid| {
        notes_dialog_txn_id.set(Some(id));
        notes_dialog_open.set(true);
    });

    use_effect(move || {
        let view = view_mode();
        PREFERENCES.write().set_transactions_view(view);
    });

    use_effect(move || {
        if *CREATE_INTENT.read() == Some(CreateIntent::Transaction) {
            *CREATE_INTENT.write() = None;
            editing_id.set(None);
            dialog_open.set(true);
        }
    });

    let bulk_delete: Callback<()> = use_callback(move |_: ()| {
        if selected().is_empty() || bulk_busy() {
            return;
        }
        let ids: Vec<Uuid> = selected().iter().copied().collect();
        bulk_busy.set(true);
        spawn(async move {
            for id in ids {
                let _ = Storage::delete_transaction(&id).await;
            }
            match Storage::list_transactions().await {
                Ok(data) => TRANSACTIONS.write().set_data(data),
                Err(error) => TRANSACTIONS.write().set_error(error.to_string()),
            }
            selected.write().clear();
            selection_mode.set(false);
            bulk_busy.set(false);
        });
    });

    let toggle_selected: Callback<Uuid> = use_callback(move |id: Uuid| {
        if selected.read().contains(&id) {
            selected.write().remove(&id);
        } else {
            selected.write().insert(id);
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
                FilterSortBar { filters: filters, sort_cfg: sort_cfg, query: query, view: view_mode, selection_mode: selection_mode }
            }

            if selection_mode() {
                div { class: "flex flex-wrap items-center gap-2 rounded-lg border border-primary/30 bg-primary/5 px-3 py-2", role: "status", aria_live: "polite",
                    span { class: "text-sm font-medium",
                        if selected().is_empty() { "Select transactions" } else { "{selected().len()} selected" }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-8 items-center rounded-md border border-border/60 bg-background px-2.5 text-xs font-medium hover:bg-muted",
                        onclick: move |_| {
                            let ids: Vec<Uuid> = visible_txn_ids.clone();
                            if selected().len() == ids.len() { selected.write().clear(); }
                            else { for id in ids { selected.write().insert(id); } }
                        },
                        "Select all"
                    }
                    select {
                        class: "h-8 rounded-md border border-border/60 bg-background px-2 text-xs",
                        value: bulk_category(),
                        onchange: move |event| bulk_category.set(event.value()),
                        option { value: "", "Set category…" }
                        for category in CATEGORIES().categories.iter().filter(|category| !category.is_system) {
                            option { value: category.id.to_string(), {category.name.clone()} }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-8 items-center rounded-md border border-border/60 bg-background px-2.5 text-xs font-medium hover:bg-muted disabled:opacity-50",
                        disabled: bulk_busy() || selected().is_empty() || bulk_category().is_empty(),
                        onclick: move |_| {
                            if bulk_busy() || bulk_category().is_empty() { return; }
                            let ids: Vec<Uuid> = selected().iter().copied().collect();
                            let category_id = Uuid::parse_str(&bulk_category()).ok();
                            let Some(category_id) = category_id else { return; };
                            bulk_busy.set(true);
                            spawn(async move {
                                for id in ids {
                                    let req = crate::models::transaction::UpdateTransactionRequest { category_id: Some(category_id), ..Default::default() };
                                    let _ = Storage::update_transaction(&id, &req).await;
                                }
                                match Storage::list_transactions().await {
                                    Ok(data) => TRANSACTIONS.write().set_data(data),
                                    Err(error) => TRANSACTIONS.write().set_error(error.to_string()),
                                }
                                selected.write().clear();
                                bulk_category.set(String::new());
                                bulk_busy.set(false);
                            });
                        },
                        "Apply category"
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-8 items-center gap-1.5 rounded-md border border-destructive/40 bg-destructive/10 px-2.5 text-xs font-medium text-destructive hover:bg-destructive/15 disabled:opacity-50",
                        disabled: bulk_busy() || selected().is_empty(),
                        onclick: move |_| bulk_delete.call(()),
                        Trash { size: 13 }
                        "Delete"
                    }
                    button {
                        r#type: "button",
                        class: "ml-auto inline-flex h-8 items-center rounded-md px-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground",
                        onclick: move |_| { selected.write().clear(); selection_mode.set(false); },
                        "Clear"
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
                        p { class: "text-sm text-muted-foreground", "No transactions match your search or filters" }
                        button {
                            onclick: move |_| {
                                query.set(String::new());
                                filters.set(FilterConfig::default());
                            },
                            class: "text-sm text-primary hover:underline",
                            "Clear search and filters"
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
                                        span { class: "amount-sensitive text-sm font-bold",
                                            {format_currency(group.total, &currency)}
                                        }
                                    }
                                    if view_mode() == "grid" {
                                        div { class: "mt-3 grid gap-3 grid-cols-1 md:grid-cols-2 xl:grid-cols-3",
                                            for txn in &group.transactions {
                                                TransactionGridCard {
                                                    txn: txn.clone(),
                                                    selection_mode: selection_mode(),
                                                    selected: selected().contains(&txn.id),
                                                    on_select: toggle_selected,
                                                    on_edit: handle_edit,
                                                    on_delete: handle_delete,
                                                    on_notes: handle_notes,
                                                }
                                            }
                                        }
                                    } else {
                                        div { class: "mt-2 space-y-2",
                                            for txn in &group.transactions {
                                                TransactionCard {
                                                    txn: txn.clone(),
                                                    selection_mode: selection_mode(),
                                                    selected: selected().contains(&txn.id),
                                                    on_select: toggle_selected,
                                                    on_edit: handle_edit,
                                                    on_delete: handle_delete,
                                                    on_notes: handle_notes,
                                                }
                                            }
                                        }
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
