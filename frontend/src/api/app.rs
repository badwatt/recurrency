use crate::api::client::{ApiError, api_get};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppInfo {
    pub status: String,
    pub smtp_configured: bool,
}

pub async fn get_app_info() -> Result<AppInfo, ApiError> {
    api_get("/health").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_smtp_enabled() {
        let json = r#"{"status":"OK","smtp_configured":true}"#;
        let info: AppInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.status, "OK");
        assert!(info.smtp_configured);
    }

    #[test]
    fn app_info_smtp_disabled() {
        let json = r#"{"status":"OK","smtp_configured":false}"#;
        let info: AppInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.status, "OK");
        assert!(!info.smtp_configured);
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn app_info_is_fetched_from_health() {
        let info: AppInfo = crate::api::test_support::with_server(
            r#"{"status":"OK","smtp_configured":true}"#,
            get_app_info(),
        )
        .await
        .unwrap();
        assert_eq!(info.status, "OK");
        assert!(info.smtp_configured);
    }
}
