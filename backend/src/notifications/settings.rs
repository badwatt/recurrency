use chrono::Utc;
use std::str::FromStr;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::notifications::channels::{
    DEFAULT_TIMEZONE, NotificationChannelRecord, NotificationChannelResponse, NotificationProvider,
    SummaryNotificationSettings, SummaryNotificationSettingsResponse, UpdateNotificationChannelRequest,
    UpdateSummaryNotificationSettingsRequest, default_channel,
};
use crate::notifications::crypto::{decrypt_with_secret, encrypt_with_secret};
use crate::notifications::providers::{
    NotificationMessage, mask_config, merge_config, send_message, validate_config,
};
use crate::notifications::scheduler::{ScheduleSpec, next_run_at, validate_schedule};
use crate::notifications::service::notification_secret;
use crate::shared::error::{AppError, AppResult};

#[async_trait::async_trait]
pub trait NotificationSettingsServiceTrait: Send + Sync {
    async fn list_channels(&self, user_id: Uuid) -> AppResult<Vec<NotificationChannelResponse>>;
    async fn update_channel(
        &self,
        user_id: Uuid,
        provider: NotificationProvider,
        req: UpdateNotificationChannelRequest,
    ) -> AppResult<NotificationChannelResponse>;
    async fn test_channel(
        &self,
        user_id: Uuid,
        provider: NotificationProvider,
        config: Option<Value>,
    ) -> AppResult<()>;
    async fn get_summary(&self, user_id: Uuid)
    -> AppResult<SummaryNotificationSettingsResponse>;
    async fn update_summary(
        &self,
        user_id: Uuid,
        req: UpdateSummaryNotificationSettingsRequest,
    ) -> AppResult<SummaryNotificationSettingsResponse>;
    async fn send_summary_now(&self, user_id: Uuid) -> AppResult<()>;
}

pub struct SqlxNotificationSettingsService {
    pool: PgPool,
}

impl SqlxNotificationSettingsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl NotificationSettingsServiceTrait for SqlxNotificationSettingsService {
    async fn list_channels(&self, user_id: Uuid) -> AppResult<Vec<NotificationChannelResponse>> {
        let rows = sqlx::query_as::<_, NotificationChannelRecord>(
            "SELECT id, user_id, provider, enabled, config, created_at, updated_at FROM notification_channels WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        let secret = notification_secret()?;
        let mut configured = Vec::new();
        for row in rows {
            let Some(provider) = NotificationProvider::parse(&row.provider) else {
                continue;
            };
            let config = if row.config.is_empty() {
                json!({})
            } else {
                decrypt_with_secret(&secret, &row.config).map_err(AppError::Internal)?
            };
            configured.push(NotificationChannelResponse {
                provider: provider.as_str().to_string(),
                enabled: row.enabled,
                configured: true,
                config: mask_config(provider, &config),
            });
        }
        Ok(NotificationProvider::ALL
            .into_iter()
            .map(|provider| {
                configured
                    .iter()
                    .find(|channel| channel.provider == provider.as_str())
                    .cloned()
                    .unwrap_or_else(|| default_channel(provider))
            })
            .collect())
    }

    async fn update_channel(
        &self,
        user_id: Uuid,
        provider: NotificationProvider,
        req: UpdateNotificationChannelRequest,
    ) -> AppResult<NotificationChannelResponse> {
        let secret = notification_secret()?;
        let existing = self.load_config(user_id, provider, &secret).await?;
        let merged = merge_config(provider, &existing, &req.config);
        let config = if req.enabled || !merged.as_object().is_none_or(|object| object.is_empty()) {
            validate_config(provider, &merged)
                .map_err(|error| AppError::BadRequest(error.to_string()))?
        } else {
            json!({})
        };
        let encrypted = encrypt_with_secret(&secret, &config).map_err(AppError::Internal)?;

        sqlx::query(
            r#"
            INSERT INTO notification_channels (id, user_id, provider, enabled, config)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, provider) DO UPDATE SET
                enabled = EXCLUDED.enabled,
                config = EXCLUDED.config,
                updated_at = NOW()
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(provider.as_str())
        .bind(req.enabled)
        .bind(encrypted)
        .execute(&self.pool)
        .await?;

        Ok(NotificationChannelResponse {
            provider: provider.as_str().to_string(),
            enabled: req.enabled,
            configured: true,
            config: mask_config(provider, &config),
        })
    }

    async fn test_channel(
        &self,
        user_id: Uuid,
        provider: NotificationProvider,
        config: Option<Value>,
    ) -> AppResult<()> {
        let secret = notification_secret()?;
        let existing = self.load_config(user_id, provider, &secret).await?;
        let incoming = config.unwrap_or(json!({}));
        let merged = merge_config(provider, &existing, &incoming);
        let config = validate_config(provider, &merged)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;
        let email: (String,) = sqlx::query_as("SELECT email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        send_message(
            provider,
            &config,
            &email.0,
            &NotificationMessage {
                title: "Recurrency test notification".to_string(),
                body: "Your notification channel is configured correctly.".to_string(),
            },
        )
        .await
    }

    async fn get_summary(
        &self,
        user_id: Uuid,
    ) -> AppResult<SummaryNotificationSettingsResponse> {
        let row = sqlx::query_as::<_, SummaryNotificationSettings>(
            r#"
            SELECT user_id, enabled, frequency, weekday, day_of_month, hour, minute,
                   timezone, next_run_at, last_sent_at, last_error, created_at, updated_at
            FROM summary_notification_settings WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .map(Into::into)
            .unwrap_or_else(default_summary))
    }

    async fn update_summary(
        &self,
        user_id: Uuid,
        req: UpdateSummaryNotificationSettingsRequest,
    ) -> AppResult<SummaryNotificationSettingsResponse> {
        let spec = ScheduleSpec {
            frequency: req.frequency.clone(),
            weekday: req.weekday,
            day_of_month: req.day_of_month,
            hour: req.hour,
            minute: req.minute,
            timezone: req.timezone.clone(),
        };
        validate_schedule(&spec)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;
        let next = if req.enabled {
            Some(
                next_run_at(Utc::now(), &spec)
                    .map_err(|error| AppError::BadRequest(error.to_string()))?,
            )
        } else {
            None
        };
        let row = sqlx::query_as::<_, SummaryNotificationSettings>(
            r#"
            INSERT INTO summary_notification_settings
                (user_id, enabled, frequency, weekday, day_of_month, hour, minute, timezone, next_run_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id) DO UPDATE SET
                enabled = EXCLUDED.enabled,
                frequency = EXCLUDED.frequency,
                weekday = EXCLUDED.weekday,
                day_of_month = EXCLUDED.day_of_month,
                hour = EXCLUDED.hour,
                minute = EXCLUDED.minute,
                timezone = EXCLUDED.timezone,
                next_run_at = EXCLUDED.next_run_at,
                updated_at = NOW()
            RETURNING user_id, enabled, frequency, weekday, day_of_month, hour, minute,
                      timezone, next_run_at, last_sent_at, last_error, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(req.enabled)
        .bind(&req.frequency)
        .bind(req.weekday)
        .bind(req.day_of_month)
        .bind(req.hour)
        .bind(req.minute)
        .bind(&req.timezone)
        .bind(next)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn send_summary_now(&self, user_id: Uuid) -> AppResult<()> {
        let secret = notification_secret()?;
        let user: (String, String) = sqlx::query_as(
            "SELECT email, currency FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        let timezone = self
            .get_summary(user_id)
            .await
            .ok()
            .and_then(|summary| chrono_tz::Tz::from_str(&summary.timezone).ok())
            .unwrap_or(chrono_tz::UTC);
        let today = Utc::now().with_timezone(&timezone).date_naive();
        let message = crate::notifications::scheduler::build_summary(
            &self.pool,
            user_id,
            &user.1,
            today,
        )
        .await
        .map_err(AppError::Internal)?;
        let channels = sqlx::query_as::<_, (String, Vec<u8>)>(
            "SELECT provider, config FROM notification_channels WHERE user_id = $1 AND enabled = true",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        if channels.is_empty() {
            return Err(AppError::BadRequest(
                "No enabled notification channels".to_string(),
            ));
        }
        let mut failures = Vec::new();
        for (provider, config) in channels {
            let Some(provider) = NotificationProvider::parse(&provider) else {
                continue;
            };
            let config = decrypt_with_secret(&secret, &config).map_err(AppError::Internal)?;
            if let Err(error) = send_message(provider, &config, &user.0, &message).await {
                failures.push(error.to_string());
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(AppError::Internal(failures.join("; ")))
        }
    }
}

impl SqlxNotificationSettingsService {
    async fn load_config(
        &self,
        user_id: Uuid,
        provider: NotificationProvider,
        secret: &str,
    ) -> AppResult<Value> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT config FROM notification_channels WHERE user_id = $1 AND provider = $2",
        )
        .bind(user_id)
        .bind(provider.as_str())
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some((config,)) if !config.is_empty() => {
                decrypt_with_secret(secret, &config).map_err(AppError::Internal)
            }
            _ => Ok(json!({})),
        }
    }
}

fn default_summary() -> SummaryNotificationSettingsResponse {
    SummaryNotificationSettingsResponse {
        enabled: false,
        frequency: "monthly".to_string(),
        weekday: None,
        day_of_month: Some(1),
        hour: 9,
        minute: 0,
        timezone: DEFAULT_TIMEZONE.to_string(),
        next_run_at: None,
        last_sent_at: None,
        last_error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_summary_is_monthly_utc() {
        let summary = default_summary();
        assert!(!summary.enabled);
        assert_eq!(summary.frequency, "monthly");
        assert_eq!(summary.day_of_month, Some(1));
        assert_eq!(summary.timezone, "UTC");
    }
}
