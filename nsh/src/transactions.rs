//! Transactions: one-time expenses grouped by month.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, theme};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::api::{self, TransactionsData};
use crate::auth::AuthSession;
use crate::components::button::{ButtonProps, button as shared_button};
use crate::components::confirm::{ConfirmProps, confirm_row};
use crate::components::converted_amount::converted_amount;
use crate::components::filter_menu::filter_menu;
use crate::components::list_toolbar::list_toolbar;
use crate::components::page_header::page_header as shared_page_header;
use crate::components::sort_menu::{SortOption, sort_menu};
use crate::components::string_select::{StringSelectProps, string_select};
use crate::components::toggle::toggle;
use crate::format::{format_currency, format_date};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

#[derive(Clone, Copy)]
struct TxForm {
    open: Signal<bool>,
    editing: Signal<Option<Uuid>>,
    name: Signal<String>,
    amount: Signal<String>,
    currency: Signal<String>,
    date: Signal<String>,
    account_id: Signal<Option<Uuid>>,
    category_id: Signal<Option<Uuid>>,
    error: Signal<String>,
    saving: Signal<bool>,
    notes: Signal<String>,
    logo_url: Signal<String>,
    logo_results: Signal<Vec<(String, String)>>,
    logo_loading: Signal<bool>,
    logo_error: Signal<String>,
}

#[derive(Clone, Copy)]
struct TxActions {
    form: TxForm,
    modal: ui::ModalHost,
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

pub fn transactions(
    auth: AuthSession,
    modal: ui::ModalHost,
    intent: Signal<Option<crate::Route>>,
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
    let sort_desc = signal(true);
    let sort_field = signal("date".to_string());
    let data = resource(
        move || Some((sort_field.get(), sort_desc.get())),
        move |(field, descending), _| {
            let _ = (field, descending);
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
    let form_currency = signal(if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    });
    let today = Local::now().date_naive();
    let date = signal(today.to_string());
    let account_id = signal(None::<Uuid>);
    let category_id = signal(None::<Uuid>);
    let form_error = signal(String::new());
    let saving = signal(false);
    let notes = signal(String::new());
    let logo_url = signal(String::new());
    let logo_results = signal(Vec::<(String, String)>::new());
    let logo_loading = signal(false);
    let logo_error = signal(String::new());
    let delete_confirm = signal(None::<Uuid>);
    let busy = signal(None::<Uuid>);
    let action_error = signal(String::new());
    let form_signals = TxForm {
        open: form_open,
        editing,
        name,
        amount,
        currency: form_currency,
        date,
        account_id,
        category_id,
        error: form_error,
        saving,
        notes,
        logo_url,
        logo_results,
        logo_loading,
        logo_error,
    };
    let actions = TxActions {
        form: form_signals,
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
        amount.set(String::new());
        form_currency.set(currency.clone());
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
        let dialog_currency = currency.clone();
        modal.open(move || {
            form(
                data,
                form_signals,
                server_signal,
                token_signal,
                dialog_currency.clone(),
                modal,
            )
            .into_element()
        });
    };

    if intent.get() == Some(crate::Route::Transactions) {
        intent.set(None);
        open_form();
    }

    let selection_mode = signal(false);
    let selected = signal(Vec::<Uuid>::new());
    let open_bulk = move || {
        if selection_mode.get() {
            modal.open(move || {
                bulk_modal(
                    data,
                    server_signal,
                    token_signal,
                    modal,
                    selected,
                    selection_mode,
                )
                .into_element()
            });
        } else {
            selected.set(Vec::new());
            selection_mode.set(true);
        }
    };

    let month_only = signal(false);
    let category_filter = signal(String::new());
    let price_min = signal(String::new());
    let price_max = signal(String::new());
    let list_view = signal(true);
    view()
        .key(format!("{}-{}", sort_field.get(), sort_desc.get()))
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(header(open_form))
        .child(toolbar(
            data,
            search,
            list_view,
            month_only,
            sort_desc,
            sort_field,
            category_filter,
            price_min,
            price_max,
            open_bulk,
        ))
        .child(when(
            move || selection_mode.get(),
            move || {
                crate::components::selection_bar::selection_bar(
                    move || selected.get().len(),
                    move || {
                        let mut values = selected.get();
                        crate::components::selection_bar::add_all(
                            &mut values,
                            data.get()
                                .unwrap_or_default()
                                .transactions
                                .into_iter()
                                .map(|item| item.id),
                        );
                        selected.set(values);
                    },
                    move || selected.set(Vec::new()),
                    move || {
                        modal.open(move || {
                            bulk_modal(
                                data,
                                server_signal,
                                token_signal,
                                modal,
                                selected,
                                selection_mode,
                            )
                            .into_element()
                        });
                    },
                    move || {
                        selected.set(Vec::new());
                        selection_mode.set(false);
                    },
                )
                .into_element()
            },
        ))
        .child(when(loading, || loading_card()).otherwise(move || {
            let value = render_currency.clone();
            when(has_error, move || {
                error_card(&data.error().unwrap_or_default(), data)
            })
            .otherwise(move || {
                content(
                    data,
                    search,
                    value.clone(),
                    actions,
                    server_signal,
                    token_signal,
                    list_view,
                    month_only,
                    sort_desc,
                    sort_field,
                    category_filter,
                    price_min,
                    price_max,
                    selection_mode,
                    selected,
                )
            })
        }))
        .into_element()
}

fn header(on_add: impl Fn() + 'static) -> impl IntoElement {
    shared_page_header(
        TITLE,
        SUBTITLE,
        view()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(px(8.0))
            .child(shared_button(
                ButtonProps::new("Add Transaction")
                    .icon(asset!("icons/plus.svg"))
                    .on_press(on_add),
            ))
            .into_element(),
    )
}

#[allow(clippy::too_many_arguments)]
fn form(
    data: Resource<TransactionsData, String>,
    form: TxForm,
    server: Signal<String>,
    token: Signal<String>,
    _currency: String,
    modal: ui::ModalHost,
) -> impl IntoElement {
    let TxForm {
        open: form_open,
        editing,
        name,
        amount,
        currency: form_currency,
        date,
        account_id,
        category_id,
        error: form_error,
        saving,
        notes,
        logo_url,
        logo_results,
        logo_loading,
        logo_error,
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
                currency: form_currency.get(),
                date: date_value,
                account_id: account_value,
                category_id: category_id.get(),
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
                    api::update_transaction(server_value, token_value, id, &request).await
                } else {
                    api::create_transaction(server_value, token_value, &request).await
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

    let title = if editing.get().is_some() {
        "Edit Transaction"
    } else {
        "Add Transaction"
    };
    let subtitle = if editing.get().is_some() {
        "Update the amount, date or account."
    } else {
        "Record a one-time expense."
    };
    let modal_for_close = modal;
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(title, subtitle, move || {
            form_open.set(false);
            editing.set(None);
            modal_for_close.close();
        })
        .width(508.0)
        .body(
            view()
                .p(px(24.0))
                .gap(px(16.0))
                .child(ui::required_field("Name", name, "Groceries"))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::required_field("Amount", amount, "42.50"))
                        .child(string_select(StringSelectProps::new(
                            "Currency *",
                            form_currency,
                            crate::currencies::CURRENCIES
                                .iter()
                                .map(|(code, label)| ((*code).to_string(), (*label).to_string()))
                                .collect(),
                        ))),
                )
                .child(ui::date_field("Date *", date))
                .child(ui::choice_picker(
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
                .child(crate::components::field::field(
                    crate::components::field::FieldProps::new("Notes"),
                    crate::components::textarea::text_area(notes, "Add notes…").into_element(),
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
                )
                .child(
                    button()
                        .cursor_pointer()
                        .items_center()
                        .justify_center()
                        .h(px(36.0))
                        .px(px(14.0))
                        .rounded(px(7.0))
                        .bg(move || theme().primary)
                        .disabled(move || saving.get())
                        .disabled_style(|state| state.opacity(0.5))
                        .on_press(move || save())
                        .child(
                            text(move || if saving.get() { "Saving..." } else { "Save" })
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().primary_foreground),
                        ),
                )
                .into_element(),
        ),
    )
}

fn toolbar(
    data: Resource<TransactionsData, String>,
    search: Signal<String>,
    list_view: Signal<bool>,
    month_only: Signal<bool>,
    sort_desc: Signal<bool>,
    sort_field: Signal<String>,
    category_filter: Signal<String>,
    price_min: Signal<String>,
    price_max: Signal<String>,
    open_bulk: impl Fn() + 'static,
) -> impl IntoElement {
    let active = move || {
        usize::from(!category_filter.get().is_empty())
            + usize::from(!price_min.get().trim().is_empty())
            + usize::from(!price_max.get().trim().is_empty())
            + usize::from(month_only.get())
    };
    let clear = move || {
        category_filter.set(String::new());
        price_min.set(String::new());
        price_max.set(String::new());
        month_only.set(false);
    };
    let category_options = move || {
        let mut options = vec![(String::new(), "All categories".to_string())];
        options.extend(
            data.get()
                .unwrap_or_default()
                .categories
                .into_iter()
                .map(|category| (category.id.to_string(), category.name)),
        );
        options
    };
    list_toolbar(
        search,
        list_view,
        view()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            .child(filter_menu(
                move || active(),
                move || {
                    view()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    text("Filters")
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(ui::action_button("Clear", false, clear)),
                        )
                        .child(string_select(StringSelectProps::new(
                            "Category",
                            category_filter,
                            category_options(),
                        )))
                        .child(
                            view()
                                .flex_row()
                                .gap(px(8.0))
                                .child(ui::field("Min", price_min, "0.00"))
                                .child(ui::field("Max", price_max, "999.99")),
                        )
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    text("This month only")
                                        .text_size(px(12.0))
                                        .text_color(move || theme().foreground),
                                )
                                .child(toggle(
                                    month_only.get(),
                                    "This month only".to_string(),
                                    move |value| month_only.set(value),
                                )),
                        )
                        .into_element()
                },
            ))
            .child(sort_menu(
                sort_field,
                sort_desc,
                vec![
                    SortOption::new("date", "Date"),
                    SortOption::new("amount", "Price"),
                    SortOption::new("name", "Name"),
                    SortOption::new("created_at", "Created"),
                    SortOption::new("updated_at", "Last modified"),
                ],
            ))
            .child(crate::components::button::icon_button(
                asset!("icons/check.svg"),
                "Select transactions",
                false,
                open_bulk,
            ))
            .into_element(),
    )
}

fn content(
    data: Resource<TransactionsData, String>,
    search: Signal<String>,
    currency: String,
    actions: TxActions,
    server: Signal<String>,
    token: Signal<String>,
    list_view: Signal<bool>,
    month_only: Signal<bool>,
    sort_desc: Signal<bool>,
    sort_field: Signal<String>,
    category_filter: Signal<String>,
    price_min: Signal<String>,
    price_max: Signal<String>,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
) -> AnyElement {
    let list_key = format!("{}-{}", sort_field.get(), sort_desc.get());
    let groups = move || {
        let snapshot = data.get().unwrap_or_default();
        let mut groups = group_transactions(
            &snapshot.transactions,
            &snapshot.accounts,
            &snapshot.rates,
            &search.get(),
            sort_desc.get(),
            (!category_filter.get().is_empty())
                .then(|| Uuid::parse_str(&category_filter.get()).ok())
                .flatten(),
            price_min.get().parse::<Decimal>().ok(),
            price_max.get().parse::<Decimal>().ok(),
        );
        if month_only.get() {
            let today = Local::now().date_naive();
            let key = format!("{:04}-{:02}", today.year(), today.month());
            groups.retain(|group| group.key == key);
        }
        for group in &mut groups {
            sort_transactions(&mut group.items, &sort_field.get(), sort_desc.get());
        }
        groups
    };
    let grid = list_view.get() == false;
    view()
        .key(list_key)
        .w_full()
        .flex_col()
        .when(grid, |view| view.flex_row().flex_wrap().gap(px(16.0)))
        .when(!grid, |view| view.gap(px(16.0)))
        .child(each(
            groups,
            move |group| format!("{}-{}-{}", group.key, sort_field.get(), sort_desc.get()),
            move |group, _| {
                group_card(
                    group.get(),
                    data,
                    currency.clone(),
                    actions,
                    server,
                    token,
                    grid,
                    selection_mode,
                    selected,
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
    grid: bool,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
) -> impl IntoElement {
    let count = ui::count_label(group.items.len(), "transaction", "transactions");
    let total = format_currency(group.total, &currency);
    view()
        .w_full()
        .when(grid, |view| view.w(px(360.0)).flex_grow(1.0))
        .flex_col()
        .gap(px(8.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(20.0))
                .py(px(12.0))
                .rounded(px(8.0))
                .bg(move || theme().muted)
                .child(
                    text(format!("{} · {count}", group.label))
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(move || {
                        if crate::ui::privacy_active() {
                            "••••".to_string()
                        } else {
                            total.clone()
                        }
                    })
                    .text_size(px(14.0))
                    .font_family(design::MONO)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(move || theme().foreground),
                ),
        )
        .child(view().w_full().flex_col().gap(px(8.0)).child(each(
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
                    selection_mode,
                    selected,
                )
            },
        )))
}

fn row(
    transaction: Transaction,
    data: Resource<TransactionsData, String>,
    currency: String,
    actions: TxActions,
    server: Signal<String>,
    token: Signal<String>,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
) -> impl IntoElement {
    let account = account_name(data, transaction.account_id);
    let action_transaction = transaction.clone();
    let logo = transaction.logo_url.clone();
    let name = transaction.name;
    let date = format_date(transaction.date);
    let rates = data.get().unwrap_or_default().rates;
    let amount_value = transaction.amount;
    let amount_currency = transaction.currency.clone();
    let select_id = transaction.id;
    let action_selected = action_transaction.clone();
    view()
        .cursor_pointer()
        .w_full()
        .flex_row()
        .items_center()
        .justify_between()
        .p(px(16.0))
        .min_h(px(128.0))
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
                        .h(px(40.0))
                        .w(px(40.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(999.0))
                        .bg(move || theme().muted)
                        .child(
                            when(logo.is_some(), move || {
                                image(logo.clone().unwrap_or_default())
                                    .size(px(22.0))
                                    .into_element()
                            })
                            .otherwise(|| {
                                image(asset!("icons/receipt.svg"))
                                    .size(px(17.0))
                                    .tint_color(move || theme().muted_foreground)
                                    .into_element()
                            }),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(3.0))
                        .min_w(px(0.0))
                        .child(
                            text(name)
                                .text_size(px(15.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(date)
                                .text_size(px(13.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            view()
                                .self_start()
                                .flex_row()
                                .items_center()
                                .gap(px(5.0))
                                .px(px(7.0))
                                .py(px(2.0))
                                .rounded(px(5.0))
                                .bg(move || theme().muted)
                                .child(
                                    image(asset!("icons/card.svg"))
                                        .size(px(11.0))
                                        .tint_color(move || theme().muted_foreground),
                                )
                                .child(
                                    text(account)
                                        .text_size(px(10.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        ),
                ),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                .child(converted_amount(
                    amount_value,
                    &amount_currency,
                    &currency,
                    &rates,
                    15.0,
                    FontWeight::SEMIBOLD,
                    true,
                ))
                .child(when(
                    move || !selection_mode.get(),
                    move || {
                        transaction_actions(
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

fn bulk_modal(
    data: Resource<TransactionsData, String>,
    server: Signal<String>,
    token: Signal<String>,
    modal: ui::ModalHost,
    selected: Signal<Vec<Uuid>>,
    selection_mode: Signal<bool>,
) -> impl IntoElement {
    let busy = signal(false);
    let error = signal(String::new());
    let confirm_delete = signal(false);
    let bulk_category = signal(None::<Uuid>);
    let toggle = move |id: Uuid| {
        let mut values = selected.get();
        crate::components::selection_bar::toggle(&mut values, id);
        selected.set(values);
    };
    let all_ids = move || {
        data.get()
            .unwrap_or_default()
            .transactions
            .into_iter()
            .map(|item| item.id)
            .collect::<Vec<_>>()
    };
    let select_all = move || {
        let mut values = selected.get();
        for id in all_ids() {
            if !values.contains(&id) {
                values.push(id);
            }
        }
        selected.set(values);
    };
    let clear = move || selected.set(Vec::new());
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
                    let _ = api::update_transaction(
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
                busy.set(false);
            });
        }
    };
    let delete = move || {
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
                let _ = api::delete_transaction(server.clone(), token.clone(), id).await;
            }
            selected.set(Vec::new());
            selection_mode.set(false);
            confirm_delete.set(false);
            data.refetch();
            busy.set(false);
        });
    };
    let rows = move || {
        data.get()
            .unwrap_or_default()
            .transactions
            .into_iter()
            .map(|transaction| {
                let id = transaction.id;
                let name = transaction.name;
                let date = format_date(transaction.date);
                let is_selected = selected.get().contains(&id);
                view()
                    .w_full()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .p(px(8.0))
                    .rounded(px(6.0))
                    .bg(move || {
                        if is_selected {
                            theme().accent
                        } else {
                            theme().card.opacity(0.0)
                        }
                    })
                    .cursor_pointer()
                    .on_press(move || toggle(id))
                    .child(
                        view()
                            .flex_col()
                            .child(
                                text(name)
                                    .text_size(px(12.0))
                                    .text_color(move || theme().foreground),
                            )
                            .child(
                                text(date)
                                    .text_size(px(10.0))
                                    .text_color(move || theme().muted_foreground),
                            ),
                    )
                    .into_element()
            })
            .collect::<Vec<_>>()
    };
    let count = move || selected.get().len();
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            "Bulk actions",
            "Select transactions, then delete them together.",
            move || modal.close(),
        )
        .width(520.0)
        .body(
            view()
                .p(px(20.0))
                .gap(px(10.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap(px(8.0))
                        .child(
                            text(move || format!("{} selected", count()))
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            view()
                                .flex_row()
                                .gap(px(8.0))
                                .child(ui::action_button("Select all", false, select_all))
                                .child(ui::action_button("Clear", false, clear)),
                        ),
                )
                .child(
                    scroll()
                        .max_h(px(320.0))
                        .w_full()
                        .child(view().w_full().flex_col().gap(px(2.0)).children(rows())),
                )
                .child(ui::choice_picker(
                    "Move selected to category",
                    move || {
                        data.get()
                            .unwrap_or_default()
                            .categories
                            .into_iter()
                            .map(|category| (category.id, category.name))
                            .collect()
                    },
                    bulk_category,
                ))
                .child(
                    when(
                        move || confirm_delete.get(),
                        move || {
                            view()
                                .w_full()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .gap(px(8.0))
                                .p(px(10.0))
                                .rounded(px(7.0))
                                .bg(move || theme().destructive.opacity(0.1))
                                .child(
                                    text(move || format!("Delete {} transactions?", count()))
                                        .text_size(px(11.0))
                                        .text_color(move || theme().destructive),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .gap(px(6.0))
                                        .child(ui::action_button("Cancel", false, move || {
                                            confirm_delete.set(false)
                                        }))
                                        .child(ui::action_button("Delete", true, delete)),
                                )
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0)).into_element()),
                )
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
                .child(ui::action_button("Apply category", false, apply_category))
                .child(ui::action_button("Delete", true, move || {
                    confirm_delete.set(true)
                }))
                .into_element(),
        ),
    )
}

fn notes_form(
    id: Uuid,
    name: String,
    notes: Signal<String>,
    saving: Signal<bool>,
    error: Signal<String>,
    server: Signal<String>,
    token: Signal<String>,
    data: Resource<TransactionsData, String>,
    modal: ui::ModalHost,
) -> impl IntoElement {
    let save = move || {
        saving.set(true);
        error.set(String::new());
        let value = notes.get();
        let server = server.get();
        let token = token.get();
        spawn(async move {
            match api::update_transaction(server, token, id, &serde_json::json!({ "notes": value }))
                .await
            {
                Ok(_) => {
                    data.refetch();
                    modal.close();
                }
                Err(message) => error.set(message),
            }
            saving.set(false);
        });
    };
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            format!("Notes: {name}"),
            "Add or update notes.",
            move || modal.close(),
        )
        .width(440.0)
        .body(
            view()
                .p(px(24.0))
                .gap(px(12.0))
                .child(crate::components::textarea::text_area(notes, "Add notes…"))
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
                .child(ui::action_button("Cancel", false, move || modal.close()))
                .child(ui::action_button(
                    if saving.get() {
                        "Saving…"
                    } else {
                        "Save notes"
                    },
                    false,
                    save,
                ))
                .into_element(),
        ),
    )
}

fn transaction_actions(
    transaction: Transaction,
    actions: TxActions,
    data: Resource<TransactionsData, String>,
    server: Signal<String>,
    token: Signal<String>,
    currency: String,
) -> AnyElement {
    let id = transaction.id;
    let form_signals = actions.form;
    let edit_transaction = transaction.clone();
    let edit = move || {
        form_signals.name.set(edit_transaction.name.clone());
        form_signals.amount.set(edit_transaction.amount.to_string());
        form_signals.currency.set(edit_transaction.currency.clone());
        form_signals.date.set(edit_transaction.date.to_string());
        form_signals
            .account_id
            .set(Some(edit_transaction.account_id));
        form_signals
            .category_id
            .set(Some(edit_transaction.category_id));
        form_signals
            .notes
            .set(edit_transaction.notes.clone().unwrap_or_default());
        form_signals
            .logo_url
            .set(edit_transaction.logo_url.clone().unwrap_or_default());
        form_signals.logo_results.set(Vec::new());
        form_signals.logo_error.set(String::new());
        form_signals.error.set(String::new());
        form_signals.saving.set(false);
        form_signals.editing.set(Some(id));
        form_signals.open.set(true);
        let dialog_currency = currency.clone();
        let modal = actions.modal;
        actions.modal.open(move || {
            form(
                data,
                form_signals,
                server,
                token,
                dialog_currency.clone(),
                modal,
            )
            .into_element()
        });
    };
    let delete = move || {
        actions.delete_confirm.set(None);
        actions.busy.set(Some(id));
        actions.error.set(String::new());
        let server_value = server.get();
        let token_value = token.get();
        spawn(async move {
            match api::delete_transaction(server_value, token_value, id).await {
                Ok(()) => {
                    crate::components::toast::success("Transaction deleted.");
                    data.refetch();
                }
                Err(message) => {
                    crate::components::toast::error(message.clone());
                    actions.error.set(message);
                }
            }
            actions.busy.set(None);
        });
    };
    let confirm = actions.delete_confirm;
    let notes_value = transaction.notes.clone().unwrap_or_default();
    let notes_name = transaction.name.clone();
    let notes_signal = actions.form.notes;
    let open_notes = move || {
        notes_signal.set(notes_value.clone());
        actions.form.error.set(String::new());
        actions.form.saving.set(false);
        let modal = actions.modal;
        let dialog_name = notes_name.clone();
        modal.open(move || {
            notes_form(
                id,
                dialog_name.clone(),
                notes_signal,
                actions.form.saving,
                actions.form.error,
                server,
                token,
                data,
                modal,
            )
            .into_element()
        });
    };
    when(
        move || actions.delete_confirm.get() == Some(id),
        move || {
            let delete = delete.clone();
            confirm_row(ConfirmProps::new(
                "Delete this transaction?",
                "Delete",
                move || delete(),
                move || actions.delete_confirm.set(None),
            ))
            .into_element()
        },
    )
    .otherwise(move || {
        let edit = edit.clone();
        let open_notes = open_notes.clone();
        view()
            .flex_row()
            .gap(px(4.0))
            .child(ui::mini_icon_button(
                asset!("icons/notes.svg"),
                false,
                open_notes,
            ))
            .child(ui::mini_icon_button(
                asset!("icons/pencil.svg"),
                false,
                edit,
            ))
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

fn sort_transactions(items: &mut [Transaction], field: &str, descending: bool) {
    items.sort_by(|a, b| {
        let ordering = match field {
            "amount" => a.amount.cmp(&b.amount),
            "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            "created_at" => a.created_at.cmp(&b.created_at),
            "updated_at" => a.updated_at.cmp(&b.updated_at),
            _ => a.date.cmp(&b.date),
        };
        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

fn group_transactions(
    transactions: &[Transaction],
    accounts: &[Account],
    rates: &Rates,
    query: &str,
    descending: bool,
    category: Option<Uuid>,
    price_min: Option<Decimal>,
    price_max: Option<Decimal>,
) -> Vec<TransactionGroup> {
    let mut groups: Vec<TransactionGroup> = Vec::new();
    for transaction in filter_transactions(transactions, accounts, query) {
        if category.is_some_and(|category| transaction.category_id != category)
            || price_min.is_some_and(|minimum| transaction.amount < minimum)
            || price_max.is_some_and(|maximum| transaction.amount > maximum)
        {
            continue;
        }
        let key = format!(
            "{:04}-{:02}",
            transaction.date.year(),
            transaction.date.month()
        );
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
    if descending {
        groups.sort_by(|a, b| b.key.cmp(&a.key));
    } else {
        groups.sort_by(|a, b| a.key.cmp(&b.key));
    }
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
            notes: None,
            logo_url: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn transaction_filters_respect_category_and_price_range() {
        let account_id = Uuid::new_v4();
        let category = Uuid::new_v4();
        let mut cheap = transaction("Coffee", account_id, Decimal::from(3), date(2026, 11, 22));
        cheap.category_id = category;
        let expensive = transaction("Rent", account_id, Decimal::from(900), date(2026, 11, 1));
        let items = vec![cheap, expensive];
        let groups = group_transactions(
            &items,
            &[],
            &Rates::default(),
            "",
            true,
            Some(category),
            None,
            None,
        );
        assert_eq!(groups[0].items.len(), 1);
        let groups = group_transactions(
            &items,
            &[],
            &Rates::default(),
            "",
            true,
            None,
            Some(Decimal::from(100)),
            None,
        );
        assert_eq!(groups[0].items.len(), 1);
        assert_eq!(groups[0].items[0].name, "Rent");
    }

    #[test]
    fn transaction_sort_uses_amount_or_name() {
        let account_id = Uuid::new_v4();
        let mut items = vec![
            transaction("Coffee", account_id, Decimal::from(3), date(2026, 11, 22)),
            transaction("Rent", account_id, Decimal::from(900), date(2026, 10, 1)),
        ];
        sort_transactions(&mut items, "amount", true);
        assert_eq!(items[0].name, "Rent");
        sort_transactions(&mut items, "name", false);
        assert_eq!(items[0].name, "Coffee");
    }

    #[test]
    fn transactions_group_by_month_and_sort_newest_first() {
        let account_id = Uuid::new_v4();
        let items = vec![
            transaction("Coffee", account_id, Decimal::from(3), date(2026, 11, 22)),
            transaction("Rent", account_id, Decimal::from(900), date(2026, 10, 1)),
            transaction("Lunch", account_id, Decimal::from(12), date(2026, 11, 20)),
        ];
        let groups = group_transactions(&items, &[], &Rates::default(), "", true, None, None, None);
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
            transaction(
                "Groceries",
                account_id,
                Decimal::from(42),
                date(2026, 11, 22),
            ),
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
            notes: None,
            logo_url: None,
        };
        let value = serde_json::to_value(request).expect("request serializes");
        assert_eq!(value["name"], json!("Groceries"));
        assert!(value.get("category_id").is_none());
    }
}
