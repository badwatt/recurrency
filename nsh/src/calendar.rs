//! Month calendar with scheduled subscriptions and one-time transactions.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, theme};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::api::{self, CalendarData};
use crate::auth::AuthSession;
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
}

pub fn calendar(auth: AuthSession) -> AnyElement {
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
        move || Some((year.get(), month.get())),
        move |(load_year, load_month), _| {
            let server = resource_server.clone();
            let token = resource_token.clone();
            let currency = load_currency.clone();
            async move {
                api::load_calendar(server, token, currency, load_year, load_month).await
            }
        },
    );
    let loading = move || data.loading() && data.get().is_none();
    let has_error = move || data.error().is_some();
    let render_currency = currency.clone();

    view()
        .w_full()
        .flex_col()
        .gap(px(24.0))
        .child(header())
        .child(navigation(year, month, today))
        .child(
            when(loading, || loading_card()).otherwise(move || {
                let currency = render_currency.clone();
                when(has_error, move || {
                    error_card(&data.error().unwrap_or_default(), data)
                })
                .otherwise(move || content(data, year, month, currency.clone(), today))
            }),
        )
        .into_element()
}

fn header() -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(4.0))
        .child(ui::page_header(TITLE, SUBTITLE))
}

fn navigation(year: Signal<i32>, month: Signal<u32>, today: NaiveDate) -> impl IntoElement {
    view()
        .w_full()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .child(
            button()
                .h(px(32.0))
                .px(px(10.0))
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .on_press(move || {
                    let (next_year, next_month) = shift_month(year.get(), month.get(), -1);
                    year.set(next_year);
                    month.set(next_month);
                })
                .child(
                    text("Previous")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                ),
        )
        .child(
            text(move || month_label(year.get(), month.get()))
                .text_size(px(20.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            button()
                .h(px(32.0))
                .px(px(10.0))
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().card)
                .on_press(move || {
                    let (next_year, next_month) = shift_month(year.get(), month.get(), 1);
                    year.set(next_year);
                    month.set(next_month);
                })
                .child(
                    text("Next")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                ),
        )
        .child(
            button()
                .h(px(32.0))
                .px(px(10.0))
                .rounded(px(6.0))
                .on_press(move || {
                    year.set(today.year());
                    month.set(today.month());
                })
                .child(
                    text("Today")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().primary),
                ),
        )
}

fn content(
    data: Resource<CalendarData, String>,
    year: Signal<i32>,
    month: Signal<u32>,
    currency: String,
    today: NaiveDate,
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
            .filter(|entry| entry.date >= today)
            .map(|entry| snapshot.rates.convert(entry.amount, &entry.currency))
            .sum::<Decimal>()
    };
    let count = move || calendar_entries(&data.get().unwrap_or_default(), year.get(), month.get()).len();
    let cells = move || cells_for(year.get(), month.get());

    view()
        .w_full()
        .flex_col()
        .gap(px(20.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .gap(px(12.0))
                .child(summary(
                    "PAYMENTS THIS MONTH",
                    move || format_currency(total(), &total_currency),
                ))
                .child(summary(
                    "PENDING FROM TODAY",
                    move || format_currency(pending(), &pending_currency),
                )),
        )
        .child(
            text(move || ui::count_label(count(), "payment", "payments"))
                .text_size(px(13.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
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
                        .grid()
                        .grid_template_columns(SharedString::from("repeat(7, minmax(0, 1fr))"))
                        .bg(move || theme().border)
                        .gap(px(1.0))
                        .child(weekday_header("MON"))
                        .child(weekday_header("TUE"))
                        .child(weekday_header("WED"))
                        .child(weekday_header("THU"))
                        .child(weekday_header("FRI"))
                        .child(weekday_header("SAT"))
                        .child(weekday_header("SUN"))
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
                                    currency.clone(),
                                    today,
                                )
                            },
                        )),
                ),
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
        .p(px(18.0))
        .child(
            text(label.to_string())
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            text(value)
                .text_size(px(28.0))
                .font_family(design::MONO)
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground)
                .mt(px(6.0)),
        )
}

fn weekday_header(label: &'static str) -> impl IntoElement {
    view()
        .py(px(8.0))
        .items_center()
        .bg(move || theme().muted)
        .child(
            text(label)
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().muted_foreground),
        )
}

fn day_cell(
    day: Option<u32>,
    data: Resource<CalendarData, String>,
    year: Signal<i32>,
    month: Signal<u32>,
    currency: String,
    today: NaiveDate,
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
    let is_today = day.is_some_and(|day| {
        year.get() == today.year() && month.get() == today.month() && day == today.day()
    });
    view()
        .w_full()
        .min_h(px(92.0))
        .p(px(6.0))
        .gap(px(2.0))
        .bg(move || if day.is_some() { theme().card } else { theme().muted.opacity(0.15) })
        .when(is_today, |cell| cell.bg(move || theme().accent))
        .when(day.is_some(), move |cell| {
            cell.child(
                text(move || day.map_or_else(String::new, |day| day.to_string()))
                    .text_size(px(11.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(move || if is_today { theme().primary } else { theme().muted_foreground }),
            )
            .child(
                view()
                    .flex_col()
                    .gap(px(1.0))
                    .child(each(
                        entries,
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
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .gap(px(2.0))
                                .child(
                                    text(entry.name)
                                        .text_size(px(9.0))
                                        .text_color(move || theme().foreground)
                                        .number_of_lines(1),
                                )
                                .child(
                                    text(amount)
                                        .text_size(px(8.0))
                                        .font_family(design::MONO)
                                        .text_color(move || theme().muted_foreground),
                                )
                        },
                    )),
            )
        })
}

fn calendar_entries(data: &CalendarData, year: i32, month: u32) -> Vec<CalendarEntry> {
    let mut entries = Vec::new();
    for item in &data.items {
        if item.billing_date.year() == year && item.billing_date.month() == month {
            entries.push(CalendarEntry {
                key: format!("subscription-{}-{}", item.subscription_id, item.billing_date),
                name: item.name.clone(),
                amount: item.amount,
                currency: item.currency.clone(),
                date: item.billing_date,
                is_transaction: false,
            });
        }
    }
    for transaction in &data.transactions {
        if transaction.date.year() == year && transaction.date.month() == month {
            entries.push(CalendarEntry {
                key: format!("transaction-{}", transaction.id),
                name: transaction.name.clone(),
                amount: transaction.amount,
                currency: transaction.currency.clone(),
                date: transaction.date,
                is_transaction: true,
            });
        }
    }
    entries.sort_by_key(|entry| (entry.date, entry.name.clone()));
    entries
}

fn cells_for(year: i32, month: u32) -> Vec<(usize, Option<u32>)> {
    let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return Vec::new();
    };
    let leading = first.weekday().num_days_from_monday() as usize;
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
