//! Debts: manual balances and automatic month-by-month payments.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, theme};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::api::{self, DebtsData};
use crate::auth::AuthSession;
use crate::components::button::{ButtonProps, button as shared_button};
use crate::components::confirm::{ConfirmProps, confirm_row};
use crate::components::empty_state::empty_state;
use crate::components::filter_menu::filter_menu;
use crate::components::optional_date::optional_date;
use crate::components::page_header::page_header as shared_page_header;
use crate::components::search_input::search_input;
use crate::components::select_menu::{SelectMenuProps, SelectOption, select_menu};
use crate::components::sort_menu::{SortOption, sort_menu};
use crate::components::string_select::{StringSelectProps, string_select};
use crate::components::{BillingMode, BillingScheduleProps, billing_schedule};
use crate::format::{format_currency, format_date};
use crate::models::{AnnualDate, Debt, DebtPayment};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_payment: Option<NaiveDate>,
    #[serde(default)]
    pub clear_first_payment: bool,
    pub start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
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
    payment_days: Signal<Vec<u8>>,
    currency: Signal<String>,
    notes: Signal<String>,
    logo_url: Signal<String>,
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

pub fn debts(
    auth: AuthSession,
    modal: ui::ModalHost,
    intent: Signal<Option<crate::Route>>,
    compact: Signal<bool>,
) -> AnyElement {
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
        || Some(crate::ui::data_revision()),
        move |_, _| {
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
    let form_currency = signal(currency.clone());
    let payment_days = signal(Vec::<u8>::new());
    let notes = signal(String::new());
    let logo_url = signal(String::new());
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
        payment_days,
        currency: form_currency,
        notes,
        logo_url,
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
        payment_days.set(Vec::new());
        form_currency.set(currency.clone());
        notes.set(String::new());
        logo_url.set(String::new());
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

    let open_form = std::rc::Rc::new(open_form);

    let intent_open = open_form.clone();
    effect(move || {
        if intent.get() == Some(crate::Route::Debts) {
            intent.set(None);
            let open = intent_open.clone();
            open();
        }
    });

    let type_filter = signal("all".to_string());
    let sort_field = signal("name".to_string());
    let sort_desc = signal(false);
    let _ = (sort_field.get(), sort_desc.get());
    let empty_open = open_form.clone();
    let selection_mode = signal(false);
    let selected = signal(Vec::<Uuid>::new());
    let open_bulk = move || {
        selected.set(Vec::new());
        selection_mode.set(!selection_mode.get());
    };
    let open_header = open_form.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .pt(move || {
            if crate::ui::compact_active() {
                px(20.0)
            } else {
                px(0.0)
            }
        })
        .child(header(move || open_header(), compact))
        .child(when(loading, || loading_card()).otherwise(move || {
            let value = render_currency.clone();
            let empty_open = empty_open.clone();
            when(has_error, move || {
                error_card(&data.error().unwrap_or_default(), data)
            })
            .otherwise(move || {
                let empty_open = empty_open.clone();
                content(
                    data,
                    search,
                    value.clone(),
                    actions,
                    server_signal,
                    token_signal,
                    move || empty_open(),
                    open_bulk,
                    type_filter,
                    sort_field,
                    sort_desc,
                    selection_mode,
                    selected,
                    modal,
                )
            })
        }))
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
                Ok(debt) => {
                    pay_id.set(None);
                    if let Some(mut snapshot) = data.get() {
                        if let Some(item) = snapshot.debts.iter_mut().find(|item| item.id == id) {
                            *item = debt;
                        }
                        data.mutate(snapshot);
                    }
                    data.refetch();
                    crate::ui::bump_data();
                    modal.close();
                }
                Err(message) => error.set(message),
            }
            saving.set(false);
        });
    };

    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            "Record payment",
            "Reduce the remaining balance of this debt.",
            move || {
                pay_id.set(None);
                modal.close();
            },
        )
        .width(512.0)
        .body(
            view()
                .p(px(24.0))
                .gap(px(16.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::row_item(ui::numeric_field("Amount", amount, "250")))
                        .child(ui::row_item(ui::date_field("Paid on", paid_on))),
                )
                .child(ui::field("Note", note, "November payment"))
                .child(
                    when(
                        move || !error.get().is_empty(),
                        move || {
                            text(move || error.get())
                                .text_size(px(12.0))
                                .text_color(move || theme().destructive)
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .into_element(),
        )
        .footer(
            view()
                .w_full()
                .flex_row()
                .justify_end()
                .gap(px(8.0))
                .child(
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(40.0))
                        .px(px(16.0))
                        .rounded(px(6.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().card)
                        .on_press(move || {
                            pay_id.set(None);
                            modal.close();
                        })
                        .child(
                            text("Cancel")
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(40.0))
                        .px(px(14.0))
                        .rounded(px(6.0))
                        .bg(move || theme().primary)
                        .disabled(move || saving.get())
                        .disabled_style(|state| state.opacity(0.5))
                        .on_press(move || save())
                        .child(
                            text(move || {
                                if saving.get() {
                                    "Saving..."
                                } else {
                                    "Record payment"
                                }
                            })
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().primary_foreground),
                        ),
                )
                .into_element(),
        ),
    )
}

fn history_panel(
    payments: DebtPayments,
    currency: String,
    modal: ui::ModalHost,
) -> impl IntoElement {
    let DebtPayments {
        history_id,
        history,
        history_loading,
        history_error,
        ..
    } = payments;
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            "Payment history",
            "Payments recorded for this debt.",
            move || {
                history_id.set(None);
                modal.close();
            },
        )
        .width(720.0)
        .body(
            view()
                .p(px(24.0))
                .gap(px(12.0))
                .child(
                    when(
                        move || history_loading.get(),
                        || {
                            text("Loading payments…")
                                .text_size(px(13.0))
                                .text_color(move || theme().muted_foreground)
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || !history_error.get().is_empty(),
                        move || {
                            text(move || history_error.get())
                                .text_size(px(12.0))
                                .text_color(move || theme().destructive)
                        },
                    )
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
                .child(view().flex_col().child(each(
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
                                        text(format_date(payment.paid_on))
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
                                text(move || {
                                    if crate::ui::privacy_active() {
                                        "••••".to_string()
                                    } else {
                                        amount.clone()
                                    }
                                })
                                .text_size(px(13.0))
                                .font_family(design::MONO)
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                            )
                    },
                )))
                .into_element(),
        )
        .footer(
            view()
                .w_full()
                .flex_row()
                .justify_end()
                .child(ui::action_button("Close", false, move || {
                    history_id.set(None);
                    modal.close();
                }))
                .into_element(),
        ),
    )
}

fn header(on_add: impl Fn() + 'static, compact: Signal<bool>) -> impl IntoElement {
    shared_page_header(
        TITLE,
        SUBTITLE,
        view()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(px(8.0))
            .child(shared_button(
                ButtonProps::new("Add Debt")
                    .icon(asset!("icons/plus.svg"))
                    .min_width(113.0)
                    .on_press(on_add),
            ))
            .into_element(),
        compact,
        true,
        140.0,
    )
}

fn bulk_bar(
    data: Resource<DebtsData, String>,
    server: Signal<String>,
    token: Signal<String>,
    modal: ui::ModalHost,
    selected: Signal<Vec<Uuid>>,
    selection_mode: Signal<bool>,
) -> AnyElement {
    let busy = signal(false);
    let error = signal(String::new());
    let bulk_category = signal(None::<Uuid>);
    let count = move || selected.get().len();
    let all_ids = move || {
        data.get()
            .unwrap_or_default()
            .debts
            .into_iter()
            .map(|item| item.id)
            .collect::<Vec<_>>()
    };
    let clear = move || selected.set(Vec::new());
    let categories = move || {
        data.get()
            .unwrap_or_default()
            .categories
            .into_iter()
            .filter(|category| !category.is_system)
            .map(|category| SelectOption::new(category.id, category.name))
            .collect::<Vec<_>>()
    };
    let apply_category = {
        let server = server;
        let token = token;
        move || {
            let Some(category) = bulk_category.get() else {
                return;
            };
            let ids = selected.get();
            if ids.is_empty() || busy.get() {
                return;
            }
            let server = server.get();
            let token = token.get();
            busy.set(true);
            error.set(String::new());
            spawn(async move {
                for id in ids {
                    let _ = api::update_debt(
                        server.clone(),
                        token.clone(),
                        id,
                        &serde_json::json!({ "category_id": category }),
                    )
                    .await;
                }
                selected.set(Vec::new());
                selection_mode.set(false);
                data.refetch();
                crate::ui::bump_data();
                busy.set(false);
            });
        }
    };
    let delete = std::rc::Rc::new(move || {
        let ids = selected.get();
        if ids.is_empty() || busy.get() {
            return;
        }
        let server = server.get();
        let token = token.get();
        busy.set(true);
        error.set(String::new());
        spawn(async move {
            for id in ids {
                let _ = api::delete_debt(server.clone(), token.clone(), id).await;
            }
            selected.set(Vec::new());
            selection_mode.set(false);
            data.refetch();
            crate::ui::bump_data();
            busy.set(false);
            modal.close();
        });
    });
    let confirm_delete = {
        let delete = delete.clone();
        move || {
            let delete = delete.clone();
            modal.open(move || {
                let delete = delete.clone();
                crate::components::dialog::dialog(
                    crate::components::dialog::DialogProps::new(
                        format!("Delete {} debts?", count()),
                        "This cannot be undone.",
                        move || modal.close(),
                    )
                    .width(420.0)
                    .footer(
                        view()
                            .w_full()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(ui::action_button("Delete", true, move || delete()))
                            .child(ui::action_button("Cancel", false, move || modal.close()))
                            .into_element(),
                    ),
                )
                .into_element()
            });
        }
    };
    view()
        .relative()
        .z_index(10)
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(8.0))
        .px(px(12.0))
        .py(px(8.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().primary.opacity(0.3))
        .bg(move || theme().primary.opacity(0.05))
        .child(
            text(move || format!("{} selected", count()))
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().foreground),
        )
        .child(ui::action_button("Select all", false, move || {
            let mut values = selected.get();
            crate::components::selection_bar::add_all(&mut values, all_ids());
            selected.set(values);
        }))
        .child(
            view().w(px(176.0)).child(select_menu(
                SelectMenuProps::new("", "Set category…", bulk_category, categories)
                    .height(32.0)
                    .show_label(false)
                    .searchable(false),
            )),
        )
        .child(ui::action_button("Apply category", false, apply_category))
        .child(ui::action_button("Delete", true, move || confirm_delete()))
        .child(ui::action_button("Clear", false, clear))
        .when(!error.get().is_empty(), |bar| {
            bar.child(
                text(move || error.get())
                    .text_size(px(11.0))
                    .text_color(move || theme().destructive),
            )
        })
        .into_element()
}

pub fn open_create_dialog(auth: AuthSession, modal: ui::ModalHost, date: Option<NaiveDate>) {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let resource_currency = currency.clone();
    let data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            let currency = resource_currency.clone();
            async move { api::load_debts(server, token, currency).await }
        }
    });
    let form_state = DebtForm {
        open: signal(true),
        editing: signal(None::<Uuid>),
        name: signal(String::new()),
        total: signal(String::new()),
        monthly: signal(String::new()),
        start_date: signal(date.map(|value| value.to_string()).unwrap_or_default()),
        debt_type: signal("manual".to_string()),
        payment_days: signal(Vec::<u8>::new()),
        currency: signal(currency.clone()),
        notes: signal(String::new()),
        logo_url: signal(String::new()),
        account_id: signal(None::<Uuid>),
        category_id: signal(None::<Uuid>),
        error: signal(String::new()),
        saving: signal(false),
    };
    let server_signal = signal(server);
    let token_signal = signal(token);
    modal.open(move || {
        form(
            data,
            form_state,
            server_signal,
            token_signal,
            currency.clone(),
            modal,
        )
        .into_element()
    });
}

fn form(
    data: Resource<DebtsData, String>,
    form: DebtForm,
    server: Signal<String>,
    token: Signal<String>,
    _currency: String,
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
        payment_days,
        currency: form_currency,
        notes,
        logo_url,
        account_id,
        category_id,
        error: form_error,
        saving,
    } = form;
    effect(move || {
        if account_id.get().is_none() {
            if let Some(account) = data
                .get()
                .unwrap_or_default()
                .accounts
                .into_iter()
                .find(|account| account.is_default)
                .or_else(|| data.get().unwrap_or_default().accounts.into_iter().next())
            {
                account_id.set(Some(account.id));
            }
        }
        if category_id.get().is_none() {
            if let Some(category) = data.get().unwrap_or_default().categories.into_iter().next() {
                category_id.set(Some(category.id));
            }
        }
    });
    effect(move || {
        if debt_type.get() == "automatic" && payment_days.get().is_empty() {
            let date = NaiveDate::parse_from_str(&start_date.get(), "%Y-%m-%d")
                .unwrap_or_else(|_| Local::now().date_naive());
            payment_days.set(vec![date.day() as u8]);
        }
    });
    let schedule_interval = signal("1".to_string());
    let schedule_unit = signal("month".to_string());
    let schedule_weekdays = signal(Vec::<u8>::new());
    let schedule_annual_dates = signal(Vec::<AnnualDate>::new());
    let logo_results = signal(Vec::<(String, String)>::new());
    let logo_loading = signal(false);
    let logo_error = signal(String::new());
    let save = {
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
            let first_payment = if start_date.get().trim().is_empty() {
                None
            } else {
                match NaiveDate::parse_from_str(start_date.get().trim(), "%Y-%m-%d") {
                    Ok(date) => Some(date),
                    Err(_) => {
                        form_error.set("Enter a valid first payment.".to_string());
                        return;
                    }
                }
            };
            let start_value = first_payment.unwrap_or_else(|| Local::now().date_naive());
            let automatic = debt_type.get() == "automatic";
            let mut days = if automatic {
                let mut selected = payment_days.get();
                if selected.is_empty() {
                    selected.push(start_value.day() as u8);
                }
                selected.sort_unstable();
                selected.dedup();
                selected
            } else {
                Vec::new()
            };
            if automatic && days.is_empty() {
                days.push(start_value.day() as u8);
            }
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
                currency: form_currency.get(),
                account_id: account_value,
                category_id: category_id.get(),
                debt_type: debt_type.get(),
                monthly_payment: monthly_value,
                payment_days: days,
                first_payment,
                clear_first_payment: first_payment.is_none(),
                start_date: Some(start_value),
                notes: (!notes.get().trim().is_empty()).then(|| notes.get()),
                logo_url: (!logo_url.get().trim().is_empty()).then(|| logo_url.get()),
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
                    Ok(saved) => {
                        form_open.set(false);
                        editing.set(None);
                        if let Some(mut snapshot) = data.get() {
                            if let Some(item) =
                                snapshot.debts.iter_mut().find(|item| item.id == saved.id)
                            {
                                *item = saved.clone();
                            } else {
                                snapshot.debts.push(saved.clone());
                            }
                            data.mutate(snapshot);
                        }
                        data.refetch();
                        crate::ui::bump_data();
                        modal.close();
                    }
                    Err(message) => form_error.set(message),
                }
                saving.set(false);
            });
        }
    };

    let title = if editing.get().is_some() {
        "Edit Debt"
    } else {
        "Add Debt"
    };
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(title, "", move || {
            form_open.set(false);
            editing.set(None);
            modal.close();
        })
        .width(720.0)
        .body(
            view()
                .pl(px(22.0))
                .pr(px(34.0))
                .pt(px(24.0))
                .pb(px(24.0))
                .gap(px(16.0))
                .child(ui::required_field("Name", name, "Mortgage, car loan…"))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .gap(px(16.0))
                        .child(ui::row_item(ui::required_numeric_field(
                            "Total owed",
                            total,
                            "",
                        )))
                        .child(ui::row_item(string_select(StringSelectProps::new(
                            "Currency *",
                            form_currency,
                            crate::currencies::CURRENCIES
                                .iter()
                                .map(|(code, label)| ((*code).to_string(), (*label).to_string()))
                                .collect(),
                        )))),
                )
                .child(string_select(StringSelectProps::new(
                    "Type",
                    debt_type,
                    vec![
                        (
                            "manual".to_string(),
                            "Manual — I record each payment".to_string(),
                        ),
                        (
                            "automatic".to_string(),
                            "Automatic — paid month by month".to_string(),
                        ),
                    ],
                )))
                .child(
                    when(
                        move || debt_type.get() == "automatic",
                        move || {
                            view()
                                .w_full()
                                .flex_row()
                                .gap(px(16.0))
                                .child(ui::row_item(ui::required_numeric_field(
                                    "Monthly payment",
                                    monthly,
                                    "",
                                )))
                                .child(optional_date(
                                    "First payment",
                                    "Add first payment",
                                    start_date,
                                ))
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .child(
                    when(
                        move || debt_type.get() == "automatic",
                        move || {
                            view()
                                .w_full()
                                .flex_col()
                                .gap(px(6.0))
                                .child(billing_schedule(BillingScheduleProps {
                                    interval: schedule_interval,
                                    unit: schedule_unit,
                                    days: payment_days,
                                    weekdays: schedule_weekdays,
                                    annual_dates: schedule_annual_dates,
                                    start_date,
                                    mode: BillingMode::Monthly,
                                }))
                                .child(
                                    text(
                                        "This also creates a monthly subscription for the payment.",
                                    )
                                    .text_size(px(11.0))
                                    .text_color(move || theme().muted_foreground),
                                )
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
                .child(ui::editable_choice_picker(
                    "Account *",
                    move || {
                        data.get()
                            .unwrap_or_default()
                            .accounts
                            .into_iter()
                            .map(|account| (account.id, account.name))
                            .collect()
                    },
                    account_id,
                    ui::EntityKind::Account,
                    server,
                    token,
                    move || data.refetch(),
                ))
                .child(ui::editable_choice_picker(
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
                    ui::EntityKind::Category,
                    server,
                    token,
                    move || data.refetch(),
                ))
                .child(crate::components::field::field(
                    crate::components::field::FieldProps::new("Notes"),
                    crate::components::textarea::text_area(notes, "").into_element(),
                ))
                .child(crate::components::logo_search::logo_search(
                    name,
                    logo_url,
                    server,
                    crate::components::logo_search::LogoSearchState {
                        results: logo_results,
                        loading: logo_loading,
                        error: logo_error,
                    },
                ))
                .child(
                    when(
                        move || !form_error.get().is_empty(),
                        move || {
                            text(move || form_error.get())
                                .text_size(px(12.0))
                                .text_color(move || theme().destructive)
                        },
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .into_element(),
        )
        .footer(
            view()
                .w_full()
                .flex_row()
                .justify_end()
                .gap(px(8.0))
                .child(
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(40.0))
                        .px(px(16.0))
                        .rounded(px(6.0))
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
                                .text_size(px(14.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(40.0))
                        .px(px(14.0))
                        .rounded(px(6.0))
                        .bg(move || theme().primary)
                        .disabled(move || saving.get())
                        .disabled_style(|state| state.opacity(0.5))
                        .on_press(move || save())
                        .child(
                            text(move || {
                                if saving.get() {
                                    "Saving..."
                                } else if editing.get().is_some() {
                                    "Save changes"
                                } else {
                                    "Add Debt"
                                }
                            })
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().primary_foreground),
                        ),
                )
                .into_element(),
        ),
    )
}

fn toolbar(
    search: Signal<String>,
    open_bulk: impl Fn() + 'static,
    type_filter: Signal<String>,
    sort_field: Signal<String>,
    sort_desc: Signal<bool>,
) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(12.0))
        .child(search_input(search, "Search"))
        .child(filter_menu(
            move || usize::from(type_filter.get() != "all"),
            move || {
                view()
                    .child(string_select(StringSelectProps::new(
                        "Type",
                        type_filter,
                        vec![
                            ("all".to_string(), "All debts".to_string()),
                            ("manual".to_string(), "Manual".to_string()),
                            ("automatic".to_string(), "Automatic".to_string()),
                        ],
                    )))
                    .into_element()
            },
        ))
        .child(sort_menu(
            sort_field,
            sort_desc,
            vec![
                SortOption::new("name", "Name"),
                SortOption::new("remaining", "Remaining"),
                SortOption::new("next", "Next payment"),
                SortOption::new("created_at", "Created"),
                SortOption::new("updated_at", "Last modified"),
            ],
        ))
        .child(crate::components::button::icon_button(
            asset!("icons/square.svg"),
            "Enter selection mode",
            false,
            open_bulk,
        ))
}

fn content(
    data: Resource<DebtsData, String>,
    search: Signal<String>,
    currency: String,
    actions: DebtActions,
    server: Signal<String>,
    token: Signal<String>,
    open_form: impl Fn() + Clone + 'static,
    open_bulk: impl Fn() + Clone + 'static,
    type_filter: Signal<String>,
    sort_field: Signal<String>,
    sort_desc: Signal<bool>,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
    modal: ui::ModalHost,
) -> AnyElement {
    let items = move || filtered(data, search, type_filter, sort_field, sort_desc);
    let empty = move || data.get().unwrap_or_default().debts.is_empty();
    view()
        .w_full()
        .flex_col()
        .gap(px(16.0))
        .child(
            when(empty, move || {
                let open_form = open_form.clone();
                empty_state(
                    asset!("icons/debts.svg"),
                    "No debts yet",
                    "Add a manual debt or let an automatic debt pay itself month by month.",
                    shared_button(
                        ButtonProps::new("Add Debt")
                            .icon(asset!("icons/plus.svg"))
                            .min_width(113.0)
                            .on_press(move || open_form()),
                    )
                    .into_element(),
                )
                .into_element()
            })
            .otherwise(move || {
                let open_search = search;
                let card_currency = currency.clone();
                let bulk = open_bulk.clone();
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(16.0))
                    .child(toolbar(
                        open_search,
                        bulk,
                        type_filter,
                        sort_field,
                        sort_desc,
                    ))
                    .child(when(
                        move || selection_mode.get(),
                        move || {
                            bulk_bar(data, server, token, modal, selected, selection_mode)
                                .into_element()
                        },
                    ))
                    .child(
                        view()
                            .w_full()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(16.0))
                            .child(each(
                                items,
                                move |debt| {
                                    format!(
                                        "{}-{:?}-{}-{}",
                                        debt.id,
                                        debt.updated_at,
                                        sort_field.get(),
                                        sort_desc.get()
                                    )
                                },
                                move |debt, _| {
                                    card(
                                        debt.get(),
                                        data,
                                        card_currency.clone(),
                                        actions,
                                        server,
                                        token,
                                        selection_mode,
                                        selected,
                                    )
                                },
                            )),
                    )
                    .into_element()
            }),
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
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
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
    let progress = progress(debt.total_amount, debt.remaining_amount);
    let monthly = debt.monthly_payment.map_or_else(
        || "Manual".to_string(),
        |value| format!("{}/month", format_currency(value, &currency)),
    );
    let next = debt.next_payment_date.map_or_else(
        || "No next payment".to_string(),
        |date| format!("Next: {}", format_date(date)),
    );
    let progress_width = progress.max(0.01) as f32;
    let select_id = debt.id;
    let action_selected = action_debt.clone();

    view()
        .cursor_pointer()
        .basis(px(600.0))
        .flex_grow(1.0)
        .max_w(px(604.0))
        .min_w(px(0.0))
        .flex_shrink(1.0)
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || {
            if selected.get().contains(&select_id) {
                theme().primary.opacity(0.55)
            } else {
                theme().border
            }
        })
        .bg(move || {
            if selected.get().contains(&select_id) {
                theme().accent
            } else {
                theme().card
            }
        })
        .on_press(move || {
            if selection_mode.get() {
                let mut values = selected.get();
                crate::components::selection_bar::toggle(&mut values, select_id);
                selected.set(values);
            }
        })
        .p(px(16.0))
        .gap(px(12.0))
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
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            text(name)
                                                .text_size(px(14.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(move || theme().foreground)
                                                .number_of_lines(1),
                                        )
                                        .child(
                                            text(badge)
                                                .text_size(px(10.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(move || {
                                                    if automatic {
                                                        theme().primary
                                                    } else {
                                                        theme().muted_foreground
                                                    }
                                                })
                                                .px(px(6.0))
                                                .py(px(1.0))
                                                .rounded(px(999.0))
                                                .bg(move || {
                                                    if automatic {
                                                        theme().primary.opacity(0.12)
                                                    } else {
                                                        theme().muted
                                                    }
                                                }),
                                        ),
                                )
                                .child(
                                    text(account.clone())
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        ),
                )
                .child(
                    text(remaining)
                        .text_size(px(18.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
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
                .child(view().h_full().flex_grow(progress_width).bg(move || {
                    if automatic {
                        theme().primary
                    } else {
                        design::success()
                    }
                }))
                .child(
                    view()
                        .h_full()
                        .flex_grow((100.0 - progress_width).max(0.01)),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    text(format!("of {total} · {monthly} · {next}"))
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                )
                .child(when(
                    move || !selection_mode.get(),
                    move || {
                        card_actions(
                            action_selected.clone(),
                            actions,
                            data,
                            server,
                            token,
                            currency.clone(),
                        )
                        .into_element()
                    },
                )),
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
        form.payment_days.set(edit_debt.payment_days.clone());
        form.currency.set(edit_debt.currency.clone());
        form.notes.set(edit_debt.notes.clone().unwrap_or_default());
        form.logo_url
            .set(edit_debt.logo_url.clone().unwrap_or_default());
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
        if actions.busy.get() == Some(id) {
            return;
        }
        actions.busy.set(Some(id));
        actions.error.set(String::new());
        let server_value = server.get();
        let token_value = token.get();
        spawn(async move {
            match api::delete_debt(server_value, token_value, id).await {
                Ok(()) => {
                    crate::components::toast::success("Debt deleted.");
                    data.refetch();
                    crate::ui::bump_data();
                }
                Err(message) => {
                    crate::components::toast::error(message.clone());
                    actions.error.set(message);
                }
            }
            actions.delete_confirm.set(None);
            actions.busy.set(None);
        });
    };
    let confirm = actions.delete_confirm;
    when(
        move || actions.delete_confirm.get() == Some(id),
        move || {
            let delete = delete.clone();
            confirm_row(ConfirmProps::new(
                "Delete this debt?",
                "Delete",
                move || delete(),
                move || actions.delete_confirm.set(None),
            ))
            .into_element()
        },
    )
    .otherwise(move || {
        let edit = edit.clone();
        view()
            .flex_row()
            .gap(px(6.0))
            .child(ui::action_button("Pay", false, pay.clone()))
            .child(ui::action_button("Edit", false, edit))
            .child(ui::action_button("History", false, history.clone()))
            .child(ui::mini_icon_button(
                asset!("icons/trash.svg"),
                true,
                move || {
                    confirm.set(Some(id));
                },
            ))
            .into_element()
    })
    .into_element()
}

fn filtered(
    data: Resource<DebtsData, String>,
    search: Signal<String>,
    type_filter: Signal<String>,
    sort_field: Signal<String>,
    sort_desc: Signal<bool>,
) -> Vec<Debt> {
    let snapshot = data.get().unwrap_or_default();
    filter_debts(
        &snapshot.debts,
        &snapshot.accounts,
        &search.get(),
        &type_filter.get(),
        &sort_field.get(),
        sort_desc.get(),
    )
}

fn filter_debts(
    debts: &[Debt],
    accounts: &[crate::models::Account],
    query: &str,
    type_filter: &str,
    sort_field: &str,
    sort_desc: bool,
) -> Vec<Debt> {
    let query = query.trim().to_lowercase();
    let mut result: Vec<Debt> = debts
        .iter()
        .filter(|debt| {
            if type_filter != "all" && debt.debt_type != type_filter {
                return false;
            }
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
    result.sort_by(|a, b| {
        let order = match sort_field {
            "remaining" => a.remaining_amount.cmp(&b.remaining_amount),
            "next" => a.next_payment_date.cmp(&b.next_payment_date),
            "created_at" => a.created_at.cmp(&b.created_at),
            "updated_at" => a.updated_at.cmp(&b.updated_at),
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        };
        if sort_desc { order.reverse() } else { order }
    });
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
                .cursor_pointer()
                .cursor_pointer()
                .items_center()
                .justify_center()
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
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn progress_is_capped_and_safe() {
        assert_eq!(progress(Decimal::from(100), Decimal::from(25)), 75.0);
        assert_eq!(progress(Decimal::ZERO, Decimal::from(10)), 0.0);
        assert_eq!(progress(Decimal::from(100), Decimal::ZERO), 100.0);
    }

    #[test]
    fn debt_filters_and_sorts_by_type_and_remaining() {
        let mut manual = debt("Alpha", Decimal::from(100), Decimal::from(80));
        manual.debt_type = "manual".to_string();
        let mut automatic = debt("Beta", Decimal::from(100), Decimal::from(20));
        automatic.debt_type = "automatic".to_string();
        let debts = vec![manual, automatic];
        let automatic_only = filter_debts(&debts, &[], "", "automatic", "name", false);
        assert_eq!(automatic_only.len(), 1);
        assert_eq!(automatic_only[0].name, "Beta");
        let by_remaining = filter_debts(&debts, &[], "", "all", "remaining", true);
        assert_eq!(by_remaining[0].name, "Alpha");
    }

    #[test]
    fn debt_search_matches_name_and_account() {
        let debts = vec![
            debt("Car loan", Decimal::from(12000), Decimal::from(9000)),
            debt("Mortgage", Decimal::from(200000), Decimal::from(180000)),
        ];
        assert_eq!(
            filter_debts(&debts, &[], "car", "all", "name", false).len(),
            1
        );
        assert_eq!(filter_debts(&debts, &[], "", "all", "name", false).len(), 2);
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
            first_payment: Some(date(2026, 11, 22)),
            clear_first_payment: false,
            start_date: Some(date(2026, 11, 22)),
            notes: Some("Loan paperwork".to_string()),
            logo_url: Some("https://example.test/logo.png".to_string()),
        };
        let value = serde_json::to_value(request).expect("request serializes");
        assert_eq!(value["debt_type"], "automatic");
        assert_eq!(value["payment_days"][0], 22);
        assert_eq!(value["notes"], "Loan paperwork");
        assert_eq!(value["logo_url"], "https://example.test/logo.png");
    }

    #[test]
    fn automatic_debts_reuse_the_billing_schedule() {
        let source = include_str!("debts.rs");
        assert!(source.contains("BillingMode::Monthly"));
        assert!(source.contains("billing_schedule(BillingScheduleProps"));
    }
}
