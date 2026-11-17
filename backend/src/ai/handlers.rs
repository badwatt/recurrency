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

    mod handler_tests {
        use super::*;
        use crate::ai::models::AiMessage;
        use crate::shared::OptionalQuery;
        use crate::shared::mocks::MockAuthService;
        use axum::http::HeaderMap;
        use std::sync::Arc;

        fn headers() -> HeaderMap {
            let mut headers = HeaderMap::new();
            headers.insert(
                "Authorization",
                "Bearer mock_token_00000000-0000-0000-0000-000000000001"
                    .parse()
                    .unwrap(),
            );
            headers
        }

        fn state() -> AppState {
            let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
            AppState::with_auth_service(pool, Arc::new(MockAuthService::success()))
        }

        #[tokio::test]
        async fn handlers_delegate_to_the_service() {
            let state = state();
            let Json(settings) = get_settings(
                State(state.clone()),
                OptionalQuery(None),
                headers(),
            )
            .await
            .unwrap();
            assert!(!settings.enabled);

            let Json(updated) = update_settings(
                State(state.clone()),
                OptionalQuery(None),
                headers(),
                Json(UpdateAiSettingsRequest {
                    enabled: true,
                    provider: Some("openai".to_string()),
                    base_url: None,
                    model: None,
                    api_key: None,
                    clear_api_key: false,
                }),
            )
            .await
            .unwrap();
            assert!(!updated.enabled);

            let Json(chat) = chat(
                State(state.clone()),
                OptionalQuery(None),
                headers(),
                Json(AiChatRequest {
                    messages: vec![AiMessage {
                        role: "user".to_string(),
                        content: "hello".to_string(),
                    }],
                }),
            )
            .await
            .unwrap();
            assert!(!chat.reply.is_empty());

            let Json(models) = models(
                State(state.clone()),
                OptionalQuery(None),
                headers(),
                Json(AiModelsRequest { base_url: None }),
            )
            .await
            .unwrap();
            assert_eq!(models.models.len(), 1);

            let Json(transcribed) = transcribe(
                State(state.clone()),
                OptionalQuery(None),
                headers(),
                Json(AiTranscriptionRequest {
                    audio_base64: "AAAA".to_string(),
                    mime_type: "audio/wav".to_string(),
                }),
            )
            .await
            .unwrap();
            assert!(!transcribed.text.is_empty());

            let unauthorized = get_settings(
                State(state),
                OptionalQuery(None),
                HeaderMap::new(),
            )
            .await;
            assert!(unauthorized.is_err());
        }

        #[tokio::test]
        async fn query_tokens_are_accepted() {
            let state = state();
            let query = OptionalQuery(Some(AuthQuery {
                token: Some("mock_token_00000000-0000-0000-0000-000000000002".to_string()),
            }));
            let result = get_settings(State(state), query, HeaderMap::new()).await;
            assert!(result.is_ok());
        }
    }
}
