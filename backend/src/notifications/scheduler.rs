use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration as StdDuration;

use crate::notifications::channels::{NotificationProvider, SummaryNotificationSettings};
use crate::notifications::crypto::decrypt_with_secret;
use crate::notifications::providers::{NotificationMessage, send_message};
use crate::notifications::service::notification_secret;
use crate::shared::date_utils::days_in_month;
use crate::notifications::summary::{build_message, SummaryAccount, SummarySubscription, SummaryTransaction};

#[derive(Debug, Clone)]
pub struct ScheduleSpec {
    pub frequency: String,
    pub weekday: Option<i16>,
    pub day_of_month: Option<i16>,
    pub hour: i16,
    pub minute: i16,
    pub timezone: String,
}

impl From<&SummaryNotificationSettings> for ScheduleSpec {
    fn from(value: &SummaryNotificationSettings) -> Self {
        Self {
            frequency: value.frequency.clone(),
            weekday: value.weekday,
            day_of_month: value.day_of_month,
            hour: value.hour,
            minute: value.minute,
            timezone: value.timezone.clone(),
        }
    }
}

pub fn validate_schedule(spec: &ScheduleSpec) -> Result<Tz, String> {
    if !(0..=23).contains(&spec.hour) || !(0..=59).contains(&spec.minute) {
        return Err("Notification time is invalid".to_string());
    }
    let timezone = Tz::from_str(&spec.timezone)
        .map_err(|_| "Notification timezone is invalid".to_string())?;
    match spec.frequency.as_str() {
        "daily" => {}
        "weekly" => {
            if !matches!(spec.weekday, Some(1..=7)) {
                return Err("Weekly notifications need a weekday".to_string());
            }
        }
        "monthly" => {
            if !matches!(spec.day_of_month, Some(1..=31)) {
                return Err("Monthly notifications need a day of month".to_string());
            }
        }
        _ => return Err("Notification frequency is invalid".to_string()),
    }
    Ok(timezone)
}

pub fn next_run_at(now: DateTime<Utc>, spec: &ScheduleSpec) -> Result<DateTime<Utc>, String> {
    let timezone = validate_schedule(spec)?;
    let local_now = now.with_timezone(&timezone);
    let candidate_date = match spec.frequency.as_str() {
        "daily" => {
            let today = local_now.date_naive();
            let candidate = local_datetime(&timezone, today, spec)?;
            if candidate <= local_now {
                local_datetime(&timezone, today + Duration::days(1), spec)?
            } else {
                candidate
            }
        }
        "weekly" => {
            let target = spec.weekday.unwrap_or(1) as u32;
            let current = local_now.weekday().num_days_from_monday() + 1;
            let days_ahead = (target + 7 - current) % 7;
            let date = local_now.date_naive() + Duration::days(i64::from(days_ahead));
            let candidate = local_datetime(&timezone, date, spec)?;
            if candidate <= local_now {
                local_datetime(&timezone, date + Duration::days(7), spec)?
            } else {
                candidate
            }
        }
        "monthly" => {
            let day = spec.day_of_month.unwrap_or(1) as u32;
            let date = local_now.date_naive();
            let candidate = local_datetime(
                &timezone,
                date.with_day(day.min(days_in_month(date.year(), date.month())))
                    .ok_or_else(|| "Could not compute next notification date".to_string())?,
                spec,
            )?;
            if candidate <= local_now {
                let (year, month) = if date.month() == 12 {
                    (date.year() + 1, 1)
                } else {
                    (date.year(), date.month() + 1)
                };
                let next = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| "Could not compute next notification date".to_string())?;
                local_datetime(
                    &timezone,
                    next.with_day(day.min(days_in_month(year, month)))
                        .ok_or_else(|| "Could not compute next notification date".to_string())?,
                    spec,
                )?
            } else {
                candidate
            }
        }
        _ => return Err("Notification frequency is invalid".to_string()),
    };
    Ok(candidate_date.with_timezone(&Utc))
}

fn local_datetime(
    timezone: &Tz,
    date: chrono::NaiveDate,
    spec: &ScheduleSpec,
) -> Result<DateTime<Tz>, String> {
    timezone
        .with_ymd_and_hms(
            date.year(),
            date.month(),
            date.day(),
            u32::try_from(spec.hour).unwrap_or(0),
            u32::try_from(spec.minute).unwrap_or(0),
            0,
        )
        .single()
        .ok_or_else(|| "Notification time does not exist in that timezone".to_string())
}



pub fn spawn(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(StdDuration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(error) = tick(&pool).await {
                tracing::warn!("Notification scheduler tick failed: {error}");
            }
        }
    });
}

async fn tick(pool: &PgPool) -> Result<(), String> {
    let due = sqlx::query_as::<_, SummaryNotificationSettings>(
        r#"
        SELECT user_id, enabled, frequency, weekday, day_of_month, hour, minute,
               timezone, next_run_at, last_sent_at, last_error, created_at, updated_at
        FROM summary_notification_settings
        WHERE enabled = true AND (next_run_at IS NULL OR next_run_at <= NOW())
        ORDER BY next_run_at NULLS FIRST
        LIMIT 20
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?;

    for settings in due {
        let user = sqlx::query_as::<_, (String, String)>(
            "SELECT email, currency FROM users WHERE id = $1",
        )
        .bind(settings.user_id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?;
        let Some((email, currency)) = user else {
            continue;
        };

        let timezone = Tz::from_str(&settings.timezone).unwrap_or(chrono_tz::UTC);
        let today = Utc::now().with_timezone(&timezone).date_naive();
        let message = build_summary(pool, settings.user_id, &currency, today).await?;
        let channels = sqlx::query_as::<_, (String, Vec<u8>)>(
            "SELECT provider, config FROM notification_channels WHERE user_id = $1 AND enabled = true",
        )
        .bind(settings.user_id)
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?;

        let mut errors = Vec::new();
        let mut sent = 0usize;
        let secret = notification_secret().map_err(|error| error.to_string())?;
        for (provider, config) in channels {
            let Some(provider) = NotificationProvider::parse(&provider) else {
                errors.push("unknown provider".to_string());
                continue;
            };
            let config = match decrypt_with_secret(&secret, &config) {
                Ok(config) => config,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
            match send_message(provider, &config, &email, &message).await {
                Ok(()) => sent += 1,
                Err(error) => errors.push(error.to_string()),
            }
        }

        let spec = ScheduleSpec::from(&settings);
        let next = next_run_at(Utc::now(), &spec).ok();
        let error = if sent == 0 && errors.is_empty() {
            Some("No enabled notification channels".to_string())
        } else if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };
        sqlx::query(
            "UPDATE summary_notification_settings SET last_sent_at = NOW(), next_run_at = $2, last_error = $3, updated_at = NOW() WHERE user_id = $1",
        )
        .bind(settings.user_id)
        .bind(next)
        .bind(error)
        .execute(pool)
        .await
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) async fn build_summary(
    pool: &PgPool,
    user_id: uuid::Uuid,
    currency: &str,
    today: chrono::NaiveDate,
) -> Result<NotificationMessage, String> {
    let accounts = sqlx::query_as::<_, (uuid::Uuid, String)>(
        "SELECT id, name FROM accounts WHERE user_id = $1 ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?
    .into_iter()
    .map(|(id, name)| SummaryAccount { id, name })
    .collect::<Vec<_>>();

    let subscriptions = sqlx::query_as::<
        _,
        (uuid::Uuid, rust_decimal::Decimal, String, chrono::NaiveDate, Option<chrono::NaiveDate>, bool),
    >(
        "SELECT account_id, amount, frequency, start_date, next_billing_date, is_active FROM subscriptions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?
    .into_iter()
    .map(|(account_id, amount, frequency, start_date, next_billing_date, is_active)| {
        SummarySubscription {
            account_id,
            amount,
            frequency,
            start_date,
            next_billing_date,
            is_active,
        }
    })
    .collect::<Vec<_>>();

    let transactions = sqlx::query_as::<_, (uuid::Uuid, rust_decimal::Decimal, chrono::NaiveDate)>(
        "SELECT account_id, amount, date FROM transactions WHERE user_id = $1 AND date >= $2 AND date <= $3",
    )
    .bind(user_id)
    .bind(today.with_day(1).ok_or_else(|| "Invalid summary date".to_string())?)
    .bind(
        today
            .with_day(days_in_month(today.year(), today.month()))
            .ok_or_else(|| "Invalid summary date".to_string())?,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?
    .into_iter()
    .map(|(account_id, amount, date)| SummaryTransaction {
        account_id,
        amount,
        date,
    })
    .collect::<Vec<_>>();

    let body = build_message(&accounts, &subscriptions, &transactions, today, currency);
    Ok(NotificationMessage {
        title: format!("Recurrency summary {}/{:02}", today.year(), today.month()),
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn spec(frequency: &str, weekday: Option<i16>, day: Option<i16>) -> ScheduleSpec {
        ScheduleSpec {
            frequency: frequency.to_string(),
            weekday,
            day_of_month: day,
            hour: 9,
            minute: 0,
            timezone: "UTC".to_string(),
        }
    }

    #[test]
    fn daily_before_time_is_today() {
        let now = Utc.with_ymd_and_hms(2026, 11, 10, 8, 0, 0).unwrap();
        let next = next_run_at(now, &spec("daily", None, None)).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 11, 10, 9, 0, 0).unwrap());
    }

    #[test]
    fn daily_after_time_is_tomorrow() {
        let now = Utc.with_ymd_and_hms(2026, 11, 10, 10, 0, 0).unwrap();
        let next = next_run_at(now, &spec("daily", None, None)).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap());
    }

    #[test]
    fn weekly_uses_next_monday() {
        let now = Utc.with_ymd_and_hms(2026, 11, 10, 10, 0, 0).unwrap();
        let next = next_run_at(now, &spec("weekly", Some(1), None)).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 11, 16, 9, 0, 0).unwrap());
    }

    #[test]
    fn december_has_31_days() {
        let now = Utc.with_ymd_and_hms(2026, 11, 30, 10, 0, 0).unwrap();
        let next = next_run_at(now, &spec("monthly", None, Some(31))).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 12, 31, 9, 0, 0).unwrap());
    }

    #[test]
    fn monthly_clamps_to_short_month() {
        let now = Utc.with_ymd_and_hms(2027, 1, 31, 10, 0, 0).unwrap();
        let next = next_run_at(now, &spec("monthly", None, Some(31))).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2027, 2, 28, 9, 0, 0).unwrap());
    }

    #[test]
    fn invalid_schedule_is_rejected() {
        let mut invalid = spec("monthly", None, Some(0));
        assert!(validate_schedule(&invalid).is_err());
        invalid = spec("weekly", Some(0), None);
        assert!(validate_schedule(&invalid).is_err());
        invalid = spec("daily", None, None);
        invalid.timezone = "Not/AZone".to_string();
        assert!(validate_schedule(&invalid).is_err());
    }
}
