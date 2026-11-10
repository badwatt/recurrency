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
struct EmailConfig {
    smtp_host: String,
    smtp_port: u16,
    #[serde(default = "default_encryption")]
    encryption: String,
    #[serde(default)]
    smtp_username: String,
    #[serde(default)]
    smtp_password: String,
    from_email: String,
}

fn default_encryption() -> String {
    "tls".to_string()
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

#[derive(Debug, Deserialize)]
struct SlackConfig {
    webhook_url: String,
}

#[derive(Debug, Deserialize)]
struct MattermostConfig {
    webhook_url: String,
}

#[derive(Debug, Deserialize)]
struct PushoverConfig {
    token: String,
    user_key: String,
}

#[derive(Debug, Deserialize)]
struct PushPlusConfig {
    token: String,
}

#[derive(Debug, Deserialize)]
struct ServerChanConfig {
    send_key: String,
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
        NotificationProvider::Email => {
            if object.is_empty() {
                return Ok(json!({}));
            }
            let smtp_host = required_string(object, "smtp_host")?;
            let smtp_port = object
                .get("smtp_port")
                .and_then(|value| value.as_u64())
                .and_then(|value| u16::try_from(value).ok())
                .filter(|value| *value > 0)
                .ok_or_else(|| "Email SMTP port is required".to_string())?;
            let from_email = required_string(object, "from_email")?;
            let encryption = optional_string(object, "encryption").unwrap_or_else(|| "tls".to_string());
            if !matches!(encryption.as_str(), "tls" | "ssl" | "none") {
                return Err("Email encryption must be tls, ssl or none".to_string());
            }
            Ok(json!({
                "smtp_host": smtp_host,
                "smtp_port": smtp_port,
                "encryption": encryption,
                "smtp_username": optional_string(object, "smtp_username").unwrap_or_default(),
                "smtp_password": optional_string(object, "smtp_password").unwrap_or_default(),
                "from_email": from_email
            }))
        }
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
        NotificationProvider::Slack => {
            let webhook_url = required_string(object, "webhook_url")?;
            let url = validate_url(&webhook_url, true)?;
            if url.host_str() != Some("hooks.slack.com") {
                return Err("Slack webhook URL must point to hooks.slack.com".to_string());
            }
            Ok(json!({"webhook_url": webhook_url}))
        }
        NotificationProvider::Mattermost => {
            let webhook_url = required_string(object, "webhook_url")?;
            validate_url(&webhook_url, false)?;
            Ok(json!({"webhook_url": webhook_url}))
        }
        NotificationProvider::Pushover => Ok(json!({
            "token": required_string(object, "token")?,
            "user_key": required_string(object, "user_key")?
        })),
        NotificationProvider::PushPlus => Ok(json!({
            "token": required_string(object, "token")?
        })),
        NotificationProvider::ServerChan => Ok(json!({
            "send_key": required_string(object, "send_key")?
        })),
    }
}

pub fn mask_config(provider: NotificationProvider, config: &Value) -> Value {
    let Some(object) = config.as_object() else {
        return json!({});
    };
    let mut masked = object.clone();
    let secret_keys: &[&str] = match provider {
        NotificationProvider::Email => &["smtp_password"],
        NotificationProvider::Ntfy => &["token"],
        NotificationProvider::Telegram => &["bot_token"],
        NotificationProvider::Discord => &["webhook_url"],
        NotificationProvider::Gotify => &["token"],
        NotificationProvider::Webhook => &["url"],
        NotificationProvider::Slack => &["webhook_url"],
        NotificationProvider::Mattermost => &["webhook_url"],
        NotificationProvider::Pushover => &["token"],
        NotificationProvider::PushPlus => &["token"],
        NotificationProvider::ServerChan => &["send_key"],
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
        NotificationProvider::Email => &["smtp_password"],
        NotificationProvider::Ntfy => &["token"],
        NotificationProvider::Telegram => &["bot_token"],
        NotificationProvider::Discord => &["webhook_url"],
        NotificationProvider::Gotify => &["token"],
        NotificationProvider::Webhook => &["url"],
        NotificationProvider::Slack => &["webhook_url"],
        NotificationProvider::Mattermost => &["webhook_url"],
        NotificationProvider::Pushover => &["token"],
        NotificationProvider::PushPlus => &["token"],
        NotificationProvider::ServerChan => &["send_key"],
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
        NotificationProvider::Email => send_email(config, recipient_email, message).await,
        NotificationProvider::Ntfy => send_ntfy(config, message).await,
        NotificationProvider::Telegram => send_telegram(config, message).await,
        NotificationProvider::Discord => send_discord(config, message).await,
        NotificationProvider::Gotify => send_gotify(config, message).await,
        NotificationProvider::Webhook => send_webhook(config, message).await,
        NotificationProvider::Slack => send_slack(config, message).await,
        NotificationProvider::Mattermost => send_mattermost(config, message).await,
        NotificationProvider::Pushover => send_pushover(config, message).await,
        NotificationProvider::PushPlus => send_pushplus(config, message).await,
        NotificationProvider::ServerChan => send_serverchan(config, message).await,
    }
}

async fn send_email(
    config: &Value,
    recipient_email: &str,
    message: &NotificationMessage,
) -> AppResult<()> {
    if config.as_object().is_none_or(|object| object.is_empty()) {
        return crate::email::send_text_email(recipient_email, &message.title, &message.body).await;
    }
    let config: EmailConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid email configuration".to_string()))?;
    let smtp = crate::email::SmtpConfig {
        host: config.smtp_host,
        port: config.smtp_port,
        user: config.smtp_username,
        password: config.smtp_password,
        from: config.from_email,
        app_name: std::env::var("APP_NAME").unwrap_or_else(|_| "Recurrency".to_string()),
        encryption: config.encryption,
    };
    crate::email::send_text_email_with_config(&smtp, recipient_email, &message.title, &message.body)
        .await
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

async fn send_slack(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: SlackConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid Slack configuration".to_string()))?;
    let response = client()
        .await?
        .post(config.webhook_url)
        .json(&json!({"text": format!("*{}*\n{}", message.title, message.body)}))
        .send()
        .await
        .map_err(|_| AppError::Internal("Slack request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_mattermost(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: MattermostConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid Mattermost configuration".to_string()))?;
    let response = client()
        .await?
        .post(config.webhook_url)
        .json(&json!({"text": format!("**{}**\n{}", message.title, message.body)}))
        .send()
        .await
        .map_err(|_| AppError::Internal("Mattermost request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_pushover(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: PushoverConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid Pushover configuration".to_string()))?;
    let response = client()
        .await?
        .post("https://api.pushover.net/1/messages.json")
        .form(&[
            ("token", config.token.as_str()),
            ("user", config.user_key.as_str()),
            ("title", message.title.as_str()),
            ("message", message.body.as_str()),
        ])
        .send()
        .await
        .map_err(|_| AppError::Internal("Pushover request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_pushplus(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: PushPlusConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid PushPlus configuration".to_string()))?;
    let response = client()
        .await?
        .post("https://www.pushplus.plus/send")
        .json(&json!({
            "token": config.token,
            "title": message.title,
            "content": message.body,
            "template": "txt"
        }))
        .send()
        .await
        .map_err(|_| AppError::Internal("PushPlus request failed".to_string()))?;
    ensure_success(response.status()).await
}

async fn send_serverchan(config: &Value, message: &NotificationMessage) -> AppResult<()> {
    let config: ServerChanConfig = serde_json::from_value(config.clone())
        .map_err(|_| AppError::BadRequest("Invalid ServerChan configuration".to_string()))?;
    let url = format!("https://sctapi.ftqq.com/{}.send", config.send_key);
    let response = client()
        .await?
        .post(url)
        .form(&[("title", message.title.as_str()), ("desp", message.body.as_str())])
        .send()
        .await
        .map_err(|_| AppError::Internal("ServerChan request failed".to_string()))?;
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
    fn validates_per_user_smtp_config() {
        let config = json!({
            "smtp_host": "smtp.example.com",
            "smtp_port": 587,
            "encryption": "tls",
            "smtp_username": "user",
            "smtp_password": "secret",
            "from_email": "recurrency@example.com"
        });
        let validated = validate_config(NotificationProvider::Email, &config).unwrap();
        assert_eq!(validated["smtp_host"], "smtp.example.com");
        assert_eq!(validated["smtp_password"], "secret");
        let masked = mask_config(NotificationProvider::Email, &validated);
        assert_eq!(masked["smtp_password"], MASKED_VALUE);
    }

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
    fn validates_pushover_config() {
        let config = json!({"token": "app", "user_key": "user"});
        assert!(validate_config(NotificationProvider::Pushover, &config).is_ok());
        let masked = mask_config(NotificationProvider::Pushover, &config);
        assert_eq!(masked["token"], MASKED_VALUE);
        assert_eq!(masked["user_key"], "user");
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
