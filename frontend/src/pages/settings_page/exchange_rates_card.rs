use crate::api::currency;
use crate::components::toast::TOAST;
use crate::models::currency::UpdateCurrencySettingsRequest;
use crate::state::auth::AUTH;
use crate::state::rates::RATES;
use dioxus::prelude::*;

/// Exchange-rate provider used for converted amounts.
#[component]
pub fn ExchangeRatesCard() -> Element {
    let mut provider = use_signal(|| "frankfurter".to_string());
    let mut fixer_key = use_signal(String::new);
    let mut fixer_configured = use_signal(|| false);
    let mut saving = use_signal(|| false);

    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            return;
        }
        if let Ok(settings) = currency::get_settings().await {
            provider.set(settings.provider);
            fixer_configured.set(settings.fixer_configured);
        }
    });

    let save: Callback<bool> = use_callback(move |clear_key: bool| {
        saving.set(true);
        let provider_value = provider();
        let key_value = fixer_key();
        spawn(async move {
            let req = UpdateCurrencySettingsRequest {
                provider: provider_value,
                fixer_api_key: if key_value.trim().is_empty() || clear_key {
                    None
                } else {
                    Some(key_value)
                },
                clear_fixer_api_key: clear_key,
            };
            match currency::update_settings(&req).await {
                Ok(settings) => {
                    provider.set(settings.provider);
                    fixer_configured.set(settings.fixer_configured);
                    fixer_key.set(String::new());
                    if let Ok(rates) = currency::get_rates(AUTH().currency()).await {
                        RATES.write().set_data(rates);
                    }
                    TOAST.write().success("Exchange-rate settings saved");
                }
                Err(error) => {
                    log::error!("Failed to save exchange-rate settings: {error}");
                    TOAST
                        .write()
                        .error(format!("Could not save exchange settings: {error}"));
                }
            }
            saving.set(false);
        });
    });

    rsx! {
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Exchange rates" }
                p { class: "text-sm text-muted-foreground",
                    "Choose where automatic currency conversion gets its daily rates."
                }
            }
            div { class: "p-6 space-y-4",
                div { class: "space-y-2",
                    label { r#for: "settings-provider", class: "text-sm font-medium", "Provider" }
                    select {
                        id: "settings-provider",
                        value: provider(),
                        onchange: move |e| provider.set(e.value()),
                        disabled: saving(),
                        class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm",
                        option { value: "frankfurter", "Frankfurter — free, no API key" }
                        option { value: "fixer", "Fixer — requires an API key" }
                    }
                }
                if provider() == "fixer" {
                    div { class: "space-y-2",
                        label { r#for: "settings-fixer-key", class: "text-sm font-medium", "Fixer API key" }
                        input {
                            id: "settings-fixer-key",
                            r#type: "password",
                            value: fixer_key(),
                            oninput: move |e| fixer_key.set(e.value()),
                            autocomplete: "off",
                            placeholder: if fixer_configured() { "Key saved — enter a new key to replace it" } else { "Paste your Fixer API key" },
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm",
                        }
                        div { class: "flex flex-wrap items-center justify-between gap-2",
                            p { class: "text-xs text-muted-foreground",
                                if fixer_configured() { "A key is configured." } else { "The key is stored encrypted on your server." }
                            }
                            if fixer_configured() {
                                button {
                                    r#type: "button",
                                    class: "text-xs text-muted-foreground hover:text-foreground",
                                    disabled: saving(),
                                    onclick: move |_| {
                                        provider.set("frankfurter".into());
                                        save.call(true);
                                    },
                                    "Remove key"
                                }
                            }
                        }
                    }
                }
                div { class: "flex items-center justify-between gap-3",
                    p { class: "text-xs text-muted-foreground", "Rates are cached for six hours." }
                    button {
                        r#type: "button",
                        class: "inline-flex h-9 items-center rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                        disabled: saving(),
                        onclick: move |_| save.call(false),
                        if saving() { "Saving…" } else { "Save rates" }
                    }
                }
            }
        }
    }
}
