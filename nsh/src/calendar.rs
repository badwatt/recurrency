//! Month calendar with scheduled subscriptions and one-time transactions.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, theme};
use nsh_core::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::api::{self, CalendarData};
use crate::auth::AuthSession;
use crate::components::page_header::page_header as shared_page_header;
use crate::format::format_currency;
use crate::recurrence::days_in_month;
use crate::theme as design;
use crate::ui;

const TITLE: &str = "Calendar";
const SUBTITLE: &str = "Upcoming payments";

#[derive(Clone, Debug, PartialEq)]
struct CalendarEntry {
    key: String,
    name: String,
    amount: Decimal,
    currency: String,
    date: NaiveDate,
    is_transaction: bool,
    subscription_id: Option<Uuid>,
    transaction_id: Option<Uuid>,
    frequency: String,
}

pub fn calendar(auth: AuthSession, compact: Signal<bool>) -> AnyElement {
    let server = crate::auth::stored_server();
    let token = auth.token.clone();
    let currency = if auth.user.currency.is_empty() {
        "EUR".to_string()
    } else {
        auth.user.currency.clone()
    };
    let today = Local::now().date_naive();
    let year = signal(today.year());
    let month = signal(today.month());
    let load_currency = currency.clone();
    let resource_server = server.clone();
    let resource_token = token.clone();
    let data = resource(
        move || Some((year.get(), month.get(), crate::ui::data_revision())),
        move |(load_year, load_month, _version), _| {
            let server = resource_server.clone();
            let token = resource_token.clone();
            let currency = load_currency.clone();
            async move { api::load_calendar(server, token, currency, load_year, load_month).await }
        },
    );
    let subs_currency = currency.clone();
    let subs_server = server.clone();
    let subs_token = token.clone();
    let subscriptions = resource(
        || Some(()),
        move |(), _| {
            let server = subs_server.clone();
            let token = subs_token.clone();
            let currency = subs_currency.clone();
            async move { api::load_subscriptions(server, token, currency).await }
        },
    );
    let loading = move || data.loading() && data.get().is_none();
    let has_error = move || data.error().is_some();
    let render_currency = currency.clone();
    let day_modal = ui::ModalHost::new();
    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .pt(move || if compact.get() { px(20.0) } else { px(0.0) })
        .child(header(compact))
        .child(when(loading, || loading_card()).otherwise(move || {
            let currency = render_currency.clone();
            let content_server = server.clone();
            let content_token = token.clone();
            when(has_error, move || {
                error_card(&data.error().unwrap_or_default(), data)
            })
            .otherwise(move || {
                content(
                    data,
                    year,
                    month,
                    currency.clone(),
                    today,
                    compact,
                    day_modal,
                    content_server.clone(),
                    content_token.clone(),
                    subscriptions,
                )
            })
        }))
        .child(
            when(
                move || day_modal.active(),
                move || ui::modal_overlay(day_modal),
            )
            .otherwise(|| view().h(px(0.0))),
        )
        .into_element()
}

fn header(compact: Signal<bool>) -> impl IntoElement {
    shared_page_header(TITLE, SUBTITLE, view().into_element(), compact, false, 0.0)
}

fn navigation(
    year: Signal<i32>,
    month: Signal<u32>,
    today: NaiveDate,
    compact: bool,
) -> impl IntoElement {
    let group = view()
        .flex_row()
        .items_center()
        .h(px(38.0))
        .p(px(2.0))
        .gap(px(4.0))
        .rounded(px(8.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .child(nav_arrow(
            asset!("icons/chevron-left.svg"),
            "Previous month",
            move || {
                let (next_year, next_month) = shift_month(year.get(), month.get(), -1);
                year.set(next_year);
                month.set(next_month);
            },
        ))
        .child(
            text(move || month_label(year.get(), month.get()))
                .w(px(if compact { 120.0 } else { 200.0 }))
                .text_size(px(if compact { 16.0 } else { 20.0 }))
                .line_height(px(28.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
        .child(nav_arrow(
            asset!("icons/chevron-right.svg"),
            "Next month",
            move || {
                let (next_year, next_month) = shift_month(year.get(), month.get(), 1);
                year.set(next_year);
                month.set(next_month);
            },
        ));
    view()
        .w_full()
        .flex_row()
        .items_center()
        .justify_between()
        .p(if compact { px(0.0) } else { px(16.0) })
        .border_b(px(if compact { 0.0 } else { 1.0 }))
        .border_color(move || theme().border)
        .child(group)
        .child(
            button()
                .cursor_pointer()
                .items_center()
                .justify_center()
                .h(px(36.0))
                .px(if compact { px(8.0) } else { px(12.0) })
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .on_press(move || {
                    year.set(today.year());
                    month.set(today.month());
                })
                .child(
                    text("Today")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                ),
        )
}

fn content(
    data: Resource<CalendarData, String>,
    year: Signal<i32>,
    month: Signal<u32>,
    currency: String,
    today: NaiveDate,
    compact: Signal<bool>,
    day_modal: ui::ModalHost,
    server: String,
    token: String,
    subscriptions: Resource<crate::api::SubscriptionsData, String>,
) -> AnyElement {
    let total_currency = currency.clone();
    let total = move || {
        let snapshot = data.get().unwrap_or_default();
        let entries = calendar_entries(&snapshot, year.get(), month.get());
        entries
            .iter()
            .map(|entry| snapshot.rates.convert(entry.amount, &entry.currency))
            .sum::<Decimal>()
    };
    let pending_currency = currency.clone();
    let pending = move || {
        let snapshot = data.get().unwrap_or_default();
        calendar_entries(&snapshot, year.get(), month.get())
            .into_iter()
            .filter(|entry| entry.date > today)
            .map(|entry| snapshot.rates.convert(entry.amount, &entry.currency))
            .sum::<Decimal>()
    };
    let cells = move || cells_for(year.get(), month.get());
    let mobile_currency = currency.clone();
    let mobile_server = server.clone();
    let mobile_token = token.clone();
    let desktop_currency = currency.clone();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child({
            let row = if compact.get() {
                view().w_full().flex_col().gap(px(12.0))
            } else {
                view().w_full().flex_row().gap(px(12.0))
            };
            row.child(summary("TRANSFER THIS MONTH", move || {
                format_currency(total(), &total_currency)
            }))
            .child(summary("PENDING FROM TODAY", move || {
                format_currency(pending(), &pending_currency)
            }))
        })
        .child(
            when(
                move || compact.get(),
                move || {
                    let mobile_currency = mobile_currency.clone();
                    mobile_calendar(
                        data,
                        year,
                        month,
                        mobile_currency,
                        today,
                        day_modal,
                        mobile_server.clone(),
                        mobile_token.clone(),
                        subscriptions,
                    )
                },
            )
            .otherwise(move || {
                let desktop_currency = desktop_currency.clone();
                let desktop_server = server.clone();
                let desktop_token = token.clone();
                view()
                    .w_full()
                    .rounded(px(12.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .overflow_hidden()
                    .child(navigation(year, month, today, compact.get()))
                    .child(
                        view()
                            .w_full()
                            .mt(px(16.0))
                            .px(px(16.0))
                            .pb(px(16.0))
                            .grid()
                            .grid_template_columns(SharedString::from("repeat(7, minmax(0, 1fr))"))
                            .bg(move || theme().border)
                            .gap(px(1.0))
                            .children({
                                let labels: [&'static str; 7] =
                                    if crate::format::week_starts_sunday() {
                                        ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
                                    } else {
                                        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                                    };
                                labels.into_iter().map(weekday_header).collect::<Vec<_>>()
                            })
                            .child(each(
                                cells,
                                |(index, _)| *index,
                                move |cell, _| {
                                    let (_, day) = cell.get();
                                    day_cell(
                                        day,
                                        data,
                                        year,
                                        month,
                                        desktop_currency.clone(),
                                        today,
                                        day_modal,
                                        desktop_server.clone(),
                                        desktop_token.clone(),
                                        subscriptions,
                                    )
                                },
                            )),
                    )
                    .into_element()
            }),
        )
        .into_element()
}

fn summary(label: &str, value: impl Fn() -> String + 'static) -> impl IntoElement {
    view()
        .flex_grow(1.0)
        .rounded(px(12.0))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .p(px(16.0))
        .child(
            text(label.to_string())
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            text(move || {
                if crate::ui::privacy_active() {
                    "••••".to_string()
                } else {
                    value()
                }
            })
            .text_size(px(24.0))
            .line_height(px(32.0))
            .letter_spacing(px(-0.6))
            .font_family(design::MONO)
            .font_weight(FontWeight::BOLD)
            .text_color(move || theme().foreground)
            .mt(px(8.0)),
        )
}

fn weekday_header(label: &'static str) -> impl IntoElement {
    view()
        .h(px(36.0))
        .items_center()
        .bg(move || theme().muted)
        .child(
            text(label)
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
}

fn mobile_calendar(
    data: Resource<CalendarData, String>,
    year: Signal<i32>,
    month: Signal<u32>,
    currency: String,
    today: NaiveDate,
    day_modal: ui::ModalHost,
    server: String,
    token: String,
    subscriptions: Resource<crate::api::SubscriptionsData, String>,
) -> AnyElement {
    let entries =
        move || calendar_entries(&data.get().unwrap_or_default(), year.get(), month.get());
    let total_currency = currency.clone();
    let total = move || {
        let snapshot = data.get().unwrap_or_default();
        entries()
            .iter()
            .map(|entry| snapshot.rates.convert(entry.amount, &entry.currency))
            .sum::<Decimal>()
    };
    let upcoming = move || {
        entries()
            .into_iter()
            .filter(|entry| entry.date >= today)
            .collect::<Vec<_>>()
    };
    let past = move || {
        entries()
            .into_iter()
            .filter(|entry| entry.date < today)
            .collect::<Vec<_>>()
    };
    let row_currency = currency.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(12.0))
        .child(
            view()
                .w_full()
                .rounded(px(12.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .child(navigation(year, month, today, true)),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            image(asset!("icons/calendar.svg"))
                                .size(px(20.0))
                                .tint_color(move || theme().foreground),
                        )
                        .child(
                            text("Payments this month")
                                .text_size(px(18.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    text(move || format_currency(total(), &total_currency))
                        .text_size(px(18.0))
                        .font_family(design::MONO)
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                ),
        )
        .child(mobile_section(
            "UPCOMING",
            upcoming,
            row_currency.clone(),
            today,
            data,
            day_modal,
            server.clone(),
            token.clone(),
            subscriptions,
        ))
        .child(mobile_section(
            "PAST",
            past,
            row_currency,
            today,
            data,
            day_modal,
            server,
            token,
            subscriptions,
        ))
        .into_element()
}

fn mobile_section(
    label: &'static str,
    entries: impl Fn() -> Vec<CalendarEntry> + Clone + 'static,
    currency: String,
    today: NaiveDate,
    data: Resource<CalendarData, String>,
    day_modal: ui::ModalHost,
    server: String,
    token: String,
    subscriptions: Resource<crate::api::SubscriptionsData, String>,
) -> impl IntoElement {
    let count_entries = entries.clone();
    let empty_entries = count_entries.clone();
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
                    text(label)
                        .text_size(px(11.0))
                        .letter_spacing(px(1.6))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    text(move || ui::count_label(count_entries().len(), "payment", "payments"))
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            when(
                move || label == "UPCOMING" && empty_entries().len() == 0,
                move || {
                    view()
                        .w_full()
                        .h(px(56.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border.opacity(0.6))
                        .child(
                            text("No upcoming payments left this month.")
                                .text_size(px(14.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .child(view().flex_col().gap(px(6.0)).child(each(
            entries,
            |entry| entry.key.clone(),
            move |entry, _| {
                let entry = entry.get();
                let past = entry.date < today;
                let date = if entry.date == today {
                    "Today".to_string()
                } else {
                    entry.date.format("%a, %b %e").to_string()
                };
                let amount = format_currency(entry.amount, &currency);
                let entry_date = entry.date;
                let dialog_currency = currency.clone();
                let dialog_server = server.clone();
                let dialog_token = token.clone();
                view()
                    .w_full()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .on_press(move || {
                        open_day_dialog(
                            entry_date,
                            data,
                            dialog_currency.clone(),
                            today,
                            day_modal,
                            dialog_server.clone(),
                            dialog_token.clone(),
                            subscriptions,
                        );
                    })
                    .gap(px(10.0))
                    .p(px(12.0))
                    .rounded(px(8.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .when(past, |row| row.opacity(0.55))
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            .min_w(px(0.0))
                            .child(
                                view()
                                    .h(px(40.0))
                                    .w(px(40.0))
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(999.0))
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
                                        text(entry.name)
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(move || theme().foreground),
                                    )
                                    .child(
                                        text(date)
                                            .text_size(px(11.0))
                                            .text_color(move || theme().muted_foreground),
                                    ),
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
}

fn nav_arrow(
    icon: ImageAsset,
    label: &'static str,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .cursor_pointer()
        .accessibility_label(label)
        .items_center()
        .justify_center()
        .h(px(32.0))
        .w(px(32.0))
        .rounded(px(6.0))
        .bg(move || theme().card.opacity(0.0))
        .on_press(on_press)
        .child(
            image(icon)
                .size(px(16.0))
                .tint_color(move || theme().foreground),
        )
}

fn open_day_dialog(
    date: NaiveDate,
    data: Resource<CalendarData, String>,
    currency: String,
    today: NaiveDate,
    modal: ui::ModalHost,
    server: String,
    token: String,
    subscriptions: Resource<crate::api::SubscriptionsData, String>,
) {
    modal.open(move || {
        day_dialog(
            date,
            data,
            currency.clone(),
            today,
            modal,
            server.clone(),
            token.clone(),
            subscriptions,
        )
    });
}

fn day_dialog(
    date: NaiveDate,
    data: Resource<CalendarData, String>,
    currency: String,
    today: NaiveDate,
    modal: ui::ModalHost,
    server: String,
    token: String,
    subscriptions: Resource<crate::api::SubscriptionsData, String>,
) -> AnyElement {
    let snapshot = data.get().unwrap_or_default();
    let money = subscriptions.get().unwrap_or_default();
    let entries = calendar_entries(&snapshot, date.year(), date.month())
        .into_iter()
        .filter(|entry| entry.date == date)
        .collect::<Vec<_>>();
    let title = date.format("%B %e %Y").to_string();
    let body = view()
        .w_full()
        .p(px(24.0))
        .gap(px(12.0))
        .flex_col()
        .children(entries.into_iter().map(|entry| {
            let past = entry.date < today;
            let is_subscription = !entry.is_transaction;
            let subscription_id = entry.subscription_id;
            let amount = format_currency(
                snapshot.rates.convert(entry.amount, &entry.currency),
                &currency,
            );
            let detail = if let Some(id) = entry.subscription_id {
                money
                    .subscriptions
                    .iter()
                    .find(|subscription| subscription.id == id)
                    .map_or_else(
                        || entry.frequency.clone(),
                        |subscription| {
                            let account = money
                                .accounts
                                .iter()
                                .find(|account| account.id == subscription.account_id)
                                .map_or_else(String::new, |account| account.name.clone());
                            let category = money
                                .categories
                                .iter()
                                .find(|category| category.id == subscription.category_id)
                                .map_or_else(
                                    || "Uncategorized".to_string(),
                                    |category| category.name.clone(),
                                );
                            format!("{account} · {category} · {}", entry.frequency)
                        },
                    )
            } else if let Some(id) = entry.transaction_id {
                snapshot
                    .transactions
                    .iter()
                    .find(|transaction| transaction.id == id)
                    .map_or_else(
                        || "One-time".to_string(),
                        |transaction| {
                            let account = money
                                .accounts
                                .iter()
                                .find(|account| account.id == transaction.account_id)
                                .map_or_else(String::new, |account| account.name.clone());
                            let category = money
                                .categories
                                .iter()
                                .find(|category| category.id == transaction.category_id)
                                .map_or_else(
                                    || "Uncategorized".to_string(),
                                    |category| category.name.clone(),
                                );
                            format!("{account} · {category} · One-time")
                        },
                    )
            } else {
                entry.frequency.clone()
            };
            let edit_server = server.clone();
            let edit_token = token.clone();
            let delete_server = server.clone();
            let delete_token = token.clone();
            view()
                .w_full()
                .p(px(14.0))
                .gap(px(8.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .when(past, |card| card.opacity(0.55))
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap(px(12.0))
                        .child(
                            view()
                                .flex_col()
                                .min_w(px(0.0))
                                .child(
                                    text(entry.name.clone())
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    text(detail)
                                        .text_size(px(11.0))
                                        .text_color(move || theme().muted_foreground),
                                ),
                        )
                        .child(
                            text(amount)
                                .text_size(px(16.0))
                                .font_family(crate::theme::MONO)
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        ),
                )
                .when(is_subscription, move |card| {
                    let edit_server = edit_server.clone();
                    let edit_token = edit_token.clone();
                    let delete_server = delete_server.clone();
                    let delete_token = delete_token.clone();
                    card.child(
                        view()
                            .flex_row()
                            .gap(px(8.0))
                            .pt(px(4.0))
                            .child(ui::action_button("Edit", false, move || {
                                let _ = (&edit_server, &edit_token);
                                if let Some(id) = subscription_id {
                                    crate::ui::request_subscription_edit(id);
                                }
                                modal.close();
                            }))
                            .child(ui::action_button("Delete", true, move || {
                                if let Some(id) = subscription_id {
                                    let server = delete_server.clone();
                                    let token = delete_token.clone();
                                    spawn(async move {
                                        let _ = crate::api::delete_subscription(server, token, id)
                                            .await;
                                        data.refetch();
                                        modal.close();
                                    });
                                }
                            })),
                    )
                })
                .into_element()
        }))
        .into_element();
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(title, "", move || modal.close())
            .width(720.0)
            .footer_visible(false)
            .body(body),
    )
    .into_element()
}

fn day_cell(
    day: Option<u32>,
    data: Resource<CalendarData, String>,
    year: Signal<i32>,
    month: Signal<u32>,
    currency: String,
    today: NaiveDate,
    day_modal: ui::ModalHost,
    server: String,
    token: String,
    subscriptions: Resource<crate::api::SubscriptionsData, String>,
) -> impl IntoElement {
    let entries = move || {
        let Some(day) = day else {
            return Vec::new();
        };
        let snapshot = data.get().unwrap_or_default();
        calendar_entries(&snapshot, year.get(), month.get())
            .into_iter()
            .filter(|entry| entry.date.day() == day)
            .collect::<Vec<_>>()
    };
    let truncated = move || {
        let mut entries = entries();
        if entries.len() > 3 {
            entries.truncate(3);
        }
        entries
    };
    let is_today = day.is_some_and(|day| {
        year.get() == today.year() && month.get() == today.month() && day == today.day()
    });
    let cell_date = day.and_then(|day| NaiveDate::from_ymd_opt(year.get(), month.get(), day));
    let press_currency_1 = currency.clone();
    let press_server_1 = server.clone();
    let press_token_1 = token.clone();
    let press_currency_2 = currency.clone();
    let press_server_2 = server.clone();
    let press_token_2 = token.clone();
    let hovered = signal(false);
    view()
        .w_full()
        .min_h(px(100.0))
        .p(px(8.0))
        .bg(move || {
            if day.is_some() {
                theme().card
            } else {
                theme().muted.opacity(0.15)
            }
        })
        .when(day.is_some(), move |cell| {
            cell.cursor_pointer()
                .on_mouse_enter(move |_: &MouseEvent| hovered.set(true))
                .on_mouse_leave(move |_: &MouseEvent| hovered.set(false))
                .on_press(move || {
                    let Some(date) = cell_date else { return };
                    if entries().is_empty() {
                        crate::ui::create_subscription(Some(date));
                    } else {
                        open_day_dialog(
                            date,
                            data,
                            press_currency_1.clone(),
                            today,
                            day_modal,
                            press_server_1.clone(),
                            press_token_1.clone(),
                            subscriptions,
                        );
                    }
                })
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .cursor_pointer()
                        .on_press(move || {
                            let Some(date) = cell_date else { return };
                            if entries().is_empty() {
                                crate::ui::create_subscription(Some(date));
                            } else {
                                open_day_dialog(
                                    date,
                                    data,
                                    press_currency_2.clone(),
                                    today,
                                    day_modal,
                                    press_server_2.clone(),
                                    press_token_2.clone(),
                                    subscriptions,
                                );
                            }
                        })
                        .child(
                            view()
                                .h(px(28.0))
                                .w(px(28.0))
                                .mb(px(4.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(999.0))
                                .bg(move || {
                                    if is_today {
                                        theme().primary
                                    } else {
                                        theme().card.opacity(0.0)
                                    }
                                })
                                .child(
                                    text(move || {
                                        day.map_or_else(String::new, |day| day.to_string())
                                    })
                                    .text_size(px(14.0))
                                    .line_height(px(20.0))
                                    .font_weight(if is_today {
                                        FontWeight::BOLD
                                    } else {
                                        FontWeight::NORMAL
                                    })
                                    .text_color(move || {
                                        if is_today {
                                            theme().primary_foreground
                                        } else {
                                            theme().foreground
                                        }
                                    }),
                                ),
                        )
                        .child(
                            button()
                                .cursor_pointer()
                                .opacity(move || if hovered.get() { 1.0_f32 } else { 0.0_f32 })
                                .h(px(16.0))
                                .w(px(16.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(4.0))
                                .accessibility_label("Add subscription on this day")
                                .on_press(move || {
                                    if let Some(date) = cell_date {
                                        crate::ui::create_subscription(Some(date));
                                    }
                                })
                                .child(
                                    image(asset!("icons/plus.svg"))
                                        .size(px(11.0))
                                        .tint_color(move || theme().muted_foreground),
                                ),
                        ),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(4.0))
                        .child(each(
                            truncated,
                            |entry| entry.key.clone(),
                            move |entry, _| {
                                let entry = entry.get();
                                let amount = format_currency(
                                    data.get()
                                        .unwrap_or_default()
                                        .rates
                                        .convert(entry.amount, &entry.currency),
                                    &currency,
                                );
                                view()
                                    .w_full()
                                    .h(px(20.0))
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(4.0))
                                    .px(px(4.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .bg(move || theme().primary.opacity(0.1))
                                    .child(
                                        view()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(4.0))
                                            .min_w(px(0.0))
                                            .child(
                                                image(asset!("icons/card.svg"))
                                                    .size(px(16.0))
                                                    .tint_color(move || theme().muted_foreground),
                                            )
                                            .child(
                                                text(entry.name)
                                                    .text_size(px(12.0))
                                                    .line_height(px(16.0))
                                                    .text_color(move || theme().foreground)
                                                    .number_of_lines(1),
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
                                        .text_size(px(12.0))
                                        .line_height(px(16.0))
                                        .font_family(design::MONO)
                                        .text_color(move || theme().muted_foreground),
                                    )
                            },
                        ))
                        .child(
                            when(
                                move || entries().len() > 3,
                                move || {
                                    text(format!("+{} more", entries().len().saturating_sub(3)))
                                        .text_size(px(9.0))
                                        .line_height(px(16.0))
                                        .text_color(move || theme().muted_foreground)
                                },
                            )
                            .otherwise(|| view().h(px(0.0))),
                        ),
                )
        })
}

fn calendar_entries(data: &CalendarData, year: i32, month: u32) -> Vec<CalendarEntry> {
    let mut entries = Vec::new();
    for item in &data.items {
        if item.billing_date.year() == year && item.billing_date.month() == month {
            entries.push(CalendarEntry {
                key: format!(
                    "subscription-{}-{}-{}-{}",
                    item.subscription_id, item.billing_date, item.name, item.amount
                ),
                name: item.name.clone(),
                amount: item.amount,
                currency: item.currency.clone(),
                date: item.billing_date,
                is_transaction: false,
                subscription_id: Some(item.subscription_id),
                transaction_id: None,
                frequency: item.frequency.clone(),
            });
        }
    }
    for transaction in &data.transactions {
        if transaction.date.year() == year && transaction.date.month() == month {
            entries.push(CalendarEntry {
                key: format!(
                    "transaction-{}-{}-{}",
                    transaction.id, transaction.name, transaction.amount
                ),
                name: transaction.name.clone(),
                amount: transaction.amount,
                currency: transaction.currency.clone(),
                date: transaction.date,
                is_transaction: true,
                subscription_id: None,
                transaction_id: Some(transaction.id),
                frequency: "One-time".to_string(),
            });
        }
    }
    entries.sort_by_key(|entry| entry.date);
    entries
}

fn cells_for(year: i32, month: u32) -> Vec<(usize, Option<u32>)> {
    let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return Vec::new();
    };
    let leading = if crate::format::week_starts_sunday() {
        first.weekday().num_days_from_sunday() as usize
    } else {
        first.weekday().num_days_from_monday() as usize
    };
    let days = days_in_month(year, month);
    let mut cells = Vec::with_capacity(42);
    for index in 0..leading {
        cells.push((index, None));
    }
    for day in 1..=days {
        cells.push((cells.len(), Some(day)));
    }
    while cells.len() % 7 != 0 {
        cells.push((cells.len(), None));
    }
    cells
}

fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let total = year * 12 + month as i32 - 1 + delta;
    (total.div_euclid(12), (total.rem_euclid(12) + 1) as u32)
}

fn month_label(year: i32, month: u32) -> String {
    NaiveDate::from_ymd_opt(year, month, 1)
        .map_or_else(String::new, |date| date.format("%B %Y").to_string())
}

fn loading_card() -> AnyElement {
    ui::surface()
        .p(px(28.0))
        .child(
            text("Loading calendar…")
                .text_size(px(14.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element()
}

fn error_card(message: &str, data: Resource<CalendarData, String>) -> AnyElement {
    let message = message.to_string();
    ui::surface()
        .p(px(24.0))
        .gap(px(12.0))
        .child(
            text("The calendar could not be loaded.")
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

    #[test]
    fn november_2026_starts_on_sunday_with_six_leading_cells() {
        let cells = cells_for(2026, 11);
        assert_eq!(cells.iter().filter(|(_, day)| day.is_none()).count(), 12);
        assert_eq!(cells[6].1, Some(1));
    }

    #[test]
    fn shift_month_crosses_year_boundaries() {
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 6, 0), (2026, 6));
    }

    #[test]
    fn month_label_uses_the_calendar_name() {
        assert_eq!(month_label(2026, 11), "November 2026");
    }
}
