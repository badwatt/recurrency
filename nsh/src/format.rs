//! Currency and date strings used by the native dashboard.

use chrono::{Datelike, NaiveDate};
use nsh_core::prelude::{Signal, signal};
use rust_decimal::Decimal;
use std::cell::RefCell;

thread_local! {
    static DATE_FORMAT: RefCell<Option<Signal<String>>> = const { RefCell::new(None) };
    static FIRST_DAY: RefCell<Option<Signal<String>>> = const { RefCell::new(None) };
}

pub fn init_date_format() {
    DATE_FORMAT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(signal("DD/MM/YYYY".to_string()));
        }
    });
    FIRST_DAY.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(signal("monday".to_string()));
        }
    });
}

pub fn first_day() -> String {
    FIRST_DAY.with(|slot| {
        slot.borrow()
            .map_or_else(|| "monday".to_string(), |day| day.get())
    })
}

pub fn set_first_day(value: &str) {
    FIRST_DAY.with(|slot| {
        if let Some(day) = *slot.borrow() {
            day.set(value.to_string());
        }
    });
}

pub fn week_starts_sunday() -> bool {
    first_day() == "sunday"
}

pub fn date_format() -> String {
    DATE_FORMAT.with(|slot| {
        slot.borrow()
            .map_or_else(|| "DD/MM/YYYY".to_string(), |format| format.get())
    })
}

pub fn set_date_format(value: &str) {
    DATE_FORMAT.with(|slot| {
        if let Some(format) = *slot.borrow() {
            format.set(value.to_string());
        }
    });
}

/// A calendar date in the user's chosen pattern.
pub fn format_date(date: NaiveDate) -> String {
    match date_format().as_str() {
        "MM/DD/YYYY" => format!("{:02}/{:02}/{:04}", date.month(), date.day(), date.year()),
        "YYYY/MM/DD" => format!("{:04}/{:02}/{:02}", date.year(), date.month(), date.day()),
        _ => format!("{:02}/{:02}/{:04}", date.day(), date.month(), date.year()),
    }
}

pub fn format_currency(amount: Decimal, currency: &str) -> String {
    if crate::ui::privacy_active() {
        return "••••".to_string();
    }
    let symbol = match currency {
        "EUR" => "€",
        "USD" | "CAD" | "AUD" => "$",
        "GBP" => "£",
        "JPY" => "¥",
        _ => currency,
    };
    let no_decimals = currency == "JPY";
    let formatted = format_decimal(amount.abs(), no_decimals);
    if amount.is_sign_negative() {
        format!("-{symbol}{formatted}")
    } else {
        format!("{symbol}{formatted}")
    }
}

fn format_decimal(decimal: Decimal, no_decimals: bool) -> String {
    let text = format!("{decimal}");
    let (integer, fraction) = text
        .split_once('.')
        .map_or((text.as_str(), None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    let integer = add_thousands_separators(integer);
    if no_decimals {
        integer
    } else {
        let fraction = fraction.unwrap_or("00");
        let padded = format!("{fraction:0<2}");
        format!("{integer}.{}", &padded[..2])
    }
}

fn add_thousands_separators(integer: &str) -> String {
    let mut result = String::new();
    let digits: Vec<char> = integer.chars().collect();
    for (index, digit) in digits.iter().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*digit);
    }
    result
}

pub fn short_date(date: NaiveDate) -> String {
    date.format("%a, %b %-d").to_string()
}

pub fn month_label(date: NaiveDate) -> String {
    date.format("%B %Y").to_string().to_uppercase()
}

pub fn days_text(date: NaiveDate, today: NaiveDate) -> String {
    let diff = (date - today).num_days();
    match diff {
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        d if d < 0 => "Due".to_string(),
        _ => format!("{diff} days"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_euros_with_thousands_and_cents() {
        assert_eq!(format_currency(Decimal::new(150099, 2), "EUR"), "€1,500.99");
    }

    #[test]
    fn formats_unknown_currency_as_code() {
        assert_eq!(format_currency(Decimal::new(1000, 2), "BRL"), "BRL10.00");
    }

    #[test]
    fn formats_yen_without_decimals() {
        assert_eq!(format_currency(Decimal::new(12345, 0), "JPY"), "¥12,345");
    }

    #[test]
    fn formats_negative_amounts() {
        assert_eq!(format_currency(Decimal::new(-1599, 2), "EUR"), "-€15.99");
    }

    #[test]
    fn formats_relative_days() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 22).expect("valid date");
        assert_eq!(days_text(today, today), "Today");
        assert_eq!(
            days_text(
                NaiveDate::from_ymd_opt(2026, 11, 25).expect("valid date"),
                today
            ),
            "3 days"
        );
    }
}
