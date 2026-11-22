//! Transactions: one-time expenses grouped by month.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::api::{self, TransactionsData};
use crate::auth::AuthSession;
use crate::format::format_currency;
use crate::models::{Account, Rates, Transaction};
use crate::theme as design;
use crate::ui;

#[derive(Serialize)]
pub struct CreateTransactionRequest {
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub account_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
}

#[derive(Clone, Copy)]
struct TxForm {
    open: Signal<bool>,
    editing: Signal<Option<Uuid>>,
    name: Signal<String>,
    amount: Signal<String>,
    date: Signal<String>,
    account_id: Signal<Option<Uuid>>,
    category_id: Signal<Option<Uuid>>,
    error: Signal<String>,
    saving: Signal<bool>,
}

#[derive(Clone, Copy)]
struct TxActions {
    form: TxForm,
    delete_confirm: Signal<Option<Uuid>>,
    busy: Signal<Option<Uuid>>,
    error: Signal<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct TransactionGroup {
    key: String,
    label: String,
    total: Decimal,
    items: Vec<Transaction>,
}

const TITLE: &str = "Transactions";
const SUBTITLE: &str = "Track your one-time expenses";

pub fn transactions(auth: AuthSession) -> AnyElement {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let load_currency = currency.clone();
    let resource_server = server.clone();
    let resource_token = token.clone();
    let data = resource(
        || Some(()),
        move |(), _| {
            let server = resource_server.clone();
            let token = resource_token.clone();
            let currency = load_currency.clone();
            async move { api::load_transactions(server, token, currency).await }
        },
    );
    let server_signal = signal(server);
    let token_signal = signal(token);
    let search = signal(String::new());
    let form_open = signal(false);
    let editing = signal(None::<Uuid>);
    let name = signal(String::new());
    let amount = signal(String::new());
    let today = Local::now().date_naive();
    let date = signal(today.to_string());
    let account_id = signal(None::<Uuid>);
    let category_id = signal(None::<Uuid>);
    let form_error = signal(String::new());
    let saving = signal(false);
    let delete_confirm = signal(None::<Uuid>);
    let busy = signal(None::<Uuid>);
    let action_error = signal(String::new());
    let form_signals = TxForm {
        open: form_open,
        editing,
        name,
        amount,
        date,
        account_id,
        category_id,
        error: form_error,
        saving,
    };
    let actions = TxActions {
        form: form_signals,
        delete_confirm,
        busy,
        error: action_error,
    };
    let loading = move || data.loading() && data.get().is_none();
    let has_error = move || data.error().is_some();
    let render_currency = currency.clone();

    let open_form = move || {
        let snapshot = data.get().unwrap_or_default();
        let today = Local::now().date_naive();
        name.set(String::new());
        amount.set(String::new());
        date.set(today.to_string());
        account_id.set(
            snapshot
                .accounts
                .iter()
                .find(|account| account.is_default)
                .or_else(|| snapshot.accounts.first())
                .map(|account| account.id),
        );
        category_id.set(snapshot.categories.first().map(|category| category.id));
        form_error.set(String::new());
        saving.set(false);
        editing.set(None);
        form_open.set(true);
    };

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(header(open_form))
        .child(
            when(move || form_open.get(), move || {
                form(
                    data,
                    form_signals,
                    server_signal,
                    token_signal,
                    currency.clone(),
                )
            })
            .otherwise(|| view().h(px(0.0))),
        )
        .child(toolbar(data, search))
        .child(
            when(move || !actions.error.get().is_empty(), move || {
                text(move || actions.error.get())
                    .text_size(px(13.0))
                    .text_color(move || theme().destructive)
            })
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(loading, || loading_card()).otherwise(move || {
                let value = render_currency.clone();
                when(has_error, move || {
                    error_card(&data.error().unwrap_or_default(), data)
                })
                .otherwise(move || {
                    content(data, search, value.clone(), actions, server_signal, token_signal)
                })
            }),
        )
        .into_element()
}

fn header(on_add: impl Fn() + 'static) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .items_end()
        .justify_between()
        .gap(px(24.0))
        .child(ui::page_header(TITLE, SUBTITLE))
        .child(
            button()
                .h(px(40.0))
                .px(px(16.0))
                .rounded(px(8.0))
                .bg(move || theme().primary)
                .on_press(on_add)
                .child(
                    text("Add transaction")
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary_foreground),
                ),
        )
}

#[allow(clippy::too_many_arguments)]
fn form(
    data: Resource<TransactionsData, String>,
    form: TxForm,
    server: Signal<String>,
    token: Signal<String>,
    currency: String,
) -> impl IntoElement {
    let TxForm {
        open: form_open,
        editing,
        name,
        amount,
        date,
        account_id,
        category_id,
        error: form_error,
        saving,
    } = form;
    let save = {
        let currency = currency.clone();
        move || {
            if saving.get() {
                return;
            }
            let Some(account_value) = account_id.get() else {
                form_error.set("Create an account first.".to_string());
                return;
            };
            let name_value = name.get().trim().to_string();
            if name_value.is_empty() {
                form_error.set("Name is required.".to_string());
                return;
            }
            let Ok(amount_value) = amount.get().trim().parse::<Decimal>() else {
                form_error.set("Enter a valid amount.".to_string());
                return;
            };
            if amount_value <= Decimal::ZERO {
                form_error.set("Amount must be positive.".to_string());
                return;
            }
            let Ok(date_value) = NaiveDate::parse_from_str(date.get().trim(), "%Y-%m-%d") else {
                form_error.set("Use YYYY-MM-DD for the date.".to_string());
                return;
            };
            let request = CreateTransactionRequest {
                name: name_value,
                amount: amount_value,
                currency: currency.clone(),
                date: date_value,
                account_id: account_value,
                category_id: category_id.get(),
            };
            saving.set(true);
            form_error.set(String::new());
            let server_value = server.get();
            let token_value = token.get();
            let editing_value = editing.get();
            spawn(async move {
                let result = if let Some(id) = editing_value {
                    api::update_transaction(server_value, token_value, id, &request).await
                } else {
                    api::create_transaction(server_value, token_value, &request).await
                };
                match result {
                    Ok(_) => {
                        form_open.set(false);
                        editing.set(None);
                        data.refetch();
                    }
                    Err(message) => form_error.set(message),
                }
                saving.set(false);
            });
        }
    };

    view()
        .w_full()
        .max_w(px(760.0))
        .rounded(px(16.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(24.0))
        .gap(px(16.0))
        .child(
            view()
                .flex_col()
                .gap(px(2.0))
                .child(
                    text(move || if editing.get().is_some() { "Edit transaction" } else { "New transaction" })
                        .text_size(px(20.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(move || if editing.get().is_some() {
                        "Update the amount, date or account."
                    } else {
                        "Record a one-time expense."
                    })
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(12.0))
                .child(ui::field("Name", name, "Groceries"))
                .child(ui::field("Amount", amount, "42.50")),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(12.0))
                .child(ui::field("Date", date, "2026-11-22")),
        )
        .child(ui::choice_picker(
            "Account",
            move || {
                data.get()
                    .unwrap_or_default()
                    .accounts
                    .into_iter()
                    .map(|account| (account.id, account.name))
                    .collect()
            },
            account_id,
        ))
        .child(ui::choice_picker(
            "Category",
            move || {
                data.get()
                    .unwrap_or_default()
                    .categories
                    .into_iter()
                    .map(|category| (category.id, category.name))
                    .collect()
            },
            category_id,
        ))
        .child(
            when(move || !form_error.get().is_empty(), move || {
                text(move || form_error.get())
                    .text_size(px(12.0))
                    .text_color(move || theme().destructive)
            })
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    button()
                        .h(px(36.0))
                        .px(px(14.0))
                        .rounded(px(7.0))
                        .bg(move || theme().primary)
                        .disabled(move || saving.get())
                        .disabled_style(|state| state.opacity(0.5))
                        .on_press(move || save())
                        .child(
                            text(move || if saving.get() { "Saving..." } else { "Save transaction" })
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().primary_foreground),
                        ),
                )
                .child(
                    button()
                        .h(px(36.0))
                        .px(px(14.0))
                        .rounded(px(7.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().card)
                        .on_press(move || {
                            form_open.set(false);
                            editing.set(None);
                        })
                        .child(
                            text("Cancel")
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        ),
                ),
        )
}

fn toolbar(data: Resource<TransactionsData, String>, search: Signal<String>) -> impl IntoElement {
    let count = move || filtered(data, search).len();
    view()
        .w_full()
        .flex_row()
        .items_center()
        .gap(px(12.0))
        .child(
            input()
                .value(search)
                .on_change(move |value: &str| search.set(value.to_string()))
                .placeholder("Search")
                .auto_fill(AutoFill::Off)
                .h(px(40.0))
                .w(px(320.0))
                .px(px(12.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().input)
                .bg(move || theme().card)
                .text_size(px(14.0))
                .text_color(move || theme().foreground),
        )
        .child(
            text(move || ui::count_label(count(), "transaction", "transactions"))
                .text_size(px(13.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn content(
    data: Resource<TransactionsData, String>,
    search: Signal<String>,
    currency: String,
    actions: TxActions,
    server: Signal<String>,
    token: Signal<String>,
) -> AnyElement {
    let groups = move || {
        let snapshot = data.get().unwrap_or_default();
        group_transactions(
            &snapshot.transactions,
            &snapshot.accounts,
            &snapshot.rates,
            &search.get(),
        )
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(16.0))
        .child(
            when(
                move || data.get().unwrap_or_default().transactions.is_empty(),
                || {
                    ui::surface()
                        .p(px(28.0))
                        .child(
                            text("No transactions yet.")
                                .text_size(px(14.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .child(each(
            groups,
            |group| group.key.clone(),
            move |group, _| {
                group_card(
                    group.get(),
                    data,
                    currency.clone(),
                    actions,
                    server,
                    token,
                )
            },
        ))
        .into_element()
}

fn group_card(
    group: TransactionGroup,
    data: Resource<TransactionsData, String>,
    currency: String,
    actions: TxActions,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let count = ui::count_label(group.items.len(), "transaction", "transactions");
    let total = format_currency(group.total, &currency);
    view()
        .w_full()
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .overflow_hidden()
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(20.0))
                .py(px(12.0))
                .bg(move || theme().muted)
                .child(
                    text(format!("{} · {count}", group.label))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(total)
                        .text_size(px(13.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_col()
                .child(each(
                    move || group.items.clone(),
                    |transaction| transaction.id,
                    move |transaction, _| {
                        row(
                            transaction.get(),
                            data,
                            currency.clone(),
                            actions,
                            server,
                            token,
                        )
                    },
                )),
        )
}

fn row(
    transaction: Transaction,
    data: Resource<TransactionsData, String>,
    currency: String,
    actions: TxActions,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let color = account_color(data, transaction.account_id);
    let color_text = color.clone();
    let color_bg = color;
    let account = account_name(data, transaction.account_id);
    let action_transaction = transaction.clone();
    let name = transaction.name;
    let date = transaction.date.format("%d/%m/%Y").to_string();
    let amount = {
        let rates = data.get().unwrap_or_default().rates;
        format_currency(rates.convert(transaction.amount, &transaction.currency), &currency)
    };
    view()
        .w_full()
        .flex_row()
        .items_center()
        .justify_between()
        .px(px(20.0))
        .py(px(12.0))
        .border_b(px(1.0))
        .border_color(move || theme().border)
        .gap(px(12.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                .min_w(px(0.0))
                .flex_1()
                .child(
                    view()
                        .h(px(36.0))
                        .w(px(36.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || rgba((parse_hex(&color_bg) << 8) | 0x24))
                        .child(
                            image(asset!("icons/transactions.svg"))
                                .size(px(16.0))
                                .tint_color(move || rgb(parse_hex(&color_text))),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text(name)
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(format!("{date} · {account}"))
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .flex_col()
                .items_end()
                .gap(px(4.0))
                .child(
                    text(amount)
                        .text_size(px(14.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(transaction_actions(
                    action_transaction,
                    actions,
                    data,
                    server,
                    token,
                )),
        )
}

fn transaction_actions(
    transaction: Transaction,
    actions: TxActions,
    data: Resource<TransactionsData, String>,
    server: Signal<String>,
    token: Signal<String>,
) -> AnyElement {
    let id = transaction.id;
    let form = actions.form;
    let edit_transaction = transaction.clone();
    let edit = move || {
        form.name.set(edit_transaction.name.clone());
        form.amount.set(edit_transaction.amount.to_string());
        form.date.set(edit_transaction.date.to_string());
        form.account_id.set(Some(edit_transaction.account_id));
        form.category_id.set(Some(edit_transaction.category_id));
        form.error.set(String::new());
        form.saving.set(false);
        form.editing.set(Some(id));
        form.open.set(true);
    };
    let delete = move || {
        actions.delete_confirm.set(None);
        actions.busy.set(Some(id));
        actions.error.set(String::new());
        let server_value = server.get();
        let token_value = token.get();
        spawn(async move {
            match api::delete_transaction(server_value, token_value, id).await {
                Ok(()) => data.refetch(),
                Err(message) => actions.error.set(message),
            }
            actions.busy.set(None);
        });
    };
    let confirm = actions.delete_confirm;
    when(move || actions.delete_confirm.get() == Some(id), move || {
        let delete = delete.clone();
        view()
            .flex_row()
            .gap(px(4.0))
            .child(ui::action_button("Delete", true, delete))
            .child(ui::action_button("Cancel", false, move || {
                actions.delete_confirm.set(None)
            }))
            .into_element()
    })
    .otherwise(move || {
        let edit = edit.clone();
        view()
            .flex_row()
            .gap(px(4.0))
            .child(ui::action_button("Edit", false, edit))
            .child(ui::action_button("Delete", true, move || {
                confirm.set(Some(id));
            }))
            .into_element()
    })
    .into_element()
}

fn filtered(
    data: Resource<TransactionsData, String>,
    search: Signal<String>,
) -> Vec<Transaction> {
    let snapshot = data.get().unwrap_or_default();
    filter_transactions(&snapshot.transactions, &snapshot.accounts, &search.get())
}

fn filter_transactions(
    transactions: &[Transaction],
    accounts: &[Account],
    query: &str,
) -> Vec<Transaction> {
    let query = query.trim().to_lowercase();
    let mut result: Vec<Transaction> = transactions
        .iter()
        .filter(|transaction| {
            if query.is_empty() {
                return true;
            }
            let account = accounts
                .iter()
                .find(|account| account.id == transaction.account_id)
                .map_or("", |account| account.name.as_str());
            transaction.name.to_lowercase().contains(&query)
                || account.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();
    result.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.name.cmp(&b.name)));
    result
}

fn group_transactions(
    transactions: &[Transaction],
    accounts: &[Account],
    rates: &Rates,
    query: &str,
) -> Vec<TransactionGroup> {
    let mut groups: Vec<TransactionGroup> = Vec::new();
    for transaction in filter_transactions(transactions, accounts, query) {
        let key = format!("{:04}-{:02}", transaction.date.year(), transaction.date.month());
        let label = transaction.date.format("%B %Y").to_string();
        let amount = rates.convert(transaction.amount, &transaction.currency);
        if let Some(group) = groups.iter_mut().find(|group| group.key == key) {
            group.total += amount;
            group.items.push(transaction);
        } else {
            groups.push(TransactionGroup {
                key,
                label,
                total: amount,
                items: vec![transaction],
            });
        }
    }
    groups.sort_by(|a, b| b.key.cmp(&a.key));
    groups
}

fn account_name(data: Resource<TransactionsData, String>, account_id: Uuid) -> String {
    data.get()
        .unwrap_or_default()
        .accounts
        .into_iter()
        .find(|account| account.id == account_id)
        .map_or_else(|| "No account".to_string(), |account| account.name)
}

fn account_color(data: Resource<TransactionsData, String>, account_id: Uuid) -> String {
    data.get()
        .unwrap_or_default()
        .accounts
        .into_iter()
        .find(|account| account.id == account_id)
        .and_then(|account| account.color)
        .unwrap_or_else(|| "#6b7280".to_string())
}

fn parse_hex(value: &str) -> u32 {
    u32::from_str_radix(value.trim_start_matches('#'), 16).unwrap_or(0x6b7280)
}

fn loading_card() -> AnyElement {
    ui::surface()
        .p(px(28.0))
        .child(
            text("Loading transactions…")
                .text_size(px(14.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn error_card(message: &str, data: Resource<TransactionsData, String>) -> AnyElement {
    let message = message.to_string();
    ui::surface()
        .p(px(24.0))
        .gap(px(12.0))
        .child(
            text("Transactions could not be loaded.")
                .text_size(px(15.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(message)
                .text_size(px(13.0))
                .text_color(move || theme().destructive),
        )
        .child(
            button()
                .h(px(34.0))
                .px(px(14.0))
                .rounded(px(7.0))
                .bg(move || theme().primary)
                .on_press(move || data.refetch())
                .child(
                    text("Try again")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary_foreground),
                ),
        )
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
    }

    fn account(id: Uuid, name: &str) -> Account {
        Account {
            id,
            name: name.to_string(),
            icon: None,
            color: None,
            is_default: false,
        }
    }

    fn transaction(name: &str, account_id: Uuid, amount: Decimal, day: NaiveDate) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: name.to_string(),
            amount,
            currency: "EUR".to_string(),
            date: day,
            logo_url: None,
        }
    }

    #[test]
    fn transactions_group_by_month_and_sort_newest_first() {
        let account_id = Uuid::new_v4();
        let items = vec![
            transaction("Coffee", account_id, Decimal::from(3), date(2026, 11, 22)),
            transaction("Rent", account_id, Decimal::from(900), date(2026, 10, 1)),
            transaction("Lunch", account_id, Decimal::from(12), date(2026, 11, 20)),
        ];
        let groups = group_transactions(&items, &[], &Rates::default(), "");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].label, "November 2026");
        assert_eq!(groups[0].items.len(), 2);
        assert_eq!(groups[0].total, Decimal::from(15));
    }

    #[test]
    fn transaction_search_matches_name_and_account() {
        let account_id = Uuid::new_v4();
        let accounts = vec![account(account_id, "Everyday")];
        let items = vec![
            transaction("Groceries", account_id, Decimal::from(42), date(2026, 11, 22)),
            transaction("Gym", Uuid::new_v4(), Decimal::from(30), date(2026, 11, 21)),
        ];
        assert_eq!(filter_transactions(&items, &accounts, "groc").len(), 1);
        assert_eq!(filter_transactions(&items, &accounts, "every").len(), 1);
        assert_eq!(filter_transactions(&items, &accounts, "").len(), 2);
    }

    #[test]
    fn create_request_serializes_without_category() {
        let request = CreateTransactionRequest {
            name: "Groceries".to_string(),
            amount: Decimal::new(4250, 2),
            currency: "EUR".to_string(),
            date: date(2026, 11, 22),
            account_id: Uuid::new_v4(),
            category_id: None,
        };
        let value = serde_json::to_value(request).expect("request serializes");
        assert_eq!(value["name"], json!("Groceries"));
        assert!(value.get("category_id").is_none());
    }
}
