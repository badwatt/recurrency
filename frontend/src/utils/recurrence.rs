//! Pure recurrence helpers shared by dashboard, calendar and local storage.

use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;

use crate::models::subscription::Recurrence;

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).expect("valid month"));
    first_next
        .pred_opt()
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, month, 1).expect("valid month"))
        .day()
}

fn date_for_day(year: i32, month: u32, day: u8) -> NaiveDate {
    let max = days_in_month(year, month);
    let day = if day == 0 {
        max
    } else {
        (day as u32).min(max).max(1)
    };
    NaiveDate::from_ymd_opt(year, month, day).expect("clamped date is valid")
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
        if date >= start
            && date >= from
            && date <= to
            && end_date.is_none_or(|end| date <= end)
        {
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
                let block_start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month");
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
                let block_start = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid year");
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

pub fn next_occurrence_on_or_after(
    start: NaiveDate,
    recurrence: &Recurrence,
    from: NaiveDate,
    end_date: Option<NaiveDate>,
) -> Option<NaiveDate> {
    let horizon = from + Duration::days(366 * 200);
    occurrences_between(start, recurrence, from, horizon, end_date, Decimal::ZERO)
        .into_iter()
        .next()
        .map(|(date, _)| date)
}

pub fn first_occurrence_on_or_after(start: NaiveDate, recurrence: &Recurrence) -> Option<NaiveDate> {
    next_occurrence_on_or_after(start, recurrence, start, None)
}


/// Progress of the next payment inside its calendar year. Only shown for
/// schedules with multiple payments per year, capped to avoid noisy daily rows.
pub fn payment_progress_for_year(
    start: NaiveDate,
    recurrence: &Recurrence,
    end_date: Option<NaiveDate>,
    next_payment: NaiveDate,
) -> Option<(usize, usize)> {
    const MAX_DISPLAY: usize = 24;
    let multiple_per_cycle = match recurrence.unit.as_str() {
        "month" => recurrence.days.len() > 1,
        "year" => recurrence.dates.len() > 1,
        _ => false,
    };
    if !multiple_per_cycle {
        return None;
    }
    if recurrence.unit == "year" {
        let mut dates = recurrence.dates.clone();
        dates.sort_by_key(|date| (date.month, date.day));
        let total = dates.len();
        if !(2..=MAX_DISPLAY).contains(&total) {
            return None;
        }
        let next_day = if next_payment.day() == days_in_month(next_payment.year(), next_payment.month()) {
            0
        } else {
            next_payment.day() as u8
        };
        let index = dates
            .iter()
            .position(|date| (date.month, date.day) >= (next_payment.month() as u8, next_day))
            .unwrap_or(0)
            + 1;
        return Some((index, total));
    }
    let year = next_payment.year();
    let year_start = NaiveDate::from_ymd_opt(year, 1, 1)?;
    let year_end = NaiveDate::from_ymd_opt(year, 12, 31)?;
    let occurrences = occurrences_between(
        start,
        recurrence,
        year_start,
        year_end,
        end_date,
        Decimal::ZERO,
    );
    let total = occurrences.len();
    if !(2..=MAX_DISPLAY).contains(&total) {
        return None;
    }
    let index = occurrences
        .iter()
        .position(|(date, _)| *date == next_payment)
        .or_else(|| occurrences.iter().position(|(date, _)| *date >= next_payment))?
        + 1;
    Some((index, total))
}

pub fn occurrences_in_month(
    start: NaiveDate,
    recurrence: &Recurrence,
    end_date: Option<NaiveDate>,
    year: i32,
    month: u32,
    base_amount: Decimal,
) -> Vec<(NaiveDate, Decimal)> {
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
    let month_end =
        NaiveDate::from_ymd_opt(year, month, days_in_month(year, month)).expect("valid month end");
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
    use crate::models::subscription::AnnualDate;
    use rust_decimal_macros::dec;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn monthly(days: Vec<u8>, interval: u16) -> Recurrence {
        Recurrence {
            interval,
            unit: "month".into(),
            days,
            ..Default::default()
        }
    }

    #[test]
    fn monthly_multiple_days_are_actual_charges() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 3),
            &monthly(vec![1, 15], 1),
            None,
            2026,
            2,
            dec!(10),
        );
        assert_eq!(
            occurrences,
            vec![(date(2026, 2, 1), dec!(10)), (date(2026, 2, 15), dec!(10))]
        );
    }

    #[test]
    fn every_two_months_skips_inactive_months() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![10], 2),
            None,
            2026,
            2,
            dec!(10),
        );
        assert!(occurrences.is_empty());
    }

    #[test]
    fn annual_amount_override() {
        let recurrence = Recurrence {
            interval: 1,
            unit: "year".into(),
            dates: vec![
                AnnualDate {
                    month: 5,
                    day: 27,
                    amount: None,
                },
                AnnualDate {
                    month: 11,
                    day: 11,
                    amount: Some(dec!(25)),
                },
            ],
            ..Default::default()
        };
        assert_eq!(
            occurrences_in_month(date(2026, 1, 1), &recurrence, None, 2026, 11, dec!(10)),
            vec![(date(2026, 11, 11), dec!(25))]
        );
    }

    #[test]
    fn end_date_inclusive_and_same_start_end() {
        let occurrences = occurrences_in_month(
            date(2026, 5, 10),
            &monthly(vec![10], 1),
            Some(date(2026, 5, 10)),
            2026,
            5,
            dec!(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 5, 10), dec!(10))]);
    }

    #[test]
    fn first_occurrence_respects_activation() {
        assert_eq!(
            first_occurrence_on_or_after(date(2026, 5, 3), &monthly(vec![1, 15], 1)),
            Some(date(2026, 5, 15))
        );
    }

    #[test]
    fn three_annual_dates_show_payment_progress() {
        let recurrence = Recurrence {
            interval: 1,
            unit: "year".into(),
            dates: vec![
                AnnualDate { month: 6, day: 24, amount: None },
                AnnualDate { month: 8, day: 8, amount: None },
                AnnualDate { month: 9, day: 22, amount: None },
            ],
            ..Default::default()
        };
        assert_eq!(
            payment_progress_for_year(
                date(2026, 6, 24),
                &recurrence,
                None,
                date(2027, 6, 24),
            ),
            Some((1, 3))
        );
    }

    #[test]
    fn annual_dates_show_payment_progress() {
        let recurrence = Recurrence {
            interval: 1,
            unit: "year".into(),
            dates: vec![
                AnnualDate { month: 5, day: 27, amount: None },
                AnnualDate { month: 11, day: 11, amount: None },
            ],
            ..Default::default()
        };
        assert_eq!(
            payment_progress_for_year(
                date(2026, 1, 1),
                &recurrence,
                None,
                date(2026, 11, 11),
            ),
            Some((2, 2))
        );
    }

    #[test]
    fn monthly_schedule_has_no_annual_progress_noise() {
        assert_eq!(
            payment_progress_for_year(
                date(2026, 1, 1),
                &monthly(vec![1], 1),
                None,
                date(2026, 11, 1),
            ),
            None
        );
    }

    #[test]
    fn last_day_clamps_to_short_month() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![0], 1),
            None,
            2026,
            2,
            dec!(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 2, 28), dec!(10))]);
    }
}
