//! Unified money page: subscriptions, budgets, one-off payments and debts.

use nsh_base::{button, theme};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::api;
use crate::auth::AuthSession;
use crate::components::card::card;
use crate::components::converted_amount::converted_amount;
use crate::components::empty_state::empty_state;
use crate::components::page_header::page_header;
use crate::components::search_input::search_input;
use crate::format::format_date;
use crate::ui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Subscription,
    Budget,
    Transaction,
    Debt,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Self::Subscription => "Subscription",
            Self::Budget => "Budget",
            Self::Transaction => "One-off payment",
            Self::Debt => "Debt",
        }
    }
}

thread_local! {
    static MONEY_EDIT: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(Kind, Uuid)>>> =
        const { std::cell::RefCell::new(None) };
    static MONEY_DELETE: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(Kind, Uuid)>>> =
        const { std::cell::RefCell::new(None) };
    static MONEY_DUPLICATE: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(Kind, Uuid)>>> =
        const { std::cell::RefCell::new(None) };
    static MONEY_NOTES: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(Kind, Uuid)>>> =
        const { std::cell::RefCell::new(None) };
    static MONEY_PAY: std::cell::RefCell<Option<std::rc::Rc<dyn Fn(Kind, Uuid)>>> =
        const { std::cell::RefCell::new(None) };
    static MONEY_SELECTED: std::cell::RefCell<Option<Signal<Vec<(Kind, Uuid)>>>> =
        const { std::cell::RefCell::new(None) };
    static MONEY_BULK: std::cell::RefCell<Option<Signal<bool>>> = const { std::cell::RefCell::new(None) };
}

fn set_money_actions(
    edit: std::rc::Rc<dyn Fn(Kind, Uuid)>,
    delete: std::rc::Rc<dyn Fn(Kind, Uuid)>,
    duplicate: std::rc::Rc<dyn Fn(Kind, Uuid)>,
    notes: std::rc::Rc<dyn Fn(Kind, Uuid)>,
    pay: std::rc::Rc<dyn Fn(Kind, Uuid)>,
    selected: Signal<Vec<(Kind, Uuid)>>,
    bulk: Signal<bool>,
) {
    MONEY_EDIT.with(|slot| *slot.borrow_mut() = Some(edit));
    MONEY_DELETE.with(|slot| *slot.borrow_mut() = Some(delete));
    MONEY_DUPLICATE.with(|slot| *slot.borrow_mut() = Some(duplicate));
    MONEY_NOTES.with(|slot| *slot.borrow_mut() = Some(notes));
    MONEY_PAY.with(|slot| *slot.borrow_mut() = Some(pay));
    MONEY_SELECTED.with(|slot| *slot.borrow_mut() = Some(selected));
    MONEY_BULK.with(|slot| *slot.borrow_mut() = Some(bulk));
}

fn money_notes(kind: Kind, id: Uuid) {
    if money_bulk_active() {
        money_toggle(kind, id);
        return;
    }
    MONEY_NOTES.with(|slot| {
        if let Some(action) = slot.borrow().as_ref() {
            action(kind, id);
        }
    });
}

fn money_pay(kind: Kind, id: Uuid) {
    if money_bulk_active() {
        money_toggle(kind, id);
        return;
    }
    MONEY_PAY.with(|slot| {
        if let Some(action) = slot.borrow().as_ref() {
            action(kind, id);
        }
    });
}

fn money_bulk_active() -> bool {
    MONEY_BULK.with(|slot| slot.borrow().as_ref().is_some_and(|bulk| bulk.get()))
}

fn money_is_selected(kind: Kind, id: Uuid) -> bool {
    MONEY_SELECTED.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|selected| selected.get().contains(&(kind, id)))
    })
}

fn money_toggle(kind: Kind, id: Uuid) {
    MONEY_SELECTED.with(|slot| {
        if let Some(selected) = *slot.borrow() {
            let mut values = selected.get();
            if let Some(position) = values.iter().position(|value| *value == (kind, id)) {
                values.remove(position);
            } else {
                values.push((kind, id));
            }
            selected.set(values);
        }
    });
}

fn money_edit(kind: Kind, id: Uuid) {
    if money_bulk_active() {
        money_toggle(kind, id);
        return;
    }
    MONEY_EDIT.with(|slot| {
        if let Some(action) = slot.borrow().as_ref() {
            action(kind, id);
        }
    });
}

fn money_duplicate(kind: Kind, id: Uuid) {
    if money_bulk_active() {
        money_toggle(kind, id);
        return;
    }
    MONEY_DUPLICATE.with(|slot| {
        if let Some(action) = slot.borrow().as_ref() {
            action(kind, id);
        }
    });
}

fn money_delete(kind: Kind, id: Uuid) {
    if money_bulk_active() {
        money_toggle(kind, id);
        return;
    }
    MONEY_DELETE.with(|slot| {
        if let Some(action) = slot.borrow().as_ref() {
            action(kind, id);
        }
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Subscription,
    Budget,
    Transaction,
    Debt,
}

impl Filter {
    fn accepts(self, kind: Kind) -> bool {
        match self {
            Self::All => true,
            Self::Subscription => kind == Kind::Subscription,
            Self::Budget => kind == Kind::Budget,
            Self::Transaction => kind == Kind::Transaction,
            Self::Debt => kind == Kind::Debt,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Subscription => "Subscriptions",
            Self::Budget => "Budgets",
            Self::Transaction => "One-off",
            Self::Debt => "Debts",
        }
    }
}

#[derive(Clone, PartialEq)]
struct Item {
    id: Uuid,
    kind: Kind,
    name: String,
    amount: Decimal,
    currency: String,
    logo: Option<String>,
    color: String,
    account: String,
    category: String,
    detail: String,
    progress: Option<f32>,
    updated: i64,
}

pub fn money(auth: AuthSession, modal: ui::ModalHost, compact: Signal<bool>) -> AnyElement {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let server_signal = signal(server.clone());
    let token_signal = signal(token.clone());
    let subscriptions = resource(|| Some(ui::data_revision()), {
        let server = server.clone();
        let token = token.clone();
        let currency = currency.clone();
        move |_, _| {
            let server = server.clone();
            let token = token.clone();
            let currency = currency.clone();
            async move { api::load_subscriptions(server, token, currency).await }
        }
    });
    let transactions = resource(|| Some(ui::data_revision()), {
        let server = server.clone();
        let token = token.clone();
        let currency = currency.clone();
        move |_, _| {
            let server = server.clone();
            let token = token.clone();
            let currency = currency.clone();
            async move { api::load_transactions(server, token, currency).await }
        }
    });
    let debts = resource(|| Some(ui::data_revision()), {
        let server = server.clone();
        let token = token.clone();
        let currency = currency.clone();
        move |_, _| {
            let server = server.clone();
            let token = token.clone();
            let currency = currency.clone();
            async move { api::load_debts(server, token, currency).await }
        }
    });
    let filter = signal(Filter::All);
    let search = signal(String::new());
    let list_view = signal(crate::view_state::money_list());
    effect(move || crate::view_state::save_money_list(list_view.get()));
    let bulk = signal(false);
    let selected = signal(Vec::<(Kind, Uuid)>::new());
    let loading = move || {
        (subscriptions.loading() && subscriptions.get().is_none())
            || (transactions.loading() && transactions.get().is_none())
            || (debts.loading() && debts.get().is_none())
    };
    let error = move || {
        subscriptions
            .error()
            .or_else(|| transactions.error())
            .or_else(|| debts.error())
    };
    let items = move || {
        build_items(
            subscriptions,
            transactions,
            debts,
            search.get(),
            filter.get(),
        )
    };
    let open_add = {
        let auth = auth.clone();
        move || open_money_add(auth.clone(), modal)
    };
    let open_filtered = move || match filter.get() {
        Filter::Budget => crate::ui::create_budget(None),
        Filter::Transaction => crate::ui::create_transaction(None),
        Filter::Debt => crate::ui::create_debt(None),
        Filter::All | Filter::Subscription => crate::ui::create_subscription(None),
    };
    let edit_action = {
        let edit_currency = currency.clone();
        move |kind: Kind, id: Uuid| match kind {
            Kind::Subscription | Kind::Budget => {
                if let Some(subscription) = subscriptions
                    .get()
                    .unwrap_or_default()
                    .subscriptions
                    .into_iter()
                    .find(|item| item.id == id)
                {
                    crate::subscriptions::open_edit_dialog(
                        subscriptions,
                        subscription,
                        server_signal,
                        token_signal,
                        modal,
                    );
                }
            }
            Kind::Transaction => {
                if let Some(transaction) = transactions
                    .get()
                    .unwrap_or_default()
                    .transactions
                    .into_iter()
                    .find(|item| item.id == id)
                {
                    crate::transactions::open_edit_dialog(
                        transactions,
                        transaction,
                        server_signal,
                        token_signal,
                        edit_currency.clone(),
                        modal,
                    );
                }
            }
            Kind::Debt => {
                if let Some(debt) = debts
                    .get()
                    .unwrap_or_default()
                    .debts
                    .into_iter()
                    .find(|item| item.id == id)
                {
                    crate::debts::open_edit_dialog(
                        debts,
                        debt,
                        server_signal,
                        token_signal,
                        edit_currency.clone(),
                        modal,
                    );
                }
            }
        }
    };
    let edit_action: std::rc::Rc<dyn Fn(Kind, Uuid)> = std::rc::Rc::new(edit_action);
    let duplicate_item = {
        let server = server.clone();
        let token = token.clone();
        let duplicate_currency = currency.clone();
        move |kind: Kind, id: Uuid| {
            let server = server.clone();
            let token = token.clone();
            let currency = duplicate_currency.clone();
            let subscriptions = subscriptions;
            let transactions = transactions;
            let debts = debts;
            spawn(async move {
                let result = match kind {
                    Kind::Subscription | Kind::Budget => {
                        let Some(item) = subscriptions
                            .get()
                            .unwrap_or_default()
                            .subscriptions
                            .into_iter()
                            .find(|item| item.id == id)
                        else {
                            return;
                        };
                        let frequency = match item.recurrence.unit.as_str() {
                            "day" => "daily",
                            "week" => "weekly",
                            "year" => "yearly",
                            _ => "monthly",
                        };
                        api::create_subscription(
                            server,
                            token,
                            &crate::subscriptions::CreateSubscriptionRequest {
                                name: format!("{} (copy)", item.name),
                                amount: item.amount,
                                currency: item.currency,
                                frequency: frequency.to_string(),
                                start_date: item.start_date,
                                kind: item.kind,
                                first_payment: item.next_billing_date,
                                end_date: item.end_date,
                                clear_first_payment: false,
                                clear_end_date: false,
                                account_id: item.account_id,
                                category_id: Some(item.category_id),
                                recurrence: item.recurrence,
                                is_active: item.is_active,
                                notes: item.notes,
                                logo_url: item.logo_url,
                                clear_logo: false,
                            },
                        )
                        .await
                        .map(|_| ())
                    }
                    Kind::Transaction => {
                        let Some(item) = transactions
                            .get()
                            .unwrap_or_default()
                            .transactions
                            .into_iter()
                            .find(|item| item.id == id)
                        else {
                            return;
                        };
                        api::create_transaction(
                            server,
                            token,
                            &crate::transactions::CreateTransactionRequest {
                                name: format!("{} (copy)", item.name),
                                amount: item.amount,
                                currency: item.currency,
                                date: item.date,
                                account_id: item.account_id,
                                category_id: Some(item.category_id),
                                notes: item.notes,
                                logo_url: item.logo_url,
                            },
                        )
                        .await
                        .map(|_| ())
                    }
                    Kind::Debt => {
                        let Some(item) = debts
                            .get()
                            .unwrap_or_default()
                            .debts
                            .into_iter()
                            .find(|item| item.id == id)
                        else {
                            return;
                        };
                        api::create_debt(
                            server,
                            token,
                            &crate::debts::CreateDebtRequest {
                                name: format!("{} (copy)", item.name),
                                total_amount: item.total_amount,
                                currency: item.currency,
                                account_id: item.account_id,
                                category_id: Some(item.category_id),
                                debt_type: item.debt_type,
                                monthly_payment: item.monthly_payment,
                                payment_days: item.payment_days,
                                recurrence: Some(item.recurrence),
                                first_payment: item.next_payment_date,
                                clear_first_payment: false,
                                start_date: Some(item.start_date),
                                notes: item.notes,
                                logo_url: item.logo_url,
                                clear_logo: false,
                            },
                        )
                        .await
                        .map(|_| ())
                    }
                };
                match result {
                    Ok(()) => {
                        crate::components::toast::success("Duplicated.");
                        subscriptions.refetch();
                        transactions.refetch();
                        debts.refetch();
                        ui::bump_data();
                    }
                    Err(message) => crate::components::toast::error(message),
                }
                let _ = currency;
            });
        }
    };
    let duplicate_item: std::rc::Rc<dyn Fn(Kind, Uuid)> = std::rc::Rc::new(duplicate_item);

    let delete_item = {
        let server = server.clone();
        let token = token.clone();
        move |kind: Kind, id: Uuid| {
            let server = server.clone();
            let token = token.clone();
            modal.open(move || {
                let close = move || modal.close();
                let server = server.clone();
                let token = token.clone();
                crate::components::dialog::dialog(
                    crate::components::dialog::DialogProps::new(
                        "Delete this item?",
                        "This cannot be undone.",
                        close,
                    )
                    .width(420.0)
                    .footer(
                        view()
                            .w_full()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(crate::ui::action_button("Cancel", false, move || {
                                modal.close()
                            }))
                            .child(crate::ui::action_button("Delete", true, move || {
                                let server = server.clone();
                                let token = token.clone();
                                spawn(async move {
                                    let result = match kind {
                                        Kind::Subscription | Kind::Budget => {
                                            api::delete_subscription(server, token, id).await
                                        }
                                        Kind::Transaction => {
                                            api::delete_transaction(server, token, id).await
                                        }
                                        Kind::Debt => api::delete_debt(server, token, id).await,
                                    };
                                    match result {
                                        Ok(()) => {
                                            crate::components::toast::success("Deleted.");
                                            subscriptions.refetch();
                                            transactions.refetch();
                                            debts.refetch();
                                            ui::bump_data();
                                        }
                                        Err(message) => crate::components::toast::error(message),
                                    }
                                    modal.close();
                                });
                            }))
                            .into_element(),
                    ),
                )
                .into_element()
            });
        }
    };
    let delete_item: std::rc::Rc<dyn Fn(Kind, Uuid)> = std::rc::Rc::new(delete_item);
    let notes_item = {
        let server = server.clone();
        let token = token.clone();
        move |kind: Kind, id: Uuid| {
            let current = match kind {
                Kind::Subscription | Kind::Budget => subscriptions
                    .get()
                    .unwrap_or_default()
                    .subscriptions
                    .into_iter()
                    .find(|item| item.id == id)
                    .and_then(|item| item.notes)
                    .unwrap_or_default(),
                Kind::Transaction => transactions
                    .get()
                    .unwrap_or_default()
                    .transactions
                    .into_iter()
                    .find(|item| item.id == id)
                    .and_then(|item| item.notes)
                    .unwrap_or_default(),
                Kind::Debt => debts
                    .get()
                    .unwrap_or_default()
                    .debts
                    .into_iter()
                    .find(|item| item.id == id)
                    .and_then(|item| item.notes)
                    .unwrap_or_default(),
            };
            let value = signal(current);
            let server = server.clone();
            let token = token.clone();
            modal.open(move || {
                let save_value = value;
                let save_kind = kind;
                let save_id = id;
                let save_server = server.clone();
                let save_token = token.clone();
                crate::components::dialog::dialog(
                    crate::components::dialog::DialogProps::new(
                        "Notes",
                        "Keep context for this item.",
                        move || modal.close(),
                    )
                    .width(460.0)
                    .body(
                        view()
                            .w_full()
                            .p(px(20.0))
                            .child(crate::components::textarea::text_area(value, "Add notes…"))
                            .into_element(),
                    )
                    .footer(
                        view()
                            .w_full()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(crate::ui::action_button("Cancel", false, move || {
                                modal.close()
                            }))
                            .child(crate::ui::action_button("Save", false, move || {
                                let notes = save_value.get();
                                let server = save_server.clone();
                                let token = save_token.clone();
                                spawn(async move {
                                    let body = if notes.trim().is_empty() {
                                        serde_json::json!({ "notes": null })
                                    } else {
                                        serde_json::json!({ "notes": notes })
                                    };
                                    let result = match save_kind {
                                        Kind::Subscription | Kind::Budget => {
                                            api::update_subscription(server, token, save_id, &body)
                                                .await
                                                .map(|_| ())
                                        }
                                        Kind::Transaction => {
                                            api::update_transaction(server, token, save_id, &body)
                                                .await
                                                .map(|_| ())
                                        }
                                        Kind::Debt => {
                                            api::update_debt(server, token, save_id, &body)
                                                .await
                                                .map(|_| ())
                                        }
                                    };
                                    match result {
                                        Ok(()) => {
                                            crate::components::toast::success("Notes saved.");
                                            subscriptions.refetch();
                                            transactions.refetch();
                                            debts.refetch();
                                            ui::bump_data();
                                        }
                                        Err(message) => crate::components::toast::error(message),
                                    }
                                    modal.close();
                                });
                            }))
                            .into_element(),
                    ),
                )
                .into_element()
            });
        }
    };
    let notes_item: std::rc::Rc<dyn Fn(Kind, Uuid)> = std::rc::Rc::new(notes_item);
    let pay_item = {
        let server = server.clone();
        let token = token.clone();
        move |kind: Kind, id: Uuid| {
            if kind != Kind::Debt {
                return;
            }
            let Some(debt) = debts
                .get()
                .unwrap_or_default()
                .debts
                .into_iter()
                .find(|item| item.id == id)
            else {
                return;
            };
            let amount = signal(
                debt.monthly_payment
                    .unwrap_or(debt.remaining_amount)
                    .to_string(),
            );
            let paid_on = signal(chrono::Local::now().date_naive().to_string());
            let note = signal(String::new());
            let error = signal(String::new());
            let server = server.clone();
            let token = token.clone();
            modal.open(move || {
                let save_server = server.clone();
                let save_token = token.clone();
                crate::components::dialog::dialog(
                    crate::components::dialog::DialogProps::new(
                        "Record payment",
                        "Reduce the remaining balance of this debt.",
                        move || modal.close(),
                    )
                    .width(460.0)
                    .body(
                        view()
                            .w_full()
                            .p(px(20.0))
                            .gap(px(12.0))
                            .child(ui::numeric_field("Amount", amount, "250"))
                            .child(ui::date_field("Paid on", paid_on))
                            .child(ui::field("Note", note, "Optional"))
                            .child(
                                text(move || error.get())
                                    .text_size(px(11.0))
                                    .text_color(move || theme().destructive),
                            )
                            .into_element(),
                    )
                    .footer(
                        view()
                            .w_full()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(crate::ui::action_button("Cancel", false, move || {
                                modal.close()
                            }))
                            .child(crate::ui::action_button("Pay", false, move || {
                                let Ok(value) = amount.get().trim().parse::<Decimal>() else {
                                    error.set("Enter a valid amount.".to_string());
                                    return;
                                };
                                let Ok(date) = chrono::NaiveDate::parse_from_str(
                                    paid_on.get().trim(),
                                    "%Y-%m-%d",
                                ) else {
                                    error.set("Enter a valid date.".to_string());
                                    return;
                                };
                                let note_value =
                                    (!note.get().trim().is_empty()).then(|| note.get());
                                let server = save_server.clone();
                                let token = save_token.clone();
                                spawn(async move {
                                    match api::record_debt_payment(
                                        server,
                                        token,
                                        id,
                                        &crate::debts::RecordDebtPaymentRequest {
                                            amount: value,
                                            paid_on: date,
                                            note: note_value,
                                        },
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            crate::components::toast::success("Payment recorded.");
                                            debts.refetch();
                                            ui::bump_data();
                                        }
                                        Err(message) => crate::components::toast::error(message),
                                    }
                                    modal.close();
                                });
                            }))
                            .into_element(),
                    ),
                )
                .into_element()
            });
        }
    };
    let pay_item: std::rc::Rc<dyn Fn(Kind, Uuid)> = std::rc::Rc::new(pay_item);
    set_money_actions(
        edit_action.clone(),
        delete_item.clone(),
        duplicate_item,
        notes_item,
        pay_item,
        selected,
        bulk,
    );
    view()
        .w_full()
        .flex_col()
        .gap(px(22.0))
        .child(page_header(
            "Money",
            "Subscriptions, budgets, one-off payments and debts in one place.",
            view()
                .child(crate::components::button::button(
                    crate::components::button::ButtonProps::new("Add")
                        .icon(asset!("icons/plus.svg"))
                        .on_press(open_add.clone()),
                ))
                .into_element(),
            compact,
            false,
            0.0,
        ))
        .child(
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(search_input(search, "Search money"))
                .child(crate::components::button::icon_button_when(
                    asset!("icons/square.svg"),
                    "Select items",
                    move || bulk.get(),
                    move || {
                        let next = !bulk.get();
                        bulk.set(next);
                        if !next {
                            selected.set(Vec::new());
                        }
                    },
                ))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .child(crate::components::list_toolbar::view_toggle(
                            asset!("icons/grid.svg"),
                            "Grid view",
                            move || !list_view.get(),
                            move || list_view.set(false),
                        ))
                        .child(crate::components::list_toolbar::view_toggle(
                            asset!("icons/list.svg"),
                            "List view",
                            move || list_view.get(),
                            move || list_view.set(true),
                        )),
                ),
        )
        .child(chips(filter, move || {
            build_items(
                subscriptions,
                transactions,
                debts,
                search.get(),
                Filter::All,
            )
        }))
        .child(
            when(
                move || bulk.get(),
                move || {
                    let all = items();
                    let set_active: std::rc::Rc<dyn Fn(bool)> = std::rc::Rc::new({
                        let server = server_signal;
                        let token = token_signal;
                        move |active: bool| {
                            let values = selected.get();
                            let server = server.get();
                            let token = token.get();
                            spawn(async move {
                                for (kind, id) in values {
                                    if matches!(kind, Kind::Subscription | Kind::Budget) {
                                        let _ = api::update_subscription(
                                            server.clone(),
                                            token.clone(),
                                            id,
                                            &serde_json::json!({ "is_active": active }),
                                        )
                                        .await;
                                    }
                                }
                                selected.set(Vec::new());
                                subscriptions.refetch();
                                transactions.refetch();
                                debts.refetch();
                                ui::bump_data();
                            });
                        }
                    });
                    view()
                        .w_full()
                        .flex_row()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.0))
                        .p(px(10.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().primary.opacity(0.4))
                        .bg(move || theme().primary.opacity(0.06))
                        .child(
                            text(move || format!("{} selected", selected.get().len()))
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(crate::ui::action_button("Select all", false, move || {
                            selected.set(all.iter().map(|item| (item.kind, item.id)).collect());
                        }))
                        .child(crate::ui::action_button("Clear", false, move || {
                            selected.set(Vec::new())
                        }))
                        .child(crate::ui::action_button("Mark active", false, {
                            let apply = set_active.clone();
                            move || apply(true)
                        }))
                        .child(crate::ui::action_button("Mark inactive", false, {
                            let apply = set_active.clone();
                            move || apply(false)
                        }))
                        .child(crate::ui::action_button("Delete", true, {
                            let server = server_signal;
                            let token = token_signal;
                            move || {
                                let values = selected.get();
                                if values.is_empty() {
                                    return;
                                }
                                let server = server.get();
                                let token = token.get();
                                spawn(async move {
                                    for (kind, id) in values {
                                        let _ = match kind {
                                            Kind::Subscription | Kind::Budget => {
                                                api::delete_subscription(
                                                    server.clone(),
                                                    token.clone(),
                                                    id,
                                                )
                                                .await
                                            }
                                            Kind::Transaction => {
                                                api::delete_transaction(
                                                    server.clone(),
                                                    token.clone(),
                                                    id,
                                                )
                                                .await
                                            }
                                            Kind::Debt => {
                                                api::delete_debt(server.clone(), token.clone(), id)
                                                    .await
                                            }
                                        };
                                    }
                                    selected.set(Vec::new());
                                    bulk.set(false);
                                    subscriptions.refetch();
                                    transactions.refetch();
                                    debts.refetch();
                                    ui::bump_data();
                                });
                            }
                        }))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .child(
            when(loading, || {
                card()
                    .p(px(24.0))
                    .child(text("Loading money…"))
                    .into_element()
            })
            .otherwise(move || {
                let rest_currency = currency.clone();
                when(
                    move || error().is_some(),
                    move || {
                        card()
                            .p(px(24.0))
                            .child(text(error().unwrap_or_default()))
                            .into_element()
                    },
                )
                .otherwise(move || {
                    let branch_currency = rest_currency.clone();
                    when(
                        move || items().is_empty(),
                        move || {
                            empty_state(
                                asset!("icons/wallet.svg"),
                                "No money items yet",
                                "Add a subscription, budget, one-off payment or debt.",
                                crate::components::button::button(
                                    crate::components::button::ButtonProps::new("Add")
                                        .icon(asset!("icons/plus.svg"))
                                        .on_press(open_filtered),
                                )
                                .into_element(),
                            )
                            .into_element()
                        },
                    )
                    .otherwise(move || {
                        let base_rates = subscriptions
                            .get()
                            .map(|data| data.rates)
                            .or_else(|| transactions.get().map(|data| data.rates))
                            .or_else(|| debts.get().map(|data| data.rates))
                            .unwrap_or_default();
                        let base_currency = branch_currency.clone();
                        let item_currency = stored(base_currency.clone());
                        let item_rates = stored(base_rates.clone());
                        when(move || list_view.get(), {
                            move || {
                                view()
                                    .w_full()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(each(
                                        items,
                                        |item| {
                                            format!(
                                                "{}-{}-{}-{}",
                                                item.kind.label(),
                                                item.id,
                                                item.updated,
                                                item.color
                                            )
                                        },
                                        move |item, _| {
                                            let currency =
                                                item_currency.with(|value| value.clone());
                                            let rates = item_rates.with(|value| value.clone());
                                            list_row(item.get(), &currency, &rates)
                                        },
                                    ))
                                    .into_element()
                            }
                        })
                        .otherwise(move || {
                            view()
                                .w_full()
                                .flex_row()
                                .flex_wrap()
                                .gap(px(16.0))
                                .child(each(
                                    items,
                                    |item| {
                                        format!(
                                            "{}-{}-{}-{}",
                                            item.kind.label(),
                                            item.id,
                                            item.updated,
                                            item.color
                                        )
                                    },
                                    move |item, _| {
                                        let currency = item_currency.with(|value| value.clone());
                                        let rates = item_rates.with(|value| value.clone());
                                        grid_card(item.get(), &currency, &rates)
                                    },
                                ))
                                .into_element()
                        })
                        .into_element()
                    })
                    .into_element()
                })
            }),
        )
        .into_element()
}

fn chips(filter: Signal<Filter>, items: impl Fn() -> Vec<Item> + Copy + 'static) -> AnyElement {
    let counts = move || {
        let all = items();
        [
            (Filter::All, all.len()),
            (
                Filter::Subscription,
                all.iter().filter(|i| i.kind == Kind::Subscription).count(),
            ),
            (
                Filter::Budget,
                all.iter().filter(|i| i.kind == Kind::Budget).count(),
            ),
            (
                Filter::Transaction,
                all.iter().filter(|i| i.kind == Kind::Transaction).count(),
            ),
            (
                Filter::Debt,
                all.iter().filter(|i| i.kind == Kind::Debt).count(),
            ),
        ]
    };
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .gap(px(6.0))
        .child(each(
            move || counts(),
            |(value, count)| format!("{}-{}", value.label(), count),
            move |item, _| {
                let (value, count) = item.get();
                let active = move || filter.get() == value;
                button()
                    .cursor_pointer()
                    .h(px(32.0))
                    .px(px(12.0))
                    .items_center()
                    .justify_center()
                    .rounded(px(999.0))
                    .border(px(1.0))
                    .border_color(move || {
                        if active() {
                            theme().primary
                        } else {
                            theme().border
                        }
                    })
                    .bg(move || {
                        if active() {
                            theme().primary.opacity(0.12)
                        } else {
                            theme().card
                        }
                    })
                    .on_press(move || filter.set(value))
                    .child(
                        text(format!("{} · {}", value.label(), count))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || {
                                if active() {
                                    theme().primary
                                } else {
                                    theme().muted_foreground
                                }
                            }),
                    )
                    .into_element()
            },
        ))
        .into_element()
}

/// Colours the cards fall back to when neither the category nor the account
/// has one: the same stable palette the Dioxus avatars use, keyed by the
/// entity's name, so every card is identifiable at a glance.
const FALLBACK_COLORS: [&str; 8] = [
    "#6366f1", "#8b5cf6", "#ec4899", "#f97316", "#14b8a6", "#0ea5e9", "#22c55e", "#eab308",
];

fn fallback_color(seed: &str) -> String {
    if seed.is_empty() {
        return "#6b7280".to_string();
    }
    let hash = seed
        .bytes()
        .fold(0usize, |acc, byte| acc.wrapping_add(usize::from(byte)));
    FALLBACK_COLORS[hash % FALLBACK_COLORS.len()].to_string()
}

fn build_items(
    subscriptions: Resource<api::SubscriptionsData, String>,
    transactions: Resource<api::TransactionsData, String>,
    debts: Resource<api::DebtsData, String>,
    search: String,
    filter: Filter,
) -> Vec<Item> {
    let needle = search.trim().to_lowercase();
    let mut items = Vec::new();
    // An automatic debt bills through a subscription of its own: the debt card
    // already shows it, so the subscription is not listed twice.
    let linked: Vec<Uuid> = debts.get().map_or_else(Vec::new, |data| {
        data.debts
            .iter()
            .filter_map(|debt| debt.subscription_id)
            .collect()
    });
    if let Some(data) = subscriptions.get() {
        for sub in data.subscriptions {
            if linked.contains(&sub.id) {
                continue;
            }
            let kind = if sub.kind == "budget" {
                Kind::Budget
            } else {
                Kind::Subscription
            };
            let account_row = data.accounts.iter().find(|a| a.id == sub.account_id);
            let account = account_row.map_or_else(String::new, |a| a.name.clone());
            let color = data
                .categories
                .iter()
                .find(|c| c.id == sub.category_id)
                .and_then(|c| c.color.clone())
                .or_else(|| account_row.and_then(|a| a.color.clone()))
                .unwrap_or_else(|| {
                    fallback_color(if account.is_empty() {
                        &sub.name
                    } else {
                        &account
                    })
                });
            let category = data
                .categories
                .iter()
                .find(|c| c.id == sub.category_id)
                .map_or_else(String::new, |c| c.name.clone());
            let detail = sub.next_billing_date.map_or_else(
                || "No next date".to_string(),
                |date| format!("Next: {}", format_date(date)),
            );
            items.push(Item {
                id: sub.id,
                kind,
                name: sub.name,
                amount: sub.amount,
                currency: sub.currency,
                logo: sub.logo_url,
                color,
                account,
                category,
                detail,
                progress: None,
                updated: sub.updated_at.map_or(0, |d| d.timestamp_millis()),
            });
        }
    }
    if let Some(data) = transactions.get() {
        for tx in data.transactions {
            let account_row = data.accounts.iter().find(|a| a.id == tx.account_id);
            let account = account_row.map_or_else(String::new, |a| a.name.clone());
            let color = data
                .categories
                .iter()
                .find(|c| c.id == tx.category_id)
                .and_then(|c| c.color.clone())
                .or_else(|| account_row.and_then(|a| a.color.clone()))
                .unwrap_or_else(|| {
                    fallback_color(if account.is_empty() {
                        &tx.name
                    } else {
                        &account
                    })
                });
            let category = data
                .categories
                .iter()
                .find(|c| c.id == tx.category_id)
                .map_or_else(String::new, |c| c.name.clone());
            items.push(Item {
                id: tx.id,
                kind: Kind::Transaction,
                name: tx.name,
                amount: tx.amount,
                currency: tx.currency,
                logo: tx.logo_url,
                color,
                account,
                category,
                detail: format_date(tx.date),
                progress: None,
                updated: tx.updated_at.map_or(0, |d| d.timestamp_millis()),
            });
        }
    }
    if let Some(data) = debts.get() {
        for debt in data.debts {
            let account_row = data.accounts.iter().find(|a| a.id == debt.account_id);
            let account = account_row.map_or_else(String::new, |a| a.name.clone());
            let color = data
                .categories
                .iter()
                .find(|c| c.id == debt.category_id)
                .and_then(|c| c.color.clone())
                .or_else(|| account_row.and_then(|a| a.color.clone()))
                .unwrap_or_else(|| {
                    fallback_color(if account.is_empty() {
                        &debt.name
                    } else {
                        &account
                    })
                });
            let category = data
                .categories
                .iter()
                .find(|c| c.id == debt.category_id)
                .map_or_else(String::new, |c| c.name.clone());
            let paid = debt.total_amount - debt.remaining_amount;
            let ratio = if debt.total_amount > Decimal::ZERO {
                (paid / debt.total_amount)
                    .to_string()
                    .parse::<f32>()
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            items.push(Item {
                id: debt.id,
                kind: Kind::Debt,
                name: debt.name,
                amount: debt.remaining_amount,
                currency: debt.currency,
                logo: debt.logo_url,
                color,
                account,
                category,
                detail: debt.next_payment_date.map_or_else(
                    || "No next payment".to_string(),
                    |date| format!("Next: {}", format_date(date)),
                ),
                progress: Some(ratio.clamp(0.0, 1.0)),
                updated: debt.updated_at.map_or(0, |d| d.timestamp_millis()),
            });
        }
    }
    items.retain(|item| {
        filter.accepts(item.kind)
            && (needle.is_empty()
                || item.name.to_lowercase().contains(&needle)
                || item.account.to_lowercase().contains(&needle)
                || item.category.to_lowercase().contains(&needle))
    });
    items.sort_by_key(|item| std::cmp::Reverse(item.updated));
    items
}

fn kind_icon(kind: Kind) -> ImageAsset {
    match kind {
        Kind::Subscription | Kind::Budget => asset!("icons/subscriptions.svg"),
        Kind::Transaction => asset!("icons/transactions.svg"),
        Kind::Debt => asset!("icons/debts.svg"),
    }
}

fn type_badge(kind: Kind) -> AnyElement {
    let color = match kind {
        Kind::Subscription => theme().primary,
        Kind::Budget => theme().accent_foreground,
        Kind::Transaction => theme().muted_foreground,
        Kind::Debt => theme().destructive,
    };
    view()
        .px(px(8.0))
        .py(px(2.0))
        .rounded(px(999.0))
        .bg(move || theme().muted.opacity(0.6))
        .child(
            text(kind.label())
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || color),
        )
        .into_element()
}

/// Cards share the row (2 columns from 620, 3 from 900, 4 from 1200) instead of
/// stacking when a fixed 360 px would not fit next to its neighbour.
const GRID_CARD_BASIS: f32 = 290.0;
const GRID_CARD_MIN: f32 = 260.0;
const GRID_CARD_MAX: f32 = 420.0;
/// Cards wear the colour of the account (or the category) the item belongs to,
/// so a tile is recognisable before reading its name.
const CARD_TINT_ALPHA: u32 = 0x1f;
const CARD_BORDER_ALPHA: u32 = 0x4d;

/// The account (or category) an item belongs to, as the small coloured chip
/// the cards wear: its colour is the one the entity's icon was given.
fn entity_chip(label: String, color: String) -> AnyElement {
    view()
        .h(px(20.0))
        .px(px(6.0))
        .items_center()
        .justify_center()
        .rounded(px(6.0))
        .border(px(1.0))
        .border_color({
            let chip = color.clone();
            move || AdaptiveColor::from(rgba((crate::theme::parse_hex(&chip) << 8) | 0x26))
        })
        .bg({
            let chip = color.clone();
            move || rgba((crate::theme::parse_hex(&chip) << 8) | 0x15)
        })
        .child(
            text(label)
                .text_size(px(11.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || rgba(crate::theme::parse_hex(&color) << 8)),
        )
        .into_element()
}

fn grid_card(item: Item, currency: &str, rates: &crate::models::Rates) -> AnyElement {
    let item_id = item.id;
    let logo = item.logo.clone();
    let name = item.name.clone();
    let detail = item.detail.clone();
    let account = item.account.clone();
    let kind = item.kind;
    let progress = item.progress;
    let card_color = item.color.clone();
    let border_color = card_color.clone();
    card()
        .basis(px(GRID_CARD_BASIS))
        .flex_grow(1.0)
        .flex_shrink(1.0)
        .min_w(px(GRID_CARD_MIN))
        .max_w(px(GRID_CARD_MAX))
        .p(px(18.0))
        .gap(px(12.0))
        .cursor_pointer()
        .bg(move || rgba((crate::theme::parse_hex(&card_color) << 8) | CARD_TINT_ALPHA))
        .border_color(move || {
            if money_is_selected(kind, item_id) {
                theme().primary
            } else {
                rgba((crate::theme::parse_hex(&border_color) << 8) | CARD_BORDER_ALPHA).into()
            }
        })
        .on_press(move || {
            if money_bulk_active() {
                money_toggle(kind, item_id);
            }
        })
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(crate::components::logo_avatar::logo_avatar(
                    logo,
                    kind_icon(kind),
                    38.0,
                    false,
                ))
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .flex_grow(1.0)
                        .child(
                            text(name)
                                .number_of_lines(1)
                                .text_size(px(15.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(detail)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(converted_amount(
                    item.amount,
                    &item.currency,
                    currency,
                    rates,
                    15.0,
                    FontWeight::BOLD,
                    true,
                )),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(type_badge(kind))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .child(entity_chip(
                            if account.is_empty() {
                                item.category.clone()
                            } else {
                                account.clone()
                            },
                            item.color.clone(),
                        ))
                        .when(kind == Kind::Debt, |row| {
                            row.child(crate::ui::mini_icon_button(
                                asset!("icons/card.svg"),
                                false,
                                move || money_pay(kind, item_id),
                            ))
                        })
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/notes.svg"),
                            false,
                            move || money_notes(kind, item_id),
                        ))
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/pencil.svg"),
                            false,
                            move || money_edit(kind, item_id),
                        ))
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/copy.svg"),
                            false,
                            move || money_duplicate(kind, item_id),
                        ))
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/trash.svg"),
                            true,
                            move || money_delete(kind, item_id),
                        )),
                ),
        )
        .when_some(progress, |card, value| {
            card.child(
                view()
                    .w_full()
                    .h(px(6.0))
                    .rounded(px(999.0))
                    .bg(move || theme().muted)
                    .child(
                        view()
                            .h_full()
                            .w(nsh_core::geometry::Length::from(
                                nsh_core::geometry::Percentage(value),
                            ))
                            .rounded(px(999.0))
                            .bg(move || theme().primary),
                    ),
            )
        })
        .into_element()
}

fn list_row(item: Item, currency: &str, rates: &crate::models::Rates) -> AnyElement {
    let item_id = item.id;
    let logo = item.logo.clone();
    let name = item.name.clone();
    let detail = item.detail.clone();
    let kind = item.kind;
    let progress = item.progress;
    let card_color = item.color.clone();
    let border_color = card_color.clone();
    view()
        .w_full()
        .rounded(px(12.0))
        .cursor_pointer()
        .border(px(1.0))
        .border_color(move || {
            if money_is_selected(kind, item_id) {
                theme().primary
            } else {
                rgba((crate::theme::parse_hex(&border_color) << 8) | CARD_BORDER_ALPHA).into()
            }
        })
        .bg(move || rgba((crate::theme::parse_hex(&card_color) << 8) | CARD_TINT_ALPHA))
        .p(px(14.0))
        .on_press(move || {
            if money_bulk_active() {
                money_toggle(kind, item_id);
            }
        })
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(crate::components::logo_avatar::logo_avatar(
                    logo,
                    kind_icon(kind),
                    36.0,
                    false,
                ))
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .flex_grow(1.0)
                        .child(
                            text(name)
                                .number_of_lines(1)
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(detail)
                                .number_of_lines(1)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(type_badge(kind)),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .mt(px(10.0))
                .pt(px(8.0))
                .border_t(px(1.0))
                .border_color(move || theme().border.opacity(0.4))
                .child(converted_amount(
                    item.amount,
                    &item.currency,
                    currency,
                    rates,
                    14.0,
                    FontWeight::SEMIBOLD,
                    true,
                ))
                .when_some(progress, |row, value| {
                    row.child(
                        view()
                            .flex_grow(1.0)
                            .min_w(px(120.0))
                            .mx(px(24.0))
                            .h(px(6.0))
                            .rounded(px(999.0))
                            .bg(move || theme().muted)
                            .child(
                                view()
                                    .h_full()
                                    .w(nsh_core::geometry::Length::from(
                                        nsh_core::geometry::Percentage(value),
                                    ))
                                    .rounded(px(999.0))
                                    .bg(move || theme().primary),
                            ),
                    )
                })
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .pointer_events(move || {
                            if money_bulk_active() {
                                PointerEvents::None
                            } else {
                                PointerEvents::Auto
                            }
                        })
                        .when(kind == Kind::Debt, |row| {
                            row.child(crate::ui::mini_icon_button(
                                asset!("icons/card.svg"),
                                false,
                                move || money_pay(kind, item_id),
                            ))
                        })
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/notes.svg"),
                            false,
                            move || money_notes(kind, item_id),
                        ))
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/pencil.svg"),
                            false,
                            move || money_edit(kind, item_id),
                        ))
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/copy.svg"),
                            false,
                            move || money_duplicate(kind, item_id),
                        ))
                        .child(crate::ui::mini_icon_button(
                            asset!("icons/trash.svg"),
                            true,
                            move || money_delete(kind, item_id),
                        )),
                ),
        )
        .into_element()
}

pub fn open_money_add(auth: AuthSession, modal: ui::ModalHost) {
    let auth = std::rc::Rc::new(auth);
    modal.open(move || add_menu(auth.clone(), modal));
}

fn add_menu(auth: std::rc::Rc<AuthSession>, modal: ui::ModalHost) -> AnyElement {
    let close = move || modal.close();
    let open = move |kind: Filter| {
        modal.close();
        match kind {
            Filter::Subscription => crate::ui::create_subscription(None),
            Filter::Budget => crate::subscriptions::open_create_dialog_kind(
                (*auth).clone(),
                modal,
                None,
                "budget",
            ),
            Filter::Transaction => crate::ui::create_transaction(None),
            Filter::Debt => crate::ui::create_debt(None),
            Filter::All => {}
        }
    };
    let row = |kind: Filter,
               icon: ImageAsset,
               description: &'static str,
               open: &std::rc::Rc<dyn Fn(Filter)>| {
        let open = open.clone();
        view()
            .w_full()
            .flex_row()
            .items_center()
            .gap(px(12.0))
            .p(px(12.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .border(px(1.0))
            .border_color(move || theme().border)
            .bg(move || theme().card)
            .on_press(move || open(kind))
            .child(
                image(icon)
                    .size(px(20.0))
                    .tint_color(move || theme().primary),
            )
            .child(
                view()
                    .flex_col()
                    .child(
                        text(kind.label())
                            .text_size(px(14.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || theme().foreground),
                    )
                    .child(
                        text(description)
                            .text_size(px(11.0))
                            .text_color(move || theme().muted_foreground),
                    ),
            )
            .into_element()
    };
    let open: std::rc::Rc<dyn Fn(Filter)> = std::rc::Rc::new(open);
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            "Add to Money",
            "Choose what you want to add.",
            close,
        )
        .width(420.0)
        .footer_visible(false)
        .body(
            view()
                .w_full()
                .flex_col()
                .gap(px(8.0))
                .p(px(20.0))
                .child(row(
                    Filter::Subscription,
                    asset!("icons/subscriptions.svg"),
                    "A recurring service like Netflix.",
                    &open,
                ))
                .child(row(
                    Filter::Budget,
                    asset!("icons/wallet.svg"),
                    "A recurring budget you plan for.",
                    &open,
                ))
                .child(row(
                    Filter::Transaction,
                    asset!("icons/transactions.svg"),
                    "A one-off payment.",
                    &open,
                ))
                .child(row(
                    Filter::Debt,
                    asset!("icons/debts.svg"),
                    "Money you owe and pay down.",
                    &open,
                ))
                .into_element(),
        ),
    )
    .into_element()
}

#[cfg(test)]
mod tests {
    use super::{FALLBACK_COLORS, Filter, Kind, fallback_color};

    #[test]
    fn the_type_filters_match_their_own_kind_only() {
        assert!(Filter::All.accepts(Kind::Budget));
        assert!(Filter::Subscription.accepts(Kind::Subscription));
        assert!(!Filter::Subscription.accepts(Kind::Budget));
        assert!(Filter::Budget.accepts(Kind::Budget));
        assert!(Filter::Transaction.accepts(Kind::Transaction));
        assert!(Filter::Debt.accepts(Kind::Debt));
    }

    #[test]
    fn cards_fall_back_to_a_stable_palette_colour() {
        assert_eq!(fallback_color("Test"), fallback_color("Test"));
        assert!(FALLBACK_COLORS.contains(&fallback_color("Test").as_str()));
        assert_eq!(fallback_color(""), "#6b7280");
    }

    #[test]
    fn money_kinds_have_user_facing_labels() {
        assert_eq!(Kind::Subscription.label(), "Subscription");
        assert_eq!(Kind::Budget.label(), "Budget");
        assert_eq!(Kind::Transaction.label(), "One-off payment");
        assert_eq!(Kind::Debt.label(), "Debt");
    }
}
