use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationProvider {
    Email,
    Ntfy,
    Telegram,
    Discord,
    Gotify,
    Webhook,
    Slack,
    Mattermost,
    Pushover,
    PushPlus,
    ServerChan,
}

impl NotificationProvider {
    pub const ALL: [Self; 11] = [
        Self::Email,
        Self::Ntfy,
        Self::Telegram,
        Self::Discord,
        Self::Gotify,
        Self::Webhook,
        Self::Slack,
        Self::Mattermost,
        Self::Pushover,
        Self::PushPlus,
        Self::ServerChan,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Ntfy => "ntfy",
            Self::Telegram => "telegram",
            Self::Discord => "discord",
            Self::Gotify => "gotify",
            Self::Webhook => "webhook",
            Self::Slack => "slack",
            Self::Mattermost => "mattermost",
            Self::Pushover => "pushover",
            Self::PushPlus => "pushplus",
            Self::ServerChan => "serverchan",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|provider| provider.as_str() == value)
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct NotificationChannelRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub enabled: bool,
    pub config: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelResponse {
    pub provider: String,
    pub enabled: bool,
    pub configured: bool,
    pub config: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNotificationChannelRequest {
    pub enabled: bool,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, FromRow)]
pub struct SummaryNotificationSettings {
    pub user_id: Uuid,
    pub enabled: bool,
    pub frequency: String,
    pub weekday: Option<i16>,
    pub day_of_month: Option<i16>,
    pub hour: i16,
    pub minute: i16,
    pub timezone: String,
    pub period: String,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_sent_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryNotificationSettingsResponse {
    pub enabled: bool,
    pub frequency: String,
    pub weekday: Option<i16>,
    pub day_of_month: Option<i16>,
    pub hour: i16,
    pub minute: i16,
    pub timezone: String,
    pub period: String,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_sent_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

impl From<SummaryNotificationSettings> for SummaryNotificationSettingsResponse {
    fn from(value: SummaryNotificationSettings) -> Self {
        Self {
            enabled: value.enabled,
            frequency: value.frequency,
            weekday: value.weekday,
            day_of_month: value.day_of_month,
            hour: value.hour,
            minute: value.minute,
            timezone: value.timezone,
            period: value.period,
            next_run_at: value.next_run_at,
            last_sent_at: value.last_sent_at,
            last_error: value.last_error,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSummaryNotificationSettingsRequest {
    pub enabled: bool,
    pub frequency: String,
    pub weekday: Option<i16>,
    pub day_of_month: Option<i16>,
    pub hour: i16,
    pub minute: i16,
    pub timezone: String,
    #[serde(default = "current_period")]
    pub period: String,
}

fn current_period() -> String {
    SUMMARY_PERIOD_CURRENT.to_string()
}

pub const DEFAULT_TIMEZONE: &str = "UTC";
pub const SUMMARY_PERIOD_CURRENT: &str = "current";
pub const SUMMARY_PERIOD_NEXT: &str = "next";
pub const MASKED_VALUE: &str = "********";

pub fn default_channel(provider: NotificationProvider) -> NotificationChannelResponse {
    NotificationChannelResponse {
        provider: provider.as_str().to_string(),
        enabled: false,
        configured: false,
        config: json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_strings_roundtrip() {
        for provider in NotificationProvider::ALL {
            assert_eq!(
                NotificationProvider::parse(provider.as_str()),
                Some(provider)
            );
        }
        assert_eq!(NotificationProvider::parse("unknown"), None);
    }

    #[test]
    fn provider_serde_is_lowercase() {
        let json = serde_json::to_string(&NotificationProvider::Telegram).unwrap();
        assert_eq!(json, "\"telegram\"");
        assert_eq!(
            serde_json::from_str::<NotificationProvider>("\"ntfy\"").unwrap(),
            NotificationProvider::Ntfy
        );
    }

    #[test]
    fn default_channel_is_disabled_and_unconfigured() {
        let channel = default_channel(NotificationProvider::Email);
        assert!(!channel.enabled);
        assert!(!channel.configured);
        assert_eq!(channel.provider, "email");
    }
}
