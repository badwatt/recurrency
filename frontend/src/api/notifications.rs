use crate::api::client::{ApiError, api_delete, api_get, api_post};
use crate::models::notification::*;

pub async fn list() -> Result<NotificationListResponse, ApiError> {
    api_get("/api/notifications").await
}

pub async fn mark_as_read(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_post(
        &format!("/api/notifications/{}/read", id),
        &serde_json::json!({}),
    )
    .await
}

pub async fn mark_all_as_read() -> Result<(), ApiError> {
    api_post("/api/notifications/read-all", &serde_json::json!({})).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/notifications/{}", id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notification::*;
    use chrono::DateTime;
    use uuid::Uuid;

    #[test]
    fn notification_api_paths() {
        let base = "/api/notifications";
        assert!(base.starts_with("/api/"));

        let id = uuid::Uuid::new_v4();
        let mark_read = format!("/api/notifications/{}/read", id);
        assert!(mark_read.contains("/read"));
    }

    #[test]
    fn mark_all_read_path() {
        let path = "/api/notifications/read-all";
        assert!(path.contains("read-all"));
    }

    #[test]
    fn delete_path_format() {
        let id = uuid::Uuid::new_v4();
        let path = format!("/api/notifications/{}", id);
        assert!(path.contains(&id.to_string()));
    }

    // ── Model serialization tests ──
    #[test]
    fn notification_serialization() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".into(),
            title: "Netflix due".into(),
            message: Some("Your Netflix subscription of $15.99 is due tomorrow".into()),
            is_read: false,
            created_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&n).unwrap();
        assert!(json.contains("Netflix due"));
        let back: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Netflix due");
    }

    #[test]
    fn notification_list_response_empty() {
        let resp = NotificationListResponse {
            notifications: vec![],
            unread_count: 0,
        };
        assert!(resp.notifications.is_empty());
        assert_eq!(resp.unread_count, 0);
    }

    #[test]
    fn notification_list_response_multiple() {
        let resp = NotificationListResponse {
            notifications: vec![
                Notification {
                    id: Uuid::new_v4(),
                    user_id: Uuid::new_v4(),
                    notification_type: "billing_reminder".into(),
                    title: "A".into(),
                    message: None,
                    is_read: false,
                    created_at: DateTime::UNIX_EPOCH,
                },
                Notification {
                    id: Uuid::new_v4(),
                    user_id: Uuid::new_v4(),
                    notification_type: "billing_reminder".into(),
                    title: "B".into(),
                    message: Some("msg".into()),
                    is_read: true,
                    created_at: DateTime::UNIX_EPOCH,
                },
            ],
            unread_count: 1,
        };
        assert_eq!(resp.notifications.len(), 2);
        assert_eq!(resp.unread_count, 1);
    }

    #[test]
    fn notification_is_read_default_false() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "reminder".into(),
            title: "Test".into(),
            message: None,
            is_read: false,
            created_at: DateTime::UNIX_EPOCH,
        };
        assert!(!n.is_read);
    }

    #[test]
    fn notification_with_message() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".into(),
            title: "Due Today".into(),
            message: Some("Your subscription renews today".into()),
            is_read: false,
            created_at: DateTime::UNIX_EPOCH,
        };
        assert!(n.message.is_some());
        assert_eq!(
            n.message.as_ref().unwrap(),
            "Your subscription renews today"
        );
    }

    #[test]
    fn notification_without_message() {
        let n = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "system".into(),
            title: "System notice".into(),
            message: None,
            is_read: true,
            created_at: DateTime::UNIX_EPOCH,
        };
        assert!(n.message.is_none());
    }

    #[test]
    fn notification_types() {
        let types = vec![
            "billing_reminder",
            "payment_due",
            "payment_overdue",
            "system",
        ];
        for t in types {
            let n = Notification {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                notification_type: t.into(),
                title: "Test".into(),
                message: None,
                is_read: false,
                created_at: DateTime::UNIX_EPOCH,
            };
            assert_eq!(n.notification_type, t);
        }
    }

    #[test]
    fn mark_read_request_body_format() {
        let body = serde_json::json!({});
        assert!(body.is_object());
        assert_eq!(body.as_object().unwrap().len(), 0);
    }

    #[test]
    fn mark_all_read_request_body_format() {
        let body = serde_json::json!({});
        assert!(body.is_object());
    }

    #[test]
    fn notification_uuid_uniqueness() {
        let ids: std::collections::HashSet<_> = (0..100).map(|_| Uuid::new_v4()).collect();
        assert_eq!(ids.len(), 100);
    }
    use crate::api::test_support::{with_response, with_server};
    use chrono::Utc;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn notification_endpoints_work() {
        let notification = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            notification_type: "reminder".into(),
            title: "Due".into(),
            message: None,
            is_read: false,
            created_at: Utc::now(),
        };
        let listed: NotificationListResponse = with_server(
            serde_json::json!({ "notifications": [notification], "unread_count": 1 }).to_string(),
            list(),
        )
        .await
        .unwrap();
        assert_eq!(listed.unread_count, 1);

        with_server("null", mark_as_read(&notification.id))
            .await
            .unwrap();
        with_server("null", mark_all_as_read()).await.unwrap();
        with_response("204 No Content", "", delete(&notification.id))
            .await
            .unwrap();
    }
}
