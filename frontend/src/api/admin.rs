use serde::{Deserialize, Serialize};

use crate::api::client::{ApiError, api_delete, api_get, api_post, api_put};

const ADMIN_USERS_PATH: &str = "/api/admin/users";
const ADMIN_SETTINGS_PATH: &str = "/api/admin/settings";
const ADMIN_BACKUP_PATH: &str = "/api/admin/backup";
const ADMIN_RESTORE_PATH: &str = "/api/admin/backup/restore";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: Option<String>,
    pub currency: String,
    pub verified: bool,
    pub is_admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminSettings {
    pub registration_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateAdminSettingsRequest {
    pub registration_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateAdminRoleRequest {
    pub is_admin: bool,
}

pub async fn users() -> Result<Vec<AdminUser>, ApiError> {
    api_get(ADMIN_USERS_PATH).await
}

pub async fn settings() -> Result<AdminSettings, ApiError> {
    api_get(ADMIN_SETTINGS_PATH).await
}

pub async fn set_registration_enabled(enabled: bool) -> Result<AdminSettings, ApiError> {
    api_put(
        ADMIN_SETTINGS_PATH,
        &UpdateAdminSettingsRequest {
            registration_enabled: enabled,
        },
    )
    .await
}

pub async fn full_backup() -> Result<serde_json::Value, ApiError> {
    api_get(ADMIN_BACKUP_PATH).await
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreResult {
    pub restored_tables: usize,
}

pub async fn restore_backup(backup: &serde_json::Value) -> Result<RestoreResult, ApiError> {
    api_post(ADMIN_RESTORE_PATH, backup).await
}

pub async fn delete_user(id: uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("{ADMIN_USERS_PATH}/{id}")).await
}

pub async fn set_admin(id: uuid::Uuid, is_admin: bool) -> Result<AdminUser, ApiError> {
    api_put(
        &format!("{ADMIN_USERS_PATH}/{id}/role"),
        &UpdateAdminRoleRequest { is_admin },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_paths_are_versioned_api_routes() {
        assert!(ADMIN_USERS_PATH.starts_with("/api/"));
        assert!(ADMIN_SETTINGS_PATH.starts_with("/api/"));
        assert!(ADMIN_BACKUP_PATH.starts_with("/api/"));
        assert!(ADMIN_RESTORE_PATH.starts_with("/api/"));
    }

    #[test]
    fn role_update_serializes_the_flag() {
        let request = UpdateAdminRoleRequest { is_admin: true };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("is_admin"));
        assert!(json.contains("true"));
    }

    #[test]
    fn registration_update_serializes_the_flag() {
        let request = UpdateAdminSettingsRequest {
            registration_enabled: false,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("registration_enabled"));
        assert!(json.contains("false"));
    }
    use crate::api::test_support::serve_once;
    use serial_test::serial;

    async fn with_server<F, T>(response: impl Into<String>, call: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let (base, server) = serve_once("200 OK", response);
        crate::api::client::set_base_url(&base);
        let result = call.await;
        crate::api::client::clear_base_url();
        server.join().unwrap();
        result
    }

    #[tokio::test]
    #[serial]
    async fn admin_endpoints_parse_their_responses() {
        let id = uuid::Uuid::new_v4();
        let user = format!(
            r#"{{"id":"{id}","email":"a@b.c","name":"A","currency":"EUR","verified":true,"is_admin":false,"created_at":"2026-01-01T00:00:00Z"}}"#
        );
        let users_json = format!("[{user}]");
        let users = with_server(users_json, users()).await.unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, id);

        let settings = with_server(r#"{"registration_enabled":true}"#, settings())
            .await
            .unwrap();
        assert!(settings.registration_enabled);

        let updated = with_server(
            r#"{"registration_enabled":false}"#,
            set_registration_enabled(false),
        )
        .await
        .unwrap();
        assert!(!updated.registration_enabled);

        let backup = with_server(r#"{"tables":{"users":[]}}"#, full_backup())
            .await
            .unwrap();
        assert!(backup["tables"].is_object());

        let restored = with_server(
            r#"{"restored_tables":14}"#,
            restore_backup(&serde_json::json!({"format_version": 1})),
        )
        .await
        .unwrap();
        assert_eq!(restored.restored_tables, 14);

        let role = with_server(user, set_admin(id, true)).await.unwrap();
        assert_eq!(role.id, id);

        let (base, server) = serve_once("204 No Content", "");
        crate::api::client::set_base_url(&base);
        delete_user(id).await.unwrap();
        crate::api::client::clear_base_url();
        server.join().unwrap();
    }
}
