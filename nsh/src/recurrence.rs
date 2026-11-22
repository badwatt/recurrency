//! Occurrence calculation for the dashboard and, later, the calendar.

use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;

use crate::models::Recurrence;

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default());
    first_next
        .pred_opt()
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default())
        .day()
}

fn date_for_day(year: i32, month: u32, day: u8) -> NaiveDate {
    let max = days_in_month(year, month);
    let day = if day == 0 {
        max
    } else {
        (day as u32).min(max).max(1)
    };
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default()
}

fn month_index(year: i32, month: u32) -> i32 {
    year * 12 + month as i32 - 1
}

fn year_month(index: i32) -> (i32, u32) {
    (index.div_euclid(12), (index.rem_euclid(12) + 1) as u32)
}

fn sorted_unique(values: &mut Vec<u8>) {
    values.sort_unstable();
    values.dedup();
}

fn amount_for(date: NaiveDate, recurrence: &Recurrence, base: Decimal) -> Decimal {
    if recurrence.unit == "year" {
        let last = date.day() == days_in_month(date.year(), date.month());
        if let Some(special) = recurrence.dates.iter().find(|entry| {
            entry.month as u32 == date.month()
                && (entry.day as u32 == date.day() || (entry.day == 0 && last))
        }) {
            return special.amount.unwrap_or(base);
        }
    }
    base
}

pub fn occurrences_between(
    start: NaiveDate,
    recurrence: &Recurrence,
    from: NaiveDate,
    to: NaiveDate,
    end_date: Option<NaiveDate>,
    base_amount: Decimal,
) -> Vec<(NaiveDate, Decimal)> {
    let mut result = Vec::new();
    if to < start {
        return result;
    }
    let mut guard = 0usize;
    let mut push = |date: NaiveDate| {
        if date >= start && date >= from && date <= to && end_date.is_none_or(|end| date <= end) {
            result.push((date, amount_for(date, recurrence, base_amount)));
        }
    };

    match recurrence.unit.as_str() {
        "day" => {
            let step = i64::from(recurrence.interval.max(1));
            let mut date = start;
            if date < from {
                let days = (from - date).num_days();
                let jumps = (days + step - 1) / step;
                date += Duration::days(jumps * step);
            }
            while date <= to && guard < 100_000 {
                push(date);
                date += Duration::days(step);
                guard += 1;
            }
        }
        "week" => {
            let mut weekdays = recurrence.weekdays.clone();
            sorted_unique(&mut weekdays);
            if weekdays.is_empty() {
                weekdays.push(start.weekday().number_from_monday() as u8);
            }
            let weekday = start.weekday().number_from_monday() as i64;
            let mut week = start - Duration::days(weekday - 1);
            while week <= to && guard < 100_000 {
                for wd in &weekdays {
                    push(week + Duration::days(i64::from(*wd) - 1));
                }
                week += Duration::weeks(i64::from(recurrence.interval.max(1)));
                guard += 1;
            }
        }
        "month" => {
            let mut days = recurrence.days.clone();
            sorted_unique(&mut days);
            if days.is_empty() {
                days.push(start.day() as u8);
            }
            let mut index = month_index(start.year(), start.month());
            while guard < 100_000 {
                let (year, month) = year_month(index);
                let block_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
                if block_start > to {
                    break;
                }
                for day in &days {
                    push(date_for_day(year, month, *day));
                }
                index += i32::from(recurrence.interval.max(1));
                guard += 1;
            }
        }
        "year" => {
            let mut dates = recurrence.dates.clone();
            dates.sort_by_key(|date| (date.month, date.day));
            let mut year = start.year();
            while guard < 10_000 {
                let block_start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or_default();
                if block_start > to {
                    break;
                }
                for entry in &dates {
                    push(date_for_day(year, u32::from(entry.month), entry.day));
                }
                year += i32::from(recurrence.interval.max(1));
                guard += 1;
            }
        }
        _ => {}
    }

    result.sort_by_key(|(date, _)| *date);
    result
}

pub fn occurrences_in_month(
    start: NaiveDate,
    recurrence: &Recurrence,
    end_date: Option<NaiveDate>,
    year: i32,
    month: u32,
    base_amount: Decimal,
) -> Vec<(NaiveDate, Decimal)> {
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
    let month_end =
        NaiveDate::from_ymd_opt(year, month, days_in_month(year, month)).unwrap_or_default();
    occurrences_between(
        start,
        recurrence,
        month_start,
        month_end,
        end_date,
        base_amount,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid test date")
    }

    fn monthly(days: Vec<u8>) -> Recurrence {
        Recurrence {
            interval: 1,
            unit: "month".to_string(),
            days,
            ..Recurrence::default()
        }
    }

    #[test]
    fn monthly_multiple_days_are_actual_charges() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 3),
            &monthly(vec![1, 15]),
            None,
            2026,
            2,
            Decimal::from(10),
        );
        assert_eq!(
            occurrences,
            vec![
                (date(2026, 2, 1), Decimal::from(10)),
                (date(2026, 2, 15), Decimal::from(10))
            ]
        );
    }

    #[test]
    fn every_two_months_skips_inactive_months() {
        let recurrence = Recurrence {
            interval: 2,
            unit: "month".to_string(),
            days: vec![10],
            ..Recurrence::default()
        };
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &recurrence,
            None,
            2026,
            2,
            Decimal::from(10),
        );
        assert!(occurrences.is_empty());
    }

    #[test]
    fn end_date_excludes_later_occurrences() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![1, 15]),
            Some(date(2026, 2, 10)),
            2026,
            2,
            Decimal::from(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 2, 1), Decimal::from(10))]);
    }

    #[test]
    fn last_day_zero_uses_month_end() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![0]),
            None,
            2026,
            2,
            Decimal::from(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 2, 28), Decimal::from(10))]);
    }
}
