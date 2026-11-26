//! Billing schedule card: interval, unit and the unit-specific date picker.

use chrono::{Datelike, Local, NaiveDate};
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::components::charge_days::charge_days;
use crate::components::select::{SelectProps, select};
use crate::components::string_select::{StringSelectProps, string_select};
use crate::models::AnnualDate;
use crate::reports::month_name;
use crate::ui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BillingMode {
    Full,
    Monthly,
}

#[derive(Clone, Copy)]
pub struct BillingScheduleProps {
    pub interval: Signal<String>,
    pub unit: Signal<String>,
    pub days: Signal<Vec<u8>>,
    pub weekdays: Signal<Vec<u8>>,
    pub annual_dates: Signal<Vec<AnnualDate>>,
    pub start_date: Signal<String>,
    pub mode: BillingMode,
}

pub fn billing_schedule(props: BillingScheduleProps) -> impl IntoElement {
    let BillingScheduleProps {
        interval,
        unit,
        days,
        weekdays,
        annual_dates,
        start_date,
        mode,
    } = props;
    effect(move || {
        if unit.get() == "week" && weekdays.get().is_empty() {
            let start = NaiveDate::parse_from_str(start_date.get().trim(), "%Y-%m-%d")
                .unwrap_or_else(|_| Local::now().date_naive());
            weekdays.set(vec![start.weekday().number_from_monday() as u8]);
        }
    });
    effect(move || {
        if unit.get() == "year" && annual_dates.get().is_empty() {
            let today = Local::now().date_naive();
            annual_dates.set(vec![AnnualDate {
                month: today.month() as u8,
                day: today.day() as u8,
                amount: None,
            }]);
        }
    });
    view()
    .w_full()
    .mt(px(-2.0))
    .rounded(px(12.0))
    .border(px(1.0))
    .border_color(move || theme().border.opacity(0.7))
    .bg(move || theme().card)
    .p(px(20.0))
    .gap(px(18.0))
    .child(
        view()
            .w_full()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(px(14.0))
            .child(
                view()
                    .h(px(42.0))
                    .w(px(42.0))
                    .items_center()
                    .justify_center()
                    .rounded(px(11.0))
                    .bg(move || theme().primary.opacity(0.12))
                    .child(
                        image(asset!("icons/calendar.svg"))
                            .size(px(20.0))
                            .tint_color(move || theme().primary),
                    ),
            )
            .child(
                view()
                    .flex_grow(1.0)
                    .basis(px(220.0))
                    .min_w(px(0.0))
                    .flex_col()
                    .gap(px(3.0))
                    .child(
                        text("Billing schedule *")
                            .text_size(px(15.0))
                            .line_height(px(22.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().foreground),
                    )
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
                        .line_height(px(18.0))
                        .text_color(move || theme().muted_foreground),
                    ),
            )
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(10.0))
                    .py(px(6.0))
                    .rounded(px(999.0))
                    .bg(move || theme().muted.opacity(0.5))
                    .child(
                        image(asset!("icons/clock.svg"))
                            .size(px(12.0))
                            .tint_color(move || theme().muted_foreground),
                    )
                    .child(
                        text(move || {
                            let first = NaiveDate::parse_from_str(
                                start_date.get().trim(),
                                "%Y-%m-%d",
                            )
                            .unwrap_or_else(|_| Local::now().date_naive());
                            format!(
                                "First charge · {}",
                                crate::format::format_date(first)
                            )
                        })
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().muted_foreground),
                    ),
            ),
    )
    .child(
        when(
            move || mode == BillingMode::Full,
            move || {
                view()
                    .w_full()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        text("REPEAT EVERY")
                    .text_size(px(10.0))
                    .letter_spacing(px(0.9))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(move || theme().muted_foreground),
            )
            .child(
                view()
                    .w_full()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.0))
                    .child(
                        view()
                            .w(px(92.0))
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                text("Interval")
                                    .text_size(px(11.0))
                                    .text_color(move || theme().muted_foreground),
                            )
                            .child(
                                input()
                                    .value(interval)
                                    .on_change(move |value: &str| {
                                        let clean = crate::components::input::sanitize_integer(
                                            value,
                                        );
                                        if clean != value {
                                            interval.set(value.to_string());
                                            let target = interval;
                                            spawn(async move { target.set(clean) });
                                        } else {
                                            interval.set(clean);
                                        }
                                    })
                                    .auto_fill(AutoFill::Off)
                                    .h(px(40.0))
                                    .w_full()
                                    .px(px(12.0))
                                    .rounded(px(8.0))
                                    .border(px(1.0))
                                    .border_color(move || theme().border)
                                    .bg(move || theme().background)
                                    .text_size(px(14.0))
                                    .text_color(move || theme().foreground),
                            ),
                    )
                    .child(
                        view()
                            .w(px(190.0))
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                text("Unit")
                                    .text_size(px(11.0))
                                    .text_color(move || theme().muted_foreground),
                            )
                            .child(
                                view().w_full().h(px(40.0)).child(string_select(
                                    StringSelectProps::new(
                                        "",
                                        unit,
                                        vec![
                                            ("day".to_string(), "day".to_string()),
                                            ("week".to_string(), "week".to_string()),
                                            ("month".to_string(), "month".to_string()),
                                            ("year".to_string(), "year".to_string()),
                                        ],
                                    )
                                    .show_label(false),
                                )),
                            ),
                    ),
            )
                .into_element()
            },
        )
        .otherwise(|| view().h(px(0.0)).into_element()),
    )
    .child(
        when(
            move || unit.get() == "year",
            move || annual_dates_editor(annual_dates).into_element(),
        )
        .otherwise(move || {
            when(
                move || unit.get() == "week",
                move || weekday_picker(weekdays).into_element(),
            )
            .otherwise(move || {
                when(
                    move || unit.get() == "day",
                    move || {
                        view()
                            .w_full()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .p(px(12.0))
                            .rounded(px(8.0))
                            .bg(move || theme().primary.opacity(0.07))
                            .border(px(1.0))
                            .border_color(move || theme().primary.opacity(0.16))
                            .child(
                                image(asset!("icons/info.svg"))
                                    .size(px(14.0))
                                    .tint_color(move || theme().primary),
                            )
                            .child(
                                text("Charges every selected number of days from the activation date.")
                                    .text_size(px(12.0))
                                    .text_color(move || theme().muted_foreground),
                            )
                            .into_element()
                    },
                )
                .otherwise(move || {
                    charge_days("Charge on these days", days).into_element()
                })
                .into_element()
            })
            .into_element()
        }),
    )
}

fn annual_dates_editor(dates: Signal<Vec<AnnualDate>>) -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            view()
                .w_full()
                .flex_col()
                .gap(px(2.0))
                .child(
                    text("Annual dates")
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("Each date creates one charge every year.")
                        .text_size(px(11.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(view().w_full().flex_col().children((0..12).map(|index| {
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
                        .mb(px(6.0))
                        .child(annual_day_field(date.day.to_string(), move |value| {
                            let mut next = dates.get();
                            if let Some(slot) = next.get_mut(index) {
                                slot.day = value.parse().unwrap_or(0);
                            }
                            dates.set(next);
                        }))
                        .child(annual_month_field(dates, index))
                        .child(annual_amount_field(
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
                        .child(view().h(px(57.0)).pt(px(22.0)).flex_shrink_0().child(
                            ui::mini_icon_button_labeled(
                                asset!("icons/trash.svg"),
                                "Remove annual date",
                                true,
                                move || {
                                    let mut next = dates.get();
                                    if index < next.len() {
                                        next.remove(index);
                                    }
                                    dates.set(next);
                                },
                            ),
                        ))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element())
            .into_element()
        })))
        .child(
            button()
                .cursor_pointer()
                .w_full()
                .h(px(40.0))
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(8.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().primary.opacity(0.35))
                .bg(move || theme().primary.opacity(0.06))
                .on_press(move || {
                    let mut next = dates.get();
                    next.push(AnnualDate {
                        month: 1,
                        day: 1,
                        amount: None,
                    });
                    dates.set(next);
                })
                .child(
                    image(asset!("icons/plus.svg"))
                        .size(px(14.0))
                        .tint_color(move || theme().primary),
                )
                .child(
                    text("Add annual date")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().primary),
                ),
        )
}

fn annual_month_field(dates: Signal<Vec<AnnualDate>>, index: usize) -> impl IntoElement {
    let options = move || {
        (1..=12u32)
            .map(|month| (month.to_string(), month_name(month).to_string()))
            .collect::<Vec<_>>()
    };
    view()
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(0.0))
        .h(px(57.0))
        .flex_col()
        .gap(px(4.0))
        .child(
            text("Month")
                .text_size(px(10.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view().h(px(40.0)).child(select(
                SelectProps::new(
                    "Month",
                    "Month",
                    move || {
                        dates.get().get(index).map_or_else(
                            || month_name(1).to_string(),
                            |date| month_name(u32::from(date.month)).to_string(),
                        )
                    },
                    move || {
                        dates
                            .get()
                            .get(index)
                            .map_or_else(String::new, |date| date.month.to_string())
                    },
                    move || dates.get().len() > index,
                    options,
                    move |value| {
                        let mut next = dates.get();
                        if let Some(slot) = next.get_mut(index) {
                            slot.month = value.parse().unwrap_or(1);
                        }
                        dates.set(next);
                    },
                )
                .searchable(false)
                .show_label(false)
                .height(40.0),
            )),
        )
}

fn annual_day_field(value: String, on_change: impl Fn(String) + 'static) -> impl IntoElement {
    view()
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(0.0))
        .flex_col()
        .gap(px(4.0))
        .child(
            text("Day")
                .text_size(px(10.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(crate::components::input::integer_input_value(
            value, "", on_change,
        ))
}

fn annual_amount_field(value: String, on_change: impl Fn(String) + 'static) -> impl IntoElement {
    view()
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(0.0))
        .flex_col()
        .gap(px(4.0))
        .child(
            text("Amount (optional)")
                .text_size(px(10.0))
                .text_color(move || theme().muted_foreground),
        )
        .child(crate::components::input::numeric_input_value(
            value, "", on_change,
        ))
}

fn weekday_picker(weekdays: Signal<Vec<u8>>) -> impl IntoElement {
    const LABELS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            text("Charge on")
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .flex_wrap()
                .gap(px(4.0))
                .children(LABELS.into_iter().enumerate().map(|(index, label)| {
                    let day = index as u8 + 1;
                    let active = move || weekdays.get().contains(&day);
                    button()
                        .cursor_pointer()
                        .h(px(28.0))
                        .px(px(10.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .border(px(1.0))
                        .border_color(move || {
                            if active() {
                                theme().primary
                            } else {
                                theme().border.opacity(0.6)
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
                            let mut values = weekdays.get();
                            if let Some(position) = values.iter().position(|value| *value == day) {
                                values.remove(position);
                            } else {
                                values.push(day);
                                values.sort_unstable();
                            }
                            weekdays.set(values);
                        })
                        .child(
                            text(label)
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(move || {
                                    if active() {
                                        theme().primary_foreground
                                    } else {
                                        theme().muted_foreground
                                    }
                                }),
                        )
                })),
        )
}
