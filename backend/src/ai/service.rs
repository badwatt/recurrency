use base64::Engine;
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::accounts::models::Account;
use crate::ai::models::{
    AI_JSON_INSTRUCTION, AiAction, AiActionResult, AiChatRequest, AiChatResponse, AiModelOutput,
    AiModelsRequest, AiModelsResponse, AiSettingsResponse, AiTranscriptionRequest,
    AiTranscriptionResponse, DEFAULT_BASE_URL, DEFAULT_MODEL, DEFAULT_PROVIDER, MAX_CHAT_MESSAGES,
    MAX_CONTEXT_ITEMS, MAX_MESSAGE_CHARS, MAX_MODELS, MAX_TRANSCRIPTION_BYTES, MODELS_PATH,
    TRANSCRIPTION_MODEL, TRANSCRIPTIONS_PATH, UpdateAiSettingsRequest,
};
use crate::categories::models::Category;
use crate::debts::models::{CreateDebtRequest, UpdateDebtRequest};
use crate::debts::service::{DebtServiceTrait, SqlxDebtService};
use crate::notifications::crypto::{decrypt_with_secret, encrypt_with_secret};
use crate::shared::error::{AppError, AppResult};
use crate::subscriptions::models::{CreateSubscriptionRequest, UpdateSubscriptionRequest};
use crate::subscriptions::service::{SqlxSubscriptionService, SubscriptionServiceTrait};
use crate::transactions::models::{CreateTransactionRequest, UpdateTransactionRequest};
use crate::transactions::service::{SqlxTransactionService, TransactionServiceTrait};

const JWT_SECRET_KEY: &str = "jwt_secret";
const API_KEY_FIELD: &str = "api_key";
const CHAT_COMPLETIONS_PATH: &str = "chat/completions";
const HTTP_TIMEOUT_SECONDS: u64 = 60;
const TRANSCRIPTION_TIMEOUT_SECONDS: u64 = 120;
const AUDIO_FILE_NAME: &str = "audio";
const MIME_ERROR_LIMIT: usize = 500;

#[derive(Debug, sqlx::FromRow)]
struct AiSettingsRow {
    enabled: bool,
    provider: String,
    base_url: String,
    model: String,
    api_key_encrypted: Option<Vec<u8>>,
}

#[derive(Debug)]
struct AiRuntime {
    base_url: String,
    model: String,
    api_key: String,
}

#[async_trait::async_trait]
pub trait AiServiceTrait: Send + Sync {
    async fn settings(&self, user_id: Uuid) -> AppResult<AiSettingsResponse>;
    async fn update_settings(
        &self,
        user_id: Uuid,
        req: UpdateAiSettingsRequest,
    ) -> AppResult<AiSettingsResponse>;
    async fn chat(&self, user_id: Uuid, req: AiChatRequest) -> AppResult<AiChatResponse>;
    async fn models(&self, user_id: Uuid, req: AiModelsRequest) -> AppResult<AiModelsResponse>;
    async fn transcribe(
        &self,
        user_id: Uuid,
        req: AiTranscriptionRequest,
    ) -> AppResult<AiTranscriptionResponse>;
}

pub struct SqlxAiService {
    pool: PgPool,
}

impl SqlxAiService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn encryption_secret(&self) -> AppResult<String> {
        if let Ok(secret) = std::env::var("JWT_SECRET")
            && !secret.trim().is_empty()
        {
            return Ok(secret);
        }
        let stored: Option<String> =
            sqlx::query_scalar("SELECT value FROM app_secrets WHERE key = $1")
                .bind(JWT_SECRET_KEY)
                .fetch_optional(&self.pool)
                .await?;
        stored
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| AppError::Internal("JWT secret is required for AI settings".to_string()))
    }

    async fn row(&self, user_id: Uuid) -> AppResult<Option<AiSettingsRow>> {
        Ok(sqlx::query_as::<_, AiSettingsRow>(
            "SELECT enabled, provider, base_url, model, api_key_encrypted \
             FROM ai_settings WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn credentials(&self, user_id: Uuid) -> AppResult<(AiSettingsRow, String)> {
        let row = self
            .row(user_id)
            .await?
            .ok_or_else(|| AppError::BadRequest("AI assistant is not configured".to_string()))?;
        let encrypted = row.api_key_encrypted.as_ref().ok_or_else(|| {
            AppError::BadRequest("Add an API key before using the AI assistant".to_string())
        })?;
        let secret = self.encryption_secret().await?;
        let value = decrypt_with_secret(&secret, encrypted)
            .map_err(|error| AppError::Internal(format!("Could not read AI key: {error}")))?;
        let api_key = value
            .get(API_KEY_FIELD)
            .and_then(Value::as_str)
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| AppError::BadRequest("AI API key is empty".to_string()))?
            .to_string();
        Ok((row, api_key))
    }

    async fn runtime(&self, user_id: Uuid) -> AppResult<AiRuntime> {
        let (row, api_key) = self.credentials(user_id).await?;
        if !row.enabled {
            return Err(AppError::BadRequest("AI assistant is disabled".to_string()));
        }
        Ok(AiRuntime {
            base_url: row.base_url,
            model: row.model,
            api_key,
        })
    }

    async fn context(&self, user_id: Uuid) -> AppResult<Value> {
        let subscriptions = SqlxSubscriptionService::new(self.pool.clone())
            .get_all(user_id)
            .await?;
        let transactions = SqlxTransactionService::new(self.pool.clone())
            .get_all(user_id)
            .await?;
        let debts = SqlxDebtService::new(self.pool.clone())
            .get_all(user_id)
            .await?;
        let accounts: Vec<Account> =
            sqlx::query_as("SELECT id, user_id, name, icon, color, is_default, created_at, updated_at FROM accounts WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;
        let categories: Vec<Category> =
            sqlx::query_as("SELECT id, user_id, name, icon, color, is_system, is_default, created_at, updated_at FROM categories WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?;
        let currency: String = sqlx::query_scalar("SELECT currency FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(json!({
            "currency": currency,
            "today": Utc::now().date_naive().to_string(),
            "subscriptions": subscriptions.into_iter().take(MAX_CONTEXT_ITEMS).map(|value| json!({
                "id": value.id,
                "name": value.name,
                "amount": value.amount,
                "currency": value.currency,
                "frequency": value.frequency,
                "recurrence": value.recurrence.0,
                "next_billing_date": value.next_billing_date,
                "is_active": value.is_active,
                "logo_url": value.logo_url,
            })).collect::<Vec<_>>(),
            "transactions": transactions.into_iter().take(MAX_CONTEXT_ITEMS).map(|value| json!({
                "id": value.id,
                "name": value.name,
                "amount": value.amount,
                "currency": value.currency,
                "date": value.date,
                "logo_url": value.logo_url,
            })).collect::<Vec<_>>(),
            "debts": debts.into_iter().take(MAX_CONTEXT_ITEMS).map(|value| json!({
                "id": value.id,
                "name": value.name,
                "total_amount": value.total_amount,
                "remaining_amount": value.remaining_amount,
                "currency": value.currency,
                "debt_type": value.debt_type,
                "logo_url": value.logo_url,
            })).collect::<Vec<_>>(),
            "accounts": accounts.into_iter().take(MAX_CONTEXT_ITEMS).map(|value| json!({
                "id": value.id,
                "name": value.name,
                "is_default": value.is_default,
            })).collect::<Vec<_>>(),
            "categories": categories.into_iter().take(MAX_CONTEXT_ITEMS).map(|value| json!({
                "id": value.id,
                "name": value.name,
                "is_default": value.is_default,
            })).collect::<Vec<_>>(),
        }))
    }

    async fn call_model(
        &self,
        runtime: &AiRuntime,
        messages: &[Value],
    ) -> AppResult<AiModelOutput> {
        let endpoint = chat_endpoint(&runtime.base_url);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
            .map_err(|error| AppError::Internal(format!("Could not create AI client: {error}")))?;
        let body = json!({
            "model": runtime.model,
            "messages": messages,
            "temperature": 0.2,
        });
        let response = client
            .post(endpoint)
            .bearer_auth(&runtime.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("AI request failed: {error}")))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "AI provider returned {status}: {}",
                body.chars().take(500).collect::<String>()
            )));
        }
        let value: Value = response
            .json()
            .await
            .map_err(|error| AppError::Internal(format!("Invalid AI response: {error}")))?;
        let content = value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .unwrap_or_default();
        Ok(parse_model_output(content))
    }

    async fn execute_action(&self, user_id: Uuid, action: &AiAction) -> AiActionResult {
        let result = match action.action_type.as_str() {
            "create_subscription" => self.create_subscription(user_id, &action.data).await,
            "update_subscription" => self.update_subscription(user_id, action).await,
            "delete_subscription" => self.delete_subscription(user_id, action).await,
            "create_transaction" => self.create_transaction(user_id, &action.data).await,
            "update_transaction" => self.update_transaction(user_id, action).await,
            "delete_transaction" => self.delete_transaction(user_id, action).await,
            "create_debt" => self.create_debt(user_id, &action.data).await,
            "update_debt" => self.update_debt(user_id, action).await,
            "delete_debt" => self.delete_debt(user_id, action).await,
            other => Err(AppError::BadRequest(format!("Unknown AI action: {other}"))),
        };
        match result {
            Ok(message) => AiActionResult {
                action: action.action_type.clone(),
                success: true,
                message,
            },
            Err(error) => AiActionResult {
                action: action.action_type.clone(),
                success: false,
                message: error.to_string(),
            },
        }
    }

    async fn create_subscription(&self, user_id: Uuid, data: &Value) -> AppResult<String> {
        let currency = self.user_currency(user_id).await?;
        let mut data = data.as_object().cloned().ok_or_else(|| {
            AppError::BadRequest("Subscription data must be an object".to_string())
        })?;
        data.entry("currency").or_insert_with(|| json!(currency));
        let find_logo = data
            .remove("find_logo")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let name = data
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if find_logo
            && data.get("logo_url").and_then(Value::as_str).is_none()
            && let Some(logo) = find_logo_url(&name).await
        {
            data.insert("logo_url".to_string(), json!(logo));
        }
        if data.get("frequency").is_none() {
            let annual = data
                .get("recurrence")
                .and_then(|value| value.get("unit"))
                .and_then(Value::as_str)
                == Some("year");
            data.insert(
                "frequency".to_string(),
                json!(if annual { "yearly" } else { "monthly" }),
            );
        }
        data.entry("start_date")
            .or_insert_with(|| json!(Utc::now().date_naive().to_string()));
        let request: CreateSubscriptionRequest = serde_json::from_value(Value::Object(data))
            .map_err(|error| AppError::BadRequest(format!("Invalid subscription: {error}")))?;
        let subscription = SqlxSubscriptionService::new(self.pool.clone())
            .create(user_id, request)
            .await?;
        Ok(format!(
            "Created subscription {} ({})",
            subscription.name, subscription.id
        ))
    }

    async fn update_subscription(&self, user_id: Uuid, action: &AiAction) -> AppResult<String> {
        let id = action_uuid(action)?;
        let data = resolve_update_logo(&action.data).await;
        let request: UpdateSubscriptionRequest = serde_json::from_value(data).map_err(|error| {
            AppError::BadRequest(format!("Invalid subscription update: {error}"))
        })?;
        let subscription = SqlxSubscriptionService::new(self.pool.clone())
            .update(user_id, id, request)
            .await?;
        Ok(format!(
            "Updated subscription {} ({})",
            subscription.name, subscription.id
        ))
    }

    async fn delete_subscription(&self, user_id: Uuid, action: &AiAction) -> AppResult<String> {
        let id = action_uuid(action)?;
        SqlxSubscriptionService::new(self.pool.clone())
            .delete(user_id, id)
            .await?;
        Ok(format!("Deleted subscription {id}"))
    }

    async fn create_transaction(&self, user_id: Uuid, data: &Value) -> AppResult<String> {
        let currency = self.user_currency(user_id).await?;
        let mut data = data.as_object().cloned().ok_or_else(|| {
            AppError::BadRequest("Transaction data must be an object".to_string())
        })?;
        data.entry("currency").or_insert_with(|| json!(currency));
        let find_logo = data
            .remove("find_logo")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let name = data
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if find_logo
            && data.get("logo_url").and_then(Value::as_str).is_none()
            && let Some(logo) = find_logo_url(&name).await
        {
            data.insert("logo_url".to_string(), json!(logo));
        }
        data.entry("date")
            .or_insert_with(|| json!(Utc::now().date_naive().to_string()));
        let request: CreateTransactionRequest = serde_json::from_value(Value::Object(data))
            .map_err(|error| AppError::BadRequest(format!("Invalid transaction: {error}")))?;
        let transaction = SqlxTransactionService::new(self.pool.clone())
            .create(user_id, request)
            .await?;
        Ok(format!(
            "Created transaction {} ({})",
            transaction.name, transaction.id
        ))
    }

    async fn update_transaction(&self, user_id: Uuid, action: &AiAction) -> AppResult<String> {
        let id = action_uuid(action)?;
        let data = resolve_update_logo(&action.data).await;
        let request: UpdateTransactionRequest = serde_json::from_value(data).map_err(|error| {
            AppError::BadRequest(format!("Invalid transaction update: {error}"))
        })?;
        let transaction = SqlxTransactionService::new(self.pool.clone())
            .update(user_id, id, request)
            .await?;
        Ok(format!(
            "Updated transaction {} ({})",
            transaction.name, transaction.id
        ))
    }

    async fn delete_transaction(&self, user_id: Uuid, action: &AiAction) -> AppResult<String> {
        let id = action_uuid(action)?;
        SqlxTransactionService::new(self.pool.clone())
            .delete(user_id, id)
            .await?;
        Ok(format!("Deleted transaction {id}"))
    }

    async fn create_debt(&self, user_id: Uuid, data: &Value) -> AppResult<String> {
        let currency = self.user_currency(user_id).await?;
        let mut data = data
            .as_object()
            .cloned()
            .ok_or_else(|| AppError::BadRequest("Debt data must be an object".to_string()))?;
        data.entry("currency").or_insert_with(|| json!(currency));
        data.entry("debt_type").or_insert_with(|| json!("manual"));
        let find_logo = data
            .remove("find_logo")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let name = data
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if find_logo
            && data.get("logo_url").and_then(Value::as_str).is_none()
            && let Some(logo) = find_logo_url(&name).await
        {
            data.insert("logo_url".to_string(), json!(logo));
        }
        let request: CreateDebtRequest = serde_json::from_value(Value::Object(data))
            .map_err(|error| AppError::BadRequest(format!("Invalid debt: {error}")))?;
        let debt = SqlxDebtService::new(self.pool.clone())
            .create(user_id, request)
            .await?;
        Ok(format!("Created debt {} ({})", debt.name, debt.id))
    }

    async fn update_debt(&self, user_id: Uuid, action: &AiAction) -> AppResult<String> {
        let id = action_uuid(action)?;
        let data = resolve_update_logo(&action.data).await;
        let request: UpdateDebtRequest = serde_json::from_value(data)
            .map_err(|error| AppError::BadRequest(format!("Invalid debt update: {error}")))?;
        let debt = SqlxDebtService::new(self.pool.clone())
            .update(user_id, id, request)
            .await?;
        Ok(format!("Updated debt {} ({})", debt.name, debt.id))
    }

    async fn delete_debt(&self, user_id: Uuid, action: &AiAction) -> AppResult<String> {
        let id = action_uuid(action)?;
        SqlxDebtService::new(self.pool.clone())
            .delete(user_id, id)
            .await?;
        Ok(format!("Deleted debt {id}"))
    }

    async fn user_currency(&self, user_id: Uuid) -> AppResult<String> {
        Ok(
            sqlx::query_scalar("SELECT currency FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?,
        )
    }
}

#[async_trait::async_trait]
impl AiServiceTrait for SqlxAiService {
    async fn models(&self, user_id: Uuid, req: AiModelsRequest) -> AppResult<AiModelsResponse> {
        let (row, api_key) = self.credentials(user_id).await?;
        let base_url = req.base_url.unwrap_or(row.base_url);
        validate_base_url(&base_url)?;
        let endpoint = models_endpoint(&base_url);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
            .map_err(|error| AppError::Internal(format!("Could not create AI client: {error}")))?;
        let response = client
            .get(endpoint)
            .bearer_auth(&api_key)
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Could not load AI models: {error}")))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "AI provider returned {status}: {}",
                body.chars().take(MIME_ERROR_LIMIT).collect::<String>()
            )));
        }
        let value: Value = response
            .json()
            .await
            .map_err(|error| AppError::Internal(format!("Invalid AI models response: {error}")))?;
        Ok(AiModelsResponse {
            models: parse_models(&value),
        })
    }

    async fn transcribe(
        &self,
        user_id: Uuid,
        req: AiTranscriptionRequest,
    ) -> AppResult<AiTranscriptionResponse> {
        let runtime = self.runtime(user_id).await?;
        let mime_type = req.mime_type.trim().to_lowercase();
        if !mime_type.starts_with("audio/") {
            return Err(AppError::BadRequest(
                "Speech audio must use an audio MIME type".to_string(),
            ));
        }
        let audio = base64::engine::general_purpose::STANDARD
            .decode(req.audio_base64.trim())
            .map_err(|_| AppError::BadRequest("Speech audio is not valid base64".to_string()))?;
        if audio.is_empty() {
            return Err(AppError::BadRequest("Speech audio is empty".to_string()));
        }
        if audio.len() > MAX_TRANSCRIPTION_BYTES {
            return Err(AppError::BadRequest("Speech audio is too long".to_string()));
        }
        let endpoint = transcription_endpoint(&runtime.base_url);
        let file_name = format!("{AUDIO_FILE_NAME}.{}", audio_extension(&mime_type));
        let part = reqwest::multipart::Part::bytes(audio)
            .file_name(file_name)
            .mime_str(&mime_type)
            .map_err(|error| AppError::Internal(format!("Invalid audio MIME type: {error}")))?;
        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", TRANSCRIPTION_MODEL)
            .text("response_format", "json");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                TRANSCRIPTION_TIMEOUT_SECONDS,
            ))
            .build()
            .map_err(|error| AppError::Internal(format!("Could not create AI client: {error}")))?;
        let response = client
            .post(endpoint)
            .bearer_auth(&runtime.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|error| AppError::Internal(format!("Speech transcription failed: {error}")))?;
        let status = response.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::BadRequest(
                    "This provider does not expose /audio/transcriptions. Choose an STT-capable provider, or use browser, Android or on-device speech.".to_string(),
                ));
            }
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "AI provider returned {status}: {}",
                body.chars().take(MIME_ERROR_LIMIT).collect::<String>()
            )));
        }
        let value: Value = response.json().await.map_err(|error| {
            AppError::Internal(format!("Invalid speech transcription response: {error}"))
        })?;
        let text = value
            .pointer("/text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            return Err(AppError::BadRequest(
                "The provider returned an empty transcription".to_string(),
            ));
        }
        Ok(AiTranscriptionResponse { text })
    }

    async fn settings(&self, user_id: Uuid) -> AppResult<AiSettingsResponse> {
        let row = self.row(user_id).await?;
        Ok(match row {
            Some(row) => AiSettingsResponse {
                enabled: row.enabled,
                provider: row.provider,
                base_url: row.base_url,
                model: row.model,
                configured: row.api_key_encrypted.is_some(),
            },
            None => AiSettingsResponse {
                enabled: false,
                provider: DEFAULT_PROVIDER.to_string(),
                base_url: DEFAULT_BASE_URL.to_string(),
                model: DEFAULT_MODEL.to_string(),
                configured: false,
            },
        })
    }

    async fn update_settings(
        &self,
        user_id: Uuid,
        req: UpdateAiSettingsRequest,
    ) -> AppResult<AiSettingsResponse> {
        let existing = self.row(user_id).await?;
        let provider = req
            .provider
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                existing
                    .as_ref()
                    .map_or(DEFAULT_PROVIDER.to_string(), |row| row.provider.clone())
            });
        if provider.len() > 40
            || !provider
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        {
            return Err(AppError::BadRequest("Invalid AI provider".to_string()));
        }
        let base_url = req.base_url.unwrap_or_else(|| {
            existing
                .as_ref()
                .map_or(DEFAULT_BASE_URL.to_string(), |row| row.base_url.clone())
        });
        let model = req.model.unwrap_or_else(|| {
            existing
                .as_ref()
                .map_or(DEFAULT_MODEL.to_string(), |row| row.model.clone())
        });
        validate_base_url(&base_url)?;
        if model.trim().is_empty() {
            return Err(AppError::BadRequest("Model is required".to_string()));
        }
        let secret = self.encryption_secret().await?;
        let encrypted = if req.clear_api_key {
            None
        } else if let Some(key) = req.api_key.filter(|key| !key.trim().is_empty()) {
            Some(
                encrypt_with_secret(&secret, &json!({ API_KEY_FIELD: key }))
                    .map_err(AppError::Internal)?,
            )
        } else {
            existing.and_then(|row| row.api_key_encrypted)
        };
        sqlx::query(
            "INSERT INTO ai_settings (user_id, enabled, provider, base_url, model, api_key_encrypted, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, NOW()) \
             ON CONFLICT (user_id) DO UPDATE SET enabled = EXCLUDED.enabled, provider = EXCLUDED.provider, base_url = EXCLUDED.base_url, model = EXCLUDED.model, api_key_encrypted = EXCLUDED.api_key_encrypted, updated_at = NOW()",
        )
        .bind(user_id)
        .bind(req.enabled)
        .bind(&provider)
        .bind(&base_url)
        .bind(&model)
        .bind(encrypted)
        .execute(&self.pool)
        .await?;
        self.settings(user_id).await
    }

    async fn chat(&self, user_id: Uuid, req: AiChatRequest) -> AppResult<AiChatResponse> {
        if req.messages.is_empty() {
            return Err(AppError::BadRequest("Send a message".to_string()));
        }
        let runtime = self.runtime(user_id).await?;
        let context = self.context(user_id).await?;
        let mut messages = vec![json!({
            "role": "system",
            "content": format!("{AI_JSON_INSTRUCTION}\n\nCurrent data JSON:\n{context}")
        })];
        for message in req.messages.iter().rev().take(MAX_CHAT_MESSAGES).rev() {
            if !matches!(message.role.as_str(), "user" | "assistant") {
                continue;
            }
            messages.push(json!({
                "role": message.role,
                "content": message.content.chars().take(MAX_MESSAGE_CHARS).collect::<String>(),
            }));
        }
        let output = self.call_model(&runtime, &messages).await?;
        let mut actions = Vec::new();
        for action in &output.actions {
            actions.push(self.execute_action(user_id, action).await);
        }
        let mut reply = output.reply.trim().to_string();
        if reply.is_empty() {
            reply = if actions.is_empty() {
                "I couldn't find an action to take.".to_string()
            } else {
                "Done.".to_string()
            };
        }
        Ok(AiChatResponse { reply, actions })
    }
}

async fn resolve_update_logo(data: &Value) -> Value {
    let mut data = data.clone();
    let Some(object) = data.as_object_mut() else {
        return data;
    };
    let find_logo = object
        .remove("find_logo")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if find_logo && object.get("logo_url").and_then(Value::as_str).is_none() {
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if let Some(logo) = find_logo_url(name).await {
            object.insert("logo_url".to_string(), json!(logo));
        }
    }
    data
}

async fn find_logo_url(name: &str) -> Option<String> {
    if name.trim().is_empty() {
        return None;
    }
    crate::logos::service::search_logos(name)
        .await
        .ok()
        .and_then(|results| results.into_iter().next())
        .map(|result| result.url)
}

fn api_root(base_url: &str) -> &str {
    let base = base_url.trim_end_matches('/');
    base.strip_suffix(CHAT_COMPLETIONS_PATH)
        .map(|value| value.trim_end_matches('/'))
        .unwrap_or(base)
}

fn chat_endpoint(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with(CHAT_COMPLETIONS_PATH) {
        base.to_string()
    } else {
        format!("{base}/{CHAT_COMPLETIONS_PATH}")
    }
}

fn models_endpoint(base_url: &str) -> String {
    format!("{}/{MODELS_PATH}", api_root(base_url))
}

fn transcription_endpoint(base_url: &str) -> String {
    format!("{}/{TRANSCRIPTIONS_PATH}", api_root(base_url))
}

fn audio_extension(mime_type: &str) -> &'static str {
    match mime_type.split(';').next().unwrap_or_default().trim() {
        "audio/wav" | "audio/x-wav" | "audio/wave" => "wav",
        "audio/mp4" | "audio/m4a" | "audio/x-m4a" => "m4a",
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/ogg" => "ogg",
        _ => "webm",
    }
}

fn parse_models(value: &Value) -> Vec<String> {
    let candidates = value
        .get("data")
        .and_then(Value::as_array)
        .or_else(|| value.as_array());
    let mut models: Vec<String> = candidates
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("id").and_then(Value::as_str))
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    models.sort();
    models.dedup();
    models.truncate(MAX_MODELS);
    models
}

fn validate_base_url(value: &str) -> AppResult<()> {
    let url = reqwest::Url::parse(value.trim())
        .map_err(|_| AppError::BadRequest("Invalid AI base URL".to_string()))?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if !(url.scheme() == "https" || (url.scheme() == "http" && local)) {
        return Err(AppError::BadRequest(
            "AI base URL must use HTTPS (HTTP is allowed for localhost)".to_string(),
        ));
    }
    Ok(())
}

fn action_uuid(action: &AiAction) -> AppResult<Uuid> {
    action
        .id
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Action id is required".to_string()))
        .and_then(|id| {
            Uuid::parse_str(id).map_err(|_| AppError::BadRequest("Invalid action id".to_string()))
        })
}

fn parse_model_output(content: &str) -> AiModelOutput {
    let trimmed = content.trim();
    let json_text = trimmed
        .find('{')
        .and_then(|start| trimmed.rfind('}').map(|end| &trimmed[start..=end]))
        .unwrap_or(trimmed);
    serde_json::from_str(json_text).unwrap_or_else(|_| AiModelOutput {
        reply: trimmed.to_string(),
        actions: Vec::new(),
    })
}

/// Test-only helper used by the integration suite without calling a provider.
#[cfg(test)]
pub fn parse_model_output_for_test(content: &str) -> AiModelOutput {
    parse_model_output(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::constants::DEMO_EMAIL;
    use crate::shared::password_utils::hash_password;

    #[test]
    fn endpoint_accepts_base_or_full_path() {
        assert_eq!(
            chat_endpoint("https://api.example.com/v1"),
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(
            chat_endpoint("https://api.example.com/v1/chat/completions"),
            "https://api.example.com/v1/chat/completions"
        );
    }

    #[test]
    fn models_endpoint_strips_a_full_chat_path() {
        assert_eq!(
            models_endpoint("https://api.example.com/v1/chat/completions"),
            "https://api.example.com/v1/models"
        );
        assert_eq!(
            models_endpoint("https://api.example.com/v1/"),
            "https://api.example.com/v1/models"
        );
    }

    #[test]
    fn transcription_endpoint_strips_a_full_chat_path() {
        assert_eq!(
            transcription_endpoint("https://api.example.com/v1/chat/completions"),
            "https://api.example.com/v1/audio/transcriptions"
        );
    }

    #[test]
    fn model_list_is_sorted_filtered_and_deduplicated() {
        let value = json!({
            "data": [
                {"id": "z-model"},
                {"id": "a-model"},
                {"id": "a-model"},
                {"id": " "},
                {}
            ]
        });
        assert_eq!(parse_models(&value), vec!["a-model", "z-model"]);
    }

    #[test]
    fn audio_extension_follows_the_recorder_mime() {
        assert_eq!(audio_extension("audio/webm;codecs=opus"), "webm");
        assert_eq!(audio_extension("audio/mp4"), "m4a");
        assert_eq!(audio_extension("audio/ogg"), "ogg");
        assert_eq!(audio_extension("audio/wav"), "wav");
    }

    #[test]
    fn base_url_requires_https_except_localhost() {
        assert!(validate_base_url("https://api.example.com/v1").is_ok());
        assert!(validate_base_url("http://localhost:11434/v1").is_ok());
        assert!(validate_base_url("http://api.example.com/v1").is_err());
    }

    #[test]
    fn model_output_keeps_plain_text() {
        let output = parse_model_output("The subscription costs 10 EUR.");
        assert_eq!(output.reply, "The subscription costs 10 EUR.");
        assert!(output.actions.is_empty());
    }

    #[test]
    fn model_output_strips_code_fences() {
        let output = parse_model_output("```json\n{\"reply\":\"Done\",\"actions\":[]}\n```");
        assert_eq!(output.reply, "Done");
    }

    #[test]
    fn action_uuid_requires_valid_id() {
        let action = AiAction {
            action_type: "delete_subscription".to_string(),
            id: Some("not-a-uuid".to_string()),
            data: Value::Null,
        };
        assert!(action_uuid(&action).is_err());
    }

    #[test]
    fn demo_email_is_not_a_placeholder() {
        assert!(DEMO_EMAIL.contains('@'));
    }

    #[test]
    fn hash_password_is_available_for_future_tools() {
        assert!(hash_password("password123").unwrap().starts_with("$argon2"));
    }
}
