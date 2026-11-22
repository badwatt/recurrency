//! Debts: manual balances and automatic month-by-month payments.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::api::{self, DebtsData};
use crate::auth::AuthSession;
use crate::format::format_currency;
use crate::models::{Debt, DebtPayment};
use crate::theme as design;
use crate::ui;

#[derive(Serialize)]
pub struct CreateDebtRequest {
    pub name: String,
    pub total_amount: Decimal,
    pub currency: String,
    pub account_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub debt_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_payment: Option<Decimal>,
    pub payment_days: Vec<u8>,
    pub start_date: Option<NaiveDate>,
}

#[derive(Clone, Copy)]
struct DebtForm {
    open: Signal<bool>,
    editing: Signal<Option<Uuid>>,
    name: Signal<String>,
    total: Signal<String>,
    monthly: Signal<String>,
    start_date: Signal<String>,
    debt_type: Signal<String>,
    account_id: Signal<Option<Uuid>>,
    category_id: Signal<Option<Uuid>>,
    error: Signal<String>,
    saving: Signal<bool>,
}

#[derive(Serialize)]
pub struct RecordDebtPaymentRequest {
    pub amount: Decimal,
    pub paid_on: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Copy)]
struct DebtPayments {
    pay_id: Signal<Option<Uuid>>,
    amount: Signal<String>,
    paid_on: Signal<String>,
    note: Signal<String>,
    error: Signal<String>,
    saving: Signal<bool>,
    history_id: Signal<Option<Uuid>>,
    history: Signal<Vec<DebtPayment>>,
    history_loading: Signal<bool>,
    history_error: Signal<String>,
}

#[derive(Clone, Copy)]
struct DebtActions {
    form: DebtForm,
    payments: DebtPayments,
    modal: ui::ModalHost,
    delete_confirm: Signal<Option<Uuid>>,
    busy: Signal<Option<Uuid>>,
    error: Signal<String>,
}

const TITLE: &str = "Debts";
const SUBTITLE: &str = "Money you owe, manual or paid automatically";

pub fn debts(auth: AuthSession, modal: ui::ModalHost) -> AnyElement {
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
            async move { api::load_debts(server, token, currency).await }
        },
    );
    let server_signal = signal(server);
    let token_signal = signal(token);
    let search = signal(String::new());
    let form_open = signal(false);
    let editing = signal(None::<Uuid>);
    let name = signal(String::new());
    let total = signal(String::new());
    let monthly = signal(String::new());
    let today = Local::now().date_naive();
    let start_date = signal(today.to_string());
    let debt_type = signal("manual".to_string());
    let account_id = signal(None::<Uuid>);
    let category_id = signal(None::<Uuid>);
    let form_error = signal(String::new());
    let saving = signal(false);
    let delete_confirm = signal(None::<Uuid>);
    let busy = signal(None::<Uuid>);
    let action_error = signal(String::new());
    let pay_id = signal(None::<Uuid>);
    let pay_amount = signal(String::new());
    let pay_date = signal(today.to_string());
    let pay_note = signal(String::new());
    let pay_error = signal(String::new());
    let paying = signal(false);
    let history_id = signal(None::<Uuid>);
    let history = signal(Vec::<DebtPayment>::new());
    let history_loading = signal(false);
    let history_error = signal(String::new());
    let payments = DebtPayments {
        pay_id,
        amount: pay_amount,
        paid_on: pay_date,
        note: pay_note,
        error: pay_error,
        saving: paying,
        history_id,
        history,
        history_loading,
        history_error,
    };
    let form_state = DebtForm {
        open: form_open,
        editing,
        name,
        total,
        monthly,
        start_date,
        debt_type,
        account_id,
        category_id,
        error: form_error,
        saving,
    };
    let actions = DebtActions {
        form: form_state,
        payments,
        modal,
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
        total.set(String::new());
        monthly.set(String::new());
        start_date.set(today.to_string());
        debt_type.set("manual".to_string());
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
        let dialog_currency = currency.clone();
        modal.open(move || {
            form(
                data,
                form_state,
                server_signal,
                token_signal,
                dialog_currency.clone(),
                modal,
            )
            .into_element()
        });
    };

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(header(open_form))
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
                .otherwise(move || content(data, search, value.clone(), actions, server_signal, token_signal))
            }),
        )
        .into_element()
}


fn payment_form(
    data: Resource<DebtsData, String>,
    payments: DebtPayments,
    server: Signal<String>,
    token: Signal<String>,
    _currency: String,
    modal: ui::ModalHost,
) -> impl IntoElement {
    let DebtPayments {
        pay_id,
        amount,
        paid_on,
        note,
        error,
        saving,
        ..
    } = payments;
    let save = move || {
        if saving.get() || pay_id.get().is_none() {
            return;
        }
        let id = pay_id.get().unwrap_or_else(Uuid::nil);
        let Ok(amount_value) = amount.get().trim().parse::<Decimal>() else {
            error.set("Enter a valid amount.".to_string());
            return;
        };
        if amount_value <= Decimal::ZERO {
            error.set("Amount must be positive.".to_string());
            return;
        }
        let Ok(paid_on_value) = NaiveDate::parse_from_str(paid_on.get().trim(), "%Y-%m-%d") else {
            error.set("Use YYYY-MM-DD for the payment date.".to_string());
            return;
        };
        let request = RecordDebtPaymentRequest {
            amount: amount_value,
            paid_on: paid_on_value,
            note: (!note.get().trim().is_empty()).then(|| note.get()),
        };
        saving.set(true);
        error.set(String::new());
        let server_value = server.get();
        let token_value = token.get();
        spawn(async move {
            match api::record_debt_payment(server_value, token_value, id, &request).await {
                Ok(_) => {
                    pay_id.set(None);
                    data.refetch();
                }
                Err(message) => error.set(message),
            }
            saving.set(false);
        });
    };

    view()
        .w_full()
        .max_w(px(620.0))
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
                    text("Record payment")
                        .text_size(px(20.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("Reduce the remaining balance of this debt.")
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(12.0))
                .child(ui::field("Amount", amount, "250"))
                .child(ui::field("Paid on", paid_on, "2026-11-22")),
        )
        .child(ui::field("Note", note, "November payment"))
        .child(
            when(move || !error.get().is_empty(), move || {
                text(move || error.get())
                    .text_size(px(12.0))
                    .text_color(move || theme().destructive)
            })
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            view()
                .flex_row()
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
                            text(move || if saving.get() { "Saving..." } else { "Record payment" })
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
                        .on_press(move || { pay_id.set(None); modal.close(); })
                        .child(
                            text("Cancel")
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        ),
                ),
        )
}

fn history_panel(payments: DebtPayments, currency: String, modal: ui::ModalHost) -> impl IntoElement {
    let DebtPayments {
        history_id,
        history,
        history_loading,
        history_error,
        ..
    } = payments;
    view()
        .w_full()
        .max_w(px(720.0))
        .rounded(px(16.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(24.0))
        .gap(px(12.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    text("Payment history")
                        .text_size(px(20.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(ui::action_button("Close", false, move || {
                    history_id.set(None);
                    modal.close();
                })),
        )
        .child(
            when(move || history_loading.get(), || {
                text("Loading payments…")
                    .text_size(px(13.0))
                    .text_color(move || theme().muted_foreground)
                    .into_element()
            })
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(move || !history_error.get().is_empty(), move || {
                text(move || history_error.get())
                    .text_size(px(12.0))
                    .text_color(move || theme().destructive)
            })
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            when(
                move || !history_loading.get() && history.get().is_empty(),
                || {
                    text("No payments recorded.")
                        .text_size(px(13.0))
                        .text_color(move || theme().muted_foreground)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            view()
                .flex_col()
                .child(each(
                    move || history.get(),
                    |payment| payment.id,
                    move |payment, _| {
                        let payment = payment.get();
                        let amount = format_currency(payment.amount, &currency);
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .py(px(8.0))
                            .border_b(px(1.0))
                            .border_color(move || theme().border)
                            .child(
                                view()
                                    .flex_col()
                                    .child(
                                        text(payment.paid_on.format("%d/%m/%Y").to_string())
                                            .text_size(px(13.0))
                                            .text_color(move || theme().foreground),
                                    )
                                    .child(
                                        text(payment.note.unwrap_or_default())
                                            .text_size(px(11.0))
                                            .text_color(move || theme().muted_foreground),
                                    ),
                            )
                            .child(
                                text(amount)
                                    .text_size(px(13.0))
                                    .font_family(design::MONO)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            )
                    },
                )),
        )
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
                    text("Add Debt")
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary_foreground),
                ),
        )
}

fn form(
    data: Resource<DebtsData, String>,
    form: DebtForm,
    server: Signal<String>,
    token: Signal<String>,
    currency: String,
    modal: ui::ModalHost,
) -> impl IntoElement {
    let DebtForm {
        open: form_open,
        editing,
        name,
        total,
        monthly,
        start_date,
        debt_type,
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
            let Ok(total_value) = total.get().trim().parse::<Decimal>() else {
                form_error.set("Enter a valid total.".to_string());
                return;
            };
            if total_value <= Decimal::ZERO {
                form_error.set("Total must be positive.".to_string());
                return;
            }
            let Ok(start_value) = NaiveDate::parse_from_str(start_date.get().trim(), "%Y-%m-%d") else {
                form_error.set("Use YYYY-MM-DD for the first payment.".to_string());
                return;
            };
            let automatic = debt_type.get() == "automatic";
            let monthly_value = if automatic {
                match monthly.get().trim().parse::<Decimal>() {
                    Ok(value) if value > Decimal::ZERO => Some(value),
                    _ => {
                        form_error.set("Monthly payment must be positive.".to_string());
                        return;
                    }
                }
            } else {
                None
            };
            let request = CreateDebtRequest {
                name: name_value,
                total_amount: total_value,
                currency: currency.clone(),
                account_id: account_value,
                category_id: category_id.get(),
                debt_type: debt_type.get(),
                monthly_payment: monthly_value,
                payment_days: if automatic {
                    vec![start_value.day() as u8]
                } else {
                    Vec::new()
                },
                start_date: Some(start_value),
            };
            saving.set(true);
            form_error.set(String::new());
            let server_value = server.get();
            let token_value = token.get();
            let editing_value = editing.get();
            spawn(async move {
                let result = if let Some(id) = editing_value {
                    api::update_debt(server_value, token_value, id, &request).await
                } else {
                    api::create_debt(server_value, token_value, &request).await
                };
                match result {
                    Ok(_) => {
                        form_open.set(false);
                        editing.set(None);
                        modal.close();
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
                    text(move || if editing.get().is_some() { "Edit debt" } else { "Add debt" })
                        .text_size(px(20.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("Manual balances or automatic payments.")
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(12.0))
                .child(ui::field("Name", name, "Car loan"))
                .child(ui::field("Total owed", total, "12000")),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(12.0))
                .child(ui::field("First payment", start_date, "2026-11-22"))
                .child(ui::field("Monthly payment", monthly, "250")),
        )
        .child(
            view()
                .flex_col()
                .gap(px(6.0))
                .child(ui::label("Type"))
                .child(
                    view()
                        .flex_row()
                        .gap(px(6.0))
                        .child(type_chip("Manual", "manual", debt_type))
                        .child(type_chip("Automatic", "automatic", debt_type)),
                ),
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
                            text(move || if saving.get() { "Saving..." } else { "Save debt" })
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
                            modal.close();
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

fn type_chip(label_text: &'static str, value: &'static str, selected: Signal<String>) -> impl IntoElement {
    let active = move || selected.get() == value;
    button()
        .h(px(30.0))
        .px(px(10.0))
        .rounded(px(6.0))
        .border(px(1.0))
        .border_color(move || if active() { theme().primary } else { theme().border })
        .bg(move || if active() { theme().accent } else { theme().card })
        .on_press(move || selected.set(value.to_string()))
        .child(
            text(label_text)
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || if active() {
                    theme().foreground
                } else {
                    theme().muted_foreground
                }),
        )
}

fn toolbar(data: Resource<DebtsData, String>, search: Signal<String>) -> impl IntoElement {
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
            text(move || ui::count_label(count(), "debt", "debts"))
                .text_size(px(13.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn content(
    data: Resource<DebtsData, String>,
    search: Signal<String>,
    currency: String,
    actions: DebtActions,
    server: Signal<String>,
    token: Signal<String>,
) -> AnyElement {
    let items = move || filtered(data, search);
    view()
        .w_full()
        .flex_col()
        .gap(px(16.0))
        .child(
            when(
                move || data.get().unwrap_or_default().debts.is_empty(),
                || {
                    ui::surface()
                        .p(px(28.0))
                        .child(
                            text("No debts yet.")
                                .text_size(px(14.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .child(
            view()
                .w_full()
                .grid()
                .grid_template_columns(SharedString::from("repeat(2, minmax(0, 1fr))"))
                .gap(px(16.0))
                .child(each(
                    items,
                    |debt| debt.id,
                    move |debt, _| {
                        card(
                            debt.get(),
                            data,
                            currency.clone(),
                            actions,
                            server,
                            token,
                        )
                    },
                )),
        )
        .into_element()
}

fn card(
    debt: Debt,
    data: Resource<DebtsData, String>,
    currency: String,
    actions: DebtActions,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let color = account_color(data, debt.account_id);
    let account = account_name(data, debt.account_id);
    let color_bg = color.clone();
    let color_text = color;
    let action_debt = debt.clone();
    let name = debt.name;
    let automatic = debt.debt_type == "automatic";
    let badge = if automatic { "Automatic" } else { "Manual" };
    let total = format_currency(debt.total_amount, &currency);
    let remaining = format_currency(debt.remaining_amount, &currency);
    let paid = format_currency(debt.paid_amount, &currency);
    let progress = progress(debt.total_amount, debt.remaining_amount);
    let monthly = debt.monthly_payment.map_or_else(
        || "Manual".to_string(),
        |value| format!("{}/month", format_currency(value, &currency)),
    );
    let next = debt.next_payment_date.map_or_else(
        || "No next payment".to_string(),
        |date| format!("Next: {}", date.format("%d/%m/%Y")),
    );
    let progress_width = progress.max(0.01) as f32;

    view()
        .w_full()
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(20.0))
        .gap(px(14.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .justify_between()
                .gap(px(12.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(10.0))
                        .min_w(px(0.0))
                        .child(
                            view()
                                .h(px(38.0))
                                .w(px(38.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(8.0))
                                .bg(move || rgba((parse_hex(&color_bg) << 8) | 0x24))
                                .child(
                                    image(asset!("icons/debts.svg"))
                                        .size(px(17.0))
                                        .tint_color(move || rgb(parse_hex(&color_text))),
                                ),
                        )
                        .child(
                            view()
                                .flex_col()
                                .min_w(px(0.0))
                                .child(
                                    text(name)
                                        .text_size(px(15.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground)
                                        .number_of_lines(1),
                                )
                                .child(
                                    text(format!("{account} · {next}"))
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        ),
                )
                .child(
                    text(badge)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .px(px(7.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .bg(move || theme().muted),
                ),
        )
        .child(
            view()
                .flex_col()
                .gap(px(4.0))
                .child(
                    text(remaining)
                        .text_size(px(28.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(format!("{paid} paid of {total}"))
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            view()
                .h(px(7.0))
                .w_full()
                .rounded(px(999.0))
                .bg(move || theme().muted)
                .overflow_hidden()
                .flex_row()
                .child(
                    view()
                        .h_full()
                        .flex_grow(progress_width)
                        .bg(move || if automatic { theme().primary } else { design::success() }),
                )
                .child(view().h_full().flex_grow((100.0 - progress_width).max(0.01))),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    text(monthly)
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(card_actions(action_debt, actions, data, server, token, currency.clone())),
        )
}

fn card_actions(
    debt: Debt,
    actions: DebtActions,
    data: Resource<DebtsData, String>,
    server: Signal<String>,
    token: Signal<String>,
    currency: String,
) -> AnyElement {
    let id = debt.id;
    let form = actions.form;
    let edit_debt = debt.clone();
    let edit = move || {
        form.name.set(edit_debt.name.clone());
        form.total.set(edit_debt.total_amount.to_string());
        form.monthly.set(
            edit_debt
                .monthly_payment
                .map_or_else(String::new, |value| value.to_string()),
        );
        form.start_date.set(edit_debt.start_date.to_string());
        form.debt_type.set(edit_debt.debt_type.clone());
        form.account_id.set(Some(edit_debt.account_id));
        form.category_id.set(Some(edit_debt.category_id));
        form.error.set(String::new());
        form.saving.set(false);
        form.editing.set(Some(id));
        form.open.set(true);
    };
    let payments = actions.payments;
    let pay_currency = currency.clone();
    let history_currency = currency.clone();
    let pay = {
        let amount = edit_debt
            .monthly_payment
            .unwrap_or(edit_debt.remaining_amount)
            .to_string();
        let today = Local::now().date_naive().to_string();
        move || {
            payments.pay_id.set(Some(id));
            payments.amount.set(amount.clone());
            payments.paid_on.set(today.clone());
            payments.note.set(String::new());
            payments.error.set(String::new());
            payments.saving.set(false);
            payments.history_id.set(None);
            let dialog_currency = pay_currency.clone();
            actions.modal.open(move || {
                payment_form(
                    data,
                    payments,
                    server,
                    token,
                    dialog_currency.clone(),
                    actions.modal,
                )
                .into_element()
            });
        }
    };
    let history = {
        let server = server;
        let token = token;
        move || {
            payments.pay_id.set(None);
            payments.history_id.set(Some(id));
            payments.history_loading.set(true);
            payments.history_error.set(String::new());
            let server_value = server.get();
            let token_value = token.get();
            let dialog_currency = history_currency.clone();
            actions.modal.open(move || {
                history_panel(payments, dialog_currency.clone(), actions.modal).into_element()
            });
            spawn(async move {
                match api::load_debt_payments(server_value, token_value, id).await {
                    Ok(list) => payments.history.set(list),
                    Err(message) => payments.history_error.set(message),
                }
                payments.history_loading.set(false);
            });
        }
    };
    let delete = move || {
        actions.delete_confirm.set(None);
        actions.busy.set(Some(id));
        actions.error.set(String::new());
        let server_value = server.get();
        let token_value = token.get();
        spawn(async move {
            match api::delete_debt(server_value, token_value, id).await {
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
            .child(ui::action_button("Pay", false, pay.clone()))
            .child(ui::action_button("History", false, history.clone()))
            .child(ui::action_button("Edit", false, edit))
            .child(ui::action_button("Delete", true, move || {
                confirm.set(Some(id));
            }))
            .into_element()
    })
    .into_element()
}

fn filtered(data: Resource<DebtsData, String>, search: Signal<String>) -> Vec<Debt> {
    let snapshot = data.get().unwrap_or_default();
    filter_debts(&snapshot.debts, &snapshot.accounts, &search.get())
}

fn filter_debts(debts: &[Debt], accounts: &[crate::models::Account], query: &str) -> Vec<Debt> {
    let query = query.trim().to_lowercase();
    let mut result: Vec<Debt> = debts
        .iter()
        .filter(|debt| {
            if query.is_empty() {
                return true;
            }
            let account = accounts
                .iter()
                .find(|account| account.id == debt.account_id)
                .map_or("", |account| account.name.as_str());
            debt.name.to_lowercase().contains(&query) || account.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    result
}

fn progress(total: Decimal, remaining: Decimal) -> f64 {
    if total <= Decimal::ZERO {
        return 0.0;
    }
    let paid = total - remaining;
    let ratio = paid / total * Decimal::from(100);
    ratio
        .max(Decimal::ZERO)
        .min(Decimal::from(100))
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0)
}

fn account_name(data: Resource<DebtsData, String>, account_id: Uuid) -> String {
    data.get()
        .unwrap_or_default()
        .accounts
        .into_iter()
        .find(|account| account.id == account_id)
        .map_or_else(|| "No account".to_string(), |account| account.name)
}

fn account_color(data: Resource<DebtsData, String>, account_id: Uuid) -> String {
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
            text("Loading debts…")
                .text_size(px(14.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn error_card(message: &str, data: Resource<DebtsData, String>) -> AnyElement {
    let message = message.to_string();
    ui::surface()
        .p(px(24.0))
        .gap(px(12.0))
        .child(
            text("Debts could not be loaded.")
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

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
    }

    fn debt(name: &str, total: Decimal, remaining: Decimal) -> Debt {
        Debt {
            id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: name.to_string(),
            currency: "EUR".to_string(),
            total_amount: total,
            remaining_amount: remaining,
            paid_amount: total - remaining,
            debt_type: "manual".to_string(),
            monthly_payment: None,
            payment_days: Vec::new(),
            start_date: date(2026, 1, 1),
            next_payment_date: None,
            subscription_id: None,
            is_active: true,
            notes: None,
            logo_url: None,
        }
    }

    #[test]
    fn progress_is_capped_and_safe() {
        assert_eq!(progress(Decimal::from(100), Decimal::from(25)), 75.0);
        assert_eq!(progress(Decimal::ZERO, Decimal::from(10)), 0.0);
        assert_eq!(progress(Decimal::from(100), Decimal::ZERO), 100.0);
    }

    #[test]
    fn debt_search_matches_name_and_account() {
        let debts = vec![
            debt("Car loan", Decimal::from(12000), Decimal::from(9000)),
            debt("Mortgage", Decimal::from(200000), Decimal::from(180000)),
        ];
        assert_eq!(filter_debts(&debts, &[], "car").len(), 1);
        assert_eq!(filter_debts(&debts, &[], "").len(), 2);
    }

    #[test]
    fn payment_request_serializes_and_skips_empty_note() {
        let request = RecordDebtPaymentRequest {
            amount: Decimal::new(250, 0),
            paid_on: date(2026, 11, 22),
            note: None,
        };
        let value = serde_json::to_value(request).expect("payment serializes");
        assert_eq!(value["amount"], "250");
        assert!(value.get("note").is_none());
    }

    #[test]
    fn create_request_serializes_automatic_fields() {
        let request = CreateDebtRequest {
            name: "Car loan".to_string(),
            total_amount: Decimal::from(12000),
            currency: "EUR".to_string(),
            account_id: Uuid::new_v4(),
            category_id: None,
            debt_type: "automatic".to_string(),
            monthly_payment: Some(Decimal::from(250)),
            payment_days: vec![22],
            start_date: Some(date(2026, 11, 22)),
        };
        let value = serde_json::to_value(request).expect("request serializes");
        assert_eq!(value["debt_type"], "automatic");
        assert_eq!(value["payment_days"][0], 22);
    }
}
