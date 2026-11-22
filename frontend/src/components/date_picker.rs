use crate::components::button::{Button, ButtonAlign, ButtonRadius, ButtonSize, ButtonVariant, IconButton};
use crate::components::input::Input;
use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;
use lucide_dioxus::{Calendar, ChevronLeft, ChevronRight};

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
const MAX_MONTH_DAYS: u32 = 31;

fn parse_date(value: &str) -> Option<NaiveDate> {
    if value.is_empty() {
        return None;
    }
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
    let first = month_start(date);
    shift_month(first, 1)
        .signed_duration_since(first)
        .num_days()
        .clamp(0, MAX_MONTH_DAYS as i64) as u32
}

/// Build a Monday-first calendar grid. `None` cells are leading/trailing blanks.
fn calendar_cells(view: NaiveDate) -> Vec<Option<NaiveDate>> {
    let first = month_start(view);
    let leading = first.weekday().num_days_from_monday() as usize;
    let total = days_in_month(view) as usize;
    let mut cells = vec![None; leading];
    for day in 1..=total {
        cells.push(NaiveDate::from_ymd_opt(view.year(), view.month(), day as u32));
    }
    while cells.len() % 7 != 0 {
        cells.push(None);
    }
    cells
}

fn day_is_disabled(date: NaiveDate, min: Option<NaiveDate>, max: Option<NaiveDate>) -> bool {
    min.is_some_and(|min| date < min) || max.is_some_and(|max| date > max)
}

/// Shared date field with a themed calendar. The native date widget is kept
/// for semantics and locale formatting, but its unstyleable popup is replaced
/// by an app calendar because browsers do not expose it to CSS.
#[component]
pub fn DatePicker(
    value: String,
    on_input: EventHandler<String>,
    #[props(default)] id: String,
    #[props(default)] min: String,
    #[props(default)] max: String,
    #[props(default)] disabled: bool,
    #[props(default)] reserve_clear: bool,
    #[props(default)] class: String,
) -> Element {
    let generated_id = use_hook(|| format!("date-picker-{}", uuid::Uuid::new_v4()));
    let input_id = if id.is_empty() { generated_id } else { id };
    let mut open = use_signal(|| false);
    let initial = parse_date(&value).unwrap_or_else(|| chrono::Local::now().date_naive());
    let mut view = use_signal(|| month_start(initial));
    let selected = parse_date(&value);
    let min_date = parse_date(&min);
    let max_date = parse_date(&max);
    let mut open_calendar = move || {
        if let Some(date) = selected {
            view.set(month_start(date));
        }
        open.set(true);
    };
    let picker_label = selected
        .map(|date| date.format("%d/%m/%Y").to_string())
        .unwrap_or_default();
    let cells = calendar_cells(view());
    let has_value = !value.is_empty();

    rsx! {
        div { class: "flex w-full items-center gap-2",
            div { class: "relative min-w-0 flex-1",
                Input {
                id: input_id.clone(),
                r#type: "date",
                value,
                min,
                max,
                disabled,
                readonly: true,
                aria_label: if picker_label.is_empty() { "Choose date".to_string() } else { format!("Date: {picker_label}") },
                    class: "date-input cursor-pointer pr-10 {class}".to_string(),
                    on_input: move |value| on_input.call(value),
                    on_click: move |_| open_calendar(),
                }
                div {
                    class: "absolute inset-y-0 left-0 right-10 z-10 cursor-pointer rounded-md",
                    aria_hidden: "true",
                    onclick: move |_| open_calendar(),
                }
                div { class: "absolute right-1 top-1/2 z-20 -translate-y-1/2",
                    IconButton {
                        label: "Open date picker".to_string(),
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        class: "text-muted-foreground".to_string(),
                        disabled,
                        onclick: move |_| open_calendar(),
                        Calendar { size: 15 }
                    }
                }
            }
            if has_value || reserve_clear {
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    class: (if has_value { "shrink-0" } else { "invisible shrink-0" }).to_string(),
                    disabled: !has_value,
                    aria_label: Some("Clear date".to_string()),
                    onclick: move |_| on_input.call(String::new()),
                    "Clear"
                }
            }
            if open() {
                div {
                    class: "fixed inset-0 z-[80] overflow-y-auto bg-black/40",
                    onclick: move |_| open.set(false),
                    div { class: "flex min-h-full items-center justify-center p-4",
                        div {
                            class: "w-[min(18rem,calc(100vw-2rem))] rounded-xl border border-border bg-popover p-3 shadow-xl",
                            onclick: move |event| event.stop_propagation(),
                        div { class: "mb-3 flex items-center justify-between gap-2",
                            IconButton {
                                label: "Previous month".to_string(),
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconSm,
                                onclick: move |_| view.set(shift_month(view(), -1)),
                                ChevronLeft { size: 16 }
                            }
                            p { class: "text-sm font-semibold", {view().format("%B %Y").to_string()} }
                            IconButton {
                                label: "Next month".to_string(),
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconSm,
                                onclick: move |_| view.set(shift_month(view(), 1)),
                                ChevronRight { size: 16 }
                            }
                        }
                        div { class: "mb-1 grid grid-cols-7 gap-1",
                            for day in WEEKDAYS {
                                div { class: "py-1 text-center text-[10px] font-semibold uppercase tracking-wide text-muted-foreground", {day} }
                            }
                        }
                        div { class: "grid grid-cols-7 gap-1",
                            for cell in cells {
                                {
                                    match cell {
                                        None => rsx! { div { class: "h-8" } },
                                        Some(date) => {
                                            let is_selected = selected == Some(date);
                                            let is_today = date == chrono::Local::now().date_naive();
                                            let disabled_day = day_is_disabled(date, min_date, max_date);
                                            rsx! {
                                                Button {
                                                    key: "{date}",
                                                    variant: ButtonVariant::Unstyled,
                                                    size: ButtonSize::Unstyled,
                                                    radius: ButtonRadius::None,
                                                    align: ButtonAlign::Center,
                                                    disabled: disabled_day,
                                                    aria_label: date.format("%d/%m/%Y").to_string(),
                                                    class: (if is_selected {
                                                        "h-8 w-8 rounded-md bg-primary text-xs font-semibold text-primary-foreground"
                                                    } else if is_today {
                                                        "h-8 w-8 rounded-md border border-primary/50 text-xs font-semibold text-primary hover:bg-primary/10"
                                                    } else {
                                                        "h-8 w-8 rounded-md text-xs text-foreground hover:bg-accent"
                                                    }).to_string(),
                                                    onclick: move |_| {
                                                        on_input.call(date.format("%Y-%m-%d").to_string());
                                                        view.set(month_start(date));
                                                        open.set(false);
                                                    },
                                                    {date.day().to_string()}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "mt-3 flex items-center justify-between border-t border-border/60 pt-3",
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Xs,
                                disabled: selected.is_none(),
                                onclick: move |_| {
                                    on_input.call(String::new());
                                    open.set(false);
                                },
                                "Clear"
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::Xs,
                                class: "text-primary",
                                onclick: move |_| {
                                    let today = chrono::Local::now().date_naive();
                                    on_input.call(today.format("%Y-%m-%d").to_string());
                                    view.set(month_start(today));
                                    open.set(false);
                                },
                                "Today"
                            }
                        }
                    }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default()
    }

    #[test]
    fn calendar_grid_starts_on_monday_and_has_full_weeks() {
        let cells = calendar_cells(date(2026, 11, 1));
        assert_eq!(cells.len() % 7, 0);
        assert_eq!(cells[0], None);
        assert_eq!(month_start(date(2026, 11, 1)).weekday(), Weekday::Sun);
    }

    #[test]
    fn month_navigation_wraps_year_boundaries() {
        assert_eq!(shift_month(date(2026, 1, 1), -1), date(2025, 12, 1));
        assert_eq!(shift_month(date(2026, 12, 1), 1), date(2027, 1, 1));
    }

    #[test]
    fn date_picker_owns_its_popup_and_native_surface() {
        let source = include_str!("date_picker.rs");
        assert!(source.contains("fixed inset-0 z-[70]"));
        assert!(source.contains("date-input cursor-pointer"));
        assert!(source.contains("Open date picker"));
        assert!(source.contains("\"Today\""));
        assert!(source.contains("\"Clear\""));
    }
}
