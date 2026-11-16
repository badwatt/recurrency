use serde::{Deserialize, Serialize};

pub const DEFAULT_PROVIDER: &str = "openai";
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_MODEL: &str = "gpt-4o-mini";
pub const MAX_CONTEXT_ITEMS: usize = 40;
pub const MAX_CHAT_MESSAGES: usize = 20;
pub const MAX_MESSAGE_CHARS: usize = 8_000;
pub const MODELS_PATH: &str = "models";
pub const TRANSCRIPTIONS_PATH: &str = "audio/transcriptions";
pub const TRANSCRIPTION_MODEL: &str = "whisper-1";
pub const MAX_MODELS: usize = 200;
pub const MAX_TRANSCRIPTION_BYTES: usize = 10_000_000;
pub const TRANSCRIPTION_BODY_LIMIT_BYTES: usize = 15 * 1024 * 1024;
pub const AI_JSON_INSTRUCTION: &str = r#"You are the Recurrency assistant. The user manages subscriptions, one-time transactions and debts.
Respond with ONLY one JSON object and no markdown fences:
{"reply":"short natural-language confirmation or answer","actions":[]}
Allowed action objects:
{"type":"create_subscription","data":{"name":"...","amount":"10.00","currency":"EUR","frequency":"monthly","start_date":"YYYY-MM-DD","notes":"...","logo_url":"https://...","find_logo":false}}
{"type":"update_subscription","id":"UUID","data":{}}
{"type":"delete_subscription","id":"UUID"}
{"type":"create_transaction","data":{"name":"...","amount":"10.00","currency":"EUR","date":"YYYY-MM-DD","notes":"...","logo_url":"https://...","find_logo":false}}
{"type":"update_transaction","id":"UUID","data":{}}
{"type":"delete_transaction","id":"UUID"}
{"type":"create_debt","data":{"name":"...","total_amount":"100.00","currency":"EUR","debt_type":"manual","start_date":"YYYY-MM-DD","notes":"...","logo_url":"https://...","find_logo":false}}
{"type":"update_debt","id":"UUID","data":{}}
{"type":"delete_debt","id":"UUID"}
Use IDs from the supplied context for updates and deletes. Never invent IDs. Amounts are decimal strings. Use the user's currency unless they specify another. 
For multiple annual billing dates in one subscription, set frequency to "yearly" and include:
"recurrence":{"interval":1,"unit":"year","days":[],"weekdays":[],"dates":[{"month":1,"day":15},{"month":6,"day":15,"amount":"10.00"}]}
Each annual date may have its own amount. Do not create duplicate dates.
If the user asks to find or add a logo, set "find_logo": true in that action's data (create or update). If they provide a logo URL, put it in "logo_url". 
If no action is needed, return an empty actions array."#;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AiModelsRequest {
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiModelsResponse {
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AiTranscriptionRequest {
    pub audio_base64: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiTranscriptionResponse {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiSettingsResponse {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub configured: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateAiSettingsRequest {
    pub enabled: bool,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub clear_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AiChatRequest {
    pub messages: Vec<AiMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiAction {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub data: serde_json::Value,
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

#[derive(Debug, Deserialize)]
pub struct AiModelOutput {
    #[serde(default)]
    pub reply: String,
    #[serde(default)]
    pub actions: Vec<AiAction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_output_parses_actions() {
        let output: AiModelOutput = serde_json::from_str(
            r#"{"reply":"Done","actions":[{"type":"delete_subscription","id":"00000000-0000-0000-0000-000000000001"}]}"#,
        )
        .unwrap();
        assert_eq!(output.reply, "Done");
        assert_eq!(output.actions.len(), 1);
        assert_eq!(output.actions[0].action_type, "delete_subscription");
    }

    #[test]
    fn settings_request_defaults_optional_fields() {
        let request: UpdateAiSettingsRequest = serde_json::from_str(r#"{"enabled":true}"#).unwrap();
        assert!(request.enabled);
        assert!(request.api_key.is_none());
        assert!(request.provider.is_none());
        assert!(!request.clear_api_key);
    }

    #[test]
    fn defaults_are_stable() {
        assert_eq!(DEFAULT_PROVIDER, "openai");
        assert!(DEFAULT_BASE_URL.starts_with("https://"));
        assert!(!DEFAULT_MODEL.is_empty());
    }
}
