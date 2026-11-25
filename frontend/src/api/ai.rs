use serde::{Deserialize, Serialize};

use crate::api::client::{ApiError, api_get, api_post, api_put};

const AI_SETTINGS_PATH: &str = "/api/ai/settings";
const AI_CHAT_PATH: &str = "/api/ai/chat";
const AI_MODELS_PATH: &str = "/api/ai/models";
const AI_TRANSCRIBE_PATH: &str = "/api/ai/transcribe";

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
    pub provider: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiModelsResponse {
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiTranscriptionResponse {
    pub text: String,
}

pub async fn models(base_url: Option<&str>) -> Result<Vec<String>, ApiError> {
    #[derive(Serialize)]
    struct Request<'a> {
        base_url: Option<&'a str>,
    }
    let response: AiModelsResponse = api_post(AI_MODELS_PATH, &Request { base_url }).await?;
    Ok(response.models)
}

pub async fn transcribe(audio_base64: &str, mime_type: &str) -> Result<String, ApiError> {
    #[derive(Serialize)]
    struct Request<'a> {
        audio_base64: &'a str,
        mime_type: &'a str,
    }
    let response: AiTranscriptionResponse = api_post(
        AI_TRANSCRIBE_PATH,
        &Request {
            audio_base64,
            mime_type,
        },
    )
    .await?;
    Ok(response.text)
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
        assert!(AI_MODELS_PATH.starts_with("/api/"));
        assert!(AI_TRANSCRIBE_PATH.starts_with("/api/"));
    }

    #[test]
    fn models_and_transcription_requests_serialize() {
        let json = serde_json::to_string(&AiModelsResponse {
            models: vec!["gpt-4o-mini".to_string()],
        })
        .unwrap();
        assert!(json.contains("gpt-4o-mini"));
        let json = serde_json::to_string(&AiTranscriptionResponse {
            text: "hello".to_string(),
        })
        .unwrap();
        assert!(json.contains("hello"));
    }

    #[test]
    fn settings_request_serializes_safely() {
        let request = UpdateAiSettingsRequest {
            enabled: true,
            provider: Some("openai".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            model: Some("model".to_string()),
            api_key: None,
            clear_api_key: false,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("https://api.example.com/v1"));
        assert!(!json.contains("secret"));
    }
    use crate::api::test_support::serve_once;
    use serial_test::serial;

    async fn with_server<F, T>(response: &'static str, call: F) -> T
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
    async fn settings_and_updates_use_the_api() {
        let settings = with_server(
            r#"{"enabled":true,"provider":"openai","base_url":"https://api.example.com/v1","model":"gpt","configured":true}"#,
            settings(),
        )
        .await
        .unwrap();
        assert!(settings.configured);
        assert_eq!(settings.provider, "openai");

        let updated = with_server(
            r#"{"enabled":false,"provider":"groq","base_url":"https://api.groq.com/openai/v1","model":"llama","configured":false}"#,
            update_settings(&UpdateAiSettingsRequest {
                enabled: false,
                provider: Some("groq".into()),
                base_url: None,
                model: None,
                api_key: None,
                clear_api_key: false,
            }),
        )
        .await
        .unwrap();
        assert_eq!(updated.provider, "groq");
    }

    #[tokio::test]
    #[serial]
    async fn models_transcription_and_chat_parse_the_api() {
        let models = with_server(r#"{"models":["a","b"]}"#, models(None))
            .await
            .unwrap();
        assert_eq!(models, vec!["a".to_string(), "b".to_string()]);

        let text = with_server(r#"{"text":"dictated"}"#, transcribe("AAAA", "audio/wav"))
            .await
            .unwrap();
        assert_eq!(text, "dictated");

        let chat = with_server(
            r#"{"reply":"Done","actions":[{"action":"create_subscription","success":true,"message":"ok"}]}"#,
            chat(&[AiMessage {
                role: "user".into(),
                content: "add one".into(),
            }]),
        )
        .await
        .unwrap();
        assert_eq!(chat.reply, "Done");
        assert_eq!(chat.actions.len(), 1);
        assert!(chat.actions[0].success);
    }
}
