use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::notifications::models::{
    NotificationPreferencesResponse, NotificationResponse, UpdateNotificationPreferencesRequest,
};
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::{AppError, AppResult};
use crate::shared::OptionalQuery;

#[derive(serde::Deserialize)]
pub struct NotificationQuery {
    pub token: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
) -> AppResult<Json<NotificationResponse>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let notifications = state.notification_service.list(claims.sub).await?;
    Ok(Json(notifications))
}

pub async fn mark_read(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let notification_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()))?;
    let success = state.notification_service.mark_read(claims.sub, notification_id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

pub async fn mark_all_read(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let count = state.notification_service.mark_all_read(claims.sub).await?;
    Ok(Json(serde_json::json!({ "marked_count": count })))
}

pub async fn delete(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let notification_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()))?;
    let success = state.notification_service.delete(claims.sub, notification_id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

// Notification Preferences handlers
pub async fn get_notification_preferences(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let preferences =
        state.notification_service.get_notification_preferences(claims.sub).await?;
    Ok(Json(NotificationPreferencesResponse { preferences }))
}

pub async fn update_notification_preferences(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<NotificationQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;
    let preferences = state.notification_service.update_notification_preferences(
        claims.sub,
        req.preferences,
    )
    .await?;
    Ok(Json(NotificationPreferencesResponse { preferences }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::models::{
        Notification, NotificationPreferenceResponse, NotificationPreferencesResponse,
        NotificationResponse, UpdateNotificationPreferencesRequest, UpdatePreferenceRequest,
    };
    use uuid::Uuid;

    #[test]
    fn notification_query_deserialize() {
        let query = "token=test-token";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("test-token"));
    }

    #[test]
    fn notification_query_empty() {
        let query = "";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn notification_query_empty_token() {
        let query = "token=";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.as_ref().unwrap().is_empty());
    }

    #[test]
    fn parse_valid_uuid_from_path() {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_uuid_from_path() {
        let id_str = "not-a-valid-uuid";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_uuid_creates_not_found_error() {
        // Simulates what the handler does: parse_str fails => NotFound error
        let id_str = "invalid";
        let result = Uuid::parse_str(id_str)
            .map_err(|_| AppError::NotFound("Invalid notification ID".to_string()));
        assert!(result.is_err());
        match result {
            Err(AppError::NotFound(msg)) => assert!(msg.contains("Invalid notification ID")),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn valid_uuid_creates_uuid_from_path() {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_ok());
        let uuid = result.unwrap();
        assert_eq!(uuid.to_string(), id_str);
    }

    #[test]
    fn mark_read_response_serialization() {
        let response = serde_json::json!({"success": true});
        assert!(response["success"].as_bool().unwrap());
    }

    #[test]
    fn mark_all_read_response_serialization() {
        let response = serde_json::json!({"marked_count": 5});
        assert_eq!(response["marked_count"].as_i64().unwrap(), 5);
    }

    #[test]
    fn delete_notification_response_serialization() {
        let response = serde_json::json!({"success": true});
        assert!(response["success"].as_bool().unwrap());
    }

    #[test]
    fn notification_preferences_response_serialization() {
        let resp = NotificationPreferencesResponse {
            preferences: vec![NotificationPreferenceResponse {
                channel: "email".to_string(),
                enabled: true,
                reminder_days_before: 3,
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("email"));
        assert!(json.contains("reminder_days_before"));
    }

    #[test]
    fn update_notification_preferences_request() {
        let json = r#"{"preferences":[{"channel":"email","enabled":true}]}"#;
        let req: UpdateNotificationPreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.preferences.len(), 1);
        assert_eq!(req.preferences[0].channel, "email");
        assert!(req.preferences[0].enabled);
    }

    #[test]
    fn update_notification_preferences_empty() {
        let json = r#"{"preferences":[]}"#;
        let req: UpdateNotificationPreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.preferences.is_empty());
    }

    #[test]
    fn notification_preference_request_all_channels() {
        for channel in &["email", "push", "system"] {
            let json = format!(r#"{{"channel":"{}","enabled":false}}"#, channel);
            let pref: UpdatePreferenceRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(pref.channel, *channel);
            assert!(!pref.enabled);
        }
    }

    #[test]
    fn notification_response_serialization() {
        let resp = NotificationResponse {
            notifications: vec![],
            unread_count: 0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("notifications"));
        assert!(json.contains("unread_count"));
    }

    #[test]
    fn notification_response_with_notifications() {
        let item = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".to_string(),
            title: "Upcoming Payment".to_string(),
            message: Some("Your Netflix subscription renews tomorrow".to_string()),
            is_read: false,
            created_at: chrono::Utc::now(),
        };

        let resp = NotificationResponse {
            notifications: vec![item],
            unread_count: 1,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("upcoming payment") || json.contains("Upcoming Payment"));
        assert_eq!(resp.unread_count, 1);
    }

    #[test]
    fn notification_without_message() {
        let item = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "system".to_string(),
            title: "Welcome".to_string(),
            message: None,
            is_read: true,
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_value(&item).unwrap();
        assert!(json.get("message").is_none() || json["message"].is_null());
    }

    #[test]
    fn notification_type_various_values() {
        for ntype in &[
            "billing_reminder",
            "new_subscription",
            "payment_due",
            "system",
        ] {
            let item = Notification {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                notification_type: ntype.to_string(),
                title: "Test".to_string(),
                message: None,
                is_read: false,
                created_at: chrono::Utc::now(),
            };
            assert_eq!(item.notification_type, *ntype);
        }
    }

    #[test]
    fn multiple_preferences_update() {
        let json = r#"{
            "preferences": [
                {"channel": "email", "enabled": true},
                {"channel": "push", "enabled": false},
                {"channel": "system", "enabled": true}
            ]
        }"#;
        let req: UpdateNotificationPreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.preferences.len(), 3);
    }

    #[test]
    fn notification_response_empty_notifications() {
        let resp = NotificationResponse {
            notifications: vec![],
            unread_count: 0,
        };
        assert!(resp.notifications.is_empty());
        assert_eq!(resp.unread_count, 0);
    }

    #[test]
    fn notification_response_multiple_unread() {
        let item1 = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".to_string(),
            title: "Test 1".to_string(),
            message: None,
            is_read: false,
            created_at: chrono::Utc::now(),
        };
        let item2 = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".to_string(),
            title: "Test 2".to_string(),
            message: None,
            is_read: false,
            created_at: chrono::Utc::now(),
        };
        let resp = NotificationResponse {
            notifications: vec![item1, item2],
            unread_count: 2,
        };
        assert_eq!(resp.notifications.len(), 2);
        assert_eq!(resp.unread_count, 2);
    }

    #[test]
    fn notification_preference_response_email() {
        let pref = NotificationPreferenceResponse {
            channel: "email".to_string(),
            enabled: true,
            reminder_days_before: 1,
        };
        assert_eq!(pref.channel, "email");
        assert!(pref.enabled);
        assert_eq!(pref.reminder_days_before, 1);
    }

    #[test]
    fn notification_preference_response_push() {
        let pref = NotificationPreferenceResponse {
            channel: "push".to_string(),
            enabled: false,
            reminder_days_before: 7,
        };
        assert_eq!(pref.channel, "push");
        assert!(!pref.enabled);
        assert_eq!(pref.reminder_days_before, 7);
    }

    #[test]
    fn notification_preference_response_system() {
        let pref = NotificationPreferenceResponse {
            channel: "system".to_string(),
            enabled: true,
            reminder_days_before: 3,
        };
        assert_eq!(pref.channel, "system");
        assert!(pref.enabled);
    }

    #[test]
    fn update_preference_request_all_channels() {
        let channels = ["email", "push", "system", "in_app"];
        for channel in channels.iter() {
            let json = serde_json::json!({"channel": channel, "enabled": true}).to_string();
            let req: UpdatePreferenceRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.channel, *channel);
        }
    }

    #[test]
    fn notification_with_long_message() {
        let long_msg = "A".repeat(1000);
        let item = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".to_string(),
            title: "Test".to_string(),
            message: Some(long_msg.clone()),
            is_read: false,
            created_at: chrono::Utc::now(),
        };
        assert!(item.message.is_some());
        assert_eq!(item.message.unwrap().len(), 1000);
    }

    #[test]
    fn notification_with_emoji_in_title() {
        let item = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".to_string(),
            title: "🎉 Your subscription renews!".to_string(),
            message: None,
            is_read: false,
            created_at: chrono::Utc::now(),
        };
        assert!(item.title.contains("🎉"));
    }

    #[test]
    fn notification_query_with_token() {
        let query = "token=abc123";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token, Some("abc123".to_string()));
    }

    #[test]
    fn notification_query_without_token() {
        let query = "";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn mark_read_success_true() {
        let response = serde_json::json!({"success": true});
        assert!(response["success"].as_bool().unwrap());
    }

    #[test]
    fn mark_read_success_false() {
        let response = serde_json::json!({"success": false});
        assert!(!response["success"].as_bool().unwrap());
    }

    #[test]
    fn mark_all_read_zero() {
        let response = serde_json::json!({"marked_count": 0});
        assert_eq!(response["marked_count"].as_i64().unwrap(), 0);
    }

    #[test]
    fn mark_all_read_large_count() {
        let response = serde_json::json!({"marked_count": 9999});
        assert_eq!(response["marked_count"].as_i64().unwrap(), 9999);
    }
}
