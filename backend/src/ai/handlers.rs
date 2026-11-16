use axum::{Json, extract::State, http::HeaderMap};

use crate::ai::models::{
    AiChatRequest, AiChatResponse, AiModelsRequest, AiModelsResponse, AiSettingsResponse,
    AiTranscriptionRequest, AiTranscriptionResponse, UpdateAiSettingsRequest,
};
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::{AuthQuery, OptionalQuery};

async fn user_id(
    state: &AppState,
    query: Option<AuthQuery>,
    headers: &HeaderMap,
) -> AppResult<uuid::Uuid> {
    let token = extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), headers)?;
    Ok(state.auth_service.verify_token(&token).await?.sub)
}

pub async fn get_settings(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<AiSettingsResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.ai_service.settings(user_id).await?))
}

pub async fn update_settings(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdateAiSettingsRequest>,
) -> AppResult<Json<AiSettingsResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.ai_service.update_settings(user_id, req).await?))
}

pub async fn chat(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<AiChatRequest>,
) -> AppResult<Json<AiChatResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.ai_service.chat(user_id, req).await?))
}

pub async fn models(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<AiModelsRequest>,
) -> AppResult<Json<AiModelsResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.ai_service.models(user_id, req).await?))
}

pub async fn transcribe(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<AiTranscriptionRequest>,
) -> AppResult<Json<AiTranscriptionResponse>> {
    let user_id = user_id(&state, query, &headers).await?;
    Ok(Json(state.ai_service.transcribe(user_id, req).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_request_deserializes_messages() {
        let request: AiChatRequest =
            serde_json::from_str(r#"{"messages":[{"role":"user","content":"Add Netflix"}]}"#)
                .unwrap();
        assert_eq!(request.messages.len(), 1);
    }

    #[test]
    fn models_request_allows_the_default_endpoint() {
        let request: AiModelsRequest = serde_json::from_str("{}").unwrap();
        assert!(request.base_url.is_none());
    }

    #[test]
    fn settings_request_deserializes() {
        let request: UpdateAiSettingsRequest = serde_json::from_str(
            r#"{"enabled":true,"base_url":"https://api.example.com/v1","model":"model","api_key":"key"}"#,
        )
        .unwrap();
        assert!(request.enabled);
        assert_eq!(request.api_key.as_deref(), Some("key"));
    }
}
