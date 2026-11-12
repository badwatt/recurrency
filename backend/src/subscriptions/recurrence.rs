use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;

use crate::shared::date_utils::days_in_month;
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{AnnualDate, Recurrence};

pub const MAX_INTERVAL: u16 = 100;

pub fn validate_recurrence(recurrence: &Recurrence) -> AppResult<()> {
    if !(1..=MAX_INTERVAL).contains(&recurrence.interval) {
        return Err(AppError::BadRequest(format!(
            "Recurrence interval must be 1-{MAX_INTERVAL}"
        )));
    }
    match recurrence.unit.as_str() {
        "day" => Ok(()),
        "week" => {
            if recurrence.weekdays.is_empty()
                || recurrence.weekdays.iter().any(|day| !(1..=7).contains(day))
            {
                return Err(AppError::BadRequest(
                    "Choose at least one valid weekday".to_string(),
                ));
            }
            Ok(())
        }
        "month" => {
            if recurrence.days.is_empty() || recurrence.days.iter().any(|day| *day > 31) {
                return Err(AppError::BadRequest(
                    "Choose at least one valid day of the month".to_string(),
                ));
            }
            Ok(())
        }
        "year" => {
            if recurrence.dates.is_empty()
                || recurrence.dates.iter().any(|date| {
                    !(1..=12).contains(&date.month) || date.day > 31
                })
            {
                return Err(AppError::BadRequest(
                    "Choose at least one valid annual date".to_string(),
                ));
            }
            Ok(())
        }
        _ => Err(AppError::BadRequest("Invalid recurrence unit".to_string())),
    }
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

fn sorted_unique<T: Ord>(values: &mut Vec<T>) {
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

/// All occurrences in `[from, to]`, bounded by the inclusive `end_date`.
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
            let step = i64::from(recurrence.interval);
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
                week += Duration::weeks(i64::from(recurrence.interval));
                guard += 1;
            }
        }
        "month" => {
            let mut days = recurrence.days.clone();
            sorted_unique(&mut days);
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
                index += i32::from(recurrence.interval);
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
                year += i32::from(recurrence.interval);
                guard += 1;
            }
        }
        _ => {}
    }

    result.sort_by_key(|(date, _)| *date);
    result
}

/// First occurrence on or after `from`, respecting the inclusive end date.
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

/// First occurrence on or after the activation date.
pub fn first_occurrence_on_or_after(
    start: NaiveDate,
    recurrence: &Recurrence,
) -> Option<NaiveDate> {
    next_occurrence_on_or_after(start, recurrence, start, None)
}

/// Occurrences in a calendar month, including completed charges.
pub fn occurrences_in_month(
    start: NaiveDate,
    recurrence: &Recurrence,
    end_date: Option<NaiveDate>,
    year: i32,
    month: u32,
    base_amount: Decimal,
) -> Vec<(NaiveDate, Decimal)> {
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month start");
    let month_end = NaiveDate::from_ymd_opt(year, month, days_in_month(year, month))
        .expect("valid month end");
    occurrences_between(
        start,
        recurrence,
        month_start,
        month_end,
        end_date,
        base_amount,
    )
}

pub fn recurrence_from_legacy(
    frequency: &str,
    start: NaiveDate,
    day: Option<i16>,
    month: Option<i16>,
) -> Recurrence {
    let fallback_day = start.day().min(31) as u8;
    let day = day.map(|value| value.clamp(1, 31) as u8).unwrap_or(fallback_day);
    match frequency {
        "daily" => Recurrence {
            interval: 1,
            unit: "day".into(),
            ..Default::default()
        },
        "weekly" => Recurrence {
            interval: 1,
            unit: "week".into(),
            weekdays: vec![day.clamp(1, 7)],
            ..Default::default()
        },
        "quarterly" => Recurrence {
            interval: 3,
            unit: "month".into(),
            days: vec![day],
            ..Default::default()
        },
        "yearly" => Recurrence {
            interval: 1,
            unit: "year".into(),
            dates: vec![AnnualDate {
                month: month
                    .map(|value| value.clamp(1, 12) as u8)
                    .unwrap_or(start.month() as u8),
                day,
                amount: None,
            }],
            ..Default::default()
        },
        _ => Recurrence {
            interval: 1,
            unit: "month".into(),
            days: vec![day],
            ..Default::default()
        },
    }
}

/// Keep the legacy columns populated for older clients and database readers.
pub fn legacy_from_recurrence(recurrence: &Recurrence) -> (String, Option<i16>, Option<i16>) {
    match recurrence.unit.as_str() {
        "day" => ("daily".into(), None, None),
        "week" => (
            "weekly".into(),
            recurrence.weekdays.first().map(|day| *day as i16),
            None,
        ),
        "year" => {
            let first = recurrence.dates.iter().min_by_key(|date| (date.month, date.day));
            (
                "yearly".into(),
                first.map(|date| date.day as i16),
                first.map(|date| date.month as i16),
            )
        }
        _ => {
            let frequency = if recurrence.interval == 3 {
                "quarterly"
            } else {
                "monthly"
            };
            (
                frequency.into(),
                recurrence.days.first().map(|day| {
                    if *day == 0 {
                        31
                    } else {
                        i16::from(*day)
                    }
                }),
                None,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
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
    fn monthly_multiple_days_generate_multiple_charges() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 3),
            &monthly(vec![1, 15], 1),
            None,
            2026,
            1,
            Decimal::from(10),
        );
        assert_eq!(
            occurrences,
            vec![(date(2026, 1, 15), Decimal::from(10))]
        );
        let occurrences = occurrences_in_month(
            date(2026, 1, 3),
            &monthly(vec![1, 15], 1),
            None,
            2026,
            2,
            Decimal::from(10),
        );
        assert_eq!(
            occurrences,
            vec![(date(2026, 2, 1), Decimal::from(10)), (date(2026, 2, 15), Decimal::from(10))]
        );
    }

    #[test]
    fn every_two_months_skips_odd_months() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![10], 2),
            None,
            2026,
            2,
            Decimal::from(10),
        );
        assert!(occurrences.is_empty());
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![10], 2),
            None,
            2026,
            3,
            Decimal::from(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 3, 10), Decimal::from(10))]);
    }

    #[test]
    fn last_day_uses_short_month_end() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![0], 1),
            None,
            2026,
            2,
            Decimal::from(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 2, 28), Decimal::from(10))]);
    }

    #[test]
    fn annual_dates_and_amount_override() {
        let recurrence = Recurrence {
            interval: 1,
            unit: "year".into(),
            dates: vec![
                AnnualDate { month: 5, day: 27, amount: None },
                AnnualDate { month: 11, day: 11, amount: Some(Decimal::from(25)) },
            ],
            ..Default::default()
        };
        assert_eq!(
            occurrences_in_month(date(2026, 1, 1), &recurrence, None, 2026, 5, Decimal::from(10)),
            vec![(date(2026, 5, 27), Decimal::from(10))]
        );
        assert_eq!(
            occurrences_in_month(date(2026, 1, 1), &recurrence, None, 2026, 11, Decimal::from(10)),
            vec![(date(2026, 11, 11), Decimal::from(25))]
        );
    }

    #[test]
    fn end_date_is_inclusive() {
        let occurrences = occurrences_in_month(
            date(2026, 1, 1),
            &monthly(vec![10], 1),
            Some(date(2026, 3, 10)),
            2026,
            3,
            Decimal::from(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 3, 10), Decimal::from(10))]);
    }

    #[test]
    fn start_and_end_same_day_produces_one_charge() {
        let occurrences = occurrences_in_month(
            date(2026, 5, 10),
            &monthly(vec![10], 1),
            Some(date(2026, 5, 10)),
            2026,
            5,
            Decimal::from(10),
        );
        assert_eq!(occurrences, vec![(date(2026, 5, 10), Decimal::from(10))]);
    }

    #[test]
    fn weekly_multiple_weekdays_start_on_activation() {
        let recurrence = Recurrence {
            interval: 1,
            unit: "week".into(),
            weekdays: vec![1, 4],
            ..Default::default()
        };
        let occurrences = occurrences_in_month(
            date(2026, 1, 6),
            &recurrence,
            None,
            2026,
            1,
            Decimal::from(5),
        );
        let expected = [8, 12, 15, 19, 22, 26, 29]
            .into_iter()
            .map(|day| (date(2026, 1, day), Decimal::from(5)))
            .collect::<Vec<_>>();
        assert_eq!(occurrences, expected);
    }

    #[test]
    fn first_occurrence_respects_activation_date() {
        let recurrence = monthly(vec![1, 15], 1);
        assert_eq!(
            first_occurrence_on_or_after(date(2026, 5, 3), &recurrence),
            Some(date(2026, 5, 15))
        );
    }

    #[test]
    fn validate_recurrence_rejects_bad_units_and_values() {
        assert!(validate_recurrence(&monthly(vec![1], 0)).is_err());
        assert!(validate_recurrence(&monthly(vec![], 1)).is_err());
        assert!(validate_recurrence(&monthly(vec![32], 1)).is_err());
        assert!(validate_recurrence(&Recurrence { unit: "year".into(), ..Default::default() }).is_err());
    }

    #[test]
    fn legacy_round_trip() {
        let recurrence = recurrence_from_legacy("quarterly", date(2026, 5, 27), Some(27), None);
        assert_eq!(recurrence.interval, 3);
        assert_eq!(recurrence.days, vec![27]);
        let (frequency, day, month) = legacy_from_recurrence(&recurrence);
        assert_eq!(frequency, "quarterly");
        assert_eq!(day, Some(27));
        assert_eq!(month, None);
    }
}
