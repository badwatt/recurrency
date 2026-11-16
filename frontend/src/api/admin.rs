use serde::{Deserialize, Serialize};

use crate::api::client::{ApiError, api_get, api_put};

const ADMIN_USERS_PATH: &str = "/api/admin/users";
const ADMIN_SETTINGS_PATH: &str = "/api/admin/settings";
const ADMIN_BACKUP_PATH: &str = "/api/admin/backup";

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_paths_are_versioned_api_routes() {
        assert!(ADMIN_USERS_PATH.starts_with("/api/"));
        assert!(ADMIN_SETTINGS_PATH.starts_with("/api/"));
        assert!(ADMIN_BACKUP_PATH.starts_with("/api/"));
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
}
