use serde::{Deserialize, Serialize};

use crate::api::client::{ApiError, api_get, api_post, api_put};

const AI_SETTINGS_PATH: &str = "/api/ai/settings";
const AI_CHAT_PATH: &str = "/api/ai/chat";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSettings {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateAiSettingsRequest {
    pub enabled: bool,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub clear_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiActionResult {
    pub action: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiChatResponse {
    pub reply: String,
    pub actions: Vec<AiActionResult>,
}

pub async fn settings() -> Result<AiSettings, ApiError> {
    api_get(AI_SETTINGS_PATH).await
}

pub async fn update_settings(req: &UpdateAiSettingsRequest) -> Result<AiSettings, ApiError> {
    api_put(AI_SETTINGS_PATH, req).await
}

pub async fn chat(messages: &[AiMessage]) -> Result<AiChatResponse, ApiError> {
    #[derive(Serialize)]
    struct Request<'a> {
        messages: &'a [AiMessage],
    }
    api_post(AI_CHAT_PATH, &Request { messages }).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_routes_are_api_paths() {
        assert!(AI_SETTINGS_PATH.starts_with("/api/"));
        assert!(AI_CHAT_PATH.starts_with("/api/"));
    }

    #[test]
    fn settings_request_serializes_safely() {
        let request = UpdateAiSettingsRequest {
            enabled: true,
            base_url: Some("https://api.example.com/v1".to_string()),
            model: Some("model".to_string()),
            api_key: None,
            clear_api_key: false,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("https://api.example.com/v1"));
        assert!(!json.contains("secret"));
    }
}
