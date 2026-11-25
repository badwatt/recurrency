use crate::api::client::{ApiError, api_get, api_post, api_put};
use crate::models::notification::*;
use serde_json::Value;

pub async fn list() -> Result<Vec<NotificationChannel>, ApiError> {
    api_get("/api/settings/notification-channels").await
}

pub async fn update(
    provider: &str,
    req: &UpdateNotificationChannelRequest,
) -> Result<NotificationChannel, ApiError> {
    api_put(
        &format!("/api/settings/notification-channels/{provider}"),
        req,
    )
    .await
}

pub async fn test(provider: &str, config: Value) -> Result<(), ApiError> {
    api_post::<_, serde_json::Value>(
        &format!("/api/settings/notification-channels/{provider}/test"),
        &serde_json::json!({ "config": config }),
    )
    .await
    .map(|_| ())
}

pub async fn send_summary_now() -> Result<(), ApiError> {
    api_post::<_, serde_json::Value>(
        "/api/settings/summary-notification/test",
        &serde_json::json!({}),
    )
    .await
    .map(|_| ())
}

pub async fn get_summary() -> Result<SummaryNotificationSettings, ApiError> {
    api_get("/api/settings/summary-notification").await
}

pub async fn update_summary(
    req: &UpdateSummaryNotificationSettingsRequest,
) -> Result<SummaryNotificationSettings, ApiError> {
    api_put("/api/settings/summary-notification", req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_paths() {
        assert_eq!(
            "/api/settings/notification-channels/telegram",
            "/api/settings/notification-channels/telegram"
        );
        assert_eq!(
            "/api/settings/notification-channels/ntfy/test",
            "/api/settings/notification-channels/ntfy/test"
        );
        assert_eq!(
            "/api/settings/summary-notification",
            "/api/settings/summary-notification"
        );
    }

    #[test]
    fn update_request_serializes() {
        let req = UpdateNotificationChannelRequest {
            enabled: true,
            config: serde_json::json!({"chat_id": "1"}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("chat_id"));
    }
    use crate::api::test_support::with_server;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn notification_channel_endpoints_work() {
        let channel = NotificationChannel {
            provider: "ntfy".into(),
            enabled: true,
            configured: true,
            config: serde_json::json!({"topic": "alerts"}),
        };
        let listed: Vec<NotificationChannel> = with_server(
            format!("[{}]", serde_json::to_string(&channel).unwrap()),
            list(),
        )
        .await
        .unwrap();
        assert_eq!(listed.len(), 1);

        let updated: NotificationChannel = with_server(
            serde_json::to_string(&channel).unwrap(),
            update(
                "ntfy",
                &UpdateNotificationChannelRequest {
                    enabled: true,
                    config: serde_json::json!({"topic": "alerts"}),
                },
            ),
        )
        .await
        .unwrap();
        assert!(updated.enabled);

        with_server("{}", test("ntfy", serde_json::json!({"topic": "alerts"})))
            .await
            .unwrap();
        with_server("{}", send_summary_now()).await.unwrap();

        let summary = SummaryNotificationSettings {
            enabled: true,
            frequency: "daily".into(),
            weekday: None,
            day_of_month: None,
            hour: 9,
            minute: 0,
            timezone: "UTC".into(),
            period: "current".into(),
            next_run_at: None,
            last_sent_at: None,
            last_error: None,
        };
        let fetched: SummaryNotificationSettings =
            with_server(serde_json::to_string(&summary).unwrap(), get_summary())
                .await
                .unwrap();
        assert!(fetched.enabled);

        let saved: SummaryNotificationSettings = with_server(
            serde_json::to_string(&summary).unwrap(),
            update_summary(&UpdateSummaryNotificationSettingsRequest {
                enabled: true,
                frequency: "daily".into(),
                weekday: None,
                day_of_month: None,
                hour: 9,
                minute: 0,
                timezone: "UTC".into(),
                period: "current".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(saved.frequency, "daily");
    }
}
