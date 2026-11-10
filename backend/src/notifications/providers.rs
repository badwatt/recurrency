use std::net::IpAddr;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::notifications::channels::{MASKED_VALUE, NotificationProvider};
use crate::notifications::crypto::encrypt_with_secret;
use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
struct NtfyConfig {
    server: String,
    topic: String,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramConfig {
    bot_token: String,
    chat_id: String,
}

#[derive(Debug, Deserialize)]
struct DiscordConfig {
    webhook_url: String,
}

#[derive(Debug, Deserialize)]
struct GotifyConfig {
    server_url: String,
    token: String,
}

#[derive(Debug, Deserialize)]
struct WebhookConfig {
    url: String,
}

fn as_object(value: &Value) -> Result<&serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| "Notification config must be an object".to_string())
}

fn required_string(object: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("Notification config field {key} is required"))
}

fn optional_string(object: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn allow_private_hosts() -> bool {
    std::env::var("NOTIFICATION_ALLOW_PRIVATE_HOSTS")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn validate_url(raw: &str, require_https: bool) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(raw).map_err(|_| "Invalid notification URL".to_string())?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Notification URL must use http or https".to_string());
    }
    if require_https && url.scheme() != "https" {
        return Err("Notification URL must use https".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "Notification URL has no host".to_string())?;
    if !allow_private_hosts() && is_private_host(host) {
        return Err("Notification URL points to a private host".to_string());
    }
    Ok(url)
}

fn is_private_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return true;
    }
    host.parse::<IpAddr>().is_ok_and(|ip| {
        if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
            return true;
        }
        match ip {
            IpAddr::V4(ip) => ip.is_private() || ip.is_link_local(),
            IpAddr::V6(ip) => ip.is_unique_local() || ip.is_unicast_link_local(),
        }
    })
}

pub fn validate_config(provider: NotificationProvider, config: &Value) -> Result<Value, String> {
    let object = as_object(config)?;
    match provider {
        NotificationProvider::Email => Ok(json!({})),
        NotificationProvider::Ntfy => {
            let server = required_string(object, "server")?;
            validate_url(&server, false)?;
            let topic = required_string(object, "topic")?;
            Ok(json!({
                "server": server,
                "topic": topic,
                "token": optional_string(object, "token")
            }))
        }
        NotificationProvider::Telegram => {
            let bot_token = required_string(object, "bot_token")?;
            let chat_id = required_string(object, "chat_id")?;
            Ok(json!({"bot_token": bot_token, "chat_id": chat_id}))
        }
        NotificationProvider::Discord => {
            let webhook_url = required_string(object, "webhook_url")?;
            let url = validate_url(&webhook_url, true)?;
            if !matches!(
                url.host_str(),
                Some("discord.com" | "discordapp.com" | "canary.discord.com")
            ) {
                return Err("Discord webhook URL must point to discord.com".to_string());
            }
            Ok(json!({"webhook_url": webhook_url}))
        }
        NotificationProvider::Gotify => {
            let server_url = required_string(object, "server_url")?;
            validate_url(&server_url, false)?;
            let token = required_string(object, "token")?;
            Ok(json!({"server_url": server_url, "token": token}))
        }
        NotificationProvider::Webhook => {
            let url = required_string(object, "url")?;
            validate_url(&url, false)?;
            Ok(json!({"url": url}))
        }
    }
}

pub fn mask_config(provider: NotificationProvider, config: &Value) -> Value {
    let Some(object) = config.as_object() else {
        return json!({});
    };
    let mut masked = object.clone();
    let secret_keys: &[&str] = match provider {
        NotificationProvider::Email => &[],
        NotificationProvider::Ntfy => &["token"],
        NotificationProvider::Telegram => &["bot_token"],
        NotificationProvider::Discord => &["webhook_url"],
        NotificationProvider::Gotify => &["token"],
        NotificationProvider::Webhook => &["url"],
    };
    for key in secret_keys {
        if masked.get(*key).and_then(Value::as_str).is_some_and(|value| !value.is_empty()) {
            masked.insert((*key).to_string(), Value::String(MASKED_VALUE.to_string()));
        }
    }
    Value::Object(masked)
}

pub fn merge_config(provider: NotificationProvider, existing: &Value, incoming: &Value) -> Value {
    let Some(incoming_object) = incoming.as_object() else {
        return existing.clone();
    };
    let mut merged = existing
        .as_object()
        .cloned()
        .unwrap_or_default();
    let secret_keys: &[&str] = match provider {
        NotificationProvider::Email => &[],
        NotificationProvider::Ntfy => &["token"],
        NotificationProvider::Telegram => &["bot_token"],
        NotificationProvider::Discord => &["webhook_url"],
        NotificationProvider::Gotify => &["token"],
        NotificationProvider::Webhook => &["url"],
    };
    for (key, value) in incoming_object {
        if value.as_str() == Some(MASKED_VALUE) {
            continue;
        }
        if secret_keys.contains(&key.as_str()) && value.as_str().is_some_and(str::is_empty) {
            continue;
        }
        merged.insert(key.clone(), value.clone());
    }
    Value::Object(merged)
}

pub fn encrypt_config(secret: &str, config: &Value) -> Result<Vec<u8>, String> {
    encrypt_with_secret(secret, config)
}

async fn client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| AppError::Internal("Could not build notification client".to_string()))
}

pub async fn send_message(
    provider: NotificationProvider,
    config: &Value,
    recipient_email: &str,
    message: &NotificationMessage,
) -> AppResult<()> {
    match provider {
        NotificationProvider::Email => {
            crate::email::send_text_email(recipient_email, &message.title, &message.body).await
        }
        NotificationProvider::Ntfy => send_ntfy(config, message).await,
        NotificationProvider::Telegram => send_telegram(config, message).await,
        NotificationProvider::Discord => send_discord(config, message).await,
        NotificationProvider::Gotify => send_gotify(config, message).await,
        NotificationProvider::Webhook => send_webhook(config, message).await,
    }
}

async fn send_ntfy(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: NtfyConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid ntfy configuration".to_string()))?;
    let base = config.server.trim_end_matches('/');
    let topic = config.topic.trim_start_matches('/');
    let mut request = client()
        .await?
        .post(format!("{base}/{topic}"))
        .header("Title", message.title.clone())
        .body(message.body.clone());
    if let Some(token) = config.token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .await
        .map_err(|_| AppError::Internal("ntfy request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_telegram(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: TelegramConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid Telegram configuration".to_string()))?;
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );
    let response = client()
        .await?
        .post(url)
        .json(&json!({
            "chat_id": config.chat_id,
            "text": format!("{}\n\n{}", message.title, message.body)
        }))
        .send()
        .await
        .map_err(|_| AppError::Internal("Telegram request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_discord(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: DiscordConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid Discord configuration".to_string()))?;
    let response = client()
        .await?
        .post(config.webhook_url)
        .json(&json!({"content": format!("**{}**\n{}", message.title, message.body)}))
        .send()
        .await
        .map_err(|_| AppError::Internal("Discord request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_gotify(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: GotifyConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid Gotify configuration".to_string()))?;
    let url = reqwest::Url::parse_with_params(
        &format!("{}/message", config.server_url.trim_end_matches('/')),
        &[("token", config.token.as_str())],
    )
    .map_err(|_| AppError::BadRequest("Invalid Gotify URL".to_string()))?;
    let response = client()
        .await?
        .post(url)
        .json(&json!({"title": message.title, "message": message.body, "priority": 0}))
        .send()
        .await
        .map_err(|_| AppError::Internal("Gotify request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_webhook(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: WebhookConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid webhook configuration".to_string()))?;
    let response = client()
        .await?
        .post(config.url)
        .json(&json!({"title": message.title, "message": message.body}))
        .send()
        .await
        .map_err(|_| AppError::Internal("Webhook request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn ensure_success(status: reqwest::StatusCode) -> AppResult<()> {
    if status.is_success() {
        Ok(())
    } else {
        Err(AppError::Internal(format!(
            "Notification provider returned HTTP {}",
            status.as_u16()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_telegram_token() {
        let masked = mask_config(
            NotificationProvider::Telegram,
            &json!({"bot_token": "secret", "chat_id": "123"}),
        );
        assert_eq!(masked["bot_token"], MASKED_VALUE);
        assert_eq!(masked["chat_id"], "123");
    }

    #[test]
    fn merge_keeps_masked_secret() {
        let merged = merge_config(
            NotificationProvider::Telegram,
            &json!({"bot_token": "old", "chat_id": "123"}),
            &json!({"bot_token": MASKED_VALUE, "chat_id": "456"}),
        );
        assert_eq!(merged["bot_token"], "old");
        assert_eq!(merged["chat_id"], "456");
    }

    #[test]
    fn rejects_private_urls_by_default() {
        assert!(validate_url("http://127.0.0.1:8080/topic", false).is_err());
        assert!(validate_url("http://localhost:8080/topic", false).is_err());
        assert!(validate_url("http://10.0.0.1/topic", false).is_err());
        assert!(validate_url("http://192.168.1.10/topic", false).is_err());
    }

    #[test]
    fn accepts_public_https_url() {
        assert!(validate_url("https://ntfy.sh/alerts", false).is_ok());
    }

    #[test]
    fn rejects_non_http_scheme() {
        assert!(validate_url("file:///tmp/notify", false).is_err());
    }

    #[test]
    fn validates_ntfy_config() {
        let config = json!({"server": "https://ntfy.sh", "topic": "alerts"});
        assert!(validate_config(NotificationProvider::Ntfy, &config).is_ok());
    }

    #[test]
    fn rejects_telegram_missing_chat() {
        let config = json!({"bot_token": "token"});
        assert!(validate_config(NotificationProvider::Telegram, &config).is_err());
    }

    #[test]
    fn rejects_discord_non_discord_host() {
        let config = json!({"webhook_url": "https://example.com/hook"});
        assert!(validate_config(NotificationProvider::Discord, &config).is_err());
    }
}
