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
use crate::components::select_menu::{SelectMenuProps, SelectOption, select_menu};
use crate::components::sort_menu::{SortOption, sort_menu};
use crate::components::string_select::{StringSelectProps, string_select};
use crate::components::toggle::toggle;
use crate::format::{format_currency, format_date};
use crate::models::{Account, Rates, Transaction};
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
    let sort_desc = signal(true);
    let sort_field = signal("date".to_string());
    let data = resource(
        move || {
            Some((
                sort_field.get(),
                sort_desc.get(),
                crate::ui::data_revision(),
            ))
        },
        move |(field, descending, _version), _| {
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

    let open_form = std::rc::Rc::new(open_form);

    let open_header = open_form.clone();

    let intent_open = open_form.clone();
    effect(move || {
        if intent.get() == Some(crate::Route::Transactions) {
            intent.set(None);
            let open = intent_open.clone();
            open();
        }
    });

    let selection_mode = signal(false);
    let selected = signal(Vec::<Uuid>::new());
    let open_bulk = move || {
        selected.set(Vec::new());
        selection_mode.set(!selection_mode.get());
    };

    let month_only = signal(false);
    let category_filter = signal(String::new());
    let price_min = signal(String::new());
    let price_max = signal(String::new());
    let list_view = signal(crate::view_state::transactions_list());
    effect(move || crate::view_state::save_transactions_list(list_view.get()));
    view()
        .key(format!("{}-{}", sort_field.get(), sort_desc.get()))
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
                bulk_bar(
                    data,
                    server_signal,
                    token_signal,
                    modal,
                    selected,
                    selection_mode,
                )
                .into_element()
            },
        ))
        .child(
            view()
                .mt(px(2.5))
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
                            compact,
                        )
                    })
                })),
        )
        .into_element()
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
                ButtonProps::new("Add Transaction")
                    .icon(asset!("icons/plus.svg"))
                    .min_width(155.0)
                    .on_press(on_add),
            ))
            .into_element(),
        compact,
        true,
        180.0,
    )
}

#[allow(clippy::too_many_arguments)]
/// Opens the full transaction form over whichever page requested it.
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
            async move { api::load_transactions(server, token, currency).await }
        }
    });
    let form_signals = TxForm {
        open: signal(true),
        editing: signal(None::<Uuid>),
        name: signal(String::new()),
        amount: signal(String::new()),
        currency: signal(currency.clone()),
        date: signal(
            date.unwrap_or_else(|| Local::now().date_naive())
                .to_string(),
        ),
        account_id: signal(None::<Uuid>),
        category_id: signal(None::<Uuid>),
        error: signal(String::new()),
        saving: signal(false),
        notes: signal(String::new()),
        logo_url: signal(String::new()),
        logo_results: signal(Vec::<(String, String)>::new()),
        logo_loading: signal(false),
        logo_error: signal(String::new()),
    };
    let server_signal = signal(server);
    let token_signal = signal(token);
    modal.open(move || {
        form(
            data,
            form_signals,
            server_signal,
            token_signal,
            currency.clone(),
            modal,
        )
        .into_element()
    });
}

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
                    Ok(saved) => {
                        form_open.set(false);
                        editing.set(None);
                        if let Some(mut snapshot) = data.get() {
                            if let Some(item) = snapshot
                                .transactions
                                .iter_mut()
                                .find(|item| item.id == saved.id)
                            {
                                *item = saved.clone();
                            } else {
                                snapshot.transactions.push(saved.clone());
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
        "Edit Transaction"
    } else {
        "Add Transaction"
    };
    let subtitle = "";
    let modal_for_close = modal;
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(title, subtitle, move || {
            form_open.set(false);
            editing.set(None);
            modal_for_close.close();
        })
        .width(512.0)
        .body(
            view()
                .pl(px(22.0))
                .pr(px(34.0))
                .pt(px(24.0))
                .pb(px(24.0))
                .gap(px(16.0))
                .child(ui::required_field("Name", name, ""))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::row_item(ui::required_field("Amount", amount, "")))
                        .child(
                            view().flex_grow(1.6).basis(px(0.0)).min_w(px(0.0)).child(
                                string_select(StringSelectProps::new(
                                    "Currency *",
                                    form_currency,
                                    crate::currencies::CURRENCIES
                                        .iter()
                                        .map(|(code, label)| {
                                            ((*code).to_string(), (*label).to_string())
                                        })
                                        .collect(),
                                )),
                            ),
                        ),
                )
                .child(
                    view().w_full().flex_row().items_end().gap(px(8.0)).child(
                        view()
                            .flex_grow(1.0)
                            .min_w(px(0.0))
                            .child(ui::date_field("Date *", date)),
                    ),
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
                    crate::components::textarea::text_area_with_height(notes, "", 120.0)
                        .into_element(),
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
                            text(move || if saving.get() { "Saving..." } else { "Save" })
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
                                .child(ui::row_item(ui::field("Min", price_min, "0.00")))
                                .child(ui::row_item(ui::field("Max", price_max, "999.99"))),
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
                asset!("icons/square.svg"),
                "Enter selection mode",
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
    compact: Signal<bool>,
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
    view()
        .key(list_key)
        .w_full()
        .mt(px(-5.5))
        .flex_col()
        .gap(px(16.0))
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
                    list_view,
                    compact,
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
    list_view: Signal<bool>,
    compact: Signal<bool>,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
) -> impl IntoElement {
    let count = ui::count_label(group.items.len(), "transaction", "transactions");
    let total = format_currency(group.total, &currency);
    let list_items = group.items.clone();
    let grid_items = group.items.clone();
    let list_currency = currency.clone();
    let grid_currency = currency.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(5.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .py(px(10.0))
                .rounded(px(8.0))
                .bg(move || theme().muted.opacity(0.5))
                .child(
                    text(format!("{} · {count}", group.label))
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .font_weight(FontWeight::SEMIBOLD)
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
                    .font_weight(FontWeight::BOLD)
                    .text_color(move || theme().foreground),
                ),
        )
        .child(
            when(
                move || list_view.get(),
                move || {
                    let items = list_items.clone();
                    let currency = list_currency.clone();
                    view()
                        .w_full()
                        .flex_col()
                        .gap(px(8.0))
                        .child(each(
                            move || items.clone(),
                            |transaction| {
                                format!("{}-{:?}", transaction.id, transaction.updated_at)
                            },
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
                        ))
                        .into_element()
                },
            )
            .otherwise(move || {
                let items = grid_items.clone();
                let currency = grid_currency.clone();
                view()
                    .w_full()
                    .grid()
                    .grid_template_columns(SharedString::from(if compact.get() {
                        "repeat(1, minmax(0, 1fr))"
                    } else {
                        "repeat(3, minmax(0, 1fr))"
                    }))
                    .gap(px(12.0))
                    .child(each(
                        move || items.clone(),
                        |transaction| format!("{}-{:?}", transaction.id, transaction.updated_at),
                        move |transaction, _| {
                            grid_card(
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
                    ))
                    .into_element()
            }),
        )
}

fn grid_card(
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
    let accent = account_color(data, transaction.account_id);
    let accent_badge = accent.clone();
    let accent_icon = accent.clone();
    let accent_text = accent.clone();
    let logo = transaction.logo_url.clone();
    let action_transaction = transaction.clone();
    let name = transaction.name;
    let date = format_date(transaction.date);
    let rates = data.get().unwrap_or_default().rates;
    let amount_value = transaction.amount;
    let amount_currency = transaction.currency.clone();
    let select_id = transaction.id;
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .p(px(16.0))
        .rounded(px(8.0))
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
        .cursor_pointer()
        .on_press(move || {
            if selection_mode.get() {
                let mut values = selected.get();
                crate::components::selection_bar::toggle(&mut values, select_id);
                selected.set(values);
            }
        })
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .gap(px(12.0))
                .child(
                    view()
                        .h(px(40.0))
                        .w(px(40.0))
                        .flex_shrink_0()
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
                                    .size(px(18.0))
                                    .tint_color(move || theme().muted_foreground)
                                    .into_element()
                            }),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .flex_grow(1.0)
                        .child(
                            text(name)
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .font_weight(FontWeight::MEDIUM)
                                .number_of_lines(1)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(date)
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            view()
                                .self_start()
                                .flex_row()
                                .items_center()
                                .gap(px(4.0))
                                .mt(px(4.0))
                                .px(px(8.0))
                                .h(px(20.0))
                                .rounded(px(999.0))
                                .bg(move || {
                                    rgb(crate::theme::parse_hex(&accent_badge)).opacity(0.13)
                                })
                                .child(
                                    image(asset!("icons/card.svg")).size(px(14.0)).tint_color(
                                        move || rgb(crate::theme::parse_hex(&accent_icon)),
                                    ),
                                )
                                .child(text(account).text_size(px(11.0)).text_color(move || {
                                    rgb(crate::theme::parse_hex(&accent_text))
                                })),
                        ),
                ),
        )
        .child(view().w_full().child(converted_amount(
            amount_value,
            &amount_currency,
            &currency,
            &rates,
            16.0,
            FontWeight::SEMIBOLD,
            true,
        )))
        .child(when(
            move || !selection_mode.get(),
            move || {
                view()
                    .w_full()
                    .flex_row()
                    .justify_end()
                    .gap(px(4.0))
                    .pt(px(2.0))
                    .border_t(px(1.0))
                    .border_color(move || theme().border.opacity(0.4))
                    .child(transaction_actions(
                        action_transaction.clone(),
                        actions,
                        data,
                        server,
                        token,
                        currency.clone(),
                    ))
                    .into_element()
            },
        ))
        .into_element()
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
    let accent = account_color(data, transaction.account_id);
    let accent_badge = accent.clone();
    let accent_text = accent.clone();
    let accent_icon = accent.clone();
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
        .rounded(px(8.0))
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
        .gap(px(16.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(16.0))
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
                                    .size(px(16.0))
                                    .tint_color(move || theme().muted_foreground)
                                    .into_element()
                            }),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .min_w(px(0.0))
                        .child(
                            text(name)
                                .text_size(px(16.0))
                                .line_height(px(24.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text(date)
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            view()
                                .self_start()
                                .flex_row()
                                .items_center()
                                .gap(px(4.0))
                                .mt(px(4.0))
                                .px(px(8.0))
                                .h(px(20.0))
                                .rounded(px(4.0))
                                .bg(move || {
                                    rgb(crate::theme::parse_hex(&accent_badge)).opacity(0.13)
                                })
                                .child(
                                    image(asset!("icons/card.svg")).size(px(16.0)).tint_color(
                                        move || rgb(crate::theme::parse_hex(&accent_icon)),
                                    ),
                                )
                                .child(text(account).text_size(px(12.0)).text_color(move || {
                                    rgb(crate::theme::parse_hex(&accent_text))
                                })),
                        ),
                ),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                .child(view().mt(px(7.0)).child(converted_amount(
                    amount_value,
                    &amount_currency,
                    &currency,
                    &rates,
                    16.0,
                    FontWeight::MEDIUM,
                    true,
                )))
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

fn bulk_bar(
    data: Resource<TransactionsData, String>,
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
            .transactions
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
                let _ = api::delete_transaction(server.clone(), token.clone(), id).await;
            }
            selected.set(Vec::new());
            selection_mode.set(false);
            data.refetch();
            crate::ui::bump_data();
            busy.set(false);
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
                        format!("Delete {} transactions?", count()),
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
                            .child(ui::action_button("Delete", true, move || {
                                delete();
                                modal.close();
                            }))
                            .child(ui::action_button("Cancel", false, move || modal.close()))
                            .into_element(),
                    ),
                )
                .into_element()
            });
        }
    };
    view()
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
                    .show_label(false),
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
                    crate::ui::bump_data();
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
            .child(ui::row_icon_button_labeled(
                asset!("icons/notes.svg"),
                "Notes",
                false,
                open_notes,
            ))
            .child(ui::row_icon_button_labeled(
                asset!("icons/pencil.svg"),
                "Edit transaction",
                false,
                edit,
            ))
            .child(ui::row_icon_button_labeled(
                asset!("icons/trash.svg"),
                "Delete transaction",
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

fn account_color(data: Resource<TransactionsData, String>, account_id: Uuid) -> String {
    data.get()
        .unwrap_or_default()
        .accounts
        .into_iter()
        .find(|account| account.id == account_id)
        .map_or_else(
            || "#6b7280".to_string(),
            |account| account.color.unwrap_or_else(|| "#6b7280".to_string()),
        )
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
