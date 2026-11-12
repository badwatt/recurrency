use dioxus::prelude::*;
use lucide_dioxus::{Plus, Trash2, Wallet};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::api;
use crate::components::dialog::Dialog;
use crate::components::logo_search::LogoSearch;
use crate::components::page_header::PageHeader;
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
use crate::utils::format::format_currency;
use crate::utils::icons::icon_display;

#[component]
pub fn DebtsPage() -> Element {
    let mut dialog_open = use_signal(|| false);
    let mut name = use_signal(String::new);
    let mut total = use_signal(String::new);
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut debt_type = use_signal(|| "manual".to_string());
    let mut monthly_payment = use_signal(String::new);
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

    let open_create = move |_| {
        name.set(String::new());
        total.set(String::new());
        currency.set(AUTH().currency().to_string());
        debt_type.set("manual".into());
        monthly_payment.set(String::new());
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
        let monthly = if debt_type() == "automatic" {
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
        let req = CreateDebtRequest {
            name: name().trim().to_string(),
            total_amount,
            currency: currency(),
            account_id: Uuid::parse_str(&account_id()).unwrap_or(Uuid::nil()),
            category_id: Uuid::parse_str(&category_id()).ok(),
            debt_type: debt_type(),
            monthly_payment: monthly,
            start_date: start,
            notes: if notes().trim().is_empty() { None } else { Some(notes()) },
            logo_url: if logo_url().trim().is_empty() {
                None
            } else {
                Some(logo_url())
            },
        };
        saving.set(true);
        spawn(async move {
            match api::debts::create(&req).await {
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
                    class: "inline-flex items-center justify-center gap-2 h-10 px-4 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90",
                    onclick: open_create,
                    Plus { size: 16 }
                    "Add Debt"
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
            } else {
                div { class: "grid gap-4 grid-cols-1 lg:grid-cols-2",
                    for debt in DEBTS().debts.iter().cloned() {
                        DebtCard {
                            debt: debt.clone(),
                            on_pay: open_pay,
                            on_history: open_history,
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
                title: "Add Debt",
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
                            onchange: move |e| debt_type.set(e.value()),
                            option { value: "manual", "Manual — I record each payment" }
                            option { value: "automatic", "Automatic — paid month by month" }
                        }
                    }
                    if debt_type() == "automatic" {
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
                            if saving() { "Saving…" } else { "Add Debt" }
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
                            "Remaining: {format_currency(debt.remaining_amount, &debt.currency)}"
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
                        for payment in history_payments() {
                            div { class: "flex items-center justify-between rounded-md border border-border/50 p-2 text-sm",
                                div {
                                    p { {payment.paid_on.to_string()} }
                                    if let Some(note) = payment.note {
                                        p { class: "text-xs text-muted-foreground", {note} }
                                    }
                                }
                                p { class: "font-medium",
                                    {format_currency(payment.amount, history_debt().map(|d| d.currency).as_deref().unwrap_or("EUR"))}
                                }
                            }
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
    on_pay: EventHandler<Debt>,
    on_history: EventHandler<Debt>,
    on_delete: EventHandler<Debt>,
) -> Element {
    let currency = debt.currency.clone();
    let remaining = format_currency(debt.remaining_amount, &currency);
    let total = format_currency(debt.total_amount, &currency);
    let account = ACCOUNTS().get_by_id(&debt.account_id).cloned();
    let category = CATEGORIES().get_by_id(&debt.category_id).cloned();
    let date_format = PREFERENCES().date_format();
    let debt_for_pay = debt.clone();
    let debt_for_history = debt.clone();
    let debt_for_delete = debt.clone();

    rsx! {
        div { class: "rounded-xl border border-border/50 bg-card p-4 flex flex-col gap-3",
            div { class: "flex items-start gap-3",
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
                p { class: "font-bold text-lg shrink-0", {remaining} }
            }
            div { class: "h-2 overflow-hidden rounded-full bg-muted",
                div { class: "h-full rounded-full bg-primary transition-all", style: "width: {debt.progress()}%" }
            }
            div { class: "flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground",
                span { "of {total}" }
                if debt.is_automatic() {
                    if let Some(payment) = debt.monthly_payment {
                        span { "Monthly {format_currency(payment, &currency)}" }
                    }
                    if let Some(next) = debt.next_payment_date {
                        span { "Next {crate::utils::dates::format_date(&next, &date_format)}" }
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
                    span { class: "inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-[11px]",
                        {icon_display(category.icon.as_deref().unwrap_or("Tag"))}
                        {category.name}
                    }
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
                    onclick: move |_| on_history.call(debt_for_history.clone()),
                    "History"
                }
                button {
                    class: "h-8 w-8 rounded-md text-destructive hover:bg-destructive/10 flex items-center justify-center",
                    aria_label: "Delete debt",
                    onclick: move |_| on_delete.call(debt_for_delete.clone()),
                    Trash2 { size: 14 }
                }
            }
        }
    }
}
