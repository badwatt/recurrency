use crate::api::currency;
use crate::api::settings;
use crate::components::server_url::ServerUrlEditor;
use crate::components::web_server_qr::WebServerQr;
use crate::components::toast::TOAST;
use crate::models::auth::UpdateSettingsRequest;
use crate::models::currency::UpdateCurrencySettingsRequest;
use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;
use crate::state::rates::RATES;
use crate::utils::constants::CURRENCIES;
use dioxus::prelude::*;

/// Preferences card: default currency and first day of week.
#[component]
pub fn PreferencesCard() -> Element {
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut saving_currency = use_signal(|| false);
    let mut provider = use_signal(|| "frankfurter".to_string());
    let mut fixer_key = use_signal(String::new);
    let mut fixer_configured = use_signal(|| false);
    let mut saving_rates = use_signal(|| false);

    use_future(move || async move {
        if crate::api::client::is_offline_mode() {
            return;
        }
        if let Ok(settings) = currency::get_settings().await {
            provider.set(settings.provider);
            fixer_configured.set(settings.fixer_configured);
        }
    });

    // Sync currency if it is empty (never overwrite the user's in-progress edit).
    use_effect(move || {
        if let Some(u) = &AUTH().user
            && currency().is_empty()
        {
            currency.set(u.currency.clone());
        }
    });

    let mut handle_save_currency = move |new_currency: String| {
        currency.set(new_currency.clone());
        saving_currency.set(true);
        spawn(async move {
            match settings::update(&UpdateSettingsRequest {
                email: None,
                avatar: None,
                name: None,
                currency: Some(new_currency),
            })
            .await
            {
                Ok(resp) => {
                    AUTH.write().user = Some(resp.user);
                    AUTH.write().persist_to_storage();
                    TOAST.write().success("Currency updated");
                    saving_currency.set(false);
                }
                Err(e) => {
                    log::error!("Failed to save currency: {}", e);
                    TOAST.write().error("Failed to update currency");
                    saving_currency.set(false);
                }
            }
        });
    };

    let save_rates: Callback<bool> = use_callback(move |clear_key: bool| {
        saving_rates.set(true);
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
                    TOAST.write().error(format!("Could not save exchange settings: {error}"));
                }
            }
            saving_rates.set(false);
        });
    });

    let cur = currency();

    rsx! {
        div { class: "flex flex-col gap-6",
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Defaults" }
                p { class: "text-sm text-muted-foreground", "Applied to new entries and reports" }
            }
            div { class: "p-6 space-y-4",
                div { class: "space-y-2",
                    label { r#for: "settings-currency", class: "text-sm font-medium", "Default Currency" }
                    select {
                        id: "settings-currency",
                        value: cur.to_string(),
                        onchange: move |e| handle_save_currency(e.value()),
                        disabled: saving_currency(),
                        class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring",
                        for (val, label) in CURRENCIES {
                            option { value: val.to_string(), {label.to_string()} }
                        }
                    }
                }
                // First day of week
                div { class: "space-y-2",
                    label { r#for: "settings-first-day", class: "text-sm font-medium", "First Day of Week" }
                    select {
                        id: "settings-first-day",
                        value: "{PREFERENCES().first_day_of_week()}",
                        onchange: move |e| {
                            let day = e.value();
                            let previous = PREFERENCES().first_day_of_week().to_string();
                            PREFERENCES.write().set_first_day_of_week(day.clone());
                            spawn(async move {
                                match crate::api::preferences::update(&crate::models::preferences::UpdatePreferencesRequest {
                                    theme: None,
                                    subscriptions_view: None,
                                    first_day_of_week: Some(day),
                                    date_format: None,
                                }).await {
                                    Ok(_) => TOAST.write().success("First day of week updated"),
                                    Err(e) => {
                                        log::error!("Failed to update first_day_of_week: {:?}", e);
                                        PREFERENCES.write().set_first_day_of_week(previous);
                                        TOAST.write().error("Failed to update preference");
                                    }
                                }
                            });
                        },
                        class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring",
                        option { value: "monday", "Monday" }
                        option { value: "sunday", "Sunday" }
                    }
                }
                div { class: "space-y-2",
                    label { r#for: "settings-date-format", class: "text-sm font-medium", "Date format" }
                    select {
                        id: "settings-date-format",
                        value: PREFERENCES().date_format(),
                        onchange: move |e| {
                            let format = e.value();
                            let previous = PREFERENCES().date_format();
                            PREFERENCES.write().set_date_format(format.clone());
                            spawn(async move {
                                match crate::api::preferences::update(&crate::models::preferences::UpdatePreferencesRequest {
                                    theme: None,
                                    subscriptions_view: None,
                                    first_day_of_week: None,
                                    date_format: Some(format),
                                }).await {
                                    Ok(_) => TOAST.write().success("Date format updated"),
                                    Err(error) => {
                                        log::error!("Failed to update date format: {error}");
                                        PREFERENCES.write().set_date_format(previous);
                                        TOAST.write().error("Failed to update date format");
                                    }
                                }
                            });
                        },
                        class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm",
                        option { value: "DD/MM/YYYY", "DD/MM/YYYY" }
                        option { value: "MM/DD/YYYY", "MM/DD/YYYY" }
                        option { value: "YYYY/MM/DD", "YYYY/MM/DD" }
                    }
                }
                div { class: "flex items-start gap-3 rounded-lg border border-border/40 p-3",
                    input {
                        r#type: "checkbox",
                        id: "settings-show-updates",
                        checked: PREFERENCES().show_updates(),
                        onchange: move |e| {
                            PREFERENCES.write().set_show_updates(e.checked());
                        },
                        class: "mt-0.5 h-4 w-4 accent-primary",
                    }
                    div { class: "space-y-0.5",
                        label { r#for: "settings-show-updates", class: "text-sm font-medium", "Show update notices" }
                        p { class: "text-xs text-muted-foreground", "Check for new Recurrency releases and show a dismissible banner." }
                    }
                }
            }
        }
        div { class: "rounded-xl bg-card/80 border border-border/40",
            div { class: "p-6 border-b border-border",
                h2 { class: "text-lg font-semibold", "Exchange rates" }
                p { class: "text-sm text-muted-foreground", "Choose where automatic currency conversion gets its daily rates." }
            }
            div { class: "p-6 space-y-4",
                div { class: "space-y-2",
                    label { r#for: "settings-provider", class: "text-sm font-medium", "Provider" }
                    select {
                        id: "settings-provider",
                        value: provider(),
                        onchange: move |e| provider.set(e.value()),
                        disabled: saving_rates(),
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
                                    disabled: saving_rates(),
                                    onclick: move |_| {
                                        provider.set("frankfurter".into());
                                        save_rates.call(true);
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
                        disabled: saving_rates(),
                        onclick: move |_| save_rates.call(false),
                        if saving_rates() { "Saving…" } else { "Save rates" }
                    }
                }
            }
        }
        if cfg!(target_arch = "wasm32") {
            WebServerQr {}
        }
        if cfg!(not(target_arch = "wasm32")) {
            div { class: "rounded-xl bg-card/80 border border-border/40",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Server" }
                    p { class: "text-sm text-muted-foreground",
                        "The Recurrency server this device connects to. Changing it signs you out."
                    }
                }
                div { class: "p-6",
                    ServerUrlEditor { logout_on_change: true }
                }
            }
        }
        }
    }
}
