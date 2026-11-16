use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A user row shown in the administration panel.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct AdminUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub currency: String,
    pub verified: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminSettings {
    pub registration_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UpdateAdminRoleRequest {
    pub is_admin: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UpdateAdminSettingsRequest {
    pub registration_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreResult {
    pub restored_tables: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullBackup {
    pub format_version: u32,
    pub app_version: String,
    pub exported_at: DateTime<Utc>,
    pub tables: std::collections::BTreeMap<String, serde_json::Value>,
}

pub const FULL_BACKUP_FORMAT_VERSION: u32 = 1;
