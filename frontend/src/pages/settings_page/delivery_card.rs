use crate::api::notification_channels;
use crate::components::toast::TOAST;
use crate::models::notification::{
    NotificationChannel, UpdateNotificationChannelRequest,
};
use dioxus::prelude::*;
use lucide_dioxus::{Bell, Gamepad2, Hash, Mail, BotMessageSquare, Plus, Send, Server, Smartphone, Webhook};
use serde_json::{Value, json};

#[derive(Clone, Copy)]
struct Provider {
    id: &'static str,
    label: &'static str,
    color: &'static str,
}

const PROVIDERS: &[Provider] = &[
    Provider { id: "email", label: "Email", color: "#6366f1" },
    Provider { id: "ntfy", label: "ntfy", color: "#22c55e" },
    Provider { id: "telegram", label: "Telegram", color: "#0ea5e9" },
    Provider { id: "discord", label: "Discord", color: "#5865f2" },
    Provider { id: "gotify", label: "Gotify", color: "#f59e0b" },
    Provider { id: "webhook", label: "Webhook", color: "#8b5cf6" },
    Provider { id: "slack", label: "Slack", color: "#e01e5a" },
    Provider { id: "mattermost", label: "Mattermost", color: "#0058cc" },
    Provider { id: "pushover", label: "Pushover", color: "#e11d48" },
    Provider { id: "pushplus", label: "PushPlus", color: "#14b8a6" },
    Provider { id: "serverchan", label: "ServerChan", color: "#64748b" },
];

fn field(config: &Value, key: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

#[derive(Clone)]
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

impl Default for ChannelFields {
    fn default() -> Self {
        Self {
            server: String::new(),
            topic: String::new(),
            token: String::new(),
            bot_token: String::new(),
            chat_id: String::new(),
            webhook_url: String::new(),
            gotify_server: String::new(),
            gotify_token: String::new(),
            email_host: String::new(),
            email_port: "587".to_string(),
            email_encryption: "tls".to_string(),
            email_username: String::new(),
            email_password: String::new(),
            email_from: String::new(),
            pushover_user: String::new(),
            serverchan_key: String::new(),
        }
    }
}

impl ChannelFields {
    fn from_config(config: &Value) -> Self {
        let encryption = field(config, "encryption");
        Self {
            server: field(config, "server"),
            topic: field(config, "topic"),
            token: field(config, "token"),
            bot_token: field(config, "bot_token"),
            chat_id: field(config, "chat_id"),
            webhook_url: field(config, "webhook_url"),
            gotify_server: field(config, "server_url"),
            gotify_token: field(config, "token"),
            email_host: field(config, "smtp_host"),
            email_port: config
                .get("smtp_port")
                .and_then(Value::as_u64)
                .unwrap_or(587)
                .to_string(),
            email_encryption: if encryption.is_empty() {
                "tls".to_string()
            } else {
                encryption
            },
            email_username: field(config, "smtp_username"),
            email_password: field(config, "smtp_password"),
            email_from: field(config, "from_email"),
            pushover_user: field(config, "user_key"),
            serverchan_key: field(config, "send_key"),
        }
    }

    fn to_config(&self, provider: &str) -> Value {
        match provider {
            "email" => {
                if self.email_host.is_empty() {
                    json!({})
                } else {
                    json!({
                        "smtp_host": self.email_host,
                        "smtp_port": self.email_port.parse::<u16>().unwrap_or(587),
                        "encryption": self.email_encryption,
                        "smtp_username": self.email_username,
                        "smtp_password": self.email_password,
                        "from_email": self.email_from
                    })
                }
            }
            "ntfy" => json!({"server": self.server, "topic": self.topic, "token": self.token}),
            "telegram" => json!({"bot_token": self.bot_token, "chat_id": self.chat_id}),
            "discord" => json!({"webhook_url": self.webhook_url}),
            "gotify" => json!({"server_url": self.gotify_server, "token": self.gotify_token}),
            "webhook" => json!({"url": self.webhook_url}),
            "slack" => json!({"webhook_url": self.webhook_url}),
            "mattermost" => json!({"webhook_url": self.webhook_url}),
            "pushover" => json!({"token": self.token, "user_key": self.pushover_user}),
            "pushplus" => json!({"token": self.token}),
            "serverchan" => json!({"send_key": self.serverchan_key}),
            _ => json!({}),
        }
    }
}

fn provider_icon(provider: &str) -> Element {
    match provider {
        "email" => rsx! { Mail { size: 15, color: "white".to_string() } },
        "ntfy" => rsx! { Bell { size: 15, color: "white".to_string() } },
        "telegram" => rsx! { Send { size: 15, color: "white".to_string() } },
        "discord" => rsx! { Gamepad2 { size: 15, color: "white".to_string() } },
        "gotify" => rsx! { Bell { size: 15, color: "white".to_string() } },
        "webhook" => rsx! { Webhook { size: 15, color: "white".to_string() } },
        "slack" => rsx! { Hash { size: 15, color: "white".to_string() } },
        "mattermost" => rsx! { BotMessageSquare { size: 15, color: "white".to_string() } },
        "pushover" => rsx! { Smartphone { size: 15, color: "white".to_string() } },
        "pushplus" => rsx! { Plus { size: 15, color: "white".to_string() } },
        "serverchan" => rsx! { Server { size: 15, color: "white".to_string() } },
        _ => rsx! { Send { size: 15, color: "white".to_string() } },
    }
}

#[component]
pub fn DeliveryCard() -> Element {
    let mut channels = use_signal(Vec::<NotificationChannel>::new);
    let mut selected = use_signal(|| "email".to_string());
    let mut enabled = use_signal(|| false);
    let mut form = use_signal(ChannelFields::default);
    let mut saving = use_signal(|| false);
    let mut testing = use_signal(|| false);

    // Load the selected provider's config into the form without going through
    // a select input. The rail calls this with the clicked provider.
    let load = Callback::new(move |(provider, list): (String, Vec<NotificationChannel>)| {
        let channel = list.iter().find(|channel| channel.provider == provider);
        let config = channel
            .map(|channel| channel.config.clone())
            .unwrap_or_else(|| json!({}));
        *enabled.write() = channel.is_some_and(|channel| channel.enabled);
        *form.write() = ChannelFields::from_config(&config);
        *selected.write() = provider;
    });

    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            return;
        }
        match notification_channels::list().await {
            Ok(list) => {
                *channels.write() = list.clone();
                load.call(("email".to_string(), list));
            }
            Err(error) => TOAST.write().error(format!(
                "Could not load delivery channels: {error} (API: {})",
                crate::api::client::get_base_url()
            )),
        }
    });

    let current_config = move || form().to_config(&selected());

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
        .find(|provider| provider.id == selected())
        .map(|provider| provider.label)
        .unwrap_or("Delivery");

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Delivery channels" }
                p { class: "text-sm text-muted-foreground", "One summary, several destinations. Secrets stay encrypted." }
            }
            div { class: "p-4 sm:p-6 grid gap-5 lg:grid-cols-[190px_minmax(0,1fr)] lg:gap-6",
                div {
                    class: "flex gap-1 overflow-x-auto pb-1 lg:flex-col lg:gap-1 lg:overflow-visible lg:border-r lg:border-border/40 lg:pr-4",
                    role: "tablist",
                    aria_label: "Delivery channels",
                    for provider in PROVIDERS {
                        {
                            let active = selected() == provider.id;
                            let is_enabled = channels()
                                .iter()
                                .find(|channel| channel.provider == provider.id)
                                .is_some_and(|channel| channel.enabled);
                            let class = if active {
                                "flex shrink-0 items-center gap-2.5 rounded-lg bg-accent px-2.5 py-2 text-left text-sm font-medium text-accent-foreground lg:w-full"
                            } else {
                                "flex shrink-0 items-center gap-2.5 rounded-lg px-2.5 py-2 text-left text-sm font-medium text-muted-foreground hover:bg-accent/50 hover:text-foreground lg:w-full"
                            };
                            rsx! {
                                button {
                                    r#type: "button",
                                    role: "tab",
                                    aria_selected: if active { "true" } else { "false" },
                                    class,
                                    onclick: move |_| load.call((provider.id.to_string(), channels())),
                                    span {
                                        class: "flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-[11px] font-bold text-white",
                                        style: "background-color: {provider.color}",
                                        {provider_icon(provider.id)}
                                    }
                                    span { class: "whitespace-nowrap", {provider.label} }
                                    if is_enabled {
                                        span { class: "ml-auto h-1.5 w-1.5 shrink-0 rounded-full bg-emerald-400", title: "Enabled" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "min-w-0 space-y-4",
                    div { class: "flex items-center justify-between gap-3 border-b border-border/40 pb-3",
                        div {
                            h3 { class: "text-sm font-semibold", {label} }
                            p { class: "text-xs text-muted-foreground", "Configure this delivery channel" }
                        }
                        button {
                            r#type: "button",
                            aria_label: if enabled() { "Disable {label}" } else { "Enable {label}" },
                            aria_pressed: if enabled() { "true" } else { "false" },
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
                                input { value: form().email_host, oninput: move |e| form.write().email_host = e.value(), placeholder: "smtp.example.com", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "SMTP port"
                                input { r#type: "number", value: form().email_port, oninput: move |e| form.write().email_port = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "Encryption"
                                select { value: form().email_encryption, onchange: move |e| form.write().email_encryption = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm",
                                    option { value: "tls", "TLS / STARTTLS" }
                                    option { value: "ssl", "SSL" }
                                    option { value: "none", "None" }
                                }
                            }
                            label { class: "space-y-1 text-sm", "From email"
                                input { value: form().email_from, oninput: move |e| form.write().email_from = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "SMTP username"
                                input { value: form().email_username, oninput: move |e| form.write().email_username = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "SMTP password"
                                input { r#type: "password", value: form().email_password, oninput: move |e| form.write().email_password = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                        }
                    } else if selected() == "ntfy" {
                        div { class: "grid gap-3 sm:grid-cols-2",
                            label { class: "space-y-1 text-sm", "Server"
                                input { value: form().server, oninput: move |e| form.write().server = e.value(), placeholder: "https://ntfy.sh", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "Topic"
                                input { value: form().topic, oninput: move |e| form.write().topic = e.value(), placeholder: "recurrency", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                        }
                        label { class: "space-y-1 text-sm block", "Access token (optional)"
                            input { r#type: "password", value: form().token, oninput: move |e| form.write().token = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    } else if selected() == "telegram" {
                        div { class: "grid gap-3 sm:grid-cols-2",
                            label { class: "space-y-1 text-sm", "Bot token"
                                input { r#type: "password", value: form().bot_token, oninput: move |e| form.write().bot_token = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "Chat ID"
                                input { value: form().chat_id, oninput: move |e| form.write().chat_id = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                        }
                    } else if selected() == "discord" {
                        label { class: "space-y-1 text-sm block", "Webhook URL"
                            input { r#type: "password", value: form().webhook_url, oninput: move |e| form.write().webhook_url = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    } else if selected() == "gotify" {
                        div { class: "grid gap-3 sm:grid-cols-2",
                            label { class: "space-y-1 text-sm", "Server URL"
                                input { value: form().gotify_server, oninput: move |e| form.write().gotify_server = e.value(), placeholder: "https://gotify.example.com", class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "App token"
                                input { r#type: "password", value: form().gotify_token, oninput: move |e| form.write().gotify_token = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                        }
                    } else if selected() == "webhook" {
                        label { class: "space-y-1 text-sm block", "Webhook URL"
                            input { r#type: "password", value: form().webhook_url, oninput: move |e| form.write().webhook_url = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                        p { class: "text-xs text-muted-foreground", "The webhook receives a JSON body with title and message." }
                    } else if selected() == "slack" || selected() == "mattermost" {
                        label { class: "space-y-1 text-sm block", "Incoming webhook URL"
                            input { r#type: "password", value: form().webhook_url, oninput: move |e| form.write().webhook_url = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    } else if selected() == "pushover" {
                        div { class: "grid gap-3 sm:grid-cols-2",
                            label { class: "space-y-1 text-sm", "Application token"
                                input { r#type: "password", value: form().token, oninput: move |e| form.write().token = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                            label { class: "space-y-1 text-sm", "User key"
                                input { value: form().pushover_user, oninput: move |e| form.write().pushover_user = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                            }
                        }
                    } else if selected() == "pushplus" {
                        label { class: "space-y-1 text-sm block", "PushPlus token"
                            input { r#type: "password", value: form().token, oninput: move |e| form.write().token = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    } else if selected() == "serverchan" {
                        label { class: "space-y-1 text-sm block", "SendKey"
                            input { r#type: "password", value: form().serverchan_key, oninput: move |e| form.write().serverchan_key = e.value(), class: "mt-1 flex h-9 w-full rounded-md border border-input bg-background px-3 text-sm" }
                        }
                    }

                    div { class: "flex flex-wrap items-center gap-2 pt-1",
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_are_unique_and_have_icons() {
        let mut ids = std::collections::HashSet::new();
        for provider in PROVIDERS {
            assert!(ids.insert(provider.id), "duplicate provider {}", provider.id);
            assert!(!provider.label.is_empty());
            assert!(provider.color.starts_with('#'));
            let _ = provider_icon(provider.id);
        }
        assert_eq!(PROVIDERS.len(), 11);
    }

    #[test]
    fn default_email_form_uses_tls_and_587() {
        let form = ChannelFields::default();
        assert_eq!(form.email_port, "587");
        assert_eq!(form.email_encryption, "tls");
    }

    #[test]
    fn config_roundtrip_for_telegram() {
        let fields = ChannelFields {
            bot_token: "bot".into(),
            chat_id: "123".into(),
            ..ChannelFields::default()
        };
        let config = fields.to_config("telegram");
        let loaded = ChannelFields::from_config(&config);
        assert_eq!(loaded.bot_token, "bot");
        assert_eq!(loaded.chat_id, "123");
    }
}
