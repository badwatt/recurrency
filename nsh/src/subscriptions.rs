//! Subscriptions list: searchable cards grouped by active state.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::api::{self, SubscriptionsData};
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
    day: Signal<String>,
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
    delete_confirm: Signal<Option<Uuid>>,
    busy: Signal<Option<Uuid>>,
    error: Signal<String>,
}

const TITLE: &str = "Subscriptions";
const SUBTITLE: &str = "Track and manage recurring payments";

pub fn subscriptions(auth: AuthSession) -> AnyElement {
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
    let form_open = signal(false);
    let editing = signal(None::<Uuid>);
    let active = signal(true);
    let name = signal(String::new());
    let amount = signal(String::new());
    let start_date = signal(String::new());
    let day = signal(String::new());
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
        day,
        unit,
        account_id,
        category_id,
        error: form_error,
        saving,
        active,
    };
    let actions = ListActions {
        form: form_signals,
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
        day.set(today.day().to_string());
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
                    content(
                        data,
                        search,
                        value.clone(),
                        actions,
                        server_signal,
                        token_signal,
                    )
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
                    text("Add subscription")
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary_foreground),
                ),
        )
}

fn form(
    data: Resource<SubscriptionsData, String>,
    form: FormSignals,
    server: Signal<String>,
    token: Signal<String>,
    currency: String,
) -> impl IntoElement {
    let FormSignals {
        open: form_open,
        editing,
        name,
        amount,
        start_date,
        day,
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
            let Ok(day_value) = day.get().trim().parse::<u8>() else {
                form_error.set("Enter a day between 1 and 31.".to_string());
                return;
            };
            if !(1..=31).contains(&day_value) {
                form_error.set("Enter a day between 1 and 31.".to_string());
                return;
            }
            let unit_value = unit.get();
            let request = CreateSubscriptionRequest {
                name: name_value,
                amount: amount_value,
                currency: currency.clone(),
                frequency: frequency_for(&unit_value).to_string(),
                start_date: start_value,
                account_id: account_id_value,
                category_id: category_id.get(),
                recurrence: recurrence_for(&unit_value, day_value, start_value),
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
                    text(move || if editing.get().is_some() { "Edit subscription" } else { "New subscription" })
                        .text_size(px(20.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(move || if editing.get().is_some() {
                        "Update the payment, account or schedule."
                    } else {
                        "Add a recurring payment. Choose an account and a schedule."
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
                .child(ui::field("Name", name, "Spotify Premium"))
                .child(ui::field("Amount", amount, "10.99")),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(12.0))
                .child(ui::field("Start date", start_date, "2026-11-22"))
                .child(ui::field("Day", day, "23")),
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
            view()
                .flex_col()
                .gap(px(6.0))
                .child(ui::label("Repeat"))
                .child(
                    view()
                        .flex_row()
                        .gap(px(6.0))
                        .child(unit_chip("Monthly", "month", unit))
                        .child(unit_chip("Yearly", "year", unit))
                        .child(unit_chip("Weekly", "week", unit)),
                ),
        )
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
                            text(move || if saving.get() { "Saving..." } else { "Save subscription" })
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

fn unit_chip(label_text: &'static str, value: &'static str, unit: Signal<String>) -> impl IntoElement {
    let active = move || unit.get() == value;
    button()
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

fn recurrence_for(unit: &str, day: u8, start: NaiveDate) -> Recurrence {
    let mut recurrence = Recurrence {
        interval: 1,
        unit: unit.to_string(),
        ..Recurrence::default()
    };
    match unit {
        "year" => {
            recurrence.dates = vec![AnnualDate {
                month: start.month() as u8,
                day,
                amount: None,
            }];
        }
        "week" => {
            recurrence.weekdays = vec![start.weekday().number_from_monday() as u8];
        }
        _ => {
            recurrence.days = vec![day];
        }
    }
    recurrence
}

fn toolbar(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
) -> impl IntoElement {
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
) -> AnyElement {
    let active = move || {
        filtered(data, search)
            .into_iter()
            .filter(|subscription| subscription.is_active)
            .collect::<Vec<_>>()
    };
    let inactive = move || {
        filtered(data, search)
            .into_iter()
            .filter(|subscription| !subscription.is_active)
            .collect::<Vec<_>>()
    };
    let has_inactive = !inactive().is_empty();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(section(
            "ACTIVE",
            active,
            data,
            currency.clone(),
            actions,
            server,
            token,
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
) -> impl IntoElement {
    let count_items = items.clone();
    let currency = currency.clone();
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
            view()
                .w_full()
                .grid()
                .grid_template_columns(SharedString::from("repeat(4, minmax(0, 1fr))"))
                .gap(px(16.0))
                .child(each(
                    items,
                    |subscription| subscription.id,
                    move |subscription, _| {
                        card(
                            subscription.get(),
                            data,
                            currency.clone(),
                            actions,
                            server,
                            token,
                        )
                    }
                )),
        )
}

fn filtered(
    data: Resource<SubscriptionsData, String>,
    search: Signal<String>,
) -> Vec<Subscription> {
    let data = data.get().unwrap_or_default();
    filter_subscriptions(&data.subscriptions, &search.get(), &data.accounts)
}

fn filter_subscriptions(
    subscriptions: &[Subscription],
    query: &str,
    accounts: &[crate::models::Account],
) -> Vec<Subscription> {
    let query = query.trim().to_lowercase();
    let mut result: Vec<Subscription> = subscriptions
        .iter()
        .filter(|subscription| {
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
        b.is_active
            .cmp(&a.is_active)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    result
}

fn card(
    subscription: Subscription,
    data: Resource<SubscriptionsData, String>,
    currency: String,
    actions: ListActions,
    server: Signal<String>,
    token: Signal<String>,
) -> impl IntoElement {
    let color = account_color(data, subscription.account_id);
    let account = account_name(data, subscription.account_id);
    let badge_bg_color = color;
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
    let badge = if active { "Active" } else { "Inactive" };
    let badge_bg = if active {
        design::success().opacity(0.15)
    } else {
        theme().muted
    };
    let badge_text = if active {
        design::success()
    } else {
        theme().muted_foreground
    };

    view()
        .w_full()
        .h(px(280.0))
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
                .child(
                    text(badge)
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || badge_text)
                        .px(px(7.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .bg(move || badge_bg),
                ),
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
            view()
                .w_full()
                .flex_row()
                .items_end()
                .justify_between()
                .gap(px(8.0))
                .child(
                    text(amount)
                        .text_size(px(24.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(8.0))
                        .py(px(4.0))
                        .rounded(px(6.0))
                        .bg(move || theme().muted)
                        .child(
                            view()
                                .h(px(12.0))
                                .w(px(12.0))
                                .rounded(px(3.0))
                                .bg(move || rgba((parse_hex(&badge_bg_color) << 8) | 0x80)),
                        )
                        .child(
                            text(account)
                                .text_size(px(11.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(card_actions(action_subscription, active, actions, data, server, token))
}

fn card_actions(
    subscription: Subscription,
    active: bool,
    actions: ListActions,
    data: Resource<SubscriptionsData, String>,
    server: Signal<String>,
    token: Signal<String>,
) -> AnyElement {
    let id = subscription.id;
    let form = actions.form;
    let edit_subscription = subscription.clone();
    let edit = move || {
        let recurrence = &edit_subscription.recurrence;
        let day = recurrence
            .dates
            .first()
            .map(|date| date.day)
            .or_else(|| recurrence.days.first().copied())
            .unwrap_or_else(|| edit_subscription.start_date.day() as u8);
        form.name.set(edit_subscription.name.clone());
        form.amount.set(edit_subscription.amount.to_string());
        form.start_date.set(edit_subscription.start_date.to_string());
        form.day.set(day.to_string());
        form.unit.set(recurrence.unit.clone());
        form.account_id.set(Some(edit_subscription.account_id));
        form.category_id.set(Some(edit_subscription.category_id));
        form.active.set(edit_subscription.is_active);
        form.error.set(String::new());
        form.saving.set(false);
        form.editing.set(Some(id));
        form.open.set(true);
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
    let action_label = if active { "Pause" } else { "Resume" };
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
        view()
            .w_full()
            .flex_row()
            .gap(px(4.0))
            .child(ui::action_button("Edit", false, edit))
            .child(ui::action_button(action_label, false, toggle))
            .child(ui::action_button("Delete", true, move || {
                confirm.set(Some(id));
            }))
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

fn account_color(data: Resource<SubscriptionsData, String>, account_id: Uuid) -> String {
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
            filter_subscriptions(&items, "spot", &accounts)[0].name,
            "Spotify"
        );
        assert_eq!(
            filter_subscriptions(&items, "every", &accounts).len(),
            1
        );
        assert_eq!(filter_subscriptions(&items, "", &accounts).len(), 2);
    }

    #[test]
    fn inactive_subscriptions_sort_after_active_ones() {
        let account_id = Uuid::new_v4();
        let items = vec![
            subscription("Aardvark", account_id, false),
            subscription("Zebra", account_id, true),
        ];
        let filtered = filter_subscriptions(&items, "", &[]);
        assert_eq!(filtered[0].name, "Zebra");
        assert!(!filtered[1].is_active);
    }

    #[test]
    fn monthly_recurrence_uses_the_chosen_day() {
        let recurrence = recurrence_for("month", 15, date(2026, 11, 22));
        assert_eq!(recurrence.unit, "month");
        assert_eq!(recurrence.days, vec![15]);
        assert!(recurrence.dates.is_empty());
        assert_eq!(frequency_for("month"), "monthly");
    }

    #[test]
    fn yearly_recurrence_uses_a_single_annual_date() {
        let recurrence = recurrence_for("year", 5, date(2026, 11, 22));
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
        let recurrence = recurrence_for("week", 1, date(2026, 11, 22));
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
            recurrence: recurrence_for("month", 23, date(2026, 11, 22)),
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
