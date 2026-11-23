//! Themed date field with a custom calendar panel, mirroring the Dioxus
//! `DatePicker`. The native date widget is replaceable on every platform,
//! so the app owns the calendar instead of the OS popup.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, theme};
use nsh_core::prelude::*;

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn month_start(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
}

fn shift_month(date: NaiveDate, months: i32) -> NaiveDate {
    let index = date.year() * 12 + date.month0() as i32 + months;
    let year = index.div_euclid(12);
    let month = index.rem_euclid(12) as u32 + 1;
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(date)
}

fn days_in_month(date: NaiveDate) -> u32 {
    let next = shift_month(month_start(date), 1);
    next.signed_duration_since(month_start(date))
        .num_days()
        .clamp(0, 31) as u32
}

pub fn date_picker(value: Signal<String>) -> impl IntoElement {
    let open = signal(false);
    let initial = parse_date(&value.get()).unwrap_or_else(|| Local::now().date_naive());
    let month = signal(month_start(initial));
    let cells = move || {
        let first = month.get();
        let leading = if crate::format::week_starts_sunday() {
            first.weekday().num_days_from_sunday() as usize
        } else {
            first.weekday().num_days_from_monday() as usize
        };
        let total = days_in_month(first) as usize;
        let selected = parse_date(&value.get());
        let today = Local::now().date_naive();
        let mut items = Vec::with_capacity(leading + total);
        for _ in 0..leading {
            items.push(view().h(px(28.0)).into_element());
        }
        for day in 1..=total {
            let date =
                NaiveDate::from_ymd_opt(first.year(), first.month(), day as u32).unwrap_or(first);
            let is_selected = selected == Some(date);
            let is_today = date == today;
            items.push(
                button()
                    .cursor_pointer()
                    .cursor_pointer()
                    .h(px(28.0))
                    .w_full()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .bg(move || {
                        if is_selected {
                            theme().primary
                        } else if is_today {
                            theme().accent
                        } else {
                            theme().card.opacity(0.0)
                        }
                    })
                    .on_press(move || {
                        value.set(date.format("%Y-%m-%d").to_string());
                        open.set(false);
                    })
                    .child(
                        text(day.to_string())
                            .text_size(px(11.0))
                            .font_weight(if is_selected {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(move || {
                                if is_selected {
                                    theme().primary_foreground
                                } else {
                                    theme().foreground
                                }
                            }),
                    )
                    .into_element(),
            );
        }
        items
    };
    let open_calendar = std::rc::Rc::new(move || {
        if let Some(date) = parse_date(&value.get()) {
            month.set(month_start(date));
        }
        open.set(!open.get());
    });
    let open_field = open_calendar.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            button()
                .cursor_pointer()
                .w_full()
                .h(px(38.0))
                .px(px(12.0))
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .rounded(px(7.0))
                .border(px(1.0))
                .border_color(move || theme().input)
                .bg(move || theme().card)
                .on_press(move || open_field())
                .child(
                    text(move || {
                        parse_date(&value.get())
                            .map_or_else(|| "Select date".to_string(), crate::format::format_date)
                    })
                    .text_size(px(13.0))
                    .text_color(move || theme().foreground),
                )
                .child(
                    image(asset!("icons/calendar.svg"))
                        .size(px(15.0))
                        .pointer_events(PointerEvents::None)
                        .tint_color(move || theme().muted_foreground),
                ),
        )
        .child(
            when(open, move || {
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(8.0))
                    .p(px(12.0))
                    .rounded(px(10.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().popover)
                    .child(
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                button()
                                    .cursor_pointer()
                                    .h(px(26.0))
                                    .w(px(26.0))
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(5.0))
                                    .on_press(move || month.set(shift_month(month.get(), -1)))
                                    .child(
                                        image(asset!("icons/chevron-left.svg"))
                                            .size(px(14.0))
                                            .tint_color(move || theme().muted_foreground),
                                    ),
                            )
                            .child(
                                text(move || month.get().format("%B %Y").to_string())
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            )
                            .child(
                                button()
                                    .cursor_pointer()
                                    .h(px(26.0))
                                    .w(px(26.0))
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(5.0))
                                    .on_press(move || month.set(shift_month(month.get(), 1)))
                                    .child(
                                        image(asset!("icons/chevron-right.svg"))
                                            .size(px(14.0))
                                            .tint_color(move || theme().muted_foreground),
                                    ),
                            ),
                    )
                    .child(
                        view()
                            .w_full()
                            .grid()
                            .grid_template_columns(SharedString::from("repeat(7, minmax(0, 1fr))"))
                            .gap(px(2.0))
                            .children({
                                let labels: [&'static str; 7] =
                                    if crate::format::week_starts_sunday() {
                                        ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
                                    } else {
                                        WEEKDAYS
                                    };
                                labels
                                    .into_iter()
                                    .map(|day| {
                                        view()
                                            .items_center()
                                            .justify_center()
                                            .py(px(2.0))
                                            .child(
                                                text(day)
                                                    .text_size(px(9.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(move || theme().muted_foreground),
                                            )
                                            .into_element()
                                    })
                                    .collect::<Vec<_>>()
                            }),
                    )
                    .child(
                        view()
                            .w_full()
                            .grid()
                            .grid_template_columns(SharedString::from("repeat(7, minmax(0, 1fr))"))
                            .gap(px(2.0))
                            .children(cells()),
                    )
                    .child(
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .pt(px(6.0))
                            .border_t(px(1.0))
                            .border_color(move || theme().border)
                            .child(
                                button()
                                    .cursor_pointer()
                                    .h(px(24.0))
                                    .px(px(6.0))
                                    .rounded(px(5.0))
                                    .on_press(move || {
                                        value.set(String::new());
                                        open.set(false);
                                    })
                                    .child(
                                        text("Clear")
                                            .text_size(px(11.0))
                                            .text_color(move || theme().muted_foreground),
                                    ),
                            )
                            .child(
                                button()
                                    .cursor_pointer()
                                    .h(px(24.0))
                                    .px(px(6.0))
                                    .rounded(px(5.0))
                                    .on_press(move || {
                                        let today = Local::now().date_naive();
                                        value.set(today.format("%Y-%m-%d").to_string());
                                        month.set(month_start(today));
                                        open.set(false);
                                    })
                                    .child(
                                        text("Today")
                                            .text_size(px(11.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(move || theme().primary),
                                    ),
                            ),
                    )
                    .into_element()
            })
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_grid_starts_on_monday() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 22).unwrap_or_default();
        assert_eq!(month_start(date).day(), 1);
        assert_eq!(days_in_month(date), 30);
        assert_eq!(shift_month(date, 1).month(), 12);
        assert_eq!(shift_month(date, -1).month(), 10);
    }

    #[test]
    fn parse_date_rejects_free_text() {
        assert!(parse_date("2026-11-22").is_some());
        assert!(parse_date("22/11/2026").is_none());
    }
}
