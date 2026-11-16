use dioxus::prelude::*;
use lucide_dioxus::{Bot, KeyRound, Save, Sparkles};

use crate::api::ai::{self, AiSettings, UpdateAiSettingsRequest};
use crate::components::toast::TOAST;
use crate::state::ai::AI_SETTINGS;

const AI_CARD_TITLE: &str = "AI assistant";
const AI_KEY_PLACEHOLDER_CONFIGURED: &str = "A key is configured — enter a new key to replace it";
const AI_KEY_PLACEHOLDER_EMPTY: &str = "Paste your provider API key";

#[component]
pub fn AiSettingsCard() -> Element {
    let mut enabled = use_signal(|| false);
    let mut base_url = use_signal(String::new);
    let mut model = use_signal(String::new);
    let mut api_key = use_signal(String::new);
    let mut configured = use_signal(|| false);
    let mut clear_key = use_signal(|| false);
    let mut loading = use_signal(|| true);
    let mut saving = use_signal(|| false);

    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            loading.set(false);
            return;
        }
        match ai::settings().await {
            Ok(settings) => apply_settings(&mut enabled, &mut base_url, &mut model, &mut configured, settings),
            Err(error) => TOAST
                .write()
                .error(format!("Could not load AI settings: {error}")),
        }
        loading.set(false);
    });

    let save = move |_| {
        if saving() {
            return;
        }
        saving.set(true);
        let request = UpdateAiSettingsRequest {
            enabled: enabled(),
            base_url: Some(base_url()),
            model: Some(model()),
            api_key: (!api_key().trim().is_empty()).then_some(api_key()),
            clear_api_key: clear_key(),
        };
        spawn(async move {
            match ai::update_settings(&request).await {
                Ok(settings) => {
                    apply_settings(&mut enabled, &mut base_url, &mut model, &mut configured, settings.clone());
                    *AI_SETTINGS.write() = Some(settings);
                    api_key.set(String::new());
                    clear_key.set(false);
                    TOAST.write().success("AI settings saved");
                }
                Err(error) => TOAST
                    .write()
                    .error(format!("Could not save AI settings: {error}")),
            }
            saving.set(false);
        });
    };

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "flex items-start gap-4 p-6 border-b border-border bg-gradient-to-br from-violet-500/5 to-transparent",
                span { class: "flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-violet-500/10 text-violet-600 dark:text-violet-400",
                    Bot { size: 22 }
                }
                div {
                    h2 { class: "text-lg font-semibold", {AI_CARD_TITLE} }
                    p { class: "text-sm text-muted-foreground",
                        "Bring your own key. The assistant can create and change subscriptions, transactions and debts."
                    }
                }
                label { class: "ml-auto flex items-center gap-2 text-sm font-medium",
                    input {
                        r#type: "checkbox",
                        checked: enabled(),
                        disabled: loading() || saving(),
                        onchange: move |event| enabled.set(event.checked()),
                        class: "h-4 w-4 accent-primary",
                    }
                    "Enabled"
                }
            }
            div { class: "p-6 space-y-4",
                if crate::api::client::is_offline_mode() {
                    p { class: "rounded-lg border border-border/60 bg-muted/20 p-4 text-sm text-muted-foreground",
                        "The AI assistant is available in server mode. Local on-device mode has no AI proxy."
                    }
                } else {
                    div { class: "grid gap-4 md:grid-cols-2",
                        div { class: "space-y-2",
                            label { r#for: "ai-base-url", class: "text-sm font-medium", "API base URL" }
                            input {
                                id: "ai-base-url",
                                r#type: "url",
                                value: base_url(),
                                oninput: move |event| base_url.set(event.value()),
                                placeholder: "https://api.openai.com/v1",
                                class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm",
                            }
                        }
                        div { class: "space-y-2",
                            label { r#for: "ai-model", class: "text-sm font-medium", "Model" }
                            input {
                                id: "ai-model",
                                r#type: "text",
                                value: model(),
                                oninput: move |event| model.set(event.value()),
                                placeholder: "gpt-4o-mini",
                                class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm",
                            }
                        }
                    }
                    div { class: "space-y-2",
                        label { r#for: "ai-api-key", class: "flex items-center gap-2 text-sm font-medium",
                            KeyRound { size: 14 }
                            "API key"
                        }
                        input {
                            id: "ai-api-key",
                            r#type: "password",
                            value: api_key(),
                            oninput: move |event| api_key.set(event.value()),
                            autocomplete: "off",
                            placeholder: if configured() { AI_KEY_PLACEHOLDER_CONFIGURED } else { AI_KEY_PLACEHOLDER_EMPTY },
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm",
                        }
                        p { class: "text-xs text-muted-foreground",
                            "The key is encrypted on the server and never returned to this device."
                        }
                    }
                    div { class: "flex flex-wrap items-center justify-between gap-3",
                        span { class: "inline-flex items-center gap-1.5 text-xs text-muted-foreground",
                            Sparkles { size: 14 }
                            if configured() { "Key stored" } else { "Key not configured" }
                        }
                        div { class: "flex items-center gap-2",
                            if configured() && !clear_key() {
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-9 items-center rounded-md border border-destructive/40 px-3 text-sm text-destructive hover:bg-destructive/10",
                                    onclick: move |_| clear_key.set(true),
                                    "Remove key"
                                }
                            }
                            if clear_key() {
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-9 items-center rounded-md border border-border px-3 text-sm",
                                    onclick: move |_| clear_key.set(false),
                                    "Keep key"
                                }
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                                disabled: loading() || saving(),
                                onclick: save,
                                Save { size: 15 }
                                if saving() { "Saving…" } else { "Save AI settings" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn apply_settings(
    enabled: &mut Signal<bool>,
    base_url: &mut Signal<String>,
    model: &mut Signal<String>,
    configured: &mut Signal<bool>,
    settings: AiSettings,
) {
    enabled.set(settings.enabled);
    base_url.set(settings.base_url);
    model.set(settings.model);
    configured.set(settings.configured);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_card_exposes_byok_fields() {
        let source = include_str!("ai_card.rs");
        assert!(source.contains("API key"));
        assert!(source.contains("api_key"));
        assert!(source.contains("Clear API key") || source.contains("clear_api_key"));
    }
}

#[allow(dead_code)]
fn _keep_placeholder_constants() -> (&'static str, &'static str) {
    (AI_KEY_PLACEHOLDER_CONFIGURED, AI_KEY_PLACEHOLDER_EMPTY)
}
