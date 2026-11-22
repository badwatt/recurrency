//! Subscriptions list: searchable cards grouped by active state.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::api::{self, SubscriptionsData};
use crate::components::button::{button as shared_button, ButtonProps};
use crate::components::page_header::page_header as shared_page_header;
use crate::components::list_toolbar::{filter_button, list_toolbar, sort_button};
use crate::components::badge::{badge, BadgeProps, BadgeTone};
use crate::components::checkbox::checkbox;
use crate::auth::AuthSession;
use crate::format::format_currency;
use crate::models::{AnnualDate, Recurrence, Subscription};
use crate::theme as design;
use crate::ui;

#[derive(Serialize)]
pub struct CreateSubscriptionRequest {
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub start_date: NaiveDate,
    pub account_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub recurrence: Recurrence,
    pub is_active: bool,
}

#[derive(Serialize)]
struct ToggleSubscriptionRequest {
    is_active: bool,
}

#[derive(Clone, Copy)]
struct FormSignals {
    open: Signal<bool>,
    editing: Signal<Option<Uuid>>,
    name: Signal<String>,
    amount: Signal<String>,
    start_date: Signal<String>,
    currency: Signal<String>,
    interval: Signal<String>,
    days: Signal<Vec<u8>>,
    unit: Signal<String>,
    account_id: Signal<Option<Uuid>>,
    category_id: Signal<Option<Uuid>>,
    error: Signal<String>,
    saving: Signal<bool>,
    active: Signal<bool>,
}

#[derive(Clone, Copy)]
struct ListActions {
    form: FormSignals,
    modal: ui::ModalHost,
    delete_confirm: Signal<Option<Uuid>>,
    busy: Signal<Option<Uuid>>,
    error: Signal<String>,
}

const TITLE: &str = "Subscriptions";
const SUBTITLE: &str = "Track and manage recurring payments";

pub fn subscriptions(auth: AuthSession, modal: ui::ModalHost, intent: Signal<Option<crate::Route>>) -> AnyElement {
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
            async move { api::load_subscriptions(server, token, currency).await }
        },
    );
    let search = signal(String::new());
    let sort_desc = signal(false);
    let list_view = signal(false);
    let active_only = signal(false);
    let form_open = signal(false);
    let editing = signal(None::<Uuid>);
    let active = signal(true);
    let name = signal(String::new());
    let amount = signal(String::new());
    let start_date = signal(String::new());
    let form_currency = signal(if auth.user.currency.is_empty() { "EUR".to_string() } else { auth.user.currency.clone() });
    let interval = signal("1".to_string());
    let days = signal(Vec::<u8>::new());
    let unit = signal("month".to_string());
    let account_id = signal(None::<Uuid>);
    let category_id = signal(None::<Uuid>);
    let form_error = signal(String::new());
    let saving = signal(false);
    let delete_confirm = signal(None::<Uuid>);
    let busy = signal(None::<Uuid>);
    let action_error = signal(String::new());
    let form_signals = FormSignals {
        open: form_open,
        editing,
        name,
        amount,
        start_date,
        currency: form_currency,
        interval,
        days,
        unit,
        account_id,
        category_id,
        error: form_error,
        saving,
        active,
    };
    let actions = ListActions {
        form: form_signals,
        modal,
        delete_confirm,
        busy,
        error: action_error,
    };
    let server_signal = signal(server.clone());
    let token_signal = signal(token.clone());
    let loading = move || data.loading() && data.get().is_none() && !form_open.get();
    let has_error = move || data.error().is_some() && !form_open.get();
    let render_currency = currency.clone();

    let open_form = move || {
        let snapshot = data.get().unwrap_or_default();
        let today = Local::now().date_naive();
        name.set(String::new());
        amount.set(String::new());
        start_date.set(today.to_string());
        form_currency.set(if auth.user.currency.is_empty() { "EUR".to_string() } else { auth.user.currency.clone() });
        interval.set("1".to_string());
        days.set(vec![today.day() as u8]);
        unit.set("month".to_string());
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
        active.set(true);
        form_open.set(true);
        modal.open(move || {
            form(
                data,
                form_signals,
                server_signal,
                token_signal,
                modal,
            )
            .into_element()
        });
    };

    if intent.get() == Some(crate::Route::Subscriptions) {
        intent.set(None);
        open_form();
    }

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(header(open_form))
        .child(toolbar(data, search, sort_desc, list_view, active_only))
        .child(
            when(loading, || loading_card()).otherwise(move || {
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
                        sort_desc,
                        list_view,
                        active_only,
                    )
                })
            }),
        )
        .into_element()
}

fn header(on_add: impl Fn() + 'static) -> impl IntoElement {
    shared_page_header(
        TITLE,
        SUBTITLE,
        shared_button(ButtonProps::new("Add Subscription").icon(asset!("icons/plus.svg")).on_press(on_add)).into_element(),
    )
}

fn form(
    data: Resource<SubscriptionsData, String>,
    form: FormSignals,
    server: Signal<String>,
    token: Signal<String>,
    modal: ui::ModalHost,
) -> impl IntoElement {
    let FormSignals {
        open: form_open,
        editing,
        name,
        amount,
        start_date,
        currency,
        interval,
        days,
        unit,
        account_id,
        category_id,
        error: form_error,
        saving,
        active,
    } = form;
    let save = {
        let currency = currency.clone();
        move || {
            if saving.get() {
                return;
            }
            let Some(account_id_value) = account_id.get() else {
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
            let Ok(start_value) = NaiveDate::parse_from_str(start_date.get().trim(), "%Y-%m-%d") else {
                form_error.set("Use YYYY-MM-DD for the start date.".to_string());
                return;
            };
            let currency_value = currency.get().trim().to_uppercase();
            if currency_value.is_empty() {
                form_error.set("Enter a currency.".to_string());
                return;
            }
            let Ok(interval_value) = interval.get().trim().parse::<u16>() else {
                form_error.set("Interval must be a positive number.".to_string());
                return;
            };
            if interval_value == 0 {
                form_error.set("Interval must be a positive number.".to_string());
                return;
            }
            let day_values = days.get();
            if day_values.is_empty() {
                form_error.set("Select at least one charge day.".to_string());
                return;
            }
            let unit_value = unit.get();
            let request = CreateSubscriptionRequest {
                name: name_value,
                amount: amount_value,
                currency: currency_value.clone(),
                frequency: frequency_for(&unit_value).to_string(),
                start_date: start_value,
                account_id: account_id_value,
                category_id: category_id.get(),
                recurrence: recurrence_for(&unit_value, &day_values, start_value, interval_value),
                is_active: active.get(),
            };
            saving.set(true);
            form_error.set(String::new());
            let server_value = server.get();
            let token_value = token.get();
            let editing_value = editing.get();
            spawn(async move {
                let result = if let Some(id) = editing_value {
                    api::update_subscription(server_value, token_value, id, &request).await
                } else {
                    api::create_subscription(server_value, token_value, &request).await
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
        .w(px(508.0))
        .rounded(px(16.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .overflow_hidden()
        .max_h(px(700.0))
        .flex_col()
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .pb(px(16.0))
                .border_b(px(1.0))
                .border_color(move || theme().border)
                .gap(px(4.0))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            text(move || if editing.get().is_some() { "Edit Subscription" } else { "Add Subscription" })
                                .text_size(px(18.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(ui::icon_button(
                            asset!("icons/x.svg"),
                            false,
                            move || {
                                form_open.set(false);
                                editing.set(None);
                                modal.close();
                            },
                        )),
                )
                .child(
                    text(move || if editing.get().is_some() {
                        "Update the recurring expense or subscription."
                    } else {
                        "Add a new recurring expense or subscription."
                    })
                        .text_size(px(13.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            scroll()
                .max_h(px(480.0))
                .child(
                    view()
                        .w_full()
                        .p(px(24.0))
                        .gap(px(16.0))
                        .child(ui::field("Name", name, "Netflix, Spotify, etc."))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::field("Amount", amount, "9.99"))
                        .child(ui::field("Currency", currency, "EUR")),
                )
                .child(
                    view()
                        .w_full()
                        .rounded(px(10.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().muted.opacity(0.15))
                        .p(px(14.0))
                        .gap(px(10.0))
                        .child(
                            text("Billing schedule")
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(ui::date_field("First charge", start_date))
                        .child(
                            view()
                                .w_full()
                                .flex_row()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    view()
                                        .w(px(64.0))
                                        .child(
                                            input()
                                                .value(interval)
                                                .on_change(move |value: &str| interval.set(value.to_string()))
                                                .auto_fill(AutoFill::Off)
                                                .h(px(34.0))
                                                .w_full()
                                                .px(px(10.0))
                                                .rounded(px(7.0))
                                                .border(px(1.0))
                                                .border_color(move || theme().input)
                                                .bg(move || theme().background)
                                                .text_size(px(13.0))
                                                .text_color(move || theme().foreground),
                                        ),
                                )
                                .child(unit_chip("Day", "day", unit))
                                .child(unit_chip("Week", "week", unit))
                                .child(unit_chip("Month", "month", unit))
                                .child(unit_chip("Year", "year", unit)),
                        )
                        .child(
                            text("Charge on these days")
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(day_grid(days)),
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
                .child(checkbox(active, "Active subscription".to_string()))
                .child(
                    when(move || !form_error.get().is_empty(), move || {
                        text(move || form_error.get())
                            .text_size(px(12.0))
                            .text_color(move || theme().destructive)
                    })
                    .otherwise(|| view().h(px(0.0))),
                ),
            ),
        )
        .child(
            view()
                .w_full()
                .p(px(24.0))
                .pt(px(16.0))
                .border_t(px(1.0))
                .border_color(move || theme().border)
                .flex_row()
                .justify_end()
                .gap(px(8.0))
                .child(
                    button()
                        .items_center()
                        .justify_center()
                        .h(px(40.0))
                        .px(px(16.0))
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
                        .items_center()
                        .justify_center()
                        .h(px(40.0))
                        .px(px(16.0))
                        .rounded(px(7.0))
                        .bg(move || theme().primary)
                        .disabled(move || saving.get())
                        .disabled_style(|state| state.opacity(0.5))
                        .on_press(move || save())
                        .child(
                            text(move || {
                                if saving.get() {
                                    "Saving..."
                                } else if editing.get().is_some() {
                                    "Save Changes"
                                } else {
                                    "Add Subscription"
                                }
                            })
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().primary_foreground),
                        ),
                ),
        )

}

fn day_grid(days: Signal<Vec<u8>>) -> impl IntoElement {
    view()
        .w_full()
        .grid()
        .grid_template_columns(SharedString::from("repeat(7, minmax(0, 1fr))"))
        .gap(px(4.0))
        .children((1u8..=31).map(|day| {
            let active = move || days.get().contains(&day);
            button()
                .items_center()
                .justify_center()
                .h(px(30.0))
                .w_full()
                .rounded(px(5.0))
                .border(px(1.0))
                .border_color(move || if active() { theme().primary } else { theme().border })
                .bg(move || if active() { theme().primary } else { theme().card })
                .on_press(move || {
                    let mut values = days.get();
                    if let Some(index) = values.iter().position(|value| *value == day) {
                        values.remove(index);
                    } else {
                        values.push(day);
                        values.sort_unstable();
                    }
                    days.set(values);
                })
                .child(
                    text(day.to_string())
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || if active() {
                            theme().primary_foreground
                        } else {
                            theme().foreground
                        }),
                )
        }))
}

fn unit_chip(label_text: &'static str, value: &'static str, unit: Signal<String>) -> impl IntoElement {
    let active = move || unit.get() == value;
    button()
        .items_center()
        .justify_center()
        .h(px(30.0))
        .px(px(10.0))
        .rounded(px(6.0))
        .border(px(1.0))
        .border_color(move || if active() { theme().primary } else { theme().border })
        .bg(move || if active() { theme().accent } else { theme().card })
        .on_press(move || unit.set(value.to_string()))
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

fn frequency_for(unit: &str) -> &'static str {
    match unit {
        "year" => "yearly",
        "week" => "weekly",
        "day" => "daily",
        _ => "monthly",
    }
}

fn recurrence_for(unit: &str, days: &[u8], start: NaiveDate, interval: u16) -> Recurrence {
    let mut recurrence = Recurrence {
        interval,
        unit: unit.to_string(),
        ..Recurrence::default()
    };
    match unit {
        "year" => {
            recurrence.dates = days
                .iter()
                .map(|day| AnnualDate {
                    month: start.month() as u8,
                    day: *day,
                    amount: None,
                })
                .collect();
        }
        "week" => {
            recurrence.weekdays = vec![start.weekday().number_from_monday() as u8];
        }
        _ => {
            recurrence.days = days.to_vec();
        }
    }
    recurrence
}

fn toolbar(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
    sort_desc: Signal<bool>,
    list_view: Signal<bool>,
    active_only: Signal<bool>,
) -> impl IntoElement {
    let count = move || filtered(data, search, active_only, sort_desc).len();
    view()
        .w_full()
        .flex_col()
        .gap(px(14.0))
        .child(list_toolbar(
            search,
            list_view,
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(filter_button(active_only))
                .child(sort_button(sort_desc))
                .into_element(),
        ))
        .child(
            text(move || ui::count_label(count(), "subscription", "subscriptions"))
                .text_size(px(13.0))
                .text_color(move || theme().muted_foreground),
        )
}

fn content(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
    currency: String,
    actions: ListActions,
    server: Signal<String>,
    token: Signal<String>,
    sort_desc: Signal<bool>,
    list_view: Signal<bool>,
    active_only: Signal<bool>,
) -> AnyElement {
    let active = move || {
        filtered(data, search, active_only, sort_desc)
            .into_iter()
            .filter(|subscription| subscription.is_active)
            .collect::<Vec<_>>()
    };
    let inactive = move || {
        filtered(data, search, active_only, sort_desc)
            .into_iter()
            .filter(|subscription| !subscription.is_active)
            .collect::<Vec<_>>()
    };
    let has_inactive = !inactive().is_empty();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .mt(px(11.0))
        .child(section(
            "ACTIVE",
            active,
            data,
            currency.clone(),
            actions,
            server,
            token,
            list_view,
        ))
        .when(has_inactive, move |page| {
            page.child(section(
                "INACTIVE",
                inactive,
                data,
                currency.clone(),
                actions,
                server,
                token,
                list_view,
            ))
        })
        .into_element()
}

fn section(
    label: &'static str,
    items: impl Fn() -> Vec<Subscription> + Clone + 'static,
    data: Resource<SubscriptionsData, String>,
    currency: String,
    actions: ListActions,
    server: Signal<String>,
    token: Signal<String>,
    list_view: Signal<bool>,
) -> impl IntoElement {
    let count_items = items.clone();
    let list_items = items.clone();
    let list_currency = currency.clone();
    let grid_currency = currency.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    text(label)
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(move || count_items().len().to_string())
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground)
                        .px(px(6.0))
                        .py(px(1.0))
                        .rounded(px(999.0))
                        .bg(move || theme().muted),
                ),
        )
        .child(
            when(
                move || list_view.get(),
                move || {
                    let list_items = list_items.clone();
                    let list_currency = list_currency.clone();
                    view()
                        .w_full()
                        .flex_col()
                        .gap(px(8.0))
                        .child(each(
                            list_items,
                            |subscription| subscription.id,
                            move |subscription, _| {
                                list_card(
                                    subscription.get(),
                                    data,
                                    list_currency.clone(),
                                    actions,
                                    server,
                                    token,
                                )
                            },
                        ))
                        .into_element()
                },
            )
            .otherwise(move || {
                let items = items.clone();
                let grid_currency = grid_currency.clone();
                view()
                    .w_full()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(16.0))
                    .child(each(
                        items,
                        |subscription| subscription.id,
                        move |subscription, _| {
                            card(
                                subscription.get(),
                                data,
                                grid_currency.clone(),
                                actions,
                                server,
                                token,
                            )
                        },
                    ))
                    .into_element()
            }),
        )

}

fn filtered(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
    active_only: Signal<bool>,
    sort_desc: Signal<bool>,
) -> Vec<Subscription> {
    let data = data.get().unwrap_or_default();
    filter_subscriptions(
        &data.subscriptions,
        &search.get(),
        &data.accounts,
        active_only.get(),
        sort_desc.get(),
    )
}

fn filter_subscriptions(
    subscriptions: &[Subscription],
    query: &str,
    accounts: &[crate::models::Account],
    active_only: bool,
    sort_desc: bool,
) -> Vec<Subscription> {
    let query = query.trim().to_lowercase();
    let mut result: Vec<Subscription> = subscriptions
        .iter()
        .filter(|subscription| {
            if active_only && !subscription.is_active {
                return false;
            }
            if query.is_empty() {
                return true;
            }
            let account = accounts
                .iter()
                .find(|account| account.id == subscription.account_id)
                .map_or("", |account| account.name.as_str());
            subscription.name.to_lowercase().contains(&query)
                || account.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();
    result.sort_by(|a, b| {
        let date = |value: &Subscription| {
            value
                .next_billing_date
                .map_or(i32::MAX, |date| date.num_days_from_ce())
        };
        b.is_active.cmp(&a.is_active).then_with(|| {
            if sort_desc {
                date(b).cmp(&date(a))
            } else {
                date(a).cmp(&date(b))
            }
        }).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    result
}

fn list_card(
    subscription: Subscription,
    data: Resource<SubscriptionsData, String>,
    currency: String,
    actions: ListActions,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let account = account_name(data, subscription.account_id);
    let action_subscription = subscription.clone();
    let name = subscription.name;
    let next = subscription
        .next_billing_date
        .map_or_else(|| "No next date".to_string(), |date| format!("Next: {}", date.format("%d/%m/%Y")));
    let amount = {
        let rates = data.get().unwrap_or_default().rates;
        format_currency(rates.convert(subscription.amount, &subscription.currency), &currency)
    };
    view()
        .w_full()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .p(px(14.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .min_w(px(0.0))
                .flex_1()
                .child(
                    view()
                        .h(px(36.0))
                        .w(px(36.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || theme().muted)
                        .child(
                            image(asset!("icons/card.svg"))
                                .size(px(16.0))
                                .tint_color(move || theme().muted_foreground),
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
                            text(format!("{account} · {next}"))
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                .child(
                    text(move || if crate::ui::privacy_active() { "••••".to_string() } else { amount.clone() })
                        .text_size(px(14.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(card_actions(
                    action_subscription,
                    subscription.is_active,
                    actions,
                    data,
                    server,
                    token,
                    currency,
                )),
        )
}

fn card(
    subscription: Subscription,
    data: Resource<SubscriptionsData, String>,
    currency: String,
    actions: ListActions,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let account = account_name(data, subscription.account_id);
    let action_subscription = subscription.clone();
    let name = subscription.name;
    let amount = {
        let rates = data.get().unwrap_or_default().rates;
        format_currency(rates.convert(subscription.amount, &subscription.currency), &currency)
    };
    let next = subscription
        .next_billing_date
        .map_or_else(|| "No next date".to_string(), |date| format!("Next: {}", date.format("%d/%m/%Y")));
    let active = subscription.is_active;
    let status = if active { "Active" } else { "Inactive" };

    view()
        .w(px(292.0))
        .min_w(px(230.0))
        .h(px(260.0))
        .p(px(20.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .flex_col()
        .justify_between()
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .justify_between()
                .child(
                    view()
                        .h(px(40.0))
                        .w(px(40.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .bg(move || theme().muted)
                        .child(
                            image(asset!("icons/card.svg"))
                                .size(px(18.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                )
                .child(badge(
                    BadgeProps::new(status).tone(if active { BadgeTone::Primary } else { BadgeTone::Neutral }),
                )),
        )
        .child(
            view()
                .flex_col()
                .gap(px(4.0))
                .child(
                    text(name)
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground)
                        .number_of_lines(1),
                )
                .child(
                    text(next)
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            text(move || if crate::ui::privacy_active() { "••••".to_string() } else { amount.clone() })
                .text_size(px(24.0))
                .font_family(design::MONO)
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            view()
                .self_start()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(6.0))
                .bg(move || theme().muted)
                .child(
                    image(asset!("icons/card.svg"))
                        .size(px(12.0))
                        .tint_color(move || theme().muted_foreground),
                )
                .child(
                    text(account)
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(card_actions(action_subscription, active, actions, data, server, token, currency.clone()))
}

fn card_actions(
    subscription: Subscription,
    active: bool,
    actions: ListActions,
    data: Resource<SubscriptionsData, String>,
    server: Signal<String>,
    token: Signal<String>,
    currency: String,
) -> AnyElement {
    let id = subscription.id;
    let duplicate_currency = currency.clone();
    let form_signals = actions.form;
    let edit_subscription = subscription.clone();
    let edit = move || {
        let recurrence = &edit_subscription.recurrence;
        let days = if recurrence.unit == "year" {
            recurrence.dates.iter().map(|date| date.day).collect::<Vec<_>>()
        } else if recurrence.days.is_empty() {
            vec![edit_subscription.start_date.day() as u8]
        } else {
            recurrence.days.clone()
        };
        form_signals.name.set(edit_subscription.name.clone());
        form_signals.amount.set(edit_subscription.amount.to_string());
        form_signals.start_date.set(edit_subscription.start_date.to_string());
        form_signals.currency.set(edit_subscription.currency.clone());
        form_signals.interval.set(recurrence.interval.max(1).to_string());
        form_signals.days.set(days);
        form_signals.unit.set(recurrence.unit.clone());
        form_signals.account_id.set(Some(edit_subscription.account_id));
        form_signals.category_id.set(Some(edit_subscription.category_id));
        form_signals.active.set(edit_subscription.is_active);
        form_signals.error.set(String::new());
        form_signals.saving.set(false);
        form_signals.editing.set(Some(id));
        form_signals.open.set(true);
        let modal = actions.modal;
        actions.modal.open(move || {
            form(
                data,
                form_signals,
                server,
                token,
                modal,
            )
            .into_element()
        });
    };
    let duplicate = {
        let original = subscription.clone();
        let server = server;
        let token = token;
        let currency = duplicate_currency.clone();
        move || {
            let request = CreateSubscriptionRequest {
                name: format!("{} (copy)", original.name),
                amount: original.amount,
                currency: currency.clone(),
                frequency: frequency_for(&original.recurrence.unit).to_string(),
                start_date: original.start_date,
                account_id: original.account_id,
                category_id: Some(original.category_id),
                recurrence: original.recurrence.clone(),
                is_active: original.is_active,
            };
            let server_value = server.get();
            let token_value = token.get();
            spawn(async move {
                if api::create_subscription(server_value, token_value, &request).await.is_ok() {
                    data.refetch();
                }
            });
        }
    };
    let toggle = {
        move || {
            if actions.busy.get().is_some() {
                return;
            }
            actions.busy.set(Some(id));
            actions.error.set(String::new());
            let server_value = server.get();
            let token_value = token.get();
            let request = ToggleSubscriptionRequest { is_active: !active };
            spawn(async move {
                match api::update_subscription(server_value, token_value, id, &request).await {
                    Ok(_) => data.refetch(),
                    Err(message) => actions.error.set(message),
                }
                actions.busy.set(None);
            });
        }
    };
    let confirm = actions.delete_confirm;
    let delete = {
        move || {
            actions.delete_confirm.set(None);
            actions.busy.set(Some(id));
            actions.error.set(String::new());
            let server_value = server.get();
            let token_value = token.get();
            spawn(async move {
                match api::delete_subscription(server_value, token_value, id).await {
                    Ok(()) => data.refetch(),
                    Err(message) => actions.error.set(message),
                }
                actions.busy.set(None);
            });
        }
    };

    when(move || actions.delete_confirm.get() == Some(id), move || {
        let delete = delete.clone();
        view()
            .w_full()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(6.0))
            .child(
                text("Delete this subscription?")
                    .text_size(px(11.0))
                    .text_color(move || theme().muted_foreground),
            )
            .child(
                view()
                    .flex_row()
                    .gap(px(4.0))
                    .child(ui::action_button("Delete", true, delete))
                    .child(ui::action_button("Cancel", false, move || {
                        actions.delete_confirm.set(None)
                    })),
            )
            .into_element()
    })
    .otherwise(move || {
        let edit = edit.clone();
        let toggle = toggle.clone();
        let duplicate = duplicate.clone();
        view()
            .w_full()
            .flex_row()
            .gap(px(4.0))
            .child(ui::mini_icon_button(
                asset!("icons/copy.svg"),
                false,
                duplicate,
            ))
            .child(ui::mini_icon_button(
                asset!("icons/notes.svg"),
                false,
                edit.clone(),
            ))
            .child(ui::mini_icon_button(
                if active { asset!("icons/pause.svg") } else { asset!("icons/play.svg") },
                false,
                toggle,
            ))
            .child(ui::mini_icon_button(
                asset!("icons/pencil.svg"),
                false,
                edit,
            ))
            .child(ui::mini_icon_button(
                asset!("icons/trash.svg"),
                true,
                move || confirm.set(Some(id)),
            ))
            .into_element()
    })
    .into_element()
}


fn account_name(data: Resource<SubscriptionsData, String>, account_id: Uuid) -> String {
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
            text("Loading subscriptions…")
                .text_size(px(14.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn error_card(message: &str, data: Resource<SubscriptionsData, String>) -> AnyElement {
    let message = message.to_string();
    ui::surface()
        .p(px(24.0))
        .gap(px(12.0))
        .child(
            text("Subscriptions could not be loaded.")
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
    use crate::models::{AnnualDate, Recurrence};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
    }

    fn subscription(name: &str, account_id: Uuid, active: bool) -> Subscription {
        Subscription {
            id: Uuid::new_v4(),
            account_id,
            category_id: Uuid::new_v4(),
            name: name.to_string(),
            amount: Decimal::from(10),
            currency: "EUR".to_string(),
            recurrence: Recurrence::default(),
            start_date: date(2026, 1, 1),
            next_billing_date: Some(date(2026, 11, 23)),
            end_date: None,
            is_active: active,
            logo_url: None,
        }
    }

    #[test]
    fn search_matches_names_and_accounts_case_insensitively() {
        let account_id = Uuid::new_v4();
        let accounts = vec![crate::models::Account {
            id: account_id,
            name: "Everyday".to_string(),
            icon: None,
            color: None,
            is_default: true,
        }];
        let items = vec![
            subscription("Spotify", account_id, true),
            subscription("Netflix", Uuid::new_v4(), true),
        ];
        assert_eq!(
            filter_subscriptions(&items, "spot", &accounts, false, false)[0].name,
            "Spotify"
        );
        assert_eq!(
            filter_subscriptions(&items, "every", &accounts, false, false).len(),
            1
        );
        assert_eq!(filter_subscriptions(&items, "", &accounts, false, false).len(), 2);
    }

    #[test]
    fn inactive_subscriptions_sort_after_active_ones() {
        let account_id = Uuid::new_v4();
        let items = vec![
            subscription("Aardvark", account_id, false),
            subscription("Zebra", account_id, true),
        ];
        let filtered = filter_subscriptions(&items, "", &[], false, false);
        assert_eq!(filtered[0].name, "Zebra");
        assert!(!filtered[1].is_active);
    }

    #[test]
    fn monthly_recurrence_uses_the_chosen_day() {
        let recurrence = recurrence_for("month", &[15], date(2026, 11, 22), 1);
        assert_eq!(recurrence.unit, "month");
        assert_eq!(recurrence.days, vec![15]);
        assert!(recurrence.dates.is_empty());
        assert_eq!(frequency_for("month"), "monthly");
    }

    #[test]
    fn yearly_recurrence_uses_a_single_annual_date() {
        let recurrence = recurrence_for("year", &[5], date(2026, 11, 22), 1);
        assert_eq!(recurrence.unit, "year");
        assert_eq!(
            recurrence.dates,
            vec![AnnualDate {
                month: 11,
                day: 5,
                amount: None,
            }]
        );
        assert_eq!(frequency_for("year"), "yearly");
    }

    #[test]
    fn weekly_recurrence_uses_the_start_weekday() {
        let recurrence = recurrence_for("week", &[1], date(2026, 11, 22), 1);
        assert_eq!(recurrence.weekdays, vec![7]);
        assert_eq!(frequency_for("week"), "weekly");
    }

    #[test]
    fn toggle_request_only_sends_the_active_flag() {
        let json = serde_json::to_value(ToggleSubscriptionRequest { is_active: false })
            .expect("toggle serializes");
        assert_eq!(json["is_active"], false);
        assert_eq!(json.as_object().expect("object").len(), 1);
    }

    #[test]
    fn create_request_serializes_the_recurrence_and_skips_no_category() {
        let request = CreateSubscriptionRequest {
            name: "Spotify".to_string(),
            amount: Decimal::new(1099, 2),
            currency: "EUR".to_string(),
            frequency: "monthly".to_string(),
            start_date: date(2026, 11, 22),
            account_id: Uuid::new_v4(),
            category_id: None,
            recurrence: recurrence_for("month", &[23], date(2026, 11, 22), 1),
            is_active: true,
        };
        let json = serde_json::to_value(request).expect("request serializes");
        assert_eq!(json["frequency"], "monthly");
        assert_eq!(json["recurrence"]["days"][0], 23);
        assert!(json.get("category_id").is_none());
    }

    #[test]
    fn subscription_serde_keeps_next_date() {
        let json = r#"{
            "id":"00000000-0000-0000-0000-000000000001",
            "account_id":"00000000-0000-0000-0000-000000000002",
            "category_id":"00000000-0000-0000-0000-000000000003",
            "name":"Spotify",
            "amount":"10.99",
            "currency":"EUR",
            "recurrence":{"interval":1,"unit":"month","days":[23],"weekdays":[],"dates":[]},
            "start_date":"2026-01-01",
            "next_billing_date":"2026-11-23",
            "end_date":null,
            "is_active":true,
            "logo_url":null
        }"#;
        let subscription: Subscription = serde_json::from_str(json).expect("subscription parses");
        assert_eq!(subscription.next_billing_date, Some(date(2026, 11, 23)));
        assert_eq!(
            subscription.recurrence,
            Recurrence {
                interval: 1,
                unit: "month".to_string(),
                days: vec![23],
                weekdays: Vec::new(),
                dates: Vec::<AnnualDate>::new(),
            }
        );
        assert_eq!(subscription.amount, Decimal::new(1099, 2));
    }
}
