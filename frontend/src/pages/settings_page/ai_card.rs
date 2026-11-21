use dioxus::prelude::*;
use lucide_dioxus::{Bot, CircleCheck, KeyRound, RefreshCw, Save, ShieldCheck, Sparkles};

use crate::api::ai::{self, AiSettings, UpdateAiSettingsRequest};
use crate::components::toast::TOAST;
use crate::state::ai::AI_SETTINGS;

pub const CUSTOM_MODEL_VALUE: &str = "__custom__";
pub const DEFAULT_PROVIDER_ID: &str = "openai";
pub const CUSTOM_PROVIDER_ID: &str = "custom";

const AI_KEY_PLACEHOLDER_CONFIGURED: &str = "A key is configured — enter a new key to replace it";
const AI_KEY_PLACEHOLDER_EMPTY: &str = "Paste your provider API key";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub base_url: &'static str,
    pub hint: &'static str,
}

pub const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "openai",
        label: "OpenAI",
        base_url: "https://api.openai.com/v1",
        hint: "GPT models with your own key.",
    },
    ProviderPreset {
        id: "groq",
        label: "Groq",
        base_url: "https://api.groq.com/openai/v1",
        hint: "Fast hosted inference.",
    },
    ProviderPreset {
        id: "openrouter",
        label: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        hint: "One key for many model providers.",
    },
    ProviderPreset {
        id: "ollama",
        label: "Ollama",
        base_url: "http://localhost:11434/v1",
        hint: "Local models served by Ollama.",
    },
    ProviderPreset {
        id: "lmstudio",
        label: "LM Studio",
        base_url: "http://localhost:1234/v1",
        hint: "Local OpenAI-compatible server.",
    },
    ProviderPreset {
        id: CUSTOM_PROVIDER_ID,
        label: "Custom endpoint",
        base_url: "",
        hint: "Any OpenAI-compatible endpoint.",
    },
];

fn preset_for(provider: &str) -> Option<ProviderPreset> {
    PROVIDER_PRESETS
        .iter()
        .copied()
        .find(|preset| preset.id == provider)
}

async fn load_models(
    base_url: String,
    current_model: String,
    models: &mut Signal<Vec<String>>,
    loading: &mut Signal<bool>,
    custom: &mut Signal<bool>,
    error: &mut Signal<String>,
) {
    loading.set(true);
    error.set(String::new());
    match ai::models(Some(&base_url)).await {
        Ok(values) => {
            let values = values.to_vec();
            custom.set(!values.is_empty() && !values.contains(&current_model));
            models.set(values);
        }
        Err(error_value) => {
            error.set(format!("Could not load models: {error_value}"));
        }
    }
    loading.set(false);
}

#[component]
pub fn AiConnectionCard() -> Element {
    let mut enabled = use_signal(|| false);
    let mut provider = use_signal(|| DEFAULT_PROVIDER_ID.to_string());
    let mut base_url = use_signal(String::new);
    let mut model = use_signal(String::new);
    let mut api_key = use_signal(String::new);
    let mut configured = use_signal(|| false);
    let mut clear_key = use_signal(|| false);
    let mut loading = use_signal(|| true);
    let mut saving = use_signal(|| false);
    let mut models = use_signal(Vec::<String>::new);
    let mut models_loading = use_signal(|| false);
    let mut custom_model = use_signal(|| true);
    let mut models_error = use_signal(String::new);

    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            loading.set(false);
            return;
        }
        match ai::settings().await {
            Ok(settings) => {
                apply_settings(
                    &mut enabled,
                    &mut provider,
                    &mut base_url,
                    &mut model,
                    &mut configured,
                    settings.clone(),
                );
                *AI_SETTINGS.write() = Some(settings.clone());
                if settings.configured {
                    load_models(
                        settings.base_url.clone(),
                        settings.model.clone(),
                        &mut models,
                        &mut models_loading,
                        &mut custom_model,
                        &mut models_error,
                    )
                    .await;
                }
            }
            Err(error) => TOAST
                .write()
                .error(format!("Could not load AI settings: {error}")),
        }
        loading.set(false);
    });

    let refresh_models = move |_| {
        if models_loading() || !configured() {
            return;
        }
        spawn(async move {
            load_models(
                base_url(),
                model(),
                &mut models,
                &mut models_loading,
                &mut custom_model,
                &mut models_error,
            )
            .await;
        });
    };

    let save = move |_| {
        if saving() {
            return;
        }
        saving.set(true);
        let request = UpdateAiSettingsRequest {
            enabled: enabled(),
            provider: Some(provider()),
            base_url: Some(base_url()),
            model: Some(model()),
            api_key: (!api_key().trim().is_empty()).then_some(api_key()),
            clear_api_key: clear_key(),
        };
        spawn(async move {
            match ai::update_settings(&request).await {
                Ok(settings) => {
                    apply_settings(
                        &mut enabled,
                        &mut provider,
                        &mut base_url,
                        &mut model,
                        &mut configured,
                        settings.clone(),
                    );
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

    let selected_provider = preset_for(&provider())
        .map(|preset| preset.label.to_string())
        .unwrap_or(provider());
    let model_select_value = if custom_model() {
        CUSTOM_MODEL_VALUE.to_string()
    } else {
        model()
    };

    rsx! {
        div { class: "relative overflow-hidden rounded-2xl border border-violet-500/20 bg-card/80 shadow-sm",
            div { class: "pointer-events-none absolute -right-24 -top-24 h-64 w-64 rounded-full bg-violet-500/10 blur-3xl" }
            div { class: "relative flex flex-wrap items-start gap-4 border-b border-border/60 p-6",
                span { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-violet-500/10 text-violet-600 ring-1 ring-violet-500/20 dark:text-violet-300",
                    Bot { size: 24 }
                }
                div { class: "min-w-0 flex-1",
                    div { class: "flex flex-wrap items-center gap-2",
                        h2 { class: "text-lg font-semibold", "Model connection" }
                        if configured() {
                            span { class: "inline-flex items-center gap-1 rounded-full border border-emerald-500/20 bg-emerald-500/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-emerald-700 dark:text-emerald-300",
                                CircleCheck { size: 11 }
                                "Connected"
                            }
                        } else {
                            span { class: "rounded-full border border-border/60 bg-background/60 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-muted-foreground",
                                "Not configured"
                            }
                        }
                    }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Bring your own key. Keys stay encrypted on the server and are never returned to this device."
                    }
                }
                label { class: "flex cursor-pointer items-center gap-2 rounded-full border border-border/60 bg-background/60 px-3 py-1.5 text-xs font-medium",
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
            div { class: "relative space-y-5 p-6",
                if crate::api::client::is_offline_mode() {
                    div { class: "rounded-xl border border-border/60 bg-muted/20 p-5 text-sm text-muted-foreground",
                        "The AI assistant is available in server mode. Local on-device mode has no AI proxy."
                    }
                } else {
                    div { class: "grid gap-4 md:grid-cols-2",
                        div { class: "space-y-2",
                            label { r#for: "ai-provider", class: "text-sm font-medium", "Provider" }
                            select {
                                id: "ai-provider",
                                value: provider(),
                                disabled: loading() || saving(),
                                class: "flex h-10 w-full rounded-lg border border-input bg-background px-3 text-sm",
                                onchange: move |event| {
                                    let next = event.value();
                                    if let Some(preset) = preset_for(&next)
                                        && !preset.base_url.is_empty()
                                    {
                                        base_url.set(preset.base_url.to_string());
                                    }
                                    provider.set(next);
                                    models.set(Vec::new());
                                    custom_model.set(true);
                                    models_error.set(String::new());
                                },
                                if preset_for(&provider()).is_none() {
                                    option { value: provider(), {provider()} }
                                }
                                for preset in PROVIDER_PRESETS {
                                    option { value: preset.id, {preset.label} }
                                }
                            }
                            p { class: "text-xs text-muted-foreground",
                                {preset_for(&provider()).map(|preset| preset.hint).unwrap_or("Choose a preset or use your own endpoint.")}
                            }
                        }
                        div { class: "space-y-2",
                            label { r#for: "ai-base-url", class: "text-sm font-medium", "API base URL" }
                            input {
                                id: "ai-base-url",
                                r#type: "url",
                                value: base_url(),
                                oninput: move |event| base_url.set(event.value()),
                                placeholder: "https://api.openai.com/v1",
                                disabled: loading() || saving(),
                                class: "flex h-10 w-full rounded-lg border border-input bg-transparent px-3 text-sm",
                            }
                            p { class: "text-xs text-muted-foreground", {selected_provider} }
                        }
                    }

                    div { class: "space-y-2",
                        div { class: "flex flex-wrap items-center justify-between gap-2",
                            label { r#for: "ai-model-select", class: "text-sm font-medium", "Model" }
                            button {
                                r#type: "button",
                                class: "inline-flex h-8 items-center gap-1.5 rounded-md border border-border px-2.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-foreground disabled:opacity-50",
                                disabled: loading() || saving() || models_loading() || !configured(),
                                title: if configured() { "Load models from the endpoint" } else { "Save an API key first" },
                                onclick: refresh_models,
                                RefreshCw { size: 13, class: if models_loading() { "animate-spin" } else { "" } }
                                if models_loading() { "Loading…" } else { "Refresh models" }
                            }
                        }
                        if models().is_empty() {
                            input {
                                id: "ai-model",
                                r#type: "text",
                                value: model(),
                                oninput: move |event| model.set(event.value()),
                                placeholder: "gpt-4o-mini",
                                disabled: loading() || saving(),
                                class: "flex h-10 w-full rounded-lg border border-input bg-transparent px-3 text-sm",
                            }
                        } else {
                            select {
                                id: "ai-model-select",
                                value: model_select_value,
                                disabled: loading() || saving(),
                                class: "flex h-10 w-full rounded-lg border border-input bg-background px-3 text-sm",
                                onchange: move |event| {
                                    let value = event.value();
                                    if value == CUSTOM_MODEL_VALUE {
                                        custom_model.set(true);
                                    } else {
                                        custom_model.set(false);
                                        model.set(value);
                                    }
                                },
                                for option_value in models() {
                                    option { value: option_value, {option_value.clone()} }
                                }
                                option { value: CUSTOM_MODEL_VALUE, "Custom model…" }
                            }
                            if custom_model() {
                                input {
                                    id: "ai-model-custom",
                                    r#type: "text",
                                    value: model(),
                                    oninput: move |event| model.set(event.value()),
                                    placeholder: "Type a model id",
                                    disabled: loading() || saving(),
                                    class: "mt-2 flex h-10 w-full rounded-lg border border-input bg-transparent px-3 text-sm",
                                }
                            }
                        }
                        if !models_error().is_empty() {
                            p { class: "text-xs text-destructive", {models_error()} }
                        } else if !models().is_empty() {
                            p { class: "text-xs text-muted-foreground",
                                "Loaded from the endpoint. Pick one or enter a custom model id."
                            }
                        } else if configured() {
                            p { class: "text-xs text-muted-foreground",
                                "Refresh to load models, or type the model id manually."
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
                            disabled: loading() || saving(),
                            class: "flex h-10 w-full rounded-lg border border-input bg-transparent px-3 text-sm",
                        }
                        p { class: "flex items-center gap-1.5 text-xs text-muted-foreground",
                            ShieldCheck { size: 13 }
                            "Encrypted at rest. Never sent back to the client."
                        }
                    }

                    div { class: "flex flex-wrap items-center justify-between gap-3 border-t border-border/50 pt-4",
                        div { class: "flex items-center gap-2",
                            if configured() && !clear_key() {
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-9 items-center rounded-lg border border-destructive/30 px-3 text-sm text-destructive transition-colors hover:bg-destructive/10",
                                    onclick: move |_| clear_key.set(true),
                                    "Remove key"
                                }
                            }
                            if clear_key() {
                                button {
                                    r#type: "button",
                                    class: "inline-flex h-9 items-center rounded-lg border border-border px-3 text-sm",
                                    onclick: move |_| clear_key.set(false),
                                    "Keep key"
                                }
                            }
                            span { class: "inline-flex items-center gap-1.5 text-xs text-muted-foreground",
                                Sparkles { size: 13 }
                                if configured() { "Key stored" } else { "Key not configured" }
                            }
                        }
                        button {
                            r#type: "button",
                            class: "inline-flex h-10 items-center gap-2 rounded-lg bg-primary px-4 text-sm font-semibold text-primary-foreground shadow-sm transition-colors hover:bg-primary/90 disabled:opacity-50",
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

fn apply_settings(
    enabled: &mut Signal<bool>,
    provider: &mut Signal<String>,
    base_url: &mut Signal<String>,
    model: &mut Signal<String>,
    configured: &mut Signal<bool>,
    settings: AiSettings,
) {
    enabled.set(settings.enabled);
    provider.set(settings.provider);
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
        assert!(source.contains("clear_api_key"));
    }

    #[test]
    fn ai_card_loads_models_and_keeps_a_custom_option() {
        let source = include_str!("ai_card.rs");
        assert!(source.contains("Refresh models"));
        assert!(source.contains("Custom model"));
        assert!(source.contains("ai::models"));
        assert!(source.contains(CUSTOM_MODEL_VALUE));
    }

    #[test]
    fn provider_presets_have_unique_ids_and_urls() {
        let mut ids = std::collections::HashSet::new();
        for preset in PROVIDER_PRESETS {
            assert!(ids.insert(preset.id));
            if preset.id != CUSTOM_PROVIDER_ID {
                assert!(preset.base_url.starts_with("http"));
            }
        }
    }

    #[test]
    fn custom_provider_has_no_forced_url() {
        assert!(preset_for(CUSTOM_PROVIDER_ID).unwrap().base_url.is_empty());
    }
}
