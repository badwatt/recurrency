use crate::api::notification_channels;
use crate::components::toast::TOAST;
use crate::models::notification::{
    NotificationChannel, UpdateNotificationChannelRequest,
};
use dioxus::prelude::*;
use serde_json::{Value, json};

const PROVIDERS: &[(&str, &str)] = &[
    ("email", "Email"),
    ("ntfy", "ntfy"),
    ("telegram", "Telegram"),
    ("discord", "Discord"),
    ("gotify", "Gotify"),
    ("webhook", "Webhook"),
    ("slack", "Slack"),
    ("mattermost", "Mattermost"),
    ("pushover", "Pushover"),
    ("pushplus", "PushPlus"),
    ("serverchan", "ServerChan"),
];

fn field(config: &Value, key: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

struct ChannelFields {
    server: String,
    topic: String,
    token: String,
    bot_token: String,
    chat_id: String,
    webhook_url: String,
    gotify_server: String,
    gotify_token: String,
    email_host: String,
    email_port: String,
    email_encryption: String,
    email_username: String,
    email_password: String,
    email_from: String,
    pushover_user: String,
    serverchan_key: String,
}

fn build_config(provider: &str, fields: &ChannelFields) -> Value {
    match provider {
        "email" => {
            if fields.email_host.is_empty() {
                json!({})
            } else {
                json!({
                    "smtp_host": fields.email_host,
                    "smtp_port": fields.email_port.parse::<u16>().unwrap_or(587),
                    "encryption": fields.email_encryption,
                    "smtp_username": fields.email_username,
                    "smtp_password": fields.email_password,
                    "from_email": fields.email_from
                })
            }
        }
        "ntfy" => json!({"server": fields.server, "topic": fields.topic, "token": fields.token}),
        "telegram" => json!({"bot_token": fields.bot_token, "chat_id": fields.chat_id}),
        "discord" => json!({"webhook_url": fields.webhook_url}),
        "gotify" => json!({"server_url": fields.gotify_server, "token": fields.gotify_token}),
        "webhook" => json!({"url": fields.webhook_url}),
        "slack" => json!({"webhook_url": fields.webhook_url}),
        "mattermost" => json!({"webhook_url": fields.webhook_url}),
        "pushover" => json!({"token": fields.token, "user_key": fields.pushover_user}),
        "pushplus" => json!({"token": fields.token}),
        "serverchan" => json!({"send_key": fields.serverchan_key}),
        _ => json!({}),
    }
}

#[component]
pub fn DeliveryCard() -> Element {
    let mut channels = use_signal(Vec::<NotificationChannel>::new);
    let mut selected = use_signal(|| "email".to_string());
    let mut enabled = use_signal(|| false);
    let mut server = use_signal(String::new);
    let mut topic = use_signal(String::new);
    let mut token = use_signal(String::new);
    let mut bot_token = use_signal(String::new);
    let mut chat_id = use_signal(String::new);
    let mut webhook_url = use_signal(String::new);
    let mut gotify_server = use_signal(String::new);
    let mut gotify_token = use_signal(String::new);
    let mut email_host = use_signal(String::new);
    let mut email_port = use_signal(|| "587".to_string());
    let mut email_encryption = use_signal(|| "tls".to_string());
    let mut email_username = use_signal(String::new);
    let mut email_password = use_signal(String::new);
    let mut email_from = use_signal(String::new);
    let mut pushover_user = use_signal(String::new);
    let mut serverchan_key = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut testing = use_signal(|| false);

    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            return;
        }
        match notification_channels::list().await {
            Ok(list) => {
                let email = list.iter().find(|c| c.provider == "email").cloned();
                channels.set(list);
                let config = email
                    .as_ref()
                    .map(|channel| channel.config.clone())
                    .unwrap_or_else(|| json!({}));
                enabled.set(email.as_ref().is_some_and(|channel| channel.enabled));
                server.set(field(&config, "server"));
                topic.set(field(&config, "topic"));
                token.set(field(&config, "token"));
                bot_token.set(field(&config, "bot_token"));
                chat_id.set(field(&config, "chat_id"));
                webhook_url.set(field(&config, "webhook_url"));
                gotify_server.set(field(&config, "server_url"));
                gotify_token.set(field(&config, "token"));
                email_host.set(field(&config, "smtp_host"));
                email_port.set(config.get("smtp_port").and_then(Value::as_u64).unwrap_or(587).to_string());
                email_encryption.set({ let value = field(&config, "encryption"); if value.is_empty() { "tls".to_string() } else { value } });
                email_username.set(field(&config, "smtp_username"));
                email_password.set(field(&config, "smtp_password"));
                email_from.set(field(&config, "from_email"));
                pushover_user.set(field(&config, "user_key"));
                serverchan_key.set(field(&config, "send_key"));
            }
            Err(error) => TOAST.write().error(format!(
                "Could not load delivery channels: {error} (API: {})",
                crate::api::client::get_base_url()
            )),
        }
    });

    let current_config = move || {
        build_config(
            &selected(),
            &ChannelFields {
                server: server(),
                topic: topic(),
                token: token(),
                bot_token: bot_token(),
                chat_id: chat_id(),
                webhook_url: webhook_url(),
                gotify_server: gotify_server(),
                gotify_token: gotify_token(),
                email_host: email_host(),
                email_port: email_port(),
                email_encryption: email_encryption(),
                email_username: email_username(),
                email_password: email_password(),
                email_from: email_from(),
                pushover_user: pushover_user(),
                serverchan_key: serverchan_key(),
            },
        )
    };

    let save = move |_| {
        saving.set(true);
        let provider = selected();
        let req = UpdateNotificationChannelRequest {
            enabled: enabled(),
            config: current_config(),
        };
        spawn(async move {
            match notification_channels::update(&provider, &req).await {
                Ok(channel) => {
                    let mut list = channels();
                    if let Some(item) = list.iter_mut().find(|item| item.provider == channel.provider) {
                        *item = channel;
                    }
                    channels.set(list);
                    TOAST.write().success("Delivery channel saved");
                }
                Err(error) => TOAST.write().error(format!(
                    "Could not save delivery channel: {error} (API: {})",
                    crate::api::client::get_base_url()
                )),
            }
            saving.set(false);
        });
    };

    let test = move |_| {
        testing.set(true);
        let provider = selected();
        let config = current_config();
        spawn(async move {
            match notification_channels::test(&provider, config).await {
                Ok(()) => TOAST.write().success("Test notification sent"),
                Err(error) => TOAST.write().error(format!(
                    "Test notification failed: {error} (API: {})",
                    crate::api::client::get_base_url()
                )),
            }
            testing.set(false);
        });
    };

    let label = PROVIDERS
        .iter()
        .find(|(key, _)| *key == selected())
        .map(|(_, label)| *label)
        .unwrap_or("Delivery");

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Delivery channels" }
                p { class: "text-sm text-muted-foreground", "One summary, several destinations. Secrets stay encrypted." }
            }
            div { class: "p-6 space-y-4",
                div { class: "flex flex-wrap items-center justify-between gap-3",
                    div { class: "flex items-center gap-2",
                        label { r#for: "delivery-provider", class: "text-sm font-medium", "Channel" }
                        select {
                            id: "delivery-provider",
                            value: selected(),
                            onchange: move |event| {
                                let provider = event.value();
                                let channel = channels().into_iter().find(|c| c.provider == provider);
                                let config = channel
                                    .as_ref()
                                    .map(|c| c.config.clone())
                                    .unwrap_or_else(|| json!({}));
                                enabled.set(channel.as_ref().is_some_and(|c| c.enabled));
                                server.set(field(&config, "server"));
                                topic.set(field(&config, "topic"));
                                token.set(field(&config, "token"));
                                bot_token.set(field(&config, "bot_token"));
                                chat_id.set(field(&config, "chat_id"));
                                webhook_url.set(field(&config, "webhook_url"));
                                gotify_server.set(field(&config, "server_url"));
                                gotify_token.set(field(&config, "token"));
                                email_host.set(field(&config, "smtp_host"));
                                email_port.set(config.get("smtp_port").and_then(Value::as_u64).unwrap_or(587).to_string());
                                email_encryption.set({ let value = field(&config, "encryption"); if value.is_empty() { "tls".to_string() } else { value } });
                                email_username.set(field(&config, "smtp_username"));
                                email_password.set(field(&config, "smtp_password"));
                                email_from.set(field(&config, "from_email"));
                                pushover_user.set(field(&config, "user_key"));
                                serverchan_key.set(field(&config, "send_key"));
                                selected.set(provider);
                            },
                            class: "h-9 rounded-md border border-input bg-background px-3 text-sm",
                            for (key, label) in PROVIDERS {
                                option { value: key.to_string(), {label.to_string()} }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: if enabled() { "relative inline-flex h-6 w-11 items-center rounded-full bg-primary transition-colors" } else { "relative inline-flex h-6 w-11 items-center rounded-full bg-muted transition-colors" },
                        onclick: move |_| enabled.toggle(),
                        span {
                            class: if enabled() { "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6" } else { "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1" }
                        }
                    }
                }

                if selected() == "email" {
                    p { class: "text-sm text-muted-foreground", "Leave the SMTP fields empty to use the server-wide SMTP configuration." }
                    div { class: "grid gap-3 sm:grid-cols-2",
                        label { class: "space-y-1 text-sm", "SMTP host"
                            input { value: email_host(), oninput: move |e| email_host.set(e.value()), placeholder: "smtp.example.com", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "SMTP port"
                            input { r#type: "number", value: email_port(), oninput: move |e| email_port.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "Encryption"
                            select { value: email_encryption(), onchange: move |e| email_encryption.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                                option { value: "tls", "TLS / STARTTLS" }
                                option { value: "ssl", "SSL" }
                                option { value: "none", "None" }
                            }
                        }
                        label { class: "space-y-1 text-sm", "From email"
                            input { value: email_from(), oninput: move |e| email_from.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "SMTP username"
                            input { value: email_username(), oninput: move |e| email_username.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "SMTP password"
                            input { r#type: "password", value: email_password(), oninput: move |e| email_password.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    }
                } else if selected() == "ntfy" {
                    div { class: "grid gap-3 sm:grid-cols-2",
                        label { class: "space-y-1 text-sm", "Server"
                            input { value: server(), oninput: move |e| server.set(e.value()), placeholder: "https://ntfy.sh", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "Topic"
                            input { value: topic(), oninput: move |e| topic.set(e.value()), placeholder: "recurrency", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    }
                    label { class: "space-y-1 text-sm block", "Access token (optional)"
                        input { r#type: "password", value: token(), oninput: move |e| token.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                    }
                } else if selected() == "telegram" {
                    div { class: "grid gap-3 sm:grid-cols-2",
                        label { class: "space-y-1 text-sm", "Bot token"
                            input { r#type: "password", value: bot_token(), oninput: move |e| bot_token.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "Chat ID"
                            input { value: chat_id(), oninput: move |e| chat_id.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    }
                } else if selected() == "discord" {
                    label { class: "space-y-1 text-sm block", "Webhook URL"
                        input { r#type: "password", value: webhook_url(), oninput: move |e| webhook_url.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                    }
                } else if selected() == "gotify" {
                    div { class: "grid gap-3 sm:grid-cols-2",
                        label { class: "space-y-1 text-sm", "Server URL"
                            input { value: gotify_server(), oninput: move |e| gotify_server.set(e.value()), placeholder: "https://gotify.example.com", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "App token"
                            input { r#type: "password", value: gotify_token(), oninput: move |e| gotify_token.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    }
                } else if selected() == "webhook" {
                    label { class: "space-y-1 text-sm block", "Webhook URL"
                        input { r#type: "password", value: webhook_url(), oninput: move |e| webhook_url.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                    }
                    p { class: "text-xs text-muted-foreground", "The webhook receives a JSON body with title and message." }
                } else if selected() == "slack" || selected() == "mattermost" {
                    label { class: "space-y-1 text-sm block", "Incoming webhook URL"
                        input { r#type: "password", value: webhook_url(), oninput: move |e| webhook_url.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                    }
                } else if selected() == "pushover" {
                    div { class: "grid gap-3 sm:grid-cols-2",
                        label { class: "space-y-1 text-sm", "Application token"
                            input { r#type: "password", value: token(), oninput: move |e| token.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        label { class: "space-y-1 text-sm", "User key"
                            input { value: pushover_user(), oninput: move |e| pushover_user.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    }
                } else if selected() == "pushplus" {
                    label { class: "space-y-1 text-sm block", "PushPlus token"
                        input { r#type: "password", value: token(), oninput: move |e| token.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                    }
                } else if selected() == "serverchan" {
                    label { class: "space-y-1 text-sm block", "SendKey"
                        input { r#type: "password", value: serverchan_key(), oninput: move |e| serverchan_key.set(e.value()), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                    }
                }

                div { class: "flex flex-wrap items-center gap-2",
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                        disabled: saving(),
                        onclick: save,
                        if saving() { "Saving..." } else { "Save {label}" }
                    }
                    button {
                        r#type: "button",
                        class: "inline-flex items-center justify-center rounded-md border border-border px-4 py-2 text-sm font-medium hover:bg-accent disabled:opacity-50",
                        disabled: testing(),
                        onclick: test,
                        if testing() { "Testing..." } else { "Send test" }
                    }
                }
            }
        }
    }
}
