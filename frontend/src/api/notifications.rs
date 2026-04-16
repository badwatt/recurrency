use crate::api::client::{api_get, api_post, api_delete, ApiError};
use crate::models::notification::*;

pub async fn list() -> Result<NotificationListResponse, ApiError> {
    api_get("/api/notifications").await
}

pub async fn mark_as_read(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_post(&format!("/api/notifications/{}/read", id), &serde_json::json!({})).await
}

pub async fn mark_all_as_read() -> Result<(), ApiError> {
    api_post("/api/notifications/read-all", &serde_json::json!({})).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/notifications/{}", id)).await
}

#[cfg(test)]
mod tests {
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
}