use crate::api::currency;
use crate::components::select_menu::{SelectMenu, SelectOption};
use crate::components::toast::TOAST;
use crate::models::currency::UpdateCurrencySettingsRequest;
use crate::state::auth::AUTH;
use crate::state::rates::RATES;
use dioxus::prelude::*;

/// Exchange-rate provider used for converted amounts.
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::SectionCard;
use crate::components::field::Field;
use crate::components::input::{Input, InputSize};
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
        SectionCard {
            title: "Exchange rates",
            description: "Choose where automatic currency conversion gets its daily rates.",
            content_class: "p-0",
            div { class: "p-6 space-y-4",
                Field { label: "Provider", html_for: "settings-provider",
                    SelectMenu {
                        id: "settings-provider",
                        value: provider(),
                        options: vec![
                            SelectOption::new("frankfurter", "Frankfurter — free, no API key"),
                            SelectOption::new("fixer", "Fixer — requires an API key"),
                        ],
                        on_change: move |value: String| provider.set(value),
                        searchable: false,
                        disabled: saving(),
                    }
                }
                if provider() == "fixer" {
                    Field { label: "Fixer API key", html_for: "settings-fixer-key",
                        Input {
                            id: "settings-fixer-key",
                            r#type: "password",
                            size: InputSize::Sm,
                            value: fixer_key(),
                            on_input: move |value| fixer_key.set(value),
                            autocomplete: "off",
                            placeholder: if fixer_configured() { "Key saved — enter a new key to replace it" } else { "Paste your Fixer API key" },
                        }
                        div { class: "flex flex-wrap items-center justify-between gap-2",
                            p { class: "text-xs text-muted-foreground",
                                if fixer_configured() { "A key is configured." } else { "The key is stored encrypted on your server." }
                            }
                            if fixer_configured() {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Xs,
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
                    Button {
                        size: ButtonSize::Sm,
                        disabled: saving(),
                        onclick: move |_| save.call(false),
                        if saving() { "Saving…" } else { "Save rates" }
                    }
                }
            }
        }
    }
}
