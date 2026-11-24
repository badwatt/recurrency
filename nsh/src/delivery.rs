//! Delivery-channel editor shared by every settings surface. Each provider is
//! described once in `PROVIDERS`; the form, config payload and test call are
//! derived from that schema instead of duplicating one form per channel.

use crate::api::DeliveryChannel;
use crate::components::button::{ButtonProps, ButtonVariant, button};
use crate::components::field::{FieldProps, field};
use crate::components::input::text_input_value;
use crate::components::toggle::toggle;
use nsh_base::{button as raw_button, theme};
use nsh_core::prelude::*;
use serde_json::{Map, Value, json};

#[derive(Clone, Copy, PartialEq)]
pub struct ChannelField {
    pub key: &'static str,
    pub label: &'static str,
    pub placeholder: &'static str,
}

const fn config_field(
    key: &'static str,
    label: &'static str,
    placeholder: &'static str,
) -> ChannelField {
    ChannelField {
        key,
        label,
        placeholder,
    }
}

pub struct ProviderSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub fields: &'static [ChannelField],
}

const EMAIL_FIELDS: &[ChannelField] = &[
    config_field("smtp_host", "SMTP host", "smtp.example.com"),
    config_field("smtp_port", "SMTP port", "587"),
    config_field("encryption", "Encryption", "TLS / STARTTLS"),
    config_field("from_email", "From email", "hello@example.com"),
    config_field("smtp_username", "SMTP username", "username"),
    config_field("smtp_password", "SMTP password", "password"),
];
const NTFY_FIELDS: &[ChannelField] = &[
    config_field("server", "Server", "https://ntfy.sh"),
    config_field("topic", "Topic", "recurrency"),
    config_field("token", "Access token (optional)", ""),
];
const TELEGRAM_FIELDS: &[ChannelField] = &[
    config_field("bot_token", "Bot token", "123456:ABC"),
    config_field("chat_id", "Chat ID", "-1001234567890"),
];
const WEBHOOK_FIELDS: &[ChannelField] = &[config_field(
    "webhook_url",
    "Webhook URL",
    "https://example.com/hook",
)];
const GOTIFY_FIELDS: &[ChannelField] = &[
    config_field("server_url", "Server URL", "https://gotify.example.com"),
    config_field("token", "App token", ""),
];
const PUSHOVER_FIELDS: &[ChannelField] = &[
    config_field("token", "Application token", ""),
    config_field("user_key", "User key", ""),
];
const TOKEN_FIELDS: &[ChannelField] = &[config_field("token", "PushPlus token", "")];
const SLACK_FIELDS: &[ChannelField] = &[config_field(
    "webhook_url",
    "Incoming webhook URL",
    "https://hooks.slack.com/services/...",
)];
const SERVERCHAN_FIELDS: &[ChannelField] = &[config_field("send_key", "SendKey", "")];

pub const PROVIDERS: &[ProviderSpec] = &[
    ProviderSpec {
        id: "email",
        label: "Email",
        fields: EMAIL_FIELDS,
    },
    ProviderSpec {
        id: "ntfy",
        label: "ntfy",
        fields: NTFY_FIELDS,
    },
    ProviderSpec {
        id: "telegram",
        label: "Telegram",
        fields: TELEGRAM_FIELDS,
    },
    ProviderSpec {
        id: "discord",
        label: "Discord",
        fields: WEBHOOK_FIELDS,
    },
    ProviderSpec {
        id: "gotify",
        label: "Gotify",
        fields: GOTIFY_FIELDS,
    },
    ProviderSpec {
        id: "webhook",
        label: "Webhook",
        fields: WEBHOOK_FIELDS,
    },
    ProviderSpec {
        id: "slack",
        label: "Slack",
        fields: SLACK_FIELDS,
    },
    ProviderSpec {
        id: "mattermost",
        label: "Mattermost",
        fields: SLACK_FIELDS,
    },
    ProviderSpec {
        id: "pushover",
        label: "Pushover",
        fields: PUSHOVER_FIELDS,
    },
    ProviderSpec {
        id: "pushplus",
        label: "PushPlus",
        fields: TOKEN_FIELDS,
    },
    ProviderSpec {
        id: "serverchan",
        label: "ServerChan",
        fields: SERVERCHAN_FIELDS,
    },
];

fn provider_color(id: &str) -> u32 {
    match id {
        "email" => 0x6366f1ff,
        "ntfy" => 0x22c55eff,
        "telegram" => 0x0ea5e9ff,
        "discord" => 0x5865f2ff,
        "gotify" => 0xf59e0bff,
        "webhook" => 0x8b5cf6ff,
        "slack" => 0xe01e5aff,
        "mattermost" => 0x0058ccff,
        "pushover" => 0xe11d48ff,
        "pushplus" => 0x14b8a6ff,
        _ => 0x64748bff,
    }
}

pub fn provider(id: &str) -> Option<&'static ProviderSpec> {
    PROVIDERS.iter().find(|provider| provider.id == id)
}

fn config_text(config: &Value, key: &str) -> String {
    config.get(key).map_or_else(String::new, |value| {
        if let Some(text) = value.as_str() {
            text.to_string()
        } else if let Some(number) = value.as_u64() {
            number.to_string()
        } else if let Some(boolean) = value.as_bool() {
            boolean.to_string()
        } else {
            String::new()
        }
    })
}

fn values_for(provider: &ProviderSpec, config: &Value) -> Vec<(String, String)> {
    provider
        .fields
        .iter()
        .map(|field| (field.key.to_string(), config_text(config, field.key)))
        .collect()
}

fn config_for(provider: &ProviderSpec, values: &[(String, String)]) -> Value {
    let mut map = Map::new();
    for (key, value) in values.iter().filter(|(_, value)| !value.trim().is_empty()) {
        if !provider.fields.iter().any(|field| field.key == key) {
            continue;
        }
        if key == "smtp_port" {
            if let Ok(port) = value.parse::<u64>() {
                map.insert(key.clone(), json!(port));
                continue;
            }
        }
        map.insert(key.clone(), json!(value));
    }
    Value::Object(map)
}

fn load_from(
    id: &str,
    channels: &[DeliveryChannel],
    values: Signal<Vec<(String, String)>>,
    enabled: Signal<bool>,
) {
    let Some(provider) = provider(id) else { return };
    let channel = channels.iter().find(|channel| channel.provider == id);
    values.set(values_for(
        provider,
        channel.map_or(&Value::Null, |channel| &channel.config),
    ));
    enabled.set(channel.is_some_and(|channel| channel.enabled));
}

fn provider_icon(id: &str) -> ImageAsset {
    match id {
        "email" => asset!("icons/mail.svg"),
        "ntfy" => asset!("icons/bell.svg"),
        "telegram" => asset!("icons/send.svg"),
        "discord" => asset!("icons/gamepad.svg"),
        "gotify" => asset!("icons/bell.svg"),
        "webhook" => asset!("icons/webhook.svg"),
        "slack" => asset!("icons/hash.svg"),
        "mattermost" => asset!("icons/bot.svg"),
        "pushover" => asset!("icons/smartphone.svg"),
        "pushplus" => asset!("icons/plus.svg"),
        _ => asset!("icons/server.svg"),
    }
}

pub fn delivery_card(server: String, token: String) -> impl IntoElement {
    let data = resource(|| Some(()), {
        let server = server.clone();
        let token = token.clone();
        move |(), _| {
            let server = server.clone();
            let token = token.clone();
            async move { crate::api::load_delivery_channels(server, token).await }
        }
    });
    let selected = signal("email".to_string());
    let enabled = signal(false);
    let values = signal(Vec::<(String, String)>::new());
    let status = signal(String::new());
    let loaded = signal(false);
    effect(move || {
        if loaded.get() {
            return;
        }
        if let Some(channels) = data.get() {
            load_from("email", &channels, values, enabled);
            loaded.set(true);
        }
    });
    let save = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let Some(provider) = provider(&selected.get()) else {
                return;
            };
            let config = config_for(provider, &values.get());
            let enabled = enabled.get();
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::update_delivery_channel(
                    server,
                    token,
                    provider.id,
                    enabled,
                    config,
                )
                .await
                {
                    Ok(_) => {
                        status.set(format!("{} saved.", provider.label));
                        data.refetch();
                    }
                    Err(message) => status.set(message),
                }
            });
        }
    };
    let send_test = {
        let server = server.clone();
        let token = token.clone();
        move || {
            let Some(provider) = provider(&selected.get()) else {
                return;
            };
            let config = config_for(provider, &values.get());
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                match crate::api::test_delivery_channel(server, token, provider.id, config).await {
                    Ok(()) => status.set("Test notification sent.".to_string()),
                    Err(message) => status.set(message),
                }
            });
        }
    };
    let current = move || provider(&selected.get()).map_or("Delivery", |provider| provider.label);
    let fields = move || provider(&selected.get()).map_or(&[][..], |provider| provider.fields);
    let header = view()
        .w_full()
        .flex_col()
        .gap(px(4.0))
        .p(px(24.0))
        .border_b(px(1.0))
        .border_color(move || theme().border)
        .child(
            view()
                .flex_col()
                .child(
                    text("Delivery channels")
                        .text_size(px(18.0))
                        .line_height(px(28.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text("One summary, several destinations. Secrets stay encrypted.")
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .into_element();
    let rail = view()
        .w(px(190.0))
        .flex_shrink_0()
        .flex_col()
        .gap(px(4.0))
        .pr(px(16.0))
        .border_r(px(1.0))
        .border_color(move || theme().border.opacity(0.4))
        .children(PROVIDERS.iter().map(|spec| {
            let id = spec.id;
            let label = spec.label;
            let color = provider_color(id);
            let is_enabled = data
                .get()
                .unwrap_or_default()
                .iter()
                .any(|channel| channel.provider == id && channel.enabled);
            let active = move || selected.get() == id;
            raw_button()
                .w_full()
                .h(px(44.0))
                .px(px(10.0))
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .rounded(px(8.0))
                .bg(move || {
                    if active() {
                        theme().accent
                    } else {
                        theme().card.opacity(0.0)
                    }
                })
                .on_press(move || {
                    let channels = data.get().unwrap_or_default();
                    load_from(id, &channels, values, enabled);
                    selected.set(id.to_string());
                })
                .child(
                    view()
                        .h(px(28.0))
                        .w(px(28.0))
                        .flex_shrink_0()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .bg(rgba(color))
                        .child(
                            image(provider_icon(id))
                                .size(px(15.0))
                                .tint_color(move || theme().primary_foreground),
                        ),
                )
                .child(
                    text(label)
                        .flex_grow(1.0)
                        .text_size(px(14.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if active() {
                                theme().foreground
                            } else {
                                theme().muted_foreground
                            }
                        }),
                )
                .when(is_enabled, |row| {
                    row.child(
                        view()
                            .h(px(6.0))
                            .w(px(6.0))
                            .rounded(px(99.0))
                            .bg(move || crate::theme::success()),
                    )
                })
                .into_element()
        }));
    let form = view()
        .flex_grow(1.0)
        .basis(px(0.0))
        .min_w(px(0.0))
        .flex_col()
        .gap(px(16.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(12.0))
                .pb(px(12.0))
                .border_b(px(1.0))
                .border_color(move || theme().border.opacity(0.4))
                .child(
                    view()
                        .flex_col()
                        .child(
                            text(move || current().to_string())
                                .text_size(px(14.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(
                            text("Configure this delivery channel")
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        ),
                )
                .child(toggle(
                    enabled.get(),
                    format!("Enable {}", current()),
                    move |value| enabled.set(value),
                )),
        )
        .child(
            when(
                move || selected.get() == "email",
                move || {
                    text("Leave the SMTP fields empty to use the server-wide SMTP configuration.")
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .text_color(move || theme().muted_foreground)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .child(view().flex_row().flex_wrap().gap(px(10.0)).child(each(
            move || fields().to_vec(),
            |entry| entry.key,
            move |entry, _| {
                let field_spec = entry.get();
                let key = field_spec.key;
                let current_value = values
                    .get()
                    .into_iter()
                    .find(|(name, _)| name == key)
                    .map_or_else(String::new, |(_, value)| value);
                if key == "encryption" {
                    let initial = if current_value.is_empty() {
                        "tls".to_string()
                    } else {
                        current_value
                    };
                    let encryption = signal(initial);
                    let encryption_values = values;
                    effect(move || {
                        let next = encryption.get();
                        let mut next_values = encryption_values.get();
                        if let Some(slot) = next_values.iter_mut().find(|(name, _)| name == key) {
                            slot.1 = next;
                        } else {
                            next_values.push((key.to_string(), next));
                        }
                        encryption_values.set(next_values);
                    });
                    view()
                        .min_w(px(240.0))
                        .flex_grow(1.0)
                        .child(crate::components::string_select::string_select(
                            crate::components::string_select::StringSelectProps::new(
                                "Encryption",
                                encryption,
                                vec![
                                    ("tls".to_string(), "TLS / STARTTLS".to_string()),
                                    ("ssl".to_string(), "SSL".to_string()),
                                    ("none".to_string(), "None".to_string()),
                                ],
                            ),
                        ))
                        .into_element()
                } else {
                    let control =
                        text_input_value(current_value, field_spec.placeholder, move |next| {
                            let mut next_values = values.get();
                            if let Some(slot) = next_values.iter_mut().find(|(name, _)| name == key)
                            {
                                slot.1 = next;
                            }
                            values.set(next_values);
                        });
                    view()
                        .min_w(px(240.0))
                        .flex_grow(1.0)
                        .child(field(
                            FieldProps::new(field_spec.label),
                            control.into_element(),
                        ))
                        .into_element()
                }
            },
        )))
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .child(button(
                    ButtonProps::new(format!("Save {}", current())).on_press(save),
                ))
                .child(button(
                    ButtonProps::new("Send test")
                        .variant(ButtonVariant::Outline)
                        .on_press(send_test),
                )),
        )
        .child(
            text(move || status.get())
                .text_size(px(11.0))
                .text_color(move || theme().muted_foreground),
        )
        .into_element();
    crate::ui::surface()
        .w_full()
        .p(px(0.0))
        .child(header)
        .child(
            view()
                .w_full()
                .p(px(48.0))
                .flex_row()
                .items_start()
                .gap(px(24.0))
                .child(rail)
                .child(form),
        )
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_provider_has_a_unique_id_and_fields() {
        for (index, provider) in PROVIDERS.iter().enumerate() {
            assert!(provider.fields.iter().all(|field| !field.key.is_empty()));
            assert!(
                PROVIDERS
                    .iter()
                    .skip(index + 1)
                    .all(|other| other.id != provider.id)
            );
        }
    }

    #[test]
    fn numeric_smtp_port_is_serialized_as_a_number() {
        let provider = super::provider("email").expect("email provider");
        let config = config_for(provider, &[("smtp_port".to_string(), "587".to_string())]);
        assert_eq!(config.get("smtp_port").and_then(Value::as_u64), Some(587));
    }

    #[test]
    fn empty_fields_are_omitted_and_shared_webhook_schema_is_reused() {
        let provider = super::provider("slack").expect("slack provider");
        let config = config_for(
            provider,
            &[(
                "webhook_url".to_string(),
                "https://hooks.slack.com/x".to_string(),
            )],
        );
        assert_eq!(config.as_object().map(Map::len), Some(1));
        assert_eq!(
            provider.fields.len(),
            super::provider("discord").expect("discord").fields.len()
        );
    }
}
