use chrono::Datelike;
use dioxus::prelude::*;
use lucide_dioxus::{ArrowUpDown, Plus, Trash, Wallet};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::api;
use crate::components::dialog::Dialog;
use crate::components::ConvertedAmount;
use crate::components::logo_search::LogoSearch;
use crate::components::page_header::PageHeader;
use crate::components::ListToolbar;
use crate::components::{AccountCombobox, CategoryCombobox};
use crate::models::debt::{
    CreateDebtRequest, Debt, DebtPayment, RecordDebtPaymentRequest,
};
use crate::state::accounts::ACCOUNTS;
use crate::state::auth::AUTH;
use crate::state::categories::CATEGORIES;
use crate::state::debts::DEBTS;
use crate::state::preferences::PREFERENCES;
use crate::utils::constants::CURRENCIES;
use crate::utils::icons::icon_display;

#[component]
pub fn DebtsPage() -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut editing_id = use_signal(|| None::<Uuid>);
    let mut name = use_signal(String::new);
    let mut total = use_signal(String::new);
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut debt_type = use_signal(|| crate::utils::constants::DEBT_TYPE_MANUAL.to_string());
    let mut monthly_payment = use_signal(String::new);
    let mut payment_days = use_signal(Vec::<u8>::new);
    let mut start_date = use_signal(|| chrono::Utc::now().date_naive().to_string());
    let mut notes = use_signal(String::new);
    let mut logo_url = use_signal(String::new);
    let mut account_id = use_signal(String::new);
    let mut category_id = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let mut pay_open = use_signal(|| false);
    let mut pay_debt = use_signal(|| None::<Debt>);
    let mut pay_amount = use_signal(String::new);
    let mut pay_date = use_signal(|| chrono::Utc::now().date_naive().to_string());
    let mut pay_note = use_signal(String::new);
    let mut pay_error = use_signal(String::new);

    let mut history_open = use_signal(|| false);
    let mut history_debt = use_signal(|| None::<Debt>);
    let mut history_payments = use_signal(Vec::<DebtPayment>::new);

    let mut delete_open = use_signal(|| false);
    let mut delete_debt = use_signal(|| None::<Debt>);
    let mut query = use_signal(String::new);
    let mut type_filter = use_signal(|| None::<String>);
    let mut sort_field = use_signal(|| "name".to_string());
    let mut sort_direction = use_signal(|| "asc".to_string());
    let mut filters_popover = use_signal(|| false);
    let mut sort_popover = use_signal(|| false);
    let mut selection_mode = use_signal(|| false);
    let mut selected = use_signal(std::collections::HashSet::<Uuid>::new);
    let mut bulk_busy = use_signal(|| false);
    let mut bulk_category = use_signal(String::new);

    let load = use_callback(move |_: ()| {
        spawn(async move {
            DEBTS.write().set_loading();
            match api::debts::list().await {
                Ok(debts) => DEBTS.write().set_data(debts),
                Err(err) => DEBTS.write().set_error(err.to_string()),
            }
        });
    });

    use_future(move || async move {
        if DEBTS().debts.is_empty() {
            load.call(());
        }
        if ACCOUNTS().accounts.is_empty()
            && let Ok(accounts) = crate::storage::Storage::list_accounts().await
        {
            ACCOUNTS.write().set_data(accounts);
        }
        if CATEGORIES().categories.is_empty()
            && let Ok(categories) = crate::storage::Storage::list_categories().await
        {
            CATEGORIES.write().set_data(categories);
        }
        if account_id().is_empty()
            && let Some(default) = ACCOUNTS().default_account()
        {
            account_id.set(default.id.to_string());
        }
        if category_id().is_empty()
            && let Some(default) = CATEGORIES().default_category()
        {
            category_id.set(default.id.to_string());
        }
    });

    let open_create = use_callback(move |_: ()| {
        editing_id.set(None);
        name.set(String::new());
        total.set(String::new());
        currency.set(AUTH().currency().to_string());
        debt_type.set(crate::utils::constants::DEBT_TYPE_MANUAL.into());
        monthly_payment.set(String::new());
        payment_days.set(vec![chrono::Utc::now().date_naive().day() as u8]);
        start_date.set(chrono::Utc::now().date_naive().to_string());
        notes.set(String::new());
        logo_url.set(String::new());
        error.set(String::new());
        if account_id().is_empty()
            && let Some(default) = ACCOUNTS().default_account()
        {
            account_id.set(default.id.to_string());
        }
        if category_id().is_empty()
            && let Some(default) = CATEGORIES().default_category()
        {
            category_id.set(default.id.to_string());
        }
        dialog_open.set(true);
    });

    use_effect(move || {
        if *crate::state::create_intent::CREATE_INTENT.read()
            == Some(crate::state::create_intent::CreateIntent::Debt)
        {
            *crate::state::create_intent::CREATE_INTENT.write() = None;
            open_create.call(());
        }
    });

    let open_edit = move |debt: Debt| {
        editing_id.set(Some(debt.id));
        name.set(debt.name.clone());
        total.set(debt.total_amount.to_string());
        currency.set(debt.currency.clone());
        let kind = if debt.debt_type.trim().is_empty() {
            if debt.monthly_payment.is_some() {
                crate::utils::constants::DEBT_TYPE_AUTOMATIC
            } else {
                crate::utils::constants::DEBT_TYPE_MANUAL
            }
        } else {
            debt.debt_type.as_str()
        };
        debt_type.set(kind.to_string());
        monthly_payment.set(
            debt.monthly_payment
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        payment_days.set(debt.payment_days.clone());
        start_date.set(debt.start_date.to_string());
        account_id.set(debt.account_id.to_string());
        category_id.set(debt.category_id.to_string());
        notes.set(debt.notes.clone().unwrap_or_default());
        logo_url.set(debt.logo_url.clone().unwrap_or_default());
        error.set(String::new());
        dialog_open.set(true);
    };

    let save = move |_| {
        let total_amount = Decimal::from_str(&total()).unwrap_or(Decimal::ZERO);
        if name().trim().is_empty() {
            error.set("Name is required".into());
            return;
        }
        if total_amount <= Decimal::ZERO {
            error.set("Total amount must be greater than zero".into());
            return;
        }
        let monthly = if debt_type() == crate::utils::constants::DEBT_TYPE_AUTOMATIC {
            let value = Decimal::from_str(&monthly_payment()).unwrap_or(Decimal::ZERO);
            if value <= Decimal::ZERO {
                error.set("Monthly payment is required for automatic debts".into());
                return;
            }
            Some(value)
        } else {
            None
        };
        let start = start_date()
            .parse::<chrono::NaiveDate>()
            .ok();
        let note_value = if notes().trim().is_empty() { None } else { Some(notes()) };
        let logo_value = if logo_url().trim().is_empty() {
            None
        } else {
            Some(logo_url())
        };
        let edit = editing_id();
        saving.set(true);
        spawn(async move {
            let result = if let Some(id) = edit {
                api::debts::update(
                    &id,
                    &crate::models::debt::UpdateDebtRequest {
                        name: Some(name().trim().to_string()),
                        total_amount: Some(total_amount),
                        currency: Some(currency()),
                        account_id: Some(Uuid::parse_str(&account_id()).unwrap_or(Uuid::nil())),
                        category_id: Uuid::parse_str(&category_id()).ok(),
                        debt_type: Some(debt_type()),
                        monthly_payment: monthly,
                        payment_days: Some(payment_days()),
                        start_date: start,
                        notes: note_value,
                        clear_notes: false,
                        logo_url: logo_value,
                    },
                )
                .await
            } else {
                api::debts::create(&CreateDebtRequest {
                    name: name().trim().to_string(),
                    total_amount,
                    currency: currency(),
                    account_id: Uuid::parse_str(&account_id()).unwrap_or(Uuid::nil()),
                    category_id: Uuid::parse_str(&category_id()).ok(),
                    debt_type: debt_type(),
                    monthly_payment: monthly,
                    payment_days: payment_days(),
                    start_date: start,
                    notes: note_value,
                    logo_url: logo_value,
                })
                .await
            };
            match result {
                Ok(_) => {
                    dialog_open.set(false);
                    load.call(());
                }
                Err(err) => error.set(err.to_string()),
            }
            saving.set(false);
        });
    };

    let open_pay = move |debt: Debt| {
        pay_amount.set(debt.remaining_amount.to_string());
        pay_date.set(chrono::Utc::now().date_naive().to_string());
        pay_note.set(String::new());
        pay_error.set(String::new());
        pay_debt.set(Some(debt));
        pay_open.set(true);
    };

    let submit_payment = move |_| {
        let Some(debt) = pay_debt() else { return };
        let amount = Decimal::from_str(&pay_amount()).unwrap_or(Decimal::ZERO);
        if amount <= Decimal::ZERO || amount > debt.remaining_amount {
            pay_error.set("Enter a valid amount up to the remaining balance".into());
            return;
        }
        let Some(date) = pay_date().parse::<chrono::NaiveDate>().ok() else {
            pay_error.set("Invalid date".into());
            return;
        };
        let req = RecordDebtPaymentRequest {
            amount,
            paid_on: date,
            note: if pay_note().trim().is_empty() {
                None
            } else {
                Some(pay_note())
            },
        };
        spawn(async move {
            match api::debts::record_payment(&debt.id, &req).await {
                Ok(_) => {
                    pay_open.set(false);
                    load.call(());
                }
                Err(err) => pay_error.set(err.to_string()),
            }
        });
    };

    let open_history = move |debt: Debt| {
        history_debt.set(Some(debt.clone()));
        history_payments.set(Vec::new());
        history_open.set(true);
        spawn(async move {
            if let Ok(items) = api::debts::payments(&debt.id).await {
                history_payments.set(items);
            }
        });
    };

    let query_value = query().trim().to_lowercase();
    let filter_value = type_filter();
    let sort_value = sort_field();
    let sort_desc = sort_direction() == "desc";
    let mut visible_debts: Vec<Debt> = DEBTS()
        .debts
        .iter()
        .filter(|debt| {
            let matches_query = query_value.is_empty()
                || debt.name.to_lowercase().contains(&query_value)
                || debt
                    .notes
                    .as_deref()
                    .is_some_and(|notes| notes.to_lowercase().contains(&query_value));
            let matches_type = filter_value
                .as_deref()
                .is_none_or(|kind| debt.debt_type == kind);
            matches_query && matches_type
        })
        .cloned()
        .collect();
    visible_debts.sort_by(|a, b| {
        let order = match sort_value.as_str() {
            "remaining" => a.remaining_amount.cmp(&b.remaining_amount),
            "next" => a.next_payment_date.cmp(&b.next_payment_date),
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        };
        if sort_desc { order.reverse() } else { order }
    });

    let visible_ids: Vec<Uuid> = visible_debts.iter().map(|debt| debt.id).collect();
    let filter_count = usize::from(filter_value.is_some());
    let toolbar_actions = rsx! {
        div { class: "relative",
            button {
                r#type: "button",
                aria_label: "Filters",
                title: "Filters",
                aria_expanded: if filters_popover() { "true" } else { "false" },
                class: if filter_count > 0 { "relative inline-flex h-10 w-10 items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary hover:bg-primary/15" } else { "relative inline-flex h-10 w-10 items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" },
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
                if filter_count > 0 {
                    span { class: "absolute -right-1 -top-1 min-w-4 rounded-full bg-primary px-1 text-[10px] leading-4 text-primary-foreground", "{filter_count}" }
                }
            }
            if filters_popover() {
                div {
                    class: "absolute right-0 top-full z-50 mt-2 w-56 rounded-lg border bg-popover p-3 shadow-md space-y-2",
                    onclick: move |e| e.stop_propagation(),
                    label { class: "text-xs font-medium text-muted-foreground", "Type" }
                    select {
                        class: "h-8 w-full rounded-md border bg-background px-2 text-sm",
                        value: filter_value.clone().unwrap_or_default(),
                        onchange: move |e| {
                            let value = e.value();
                            type_filter.set(if value.is_empty() { None } else { Some(value) });
                        },
                        option { value: "", "All debts" }
                        option { value: crate::utils::constants::DEBT_TYPE_MANUAL, "Manual" }
                        option { value: crate::utils::constants::DEBT_TYPE_AUTOMATIC, "Automatic" }
                    }
                    if filter_count > 0 {
                        button {
                            r#type: "button",
                            class: "text-xs text-muted-foreground hover:text-foreground",
                            onclick: move |_| type_filter.set(None),
                            "Clear filter"
                        }
                    }
                }
                div { class: "fixed inset-0 z-40", onclick: move |_| filters_popover.set(false) }
            }
        }
        div { class: "relative",
            button {
                r#type: "button",
                aria_label: "Sort",
                title: "Sort",
                aria_expanded: if sort_popover() { "true" } else { "false" },
                class: "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground",
                onclick: move |_| sort_popover.set(!sort_popover()),
                ArrowUpDown { size: 16 }
            }
            if sort_popover() {
                div {
                    class: "absolute right-0 top-full z-50 mt-2 w-52 rounded-lg border bg-popover p-3 shadow-md space-y-2",
                    onclick: move |e| e.stop_propagation(),
                    label { class: "text-xs font-medium text-muted-foreground", "Sort by" }
                    select {
                        class: "h-8 w-full rounded-md border bg-background px-2 text-sm",
                        value: sort_field(),
                        onchange: move |e| sort_field.set(e.value()),
                        option { value: "name", "Name" }
                        option { value: "remaining", "Remaining" }
                        option { value: "next", "Next payment" }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-8 w-full items-center justify-between rounded-md border bg-background px-2 text-sm",
                        onclick: move |_| sort_direction.set(if sort_direction() == "asc" { "desc".into() } else { "asc".into() }),
                        span { if sort_direction() == "asc" { "Ascending" } else { "Descending" } }
                        span { if sort_direction() == "asc" { "↑" } else { "↓" } }
                    }
                }
                div { class: "fixed inset-0 z-40", onclick: move |_| sort_popover.set(false) }
            }
        }
        button {
            r#type: "button",
            aria_label: if selection_mode() { "Exit selection mode" } else { "Enter selection mode" },
            aria_pressed: if selection_mode() { "true" } else { "false" },
            class: if selection_mode() { "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-primary/40 bg-primary/10 text-primary" } else { "inline-flex h-10 w-10 items-center justify-center rounded-lg border border-border/60 bg-background text-muted-foreground hover:bg-muted hover:text-foreground" },
            onclick: move |_| selection_mode.toggle(),
            svg {
                view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "2",
                rect { x: "3", y: "3", width: "18", height: "18", rx: "3" }
                if selection_mode() { path { d: "m7 12 3 3 7-7" } }
            }
        }
    };

    let bulk_delete: Callback<()> = use_callback(move |_: ()| {
        if selected().is_empty() || bulk_busy() {
            return;
        }
        let ids: Vec<Uuid> = selected().iter().copied().collect();
        bulk_busy.set(true);
        spawn(async move {
            for id in ids {
                let _ = api::debts::delete(&id).await;
            }
            load.call(());
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

    if let Some(err) = &DEBTS().error {
        return rsx! {
            div { class: "p-4 md:p-6",
                div { class: "rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive", {err.clone()} }
            }
        };
    }

    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6 animate-in fade-in duration-300",
            div { class: "flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3",
                PageHeader { title: "Debts", subtitle: "Money you owe, manual or paid automatically" }
                button {
                    class: "hidden md:inline-flex items-center justify-center gap-2 h-10 px-4 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90",
                    onclick: move |_| open_create.call(()),
                    Plus { size: 16 }
                    "Add Debt"
                }
            }

            if !DEBTS().debts.is_empty() {
                ListToolbar {
                    value: query(),
                    oninput: move |value| query.set(value),
                    view: "grid".to_string(),
                    onview: move |_| {},
                    show_view: false,
                    actions: toolbar_actions,
                }
            }

            if selection_mode() {
                div { class: "flex flex-wrap items-center gap-2 rounded-lg border border-primary/30 bg-primary/5 px-3 py-2", role: "status", aria_live: "polite",
                    span { class: "text-sm font-medium",
                        if selected().is_empty() { "Select debts" } else { "{selected().len()} selected" }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex h-8 items-center rounded-md border border-border/60 bg-background px-2.5 text-xs font-medium hover:bg-muted",
                        onclick: move |_| {
                            if selected().len() == visible_ids.len() { selected.write().clear(); }
                            else { for id in &visible_ids { selected.write().insert(*id); } }
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
                                    let req = crate::models::debt::UpdateDebtRequest { category_id: Some(category_id), ..Default::default() };
                                    let _ = api::debts::update(&id, &req).await;
                                }
                                load.call(());
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

            if DEBTS().is_loading && DEBTS().debts.is_empty() {
                div { class: "flex h-52 items-center justify-center rounded-xl border border-border/40 bg-card/80",
                    p { class: "text-sm text-muted-foreground", "Loading debts…" }
                }
            } else if DEBTS().debts.is_empty() {
                div { class: "rounded-xl border border-border/40 bg-card/80 p-8 flex flex-col items-center gap-3",
                    Wallet { size: 36 }
                    p { class: "text-lg font-medium", "No debts yet" }
                    p { class: "text-sm text-muted-foreground text-center",
                        "Add a manual debt or let an automatic debt pay itself month by month."
                    }
                }
            } else if visible_debts.is_empty() {
                div { class: "rounded-xl border border-border/40 bg-card/80 p-8 flex flex-col items-center gap-3",
                    Wallet { size: 36 }
                    p { class: "text-lg font-medium", "No debts match your search" }
                    button {
                        r#type: "button",
                        class: "text-sm text-primary hover:underline",
                        onclick: move |_| {
                            query.set(String::new());
                            type_filter.set(None);
                        },
                        "Clear search and filters"
                    }
                }
            } else {
                div { class: "grid gap-4 grid-cols-1 lg:grid-cols-2",
                    for debt in visible_debts.iter().cloned() {
                        DebtCard {
                            debt: debt.clone(),
                            selection_mode: selection_mode(),
                            selected: selected().contains(&debt.id),
                            on_select: toggle_selected,
                            on_pay: open_pay,
                            on_history: open_history,
                            on_edit: open_edit,
                            on_delete: move |debt| {
                                delete_debt.set(Some(debt));
                                delete_open.set(true);
                            },
                        }
                    }
                }
            }

            Dialog {
                open: dialog_open(),
                on_close: move |_| dialog_open.set(false),
                title: if editing_id().is_some() { "Edit Debt".to_string() } else { "Add Debt".to_string() },
                div { class: "space-y-4",
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Name " span { class: "text-destructive", "*" } }
                        input {
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                            value: name(),
                            oninput: move |e| name.set(e.value()),
                            placeholder: "Mortgage, car loan…",
                        }
                    }
                    div { class: "grid grid-cols-2 gap-3",
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Total owed " span { class: "text-destructive", "*" } }
                            input {
                                r#type: "number",
                                step: "0.01",
                                class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                                value: total(),
                                oninput: move |e| total.set(e.value()),
                            }
                        }
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Currency " span { class: "text-destructive", "*" } }
                            select {
                                class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                                value: currency(),
                                onchange: move |e| currency.set(e.value()),
                                for (code, label) in CURRENCIES {
                                    option { value: code.to_string(), {label.to_string()} }
                                }
                            }
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Type" }
                        select {
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                            value: debt_type(),
                            onchange: move |e| {
                                let value = e.value();
                                if value == crate::utils::constants::DEBT_TYPE_AUTOMATIC
                                    && payment_days().is_empty()
                                {
                                    payment_days.set(vec![start_date()
                                        .parse::<chrono::NaiveDate>()
                                        .map(|date| chrono::Datelike::day(&date) as u8)
                                        .unwrap_or(1)]);
                                }
                                debt_type.set(value);
                            },
                            option {
                                value: crate::utils::constants::DEBT_TYPE_MANUAL,
                                selected: debt_type() == crate::utils::constants::DEBT_TYPE_MANUAL,
                                "Manual — I record each payment"
                            }
                            option {
                                value: crate::utils::constants::DEBT_TYPE_AUTOMATIC,
                                selected: debt_type() == crate::utils::constants::DEBT_TYPE_AUTOMATIC,
                                "Automatic — paid month by month"
                            }
                        }
                    }
                    if debt_type() == crate::utils::constants::DEBT_TYPE_AUTOMATIC {
                        div { class: "grid grid-cols-2 gap-3",
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "Monthly payment " span { class: "text-destructive", "*" } }
                                input {
                                    r#type: "number",
                                    step: "0.01",
                                    class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                                    value: monthly_payment(),
                                    oninput: move |e| monthly_payment.set(e.value()),
                                }
                            }
                            div { class: "space-y-1.5",
                                label { class: "text-sm font-medium", "First payment" }
                                input {
                                    r#type: "date",
                                    class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                                    value: start_date(),
                                    oninput: move |e| start_date.set(e.value()),
                                }
                            }
                        }
                        div { class: "space-y-1.5",
                            label { class: "text-sm font-medium", "Charge days" }
                            div { class: "flex flex-wrap gap-1",
                                for day in 1..=31u8 {
                                    {
                                        let selected = payment_days().contains(&day);
                                        rsx! {
                                            button {
                                                r#type: "button",
                                                class: if selected {
                                                    "h-8 w-8 rounded-md bg-primary text-xs font-semibold text-primary-foreground"
                                                } else {
                                                    "h-8 w-8 rounded-md border border-border/60 text-xs font-medium text-muted-foreground hover:bg-accent"
                                                },
                                                onclick: move |_| {
                                                    let mut days = payment_days();
                                                    if days.contains(&day) {
                                                        days.retain(|value| *value != day);
                                                    } else {
                                                        days.push(day);
                                                    }
                                                    days.sort_unstable();
                                                    payment_days.set(days);
                                                },
                                                {day.to_string()}
                                            }
                                        }
                                    }
                                }
                                if payment_days().iter().any(|day| *day >= 29)
                                    || payment_days().contains(&0)
                                {
                                    button {
                                        r#type: "button",
                                        class: if payment_days().contains(&0) {
                                            "h-8 rounded-md bg-primary px-2 text-xs font-semibold text-primary-foreground"
                                        } else {
                                            "h-8 rounded-md border border-border/60 px-2 text-xs font-medium text-muted-foreground hover:bg-accent"
                                        },
                                        onclick: move |_| {
                                            let mut days = payment_days();
                                            if days.contains(&0) {
                                                days.retain(|value| *value != 0);
                                            } else {
                                                days.retain(|value| *value < 29);
                                                days.push(0);
                                            }
                                            days.sort_unstable();
                                            payment_days.set(days);
                                        },
                                        "Last day"
                                    }
                                }
                            }
                            p { class: "text-xs text-muted-foreground",
                                "Each selected day is a separate payment. Days 29-31 can use “Last day”."
                            }
                        }
                        p { class: "text-xs text-muted-foreground",
                            "This also creates a monthly subscription for the payment."
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Account " span { class: "text-destructive", "*" } }
                        AccountCombobox {
                            selected_id: account_id(),
                            on_change: move |id| account_id.set(id),
                            error: None,
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Category" }
                        CategoryCombobox {
                            selected_id: category_id(),
                            on_change: move |id| category_id.set(id),
                            error: None,
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Notes" }
                        textarea {
                            class: "flex min-h-20 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                            value: notes(),
                            oninput: move |e| notes.set(e.value()),
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Logo" }
                        LogoSearch {
                            selected_url: logo_url(),
                            default_query: name(),
                            on_select: move |url| logo_url.set(url),
                        }
                    }
                    if !error().is_empty() {
                        p { class: "text-sm text-destructive", {error()} }
                    }
                    div { class: "flex justify-end gap-2 pt-2",
                        button {
                            class: "h-10 px-4 rounded-md border text-sm",
                            onclick: move |_| dialog_open.set(false),
                            "Cancel"
                        }
                        button {
                            class: "h-10 px-4 rounded-md bg-primary text-primary-foreground text-sm disabled:opacity-50",
                            disabled: saving(),
                            onclick: save,
                            if saving() { "Saving…" } else if editing_id().is_some() { "Save changes" } else { "Add Debt" }
                        }
                    }
                }
            }

            Dialog {
                open: pay_open(),
                on_close: move |_| pay_open.set(false),
                title: "Record payment",
                div { class: "space-y-4",
                    if let Some(debt) = pay_debt() {
                        p { class: "text-sm text-muted-foreground",
                            "Remaining: "
                            ConvertedAmount { amount: debt.remaining_amount, currency: debt.currency.clone() }
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Amount" }
                        input {
                            r#type: "number",
                            step: "0.01",
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                            value: pay_amount(),
                            oninput: move |e| pay_amount.set(e.value()),
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Date" }
                        input {
                            r#type: "date",
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                            value: pay_date(),
                            oninput: move |e| pay_date.set(e.value()),
                        }
                    }
                    div { class: "space-y-1.5",
                        label { class: "text-sm font-medium", "Note" }
                        input {
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 text-sm",
                            value: pay_note(),
                            oninput: move |e| pay_note.set(e.value()),
                        }
                    }
                    if !pay_error().is_empty() {
                        p { class: "text-sm text-destructive", {pay_error()} }
                    }
                    div { class: "flex justify-end gap-2",
                        button {
                            class: "h-10 px-4 rounded-md border text-sm",
                            onclick: move |_| pay_open.set(false),
                            "Cancel"
                        }
                        button {
                            class: "h-10 px-4 rounded-md bg-primary text-primary-foreground text-sm",
                            onclick: submit_payment,
                            "Record payment"
                        }
                    }
                }
            }

            Dialog {
                open: history_open(),
                on_close: move |_| history_open.set(false),
                title: "Payment history",
                div { class: "space-y-2 max-h-80 overflow-y-auto",
                    if history_payments().is_empty() {
                        p { class: "text-sm text-muted-foreground", "No payments yet." }
                    } else {
                        for (index, payment) in history_payments().iter().enumerate() {
                            {let count = history_payments().len();
                             let number = count.saturating_sub(index);
                             rsx! {
                            div { class: "flex items-center justify-between rounded-md border border-border/50 p-2 text-sm",
                                div {
                                    p { "Payment {number}/{count} · {payment.paid_on}" }
                                    if let Some(note) = payment.note.clone() {
                                        p { class: "text-xs text-muted-foreground", {note} }
                                    }
                                }
                                p { class: "font-medium",
                                    ConvertedAmount {
                                        amount: payment.amount,
                                        currency: history_debt().map(|d| d.currency).unwrap_or_else(|| "EUR".to_string()),
                                    }
                                }
                            }
                             }}
                        }
                    }
                }
            }

            Dialog {
                open: delete_open(),
                on_close: move |_| delete_open.set(false),
                title: "Delete debt",
                div { class: "space-y-4",
                    if let Some(debt) = delete_debt() {
                        p { class: "text-sm",
                            "Delete “{debt.name}”? Its payment history will be removed. Automatic debts also remove their linked subscription."
                        }
                    }
                    div { class: "flex justify-end gap-2",
                        button {
                            class: "h-10 px-4 rounded-md border text-sm",
                            onclick: move |_| delete_open.set(false),
                            "Cancel"
                        }
                        button {
                            class: "h-10 px-4 rounded-md bg-destructive text-destructive-foreground text-sm",
                            onclick: move |_| {
                                let Some(debt) = delete_debt() else { return };
                                spawn(async move {
                                    let _ = api::debts::delete(&debt.id).await;
                                    delete_open.set(false);
                                    load.call(());
                                });
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DebtCard(
    debt: Debt,
    selection_mode: bool,
    selected: bool,
    on_select: Callback<Uuid>,
    on_pay: EventHandler<Debt>,
    on_history: EventHandler<Debt>,
    on_edit: EventHandler<Debt>,
    on_delete: EventHandler<Debt>,
) -> Element {
    let account = ACCOUNTS().get_by_id(&debt.account_id).cloned();
    let category = CATEGORIES().get_by_id(&debt.category_id).cloned();
    let date_format = PREFERENCES().date_format();
    let debt_for_pay = debt.clone();
    let debt_for_history = debt.clone();
    let debt_for_delete = debt.clone();
    let debt_for_edit = debt.clone();
    let debt_for_checkbox_id = debt.id;

    rsx! {
        div { class: "rounded-xl border border-border/50 bg-card p-4 flex flex-col gap-3",
            div { class: "flex items-start gap-3",
                if selection_mode {
                    input {
                        r#type: "checkbox",
                        checked: selected,
                        aria_label: "Select debt",
                        class: "mt-1 h-4 w-4 shrink-0 cursor-pointer accent-primary",
                        onchange: move |_| on_select.call(debt_for_checkbox_id),
                    }
                }
                div { class: "flex h-11 w-11 shrink-0 items-center justify-center rounded-xl overflow-hidden",
                    if let Some(logo) = debt.logo_url.clone() {
                        img { src: logo, alt: debt.name.clone(), class: "h-11 w-11 object-contain" }
                    } else {
                        Wallet { size: 22 }
                    }
                }
                div { class: "min-w-0 flex-1",
                    div { class: "flex items-center gap-2",
                        p { class: "font-semibold truncate", {debt.name.clone()} }
                        span { class: if debt.is_automatic() {
                            "rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary"
                        } else {
                            "rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground"
                        }, if debt.is_automatic() { "Automatic" } else { "Manual" } }
                    }
                    p { class: "text-xs text-muted-foreground",
                        if debt.is_active { "Remaining" } else { "Paid" }
                    }
                }
                p { class: "font-bold text-lg shrink-0",
                    ConvertedAmount { amount: debt.remaining_amount, currency: debt.currency.clone() }
                }
            }
            div { class: "h-2 overflow-hidden rounded-full bg-muted",
                div { class: "h-full rounded-full bg-primary transition-all", style: "width: {debt.progress()}%" }
            }
            div { class: "flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground",
                span { "of " ConvertedAmount { amount: debt.total_amount, currency: debt.currency.clone() } }
                if debt.is_automatic() {
                    if let Some(payment) = debt.monthly_payment {
                        span { "Monthly " ConvertedAmount { amount: payment, currency: debt.currency.clone() } }
                    }
                    if let Some(next) = debt.next_payment_date {
                        span { "Next {crate::utils::dates::format_date(&next, &date_format)}" }
                    }
                    if let Some((current, total)) = debt.installment() {
                        span { class: "rounded-full bg-primary/10 px-2 py-0.5 font-medium text-primary",
                            "Payment {current}/{total}"
                        }
                    }
                }
            }
            div { class: "flex flex-wrap items-center gap-1.5",
                if let Some(account) = account {
                    span {
                        class: "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px]",
                        style: "color: {account.color.clone().unwrap_or_else(|| \"#6b7280\".into())}; background-color: {account.color.clone().unwrap_or_else(|| \"#6b7280\".into())}15;",
                        {icon_display(account.icon.as_deref().unwrap_or("Wallet"))}
                        {account.name}
                    }
                }
                if let Some(category) = category.filter(|c| !c.is_system) {
                    {let color = category.color.clone().unwrap_or_else(|| "#6b7280".into());
                     rsx! {
                        span { class: "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px]",
                            style: "color: {color}; background-color: {color}15;",
                            {icon_display(category.icon.as_deref().unwrap_or("Tag"))}
                            {category.name}
                        }
                     }}
                }
            }
            div { class: "flex items-center justify-end gap-1 border-t border-border/40 pt-2",
                if debt.is_active {
                    button {
                        class: "h-8 rounded-md border border-border/60 px-3 text-xs font-medium hover:bg-accent",
                        onclick: move |_| on_pay.call(debt_for_pay.clone()),
                        "Pay"
                    }
                }
                button {
                    class: "h-8 rounded-md border border-border/60 px-3 text-xs font-medium hover:bg-accent",
                    onclick: move |_| on_edit.call(debt_for_edit.clone()),
                    "Edit"
                }
                button {
                    class: "h-8 rounded-md border border-border/60 px-3 text-xs font-medium hover:bg-accent",
                    onclick: move |_| on_history.call(debt_for_history.clone()),
                    "History"
                }
                button {
                    class: "h-8 w-8 rounded-md text-destructive hover:bg-destructive/10 flex items-center justify-center",
                    aria_label: "Delete debt",
                    onclick: move |_| on_delete.call(debt_for_delete.clone()),
                    Trash { size: 14 }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn debts_toolbar_has_search_filters_and_sort_but_no_view_toggle() {
        let src = include_str!("debts_page.rs");
        assert!(src.contains("ListToolbar"));
        assert!(src.contains("aria_label: \"Filters\""));
        assert!(src.contains("aria_label: \"Sort\""));
        assert!(src.contains("show_view: false"));
        assert!(src.contains("visible_debts"));
    }
}
