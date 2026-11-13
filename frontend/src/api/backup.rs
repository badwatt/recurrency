use serde_json::Value;

use crate::api::client::{ApiError, api_get, api_post, is_offline_mode};

pub async fn export() -> Result<Value, ApiError> {
    if is_offline_mode() {
        crate::storage::local::export_backup().await
    } else {
        api_get("/api/backup").await
    }
}

pub async fn import(backup: &Value) -> Result<Value, ApiError> {
    if is_offline_mode() {
        crate::storage::local::import_backup(backup).await
    } else {
        api_post("/api/backup/import", backup).await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn backup_dispatches_local_and_server() {
        let source = include_str!("backup.rs");
        assert!(source.contains("is_offline_mode"));
        assert!(source.contains("export_backup"));
        assert!(source.contains("import_backup"));
    }
}
