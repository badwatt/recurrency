use crate::api::settings;
use crate::components::server_url::ServerUrlEditor;
use crate::components::toast::TOAST;
use crate::models::auth::UpdateSettingsRequest;
use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;
use crate::utils::constants::CURRENCIES;
use dioxus::prelude::*;

/// Preferences card: default currency and first day of week.
#[component]
pub fn PreferencesCard() -> Element {
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut saving_currency = use_signal(|| false);

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
            }
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
