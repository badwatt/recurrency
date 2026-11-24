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
use crate::components::badge::{BadgeProps, BadgeTone, badge};
use crate::components::button::{ButtonProps, button as shared_button};
use crate::components::confirm::{ConfirmProps, confirm_row};
use crate::components::converted_amount::converted_amount;
use crate::components::filter_menu::filter_menu;
use crate::components::list_toolbar::list_toolbar;
use crate::components::sort_menu::{SortOption, sort_menu};
use crate::components::string_select::{StringSelectProps, string_select};
use crate::format::format_date;
use crate::models::{AnnualDate, Recurrence, Subscription};
use crate::ui;

#[derive(Serialize)]
pub struct CreateSubscriptionRequest {
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub start_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<NaiveDate>,
    pub account_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub recurrence: Recurrence,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
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
    end_date: Signal<String>,
    currency: Signal<String>,
    interval: Signal<String>,
    days: Signal<Vec<u8>>,
    annual_dates: Signal<Vec<AnnualDate>>,
    unit: Signal<String>,
    account_id: Signal<Option<Uuid>>,
    category_id: Signal<Option<Uuid>>,
    error: Signal<String>,
    saving: Signal<bool>,
    active: Signal<bool>,
    notes: Signal<String>,
    logo_url: Signal<String>,
    logo_results: Signal<Vec<(String, String)>>,
    logo_loading: Signal<bool>,
    logo_error: Signal<String>,
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

pub fn subscriptions(
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
    let sort_field = signal("next_billing_date".to_string());
    let list_view = signal(false);
    let _ = (sort_field.get(), sort_desc.get());
    let status = signal("all".to_string());
    let frequency_filter = signal("all".to_string());
    let category_filter = signal("all".to_string());
    let price_min = signal(String::new());
    let price_max = signal(String::new());
    let form_open = signal(false);
    let editing = signal(None::<Uuid>);
    let active = signal(true);
    let name = signal(String::new());
    let amount = signal(String::new());
    let start_date = signal(String::new());
    let end_date = signal(String::new());
    let form_currency = signal(if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    });
    let interval = signal("1".to_string());
    let days = signal(Vec::<u8>::new());
    let annual_dates = signal(Vec::<AnnualDate>::new());
    let unit = signal("month".to_string());
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
    let form_signals = FormSignals {
        open: form_open,
        editing,
        name,
        amount,
        start_date,
        end_date,
        currency: form_currency,
        interval,
        days,
        annual_dates,
        unit,
        account_id,
        category_id,
        error: form_error,
        saving,
        active,
        notes,
        logo_url,
        logo_results,
        logo_loading,
        logo_error,
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
    let requested_date = signal(None::<chrono::NaiveDate>);
    let pending_edit = signal(None::<Uuid>);
    let loading = move || data.loading() && data.get().is_none() && !form_open.get();
    let has_error = move || data.error().is_some() && !form_open.get();
    let render_currency = currency.clone();

    let open_form = move || {
        let snapshot = data.get().unwrap_or_default();
        let today = Local::now().date_naive();
        let requested = requested_date.get();
        requested_date.set(None);
        let date = requested.unwrap_or(today);
        name.set(String::new());
        amount.set(String::new());
        start_date.set(requested.map(|date| date.to_string()).unwrap_or_default());
        end_date.set(String::new());
        form_currency.set(if auth.user.currency.is_empty() {
            "EUR".to_string()
        } else {
            auth.user.currency.clone()
        });
        interval.set("1".to_string());
        days.set(vec![date.day() as u8]);
        annual_dates.set(Vec::new());
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
            form(data, form_signals, server_signal, token_signal, modal).into_element()
        });
    };

    if let Some(request) = crate::ui::take_subscription_intent() {
        match request {
            crate::ui::SubscriptionIntent::Create(date) => {
                requested_date.set(Some(date));
                open_form();
            }
            crate::ui::SubscriptionIntent::Edit(id) => {
                pending_edit.set(Some(id));
            }
        }
    }

    effect(move || {
        let Some(id) = pending_edit.get() else {
            return;
        };
        if let Some(subscription) = data
            .get()
            .unwrap_or_default()
            .subscriptions
            .into_iter()
            .find(|subscription| subscription.id == id)
        {
            pending_edit.set(None);
            open_edit_subscription(
                subscription,
                form_signals,
                data,
                server_signal,
                token_signal,
                modal,
            );
        }
    });

    if intent.get() == Some(crate::Route::Subscriptions) {
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

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .pt(move || if compact.get() { px(10.0) } else { px(0.0) })
        .child(header(open_form, compact))
        .child(toolbar(
            data,
            search,
            sort_desc,
            sort_field,
            list_view,
            status,
            frequency_filter,
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
                                .subscriptions
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
                    sort_desc,
                    sort_field,
                    list_view,
                    status,
                    frequency_filter,
                    category_filter,
                    price_min,
                    price_max,
                    selection_mode,
                    selected,
                    compact,
                )
            })
        }))
        .into_element()
}

fn header(on_add: impl Fn() + 'static, compact: Signal<bool>) -> impl IntoElement {
    let on_add = std::rc::Rc::new(on_add);
    view()
        .w_full()
        .flex_row()
        .flex_wrap()
        .items_end()
        .justify_between()
        .gap(px(12.0))
        .pb(px(2.0))
        .child(
            view()
                .min_w(px(0.0))
                .mt(px(-10.0))
                .pb(px(10.0))
                .flex_col()
                .child(
                    text(TITLE)
                        .h(move || if compact.get() { px(40.0) } else { px(48.0) })
                        .mb(px(-8.0))
                        .text_size(move || if compact.get() { px(36.0) } else { px(48.0) })
                        .line_height(move || if compact.get() { px(40.0) } else { px(48.0) })
                        .letter_spacing(px(-2.4))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(SUBTITLE)
                        .text_size(px(16.0))
                        .line_height(px(24.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            when(move || !compact.get(), {
                let on_add = on_add.clone();
                move || {
                    let on_add = on_add.clone();
                    view()
                        .flex_row()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.0))
                        .mt(px(1.0))
                        .mb(px(-3.0))
                        .child(shared_button(
                            ButtonProps::new("Add Subscription")
                                .icon(asset!("icons/plus.svg"))
                                .min_width(160.0)
                                .on_press(move || on_add()),
                        ))
                        .into_element()
                }
            })
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}

fn open_edit_subscription(
    subscription: Subscription,
    form_signals: FormSignals,
    data: Resource<SubscriptionsData, String>,
    server: Signal<String>,
    token: Signal<String>,
    modal: ui::ModalHost,
) {
    let recurrence = &subscription.recurrence;
    let days = if recurrence.unit == "year" {
        recurrence
            .dates
            .iter()
            .map(|date| date.day)
            .collect::<Vec<_>>()
    } else if recurrence.days.is_empty() {
        vec![subscription.start_date.day() as u8]
    } else {
        recurrence.days.clone()
    };
    form_signals.name.set(subscription.name.clone());
    form_signals.amount.set(subscription.amount.to_string());
    form_signals
        .start_date
        .set(subscription.start_date.to_string());
    form_signals.currency.set(subscription.currency.clone());
    form_signals
        .interval
        .set(recurrence.interval.max(1).to_string());
    form_signals.days.set(days);
    form_signals.annual_dates.set(recurrence.dates.clone());
    form_signals.unit.set(recurrence.unit.clone());
    form_signals.account_id.set(Some(subscription.account_id));
    form_signals.category_id.set(Some(subscription.category_id));
    form_signals.active.set(subscription.is_active);
    form_signals
        .notes
        .set(subscription.notes.clone().unwrap_or_default());
    form_signals
        .logo_url
        .set(subscription.logo_url.clone().unwrap_or_default());
    form_signals.logo_results.set(Vec::new());
    form_signals.logo_error.set(String::new());
    form_signals.error.set(String::new());
    form_signals.saving.set(false);
    form_signals.editing.set(Some(subscription.id));
    form_signals.open.set(true);
    modal.open(move || form(data, form_signals, server, token, modal).into_element());
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
        end_date,
        currency,
        interval,
        days,
        annual_dates,
        unit,
        account_id,
        category_id,
        error: form_error,
        saving,
        active,
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
            let Ok(start_value) = NaiveDate::parse_from_str(start_date.get().trim(), "%Y-%m-%d")
            else {
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
            let unit_value = unit.get();
            let day_values = days.get();
            let mut annual_values = annual_dates.get();
            if unit_value == "year" {
                if annual_values.is_empty() {
                    form_error.set("Add at least one annual date.".to_string());
                    saving.set(false);
                    return;
                }
                if annual_values
                    .iter()
                    .any(|date| !(1..=12).contains(&date.month) || !(1..=31).contains(&date.day))
                {
                    form_error.set("Check the annual dates.".to_string());
                    saving.set(false);
                    return;
                }
                annual_values.sort_by_key(|date| (date.month, date.day));
                annual_values.dedup_by_key(|date| (date.month, date.day));
            }
            if unit_value != "year" && day_values.is_empty() {
                form_error.set("Select at least one charge day.".to_string());
                return;
            }
            let end_value = if end_date.get().trim().is_empty() {
                None
            } else {
                match NaiveDate::parse_from_str(end_date.get().trim(), "%Y-%m-%d") {
                    Ok(date) => Some(date),
                    Err(_) => {
                        form_error.set("Enter a valid end date.".to_string());
                        saving.set(false);
                        return;
                    }
                }
            };
            let request = CreateSubscriptionRequest {
                name: name_value,
                amount: amount_value,
                currency: currency_value.clone(),
                frequency: frequency_for(&unit_value).to_string(),
                start_date: start_value,
                end_date: end_value,
                account_id: account_id_value,
                category_id: category_id.get(),
                recurrence: recurrence_for(
                    &unit_value,
                    &day_values,
                    &annual_values,
                    start_value,
                    interval_value,
                ),
                is_active: active.get(),
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
                    api::update_subscription(server_value, token_value, id, &request).await
                } else {
                    api::create_subscription(server_value, token_value, &request).await
                };
                match result {
                    Ok(_) => {
                        form_open.set(false);
                        editing.set(None);
                        modal.close();
                        crate::components::toast::success("Subscription saved.");
                        data.refetch();
                    }
                    Err(message) => {
                        crate::components::toast::error(message.clone());
                        form_error.set(message);
                    }
                }
                saving.set(false);
            });
        }
    };

    let title = if editing.get().is_some() {
        "Edit Subscription"
    } else {
        "Add Subscription"
    };
    let subtitle = if editing.get().is_some() {
        "Update the recurring expense or subscription."
    } else {
        "Add a new recurring expense or subscription."
    };
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(title, subtitle, move || {
            form_open.set(false);
            editing.set(None);
            modal.close();
        })
        .width(512.0)
        .body(
            view()
                .pl(px(22.0))
                .pr(px(34.0))
                .pt(px(2.0))
                .pb(px(24.0))
                .gap(px(10.0))
                .child(ui::required_field("Name", name, "Netflix, Spotify, etc."))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .gap(px(12.0))
                        .child(ui::row_item(ui::required_field("Amount", amount, "9.99")))
                        .child(
                            view()
                                .flex_grow(1.6)
                                .basis(px(0.0))
                                .min_w(px(0.0))
                                .child(string_select(StringSelectProps::new(
                                    "Currency *",
                                    currency,
                                    crate::currencies::CURRENCIES
                                        .iter()
                                        .map(|(code, label)| {
                                            ((*code).to_string(), (*label).to_string())
                                        })
                                        .collect(),
                                ))),
                        ),
                )
                .child(
                    view()
                        .w_full()
                        .mt(px(-2.0))
                        .min_h(px(315.0))
                        .rounded(px(10.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().muted.opacity(0.15))
                        .p(px(14.0))
                        .gap(px(0.0))
                        .child(
                            text("Billing schedule *")
                                .text_size(px(14.0))
                                .line_height(px(20.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            view()
                                .flex_col()
                                .gap(px(0.0))
                                .child(
                                    text(move || {
                                        let interval_value = interval.get();
                                        let unit_value = unit.get();
                                        let today = Local::now().date_naive();
                                        let selected = days.get();
                                        if unit_value == "month" && interval_value == "1" {
                                            let mut values = selected.clone();
                                            if values.is_empty() {
                                                values.push(today.day() as u8);
                                            }
                                            values.sort_unstable();
                                            values.dedup();
                                            if values.len() == 1 {
                                                format!(
                                                    "Charges monthly on day {}.",
                                                    values[0]
                                                )
                                            } else {
                                                let list = values
                                                    .iter()
                                                    .map(u8::to_string)
                                                    .collect::<Vec<_>>()
                                                    .join(", ");
                                                format!("Charges monthly on days {list}.")
                                            }
                                        } else {
                                            format!(
                                                "Charges every {interval_value} {unit_value}."
                                            )
                                        }
                                    })
                                    .text_size(px(12.0))
                                    .line_height(px(16.0))
                                    .text_color(move || theme().muted_foreground),
                                )
                                .child(
                                    text(move || {
                                        let first = NaiveDate::parse_from_str(
                                            start_date.get().trim(),
                                            "%Y-%m-%d",
                                        )
                                        .unwrap_or_else(|_| Local::now().date_naive());
                                        format!(
                                            "First charge: {}",
                                            crate::format::format_date(first)
                                        )
                                    })
                                    .text_size(px(11.0))
                                    .line_height(px(16.0))
                                    .text_color(move || theme().muted_foreground),
                                ),
                        )
                        .child(
                            view()
                                .w_full()
                                .flex_row()
                                .items_center()
                                .gap(px(8.0))
                                .mt(px(1.0))
                                .child(
                                    text("Every")
                                        .text_size(px(14.0))
                                        .line_height(px(20.0))
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    view().w(px(64.0)).child(
                                        input()
                                            .value(interval)
                                            .on_change(move |value: &str| {
                                                interval.set(value.to_string())
                                            })
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
                                .child(
                                    view().w(px(140.0)).child(string_select(
                                        StringSelectProps::new(
                                            "",
                                            unit,
                                            vec![
                                                ("day".to_string(), "day".to_string()),
                                                ("week".to_string(), "week".to_string()),
                                                ("month".to_string(), "month".to_string()),
                                                ("year".to_string(), "year".to_string()),
                                            ],
                                        ),
                                    )),
                                ),
                        )
                        .child(
                            when(
                                move || unit.get() == "year",
                                move || annual_dates_editor(annual_dates).into_element(),
                            )
                            .otherwise(move || {
                                view()
                                    .w_full()
                                    .flex_col()
                                    .gap(px(6.0))
                                    .child(
                                        view()
                                            .w_full()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                text("Charge on these days")
                                                    .text_size(px(12.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(move || theme().muted_foreground),
                                            )
                                            .child(
                                                view()
                                                    .flex_row()
                                                    .items_center()
                                                    .gap(px(8.0))
                                                    .child(
                                                        button()
                                                            .cursor_pointer()
                                                            .h(px(24.0))
                                                            .px(px(6.0))
                                                            .rounded(px(4.0))
                                                            .on_press(move || {
                                                                days.set((1u8..=31).collect())
                                                            })
                                                            .child(
                                                                text("Select all")
                                                                    .text_size(px(11.0))
                                                                    .text_color(move || theme().primary),
                                                            ),
                                                    )
                                                    .child(
                                                        button()
                                                            .cursor_pointer()
                                                            .h(px(24.0))
                                                            .px(px(6.0))
                                                            .rounded(px(4.0))
                                                            .on_press(move || days.set(Vec::new()))
                                                            .child(
                                                                text("Clear")
                                                                    .text_size(px(11.0))
                                                                    .text_color(move || theme().primary),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(day_grid(days))
                                    .child(
                                        text("Days 29-31 use 'last day' for short months. Each selected day is a separate charge.")
                                            .text_size(px(10.0))
                                            .mt(px(9.0))
                                            .text_color(move || theme().muted_foreground),
                                    )
                                    .into_element()
                            }),
                        ),
                )
                .child(
                    view()
                        .w_full()
                        .mt(px(4.0))
                        .flex_row()
                        .gap(px(12.0))
                        .child(
                            view().flex_grow(1.0).min_w(px(0.0)).child(
                                crate::components::field::field(
                                    crate::components::field::FieldProps::new("Start date")
                                        .hint("Leave empty to start from today."),
                                    view().w(px(155.0)).child(
                                        crate::components::date_picker::date_picker(start_date),
                                    )
                                    .into_element(),
                                ),
                            ),
                        )
                        .child(
                            view().flex_grow(1.0).min_w(px(0.0)).child(
                                crate::components::field::field(
                                    crate::components::field::FieldProps::new("End date")
                                        .hint("Last billing date (inclusive)."),
                                    view().w(px(155.0)).child(
                                        crate::components::date_picker::date_picker(end_date),
                                    )
                                    .into_element(),
                                ),
                            ),
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
                .child(crate::components::field::field(
                    crate::components::field::FieldProps::new("Notes"),
                    crate::components::textarea::text_area(notes, "Additional notes...").into_element(),
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
                        .px(px(17.0))
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
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().primary_foreground),
                        ),
                )
                .into_element(),
        ),
    )
}

fn annual_dates_editor(dates: Signal<Vec<AnnualDate>>) -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            text("Annual dates")
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .w_full()
                .flex_col()
                .gap(px(6.0))
                .children((0..12).map(|index| {
                    when(
                        move || dates.get().len() > index,
                        move || {
                            let date = dates.get().get(index).cloned().unwrap_or(AnnualDate {
                                month: 1,
                                day: 1,
                                amount: None,
                            });
                            view()
                                .w_full()
                                .flex_row()
                                .items_end()
                                .gap(px(6.0))
                                .child(annual_field(
                                    "Month",
                                    date.month.to_string(),
                                    move |value| {
                                        let mut next = dates.get();
                                        if let Some(slot) = next.get_mut(index) {
                                            slot.month = value.parse().unwrap_or(0);
                                        }
                                        dates.set(next);
                                    },
                                ))
                                .child(annual_field("Day", date.day.to_string(), move |value| {
                                    let mut next = dates.get();
                                    if let Some(slot) = next.get_mut(index) {
                                        slot.day = value.parse().unwrap_or(0);
                                    }
                                    dates.set(next);
                                }))
                                .child(annual_field(
                                    "Amount (optional)",
                                    date.amount
                                        .map(|value| value.to_string())
                                        .unwrap_or_default(),
                                    move |value| {
                                        let mut next = dates.get();
                                        if let Some(slot) = next.get_mut(index) {
                                            slot.amount = value.trim().parse::<Decimal>().ok();
                                        }
                                        dates.set(next);
                                    },
                                ))
                                .child(ui::action_button("Remove", true, move || {
                                    let mut next = dates.get();
                                    if index < next.len() {
                                        next.remove(index);
                                    }
                                    dates.set(next);
                                }))
                                .into_element()
                        },
                    )
                    .otherwise(|| view().h(px(0.0)).into_element())
                    .into_element()
                })),
        )
        .child(ui::action_button("Add annual date", false, move || {
            let mut next = dates.get();
            next.push(AnnualDate {
                month: 1,
                day: 1,
                amount: None,
            });
            dates.set(next);
        }))
}

fn annual_field(
    label: &'static str,
    value: String,
    on_change: impl Fn(String) + 'static,
) -> impl IntoElement {
    view()
        .flex_grow(1.0)
        .min_w(px(0.0))
        .flex_col()
        .gap(px(4.0))
        .child(
            text(label)
                .text_size(px(10.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(crate::components::input::text_input_value(
            value, "", on_change,
        ))
}

fn day_grid(days: Signal<Vec<u8>>) -> impl IntoElement {
    view()
        .w_full()
        .grid()
        .grid_template_columns(SharedString::from("repeat(11, minmax(0, 1fr))"))
        .gap(px(4.0))
        .mt(px(-4.0))
        .children((1u8..=31).map(|day| {
            let active = move || days.get().contains(&day);
            button()
                .cursor_pointer()
                .cursor_pointer()
                .items_center()
                .justify_center()
                .h(px(32.0))
                .w_full()
                .rounded(px(6.0))
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
                        theme().primary
                    } else {
                        theme().card
                    }
                })
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
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if active() {
                                theme().primary_foreground
                            } else {
                                theme().foreground
                            }
                        }),
                )
        }))
}

fn frequency_for(unit: &str) -> &'static str {
    match unit {
        "year" => "yearly",
        "week" => "weekly",
        "day" => "daily",
        _ => "monthly",
    }
}

fn recurrence_for(
    unit: &str,
    days: &[u8],
    annual: &[AnnualDate],
    start: NaiveDate,
    interval: u16,
) -> Recurrence {
    let mut recurrence = Recurrence {
        interval,
        unit: unit.to_string(),
        ..Recurrence::default()
    };
    match unit {
        "year" => {
            recurrence.dates = if annual.is_empty() {
                if days.is_empty() {
                    vec![AnnualDate {
                        month: start.month() as u8,
                        day: start.day() as u8,
                        amount: None,
                    }]
                } else {
                    days.iter()
                        .map(|day| AnnualDate {
                            month: start.month() as u8,
                            day: *day,
                            amount: None,
                        })
                        .collect()
                }
            } else {
                annual.to_vec()
            };
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
    sort_field: Signal<String>,
    list_view: Signal<bool>,
    status: Signal<String>,
    frequency_filter: Signal<String>,
    category_filter: Signal<String>,
    price_min: Signal<String>,
    price_max: Signal<String>,
    open_bulk: impl Fn() + 'static,
) -> impl IntoElement {
    let count = move || {
        filtered(
            data,
            search,
            status,
            sort_desc,
            &sort_field.get(),
            frequency_filter,
            category_filter,
            price_min,
            price_max,
        )
        .len()
    };
    let clear_filters = move || {
        status.set("all".to_string());
        frequency_filter.set("all".to_string());
        category_filter.set("all".to_string());
        price_min.set(String::new());
        price_max.set(String::new());
    };
    let filter_count = move || {
        usize::from(status.get() != "all")
            + usize::from(frequency_filter.get() != "all")
            + usize::from(category_filter.get() != "all")
            + usize::from(!price_min.get().trim().is_empty())
            + usize::from(!price_max.get().trim().is_empty())
    };
    let category_options = move || {
        let mut options = vec![("all".to_string(), "All categories".to_string())];
        options.extend(
            data.get()
                .unwrap_or_default()
                .categories
                .into_iter()
                .map(|category| (category.id.to_string(), category.name)),
        );
        options
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(27.0))
        .child(list_toolbar(
            search,
            list_view,
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(filter_menu(
                    move || filter_count(),
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
                                    .child(ui::action_button("Clear all", false, clear_filters)),
                            )
                            .child(string_select(StringSelectProps::new(
                                "Status",
                                status,
                                vec![
                                    ("all".to_string(), "All".to_string()),
                                    ("active".to_string(), "Active".to_string()),
                                    ("inactive".to_string(), "Inactive".to_string()),
                                ],
                            )))
                            .child(string_select(StringSelectProps::new(
                                "Repeats every",
                                frequency_filter,
                                vec![
                                    ("all".to_string(), "All units".to_string()),
                                    ("day".to_string(), "Day".to_string()),
                                    ("week".to_string(), "Week".to_string()),
                                    ("month".to_string(), "Month".to_string()),
                                    ("year".to_string(), "Year".to_string()),
                                ],
                            )))
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
                            .into_element()
                    },
                ))
                .child(sort_menu(
                    sort_field,
                    sort_desc,
                    vec![
                        SortOption::new("next_billing_date", "Upcoming date"),
                        SortOption::new("amount", "Price"),
                        SortOption::new("name", "Name"),
                        SortOption::new("frequency", "Repeat"),
                        SortOption::new("start_date", "Start date"),
                        SortOption::new("created_at", "Created"),
                        SortOption::new("updated_at", "Last modified"),
                    ],
                ))
                .child(crate::components::button::icon_button_when_sized(
                    asset!("icons/square.svg"),
                    "Enter selection mode",
                    crate::components::button::ButtonSize::Icon,
                    || false,
                    open_bulk,
                ))
                .into_element(),
        ))
        .child(
            text(move || ui::count_label(count(), "subscription", "subscriptions"))
                .text_size(px(14.0))
                .mt(px(-2.0))
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
    sort_field: Signal<String>,
    list_view: Signal<bool>,
    status: Signal<String>,
    frequency_filter: Signal<String>,
    category_filter: Signal<String>,
    price_min: Signal<String>,
    price_max: Signal<String>,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
    compact: Signal<bool>,
) -> AnyElement {
    let active = move || {
        filtered(
            data,
            search,
            status,
            sort_desc,
            &sort_field.get(),
            frequency_filter,
            category_filter,
            price_min,
            price_max,
        )
        .into_iter()
        .filter(|subscription| subscription.is_active)
        .collect::<Vec<_>>()
    };
    let inactive = move || {
        filtered(
            data,
            search,
            status,
            sort_desc,
            &sort_field.get(),
            frequency_filter,
            category_filter,
            price_min,
            price_max,
        )
        .into_iter()
        .filter(|subscription| !subscription.is_active)
        .collect::<Vec<_>>()
    };
    let has_inactive = !inactive().is_empty();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .mt(px(0.0))
        .child(section(
            "ACTIVE",
            active,
            data,
            currency.clone(),
            actions,
            server,
            token,
            list_view,
            sort_field,
            sort_desc,
            selection_mode,
            selected,
            compact,
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
                sort_field,
                sort_desc,
                selection_mode,
                selected,
                compact,
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
    sort_field: Signal<String>,
    sort_desc: Signal<bool>,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
    compact: Signal<bool>,
) -> impl IntoElement {
    let count_items = items.clone();
    let list_items = items.clone();
    let list_currency = currency.clone();
    let grid_currency = currency.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(17.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    text(label)
                        .text_size(px(12.0))
                        .letter_spacing(px(0.7))
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
                            move |subscription| {
                                format!(
                                    "{}-{}-{}",
                                    subscription.id,
                                    sort_field.get(),
                                    sort_desc.get()
                                )
                            },
                            move |subscription, _| {
                                list_card(
                                    subscription.get(),
                                    data,
                                    list_currency.clone(),
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
                let items = items.clone();
                let grid_currency = grid_currency.clone();
                view()
                    .w_full()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(16.0))
                    .child(each(
                        items,
                        move |subscription| {
                            format!(
                                "{}-{}-{}",
                                subscription.id,
                                sort_field.get(),
                                sort_desc.get()
                            )
                        },
                        move |subscription, _| {
                            card(
                                subscription.get(),
                                data,
                                grid_currency.clone(),
                                actions,
                                server,
                                token,
                                selection_mode,
                                selected,
                                compact,
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
    status: Signal<String>,
    sort_desc: Signal<bool>,
    sort_field: &str,
    frequency_filter: Signal<String>,
    category_filter: Signal<String>,
    price_min: Signal<String>,
    price_max: Signal<String>,
) -> Vec<Subscription> {
    let data = data.get().unwrap_or_default();
    filter_subscriptions(
        &data.subscriptions,
        &search.get(),
        &data.accounts,
        &status.get(),
        sort_desc.get(),
        sort_field,
        &frequency_filter.get(),
        &category_filter.get(),
        price_min.get().parse::<Decimal>().ok(),
        price_max.get().parse::<Decimal>().ok(),
    )
}

fn filter_subscriptions(
    subscriptions: &[Subscription],
    query: &str,
    accounts: &[crate::models::Account],
    status: &str,
    sort_desc: bool,
    sort_field: &str,
    frequency: &str,
    category: &str,
    price_min: Option<Decimal>,
    price_max: Option<Decimal>,
) -> Vec<Subscription> {
    let query = query.trim().to_lowercase();
    let mut result: Vec<Subscription> = subscriptions
        .iter()
        .filter(|subscription| {
            if (status == "active" && !subscription.is_active)
                || (status == "inactive" && subscription.is_active)
            {
                return false;
            }
            if frequency != "all" && subscription.recurrence.unit != frequency {
                return false;
            }
            if category != "all" && subscription.category_id.to_string() != category {
                return false;
            }
            if price_min.is_some_and(|minimum| subscription.amount < minimum)
                || price_max.is_some_and(|maximum| subscription.amount > maximum)
            {
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
        let ordering = match sort_field {
            "amount" => a.amount.cmp(&b.amount),
            "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            "frequency" => a.recurrence.unit.cmp(&b.recurrence.unit),
            "start_date" => a.start_date.cmp(&b.start_date),
            "created_at" => a.created_at.cmp(&b.created_at),
            "updated_at" => a.updated_at.cmp(&b.updated_at),
            _ => date(a).cmp(&date(b)),
        };
        b.is_active
            .cmp(&a.is_active)
            .then_with(|| {
                if sort_desc {
                    ordering.reverse()
                } else {
                    ordering
                }
            })
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
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
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
) -> impl IntoElement {
    let account = account_name(data, subscription.account_id);
    let action_subscription = subscription.clone();
    let logo = subscription.logo_url.clone();
    let name = subscription.name;
    let select_id = subscription.id;
    let next = subscription.next_billing_date.map_or_else(
        || "No next date".to_string(),
        |date| format!("Next: {}", format_date(date)),
    );
    let rates = data.get().unwrap_or_default().rates;
    let amount_value = subscription.amount;
    let amount_currency = subscription.currency.clone();
    let action_list = action_subscription.clone();
    view()
        .cursor_pointer()
        .w_full()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .p(px(14.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || {
            if selected.get().contains(&select_id) {
                theme().primary.opacity(0.55)
            } else {
                theme().border
            }
        })
        .bg(move || theme().card)
        .on_press(move || {
            if selection_mode.get() {
                let mut values = selected.get();
                crate::components::selection_bar::toggle(&mut values, select_id);
                selected.set(values);
            }
        })
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .min_w(px(0.0))
                .flex_1()
                .child(crate::components::logo_avatar::logo_avatar(
                    logo.clone(),
                    asset!("icons/card.svg"),
                    36.0,
                    false,
                ))
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
                .child(converted_amount(
                    amount_value,
                    &amount_currency,
                    &currency,
                    &rates,
                    14.0,
                    FontWeight::SEMIBOLD,
                    true,
                ))
                .child(when(
                    move || !selection_mode.get(),
                    move || {
                        card_actions(
                            action_list.clone(),
                            subscription.is_active,
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

fn card(
    subscription: Subscription,
    data: Resource<SubscriptionsData, String>,
    currency: String,
    actions: ListActions,
    server: Signal<String>,
    token: Signal<String>,
    selection_mode: Signal<bool>,
    selected: Signal<Vec<Uuid>>,
    compact: Signal<bool>,
) -> impl IntoElement {
    let account = account_name(data, subscription.account_id);
    let accent = account_color(data, subscription.account_id);
    let accent_bg = accent.clone();
    let accent_border = accent.clone();
    let accent_fill = accent.clone();
    let accent_icon = accent.clone();
    let accent_text = accent.clone();
    let action_subscription = subscription.clone();
    let action_card = action_subscription.clone();
    let logo = subscription.logo_url.clone();
    let wash_logo = logo.clone();
    let name = subscription.name;
    let rates = data.get().unwrap_or_default().rates;
    let amount_value = subscription.amount;
    let amount_currency = subscription.currency.clone();
    let select_id = subscription.id;
    let next = subscription.next_billing_date.map_or_else(
        || "No next date".to_string(),
        |date| format!("Next: {}", format_date(date)),
    );
    let active = subscription.is_active;
    let status = if active { "Active" } else { "Inactive" };

    view()
        .relative()
        .overflow_hidden()
        .cursor_pointer()
        .w_full()
        .max_w(move || {
            if compact.get() {
                px(10000.0)
            } else {
                px(296.0)
            }
        })
        .min_h(px(266.0))
        .p(px(20.0))
        .pb(px(16.0))
        .gap(px(12.0))
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || {
            if selected.get().contains(&select_id) {
                theme().primary.opacity(0.55)
            } else {
                theme().border.opacity(0.4)
            }
        })
        .bg(move || theme().card.opacity(0.8))
        .on_press(move || {
            if selection_mode.get() {
                let mut values = selected.get();
                crate::components::selection_bar::toggle(&mut values, select_id);
                selected.set(values);
            }
        })
        .when(logo.is_some(), |card| {
            card.child(
                image(wash_logo.clone().unwrap_or_default())
                    .absolute()
                    .inset(px(0.0))
                    .w_full()
                    .h_full()
                    .object_fit(ResizeMode::Cover)
                    .pointer_events(PointerEvents::None)
                    .opacity(0.28)
                    .blur(px(52.0))
                    .saturate(1.7)
                    .scale(1.35),
            )
            .child(
                view()
                    .absolute()
                    .inset(px(0.0))
                    .pointer_events(PointerEvents::None)
                    .bg(move || theme().card.opacity(0.75)),
            )
        })
        .child(
            view()
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(3.0))
                .pointer_events(PointerEvents::None)
                .bg(move || rgb(crate::theme::parse_hex(&accent_bg))),
        )
        .flex_col()
        .child(
            view()
                .w_full()
                .flex_row()
                .items_start()
                .justify_between()
                .child(
                    view()
                        .h(px(48.0))
                        .w(px(48.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(12.0))
                        .overflow_hidden()
                        .child(
                            when(logo.is_some(), move || {
                                image(logo.clone().unwrap_or_default())
                                    .size(px(48.0))
                                    .object_fit(ResizeMode::Cover)
                                    .into_element()
                            })
                            .otherwise(|| {
                                image(asset!("icons/card.svg"))
                                    .size(px(22.0))
                                    .tint_color(move || theme().muted_foreground)
                                    .into_element()
                            }),
                        ),
                )
                .child(badge(BadgeProps::new(status).tone(if active {
                    BadgeTone::Success
                } else {
                    BadgeTone::Neutral
                }))),
        )
        .child(
            view()
                .flex_col()
                .gap(px(4.0))
                .child(
                    text(name)
                        .text_size(px(14.0))
                        .line_height(px(18.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground)
                        .number_of_lines(1),
                )
                .child(
                    text(next)
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(view().mt(px(3.0)).child(converted_amount(
            amount_value,
            &amount_currency,
            &currency,
            &rates,
            24.0,
            FontWeight::EXTRA_BOLD,
            false,
        )))
        .child(
            view()
                .self_start()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || rgb(crate::theme::parse_hex(&accent_border)).opacity(0.15))
                .bg(move || rgb(crate::theme::parse_hex(&accent_fill)).opacity(0.08))
                .child(
                    image(asset!("icons/card.svg"))
                        .size(px(16.0))
                        .tint_color(move || rgb(crate::theme::parse_hex(&accent_icon))),
                )
                .child(
                    text(account)
                        .text_size(px(12.0))
                        .text_color(move || rgb(crate::theme::parse_hex(&accent_text))),
                ),
        )
        .child(when(
            move || !selection_mode.get(),
            move || {
                card_actions(
                    action_card.clone(),
                    active,
                    actions,
                    data,
                    server,
                    token,
                    currency.clone(),
                )
                .into_element()
            },
        ))
}

fn bulk_modal(
    data: Resource<SubscriptionsData, String>,
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
            .subscriptions
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
    let set_active = move |active: bool| {
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
                let _ = api::update_subscription(
                    server.clone(),
                    token.clone(),
                    id,
                    &serde_json::json!({ "is_active": active }),
                )
                .await;
            }
            selected.set(Vec::new());
            selection_mode.set(false);
            data.refetch();
            busy.set(false);
        });
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
                    let _ = api::update_subscription(
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
                let _ = api::delete_subscription(server.clone(), token.clone(), id).await;
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
            .subscriptions
            .into_iter()
            .map(|subscription| {
                let id = subscription.id;
                let name = subscription.name;
                let active = subscription.is_active;
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
                                text(if active { "Active" } else { "Inactive" })
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
            "Select subscriptions, then choose an action.",
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
                        .max_h(px(300.0))
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
                                    text(move || format!("Delete {} subscriptions?", count()))
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
                .flex_wrap()
                .justify_end()
                .gap(px(8.0))
                .child(ui::action_button("Apply category", false, apply_category))
                .child(ui::action_button("Activate", false, move || {
                    set_active(true)
                }))
                .child(ui::action_button("Pause", false, move || set_active(false)))
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
    data: Resource<SubscriptionsData, String>,
    modal: ui::ModalHost,
) -> impl IntoElement {
    let save = move || {
        saving.set(true);
        error.set(String::new());
        let value = notes.get();
        let server = server.get();
        let token = token.get();
        spawn(async move {
            match api::update_subscription(
                server,
                token,
                id,
                &serde_json::json!({ "notes": value }),
            )
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
            recurrence
                .dates
                .iter()
                .map(|date| date.day)
                .collect::<Vec<_>>()
        } else if recurrence.days.is_empty() {
            vec![edit_subscription.start_date.day() as u8]
        } else {
            recurrence.days.clone()
        };
        form_signals.name.set(edit_subscription.name.clone());
        form_signals
            .amount
            .set(edit_subscription.amount.to_string());
        form_signals
            .start_date
            .set(edit_subscription.start_date.to_string());
        form_signals
            .currency
            .set(edit_subscription.currency.clone());
        form_signals
            .interval
            .set(recurrence.interval.max(1).to_string());
        form_signals.days.set(days);
        form_signals.annual_dates.set(recurrence.dates.clone());
        form_signals.unit.set(recurrence.unit.clone());
        form_signals
            .account_id
            .set(Some(edit_subscription.account_id));
        form_signals
            .category_id
            .set(Some(edit_subscription.category_id));
        form_signals.active.set(edit_subscription.is_active);
        form_signals
            .notes
            .set(edit_subscription.notes.clone().unwrap_or_default());
        form_signals
            .logo_url
            .set(edit_subscription.logo_url.clone().unwrap_or_default());
        form_signals.logo_results.set(Vec::new());
        form_signals.logo_error.set(String::new());
        form_signals.error.set(String::new());
        form_signals.saving.set(false);
        form_signals.editing.set(Some(id));
        form_signals.open.set(true);
        let modal = actions.modal;
        actions
            .modal
            .open(move || form(data, form_signals, server, token, modal).into_element());
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
                end_date: original.end_date,
                account_id: original.account_id,
                category_id: Some(original.category_id),
                recurrence: original.recurrence.clone(),
                is_active: original.is_active,
                notes: None,
                logo_url: original.logo_url.clone(),
            };
            let server_value = server.get();
            let token_value = token.get();
            spawn(async move {
                if api::create_subscription(server_value, token_value, &request)
                    .await
                    .is_ok()
                {
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
                    Ok(_) => {
                        crate::components::toast::success("Subscription updated.");
                        data.refetch();
                    }
                    Err(message) => {
                        crate::components::toast::error(message.clone());
                        actions.error.set(message);
                    }
                }
                actions.busy.set(None);
            });
        }
    };
    let notes_value = subscription.notes.clone().unwrap_or_default();
    let notes_name = subscription.name.clone();
    let notes_signal = actions.form.notes;
    let open_notes = move || {
        notes_signal.set(notes_value.clone());
        form_signals.error.set(String::new());
        form_signals.saving.set(false);
        let modal = actions.modal;
        let dialog_name = notes_name.clone();
        modal.open(move || {
            notes_form(
                id,
                dialog_name.clone(),
                notes_signal,
                form_signals.saving,
                form_signals.error,
                server,
                token,
                data,
                modal,
            )
            .into_element()
        });
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
                    Ok(()) => {
                        crate::components::toast::success("Subscription deleted.");
                        data.refetch();
                    }
                    Err(message) => {
                        crate::components::toast::error(message.clone());
                        actions.error.set(message);
                    }
                }
                actions.busy.set(None);
            });
        }
    };

    when(
        move || actions.delete_confirm.get() == Some(id),
        move || {
            let delete = delete.clone();
            confirm_row(ConfirmProps::new(
                "Delete this subscription?",
                "Delete",
                move || delete(),
                move || actions.delete_confirm.set(None),
            ))
            .into_element()
        },
    )
    .otherwise(move || {
        let edit = edit.clone();
        let toggle = toggle.clone();
        let duplicate = duplicate.clone();
        let open_notes = open_notes.clone();
        view()
            .w_full()
            .flex_row()
            .gap(px(2.0))
            .pt(px(4.0))
            .child(ui::mini_icon_button_labeled(
                asset!("icons/copy.svg"),
                "Duplicate subscription",
                false,
                duplicate,
            ))
            .child(ui::mini_icon_button_labeled(
                asset!("icons/notes.svg"),
                "Subscription notes",
                false,
                open_notes,
            ))
            .child(ui::mini_icon_button_labeled(
                if active {
                    asset!("icons/pause.svg")
                } else {
                    asset!("icons/play.svg")
                },
                if active {
                    "Pause subscription"
                } else {
                    "Activate subscription"
                },
                false,
                toggle,
            ))
            .child(ui::mini_icon_button_labeled(
                asset!("icons/pencil.svg"),
                "Edit subscription",
                false,
                edit,
            ))
            .child(ui::mini_icon_button_labeled(
                asset!("icons/trash.svg"),
                "Delete subscription",
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

fn account_color(data: Resource<SubscriptionsData, String>, account_id: Uuid) -> String {
    data.get()
        .unwrap_or_default()
        .accounts
        .into_iter()
        .find(|account| account.id == account_id)
        .and_then(|account| account.color)
        .unwrap_or_else(|| "#6b7280".to_string())
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
            notes: None,
            logo_url: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn sorting_uses_the_selected_field_and_direction() {
        let account_id = Uuid::new_v4();
        let mut cheap = subscription("Cheap", account_id, true);
        cheap.amount = Decimal::from(5);
        let mut pricey = subscription("Pricey", account_id, true);
        pricey.amount = Decimal::from(50);
        let items = vec![cheap, pricey];
        let ascending = filter_subscriptions(
            &items,
            "",
            &[],
            "all",
            false,
            "amount",
            "all",
            "all",
            None,
            None,
        );
        assert_eq!(ascending[0].name, "Cheap");
        let descending = filter_subscriptions(
            &items,
            "",
            &[],
            "all",
            true,
            "amount",
            "all",
            "all",
            None,
            None,
        );
        assert_eq!(descending[0].name, "Pricey");
    }

    #[test]
    fn sort_menu_exposes_every_sortable_field() {
        let source = include_str!("components/sort_menu.rs");
        assert!(source.contains("Sort by"));
        assert!(source.contains("Ascending"));
        assert!(source.contains("icons/sort.svg"));
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
            filter_subscriptions(
                &items,
                "spot",
                &accounts,
                "all",
                false,
                "next_billing_date",
                "all",
                "all",
                None,
                None
            )[0]
            .name,
            "Spotify"
        );
        assert_eq!(
            filter_subscriptions(
                &items,
                "every",
                &accounts,
                "all",
                false,
                "next_billing_date",
                "all",
                "all",
                None,
                None
            )
            .len(),
            1
        );
        assert_eq!(
            filter_subscriptions(
                &items,
                "",
                &accounts,
                "all",
                false,
                "next_billing_date",
                "all",
                "all",
                None,
                None
            )
            .len(),
            2
        );
    }

    #[test]
    fn the_status_filter_limits_to_active_or_inactive() {
        let account_id = Uuid::new_v4();
        let items = vec![
            subscription("Active", account_id, true),
            subscription("Paused", account_id, false),
        ];
        let active = filter_subscriptions(
            &items,
            "",
            &[],
            "active",
            false,
            "next_billing_date",
            "all",
            "all",
            None,
            None,
        );
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Active");
        let inactive = filter_subscriptions(
            &items,
            "",
            &[],
            "inactive",
            false,
            "next_billing_date",
            "all",
            "all",
            None,
            None,
        );
        assert_eq!(inactive.len(), 1);
        assert_eq!(inactive[0].name, "Paused");
    }

    #[test]
    fn inactive_subscriptions_sort_after_active_ones() {
        let account_id = Uuid::new_v4();
        let items = vec![
            subscription("Aardvark", account_id, false),
            subscription("Zebra", account_id, true),
        ];
        let filtered = filter_subscriptions(
            &items,
            "",
            &[],
            "all",
            false,
            "next_billing_date",
            "all",
            "all",
            None,
            None,
        );
        assert_eq!(filtered[0].name, "Zebra");
        assert!(!filtered[1].is_active);
    }

    #[test]
    fn monthly_recurrence_uses_the_chosen_day() {
        let recurrence = recurrence_for("month", &[15], &[], date(2026, 11, 22), 1);
        assert_eq!(recurrence.unit, "month");
        assert_eq!(recurrence.days, vec![15]);
        assert!(recurrence.dates.is_empty());
        assert_eq!(frequency_for("month"), "monthly");
    }

    #[test]
    fn yearly_recurrence_preserves_multiple_annual_dates() {
        let dates = vec![
            AnnualDate {
                month: 3,
                day: 5,
                amount: Some(Decimal::from(5)),
            },
            AnnualDate {
                month: 9,
                day: 20,
                amount: None,
            },
        ];
        let recurrence = recurrence_for("year", &[], &dates, date(2026, 11, 22), 1);
        assert_eq!(recurrence.dates, dates);
    }

    #[test]
    fn yearly_recurrence_uses_a_single_annual_date() {
        let recurrence = recurrence_for("year", &[5], &[], date(2026, 11, 22), 1);
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
        let recurrence = recurrence_for("week", &[1], &[], date(2026, 11, 22), 1);
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
            end_date: None,
            account_id: Uuid::new_v4(),
            category_id: None,
            recurrence: recurrence_for("month", &[23], &[], date(2026, 11, 22), 1),
            is_active: true,
            notes: None,
            logo_url: None,
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
