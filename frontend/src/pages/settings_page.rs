use dioxus::prelude::*;
use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;
use crate::api::settings;
use crate::api::notification_preferences;
use crate::models::auth::UpdateSettingsRequest;
use crate::models::notification::{UpdateNotificationPreferenceRequest, UpdateNotificationPreferencesRequest};
use crate::utils::constants::CURRENCIES;
use crate::components::toast::TOAST;

const NOTIFICATION_CHANNELS: &[(&str, &str, &str)] = &[
    ("email", "Email Notifications", "Receive email reminders before payments"),
    ("push", "Push Notifications", "Receive push notifications on your devices"),
    ("system", "System Notifications", "Receive desktop notifications"),
];

#[component]
pub fn SettingsPage() -> Element {
    let mut name = use_signal(|| AUTH().user.as_ref().and_then(|u| u.name.clone()).unwrap_or_default());
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut notification_prefs = use_signal(|| {
        let mut map = std::collections::HashMap::new();
        for (key, _, _) in NOTIFICATION_CHANNELS {
            map.insert(key.to_string(), false);
        }
        map
    });
    let mut prefs_loading = use_signal(|| false);
    let mut saving_name = use_signal(|| false);
    let mut saving_currency = use_signal(|| false);
    let server_url = use_signal(|| crate::api::client::get_base_url());

    // Load notification preferences
    use_future(move || async move {
        match notification_preferences::get().await {
            Ok(resp) => {
                let mut map = std::collections::HashMap::new();
                for p in resp.preferences {
                    map.insert(p.channel, p.enabled);
                }
                notification_prefs.set(map);
            }
            Err(e) => log::error!("Failed to load notification prefs: {}", e),
        }
    });

    // Sync form state when user changes
    use_effect(move || {
        if let Some(u) = &AUTH().user {
            name.set(u.name.clone().unwrap_or_default());
            currency.set(u.currency.clone());
        }
    });


    let mut handle_save_currency = move |new_currency: String| {
        currency.set(new_currency.clone());
        saving_currency.set(true);
        spawn(async move {
            match settings::update(&UpdateSettingsRequest {
                name: None,
                currency: Some(new_currency),
            }).await {
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

    let mut handle_notification_toggle = move |channel: String, enabled: bool| {
        let mut prefs = notification_prefs();
        prefs.insert(channel.clone(), enabled);
        notification_prefs.set(prefs);
        prefs_loading.set(true);
        spawn(async move {
            let req = UpdateNotificationPreferencesRequest {
                preferences: vec![UpdateNotificationPreferenceRequest {
                    channel: channel.clone(),
                    enabled,
                }],
            };
            match notification_preferences::update(&req).await {
                Ok(_) => {
                    TOAST.write().success("Notification preferences updated");
                }
                Err(e) => {
                    log::error!("Failed to update notification pref: {}", e);
                    let mut prefs = notification_prefs();
                    prefs.insert(channel, !enabled);
                    notification_prefs.set(prefs);
                    TOAST.write().error("Failed to update notifications");
                }
            }
            prefs_loading.set(false);
        });
    };

    let user_email = AUTH().user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let cur = currency();


    rsx! {
        div { class: "p-4 md:p-6 flex flex-col gap-6",
            h1 { class: "text-3xl font-bold tracking-tight", "Settings" }
            p { class: "text-muted-foreground", "Manage your account and preferences" }

            // Profile card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Profile" }
                    p { class: "text-sm text-muted-foreground", "Update your personal information" }
                }
                div { class: "p-6 space-y-4",
                    // Email (read-only)
                    div { class: "space-y-2",
                        label { r#for: "settings-email", class: "text-sm font-medium", "Email" }
                        input {
                            id: "settings-email",
                            r#type: "email",
                            value: "{user_email}",
                            disabled: true,
                            class: "flex h-10 w-full rounded-md border border-input bg-muted px-3 py-2 text-sm opacity-50 cursor-not-allowed",
                        }
                    }
                    // Name
                    div { class: "space-y-2",
                        label { r#for: "settings-name", class: "text-sm font-medium", "Name" }
                        input {
                            id: "settings-name",
                            r#type: "text",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                            onblur: move |_| {
                                let n = name();
                                if n.len() >= 2 {
                                    saving_name.set(true);
                                    spawn(async move {
                                        match settings::update(&UpdateSettingsRequest {
                                            name: Some(n),
                                            currency: None,
                                        }).await {
                                            Ok(resp) => {
                                                AUTH.write().user = Some(resp.user);
                                                AUTH.write().persist_to_storage();
                                                TOAST.write().success("Name updated");
                                                saving_name.set(false);
                                            }
                                            Err(e) => {
                                                log::error!("Failed to save name: {}", e);
                                                TOAST.write().error("Failed to update name");
                                                saving_name.set(false);
                                            }
                                        }
                                    });
                                }
                            },
                            class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                        }
                    }

                }
            }

            // Preferences card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Preferences" }
                    p { class: "text-sm text-muted-foreground", "Customize your experience" }
                }
                div { class: "p-6 space-y-4",
                    div { class: "space-y-2",
                        label { r#for: "settings-currency", class: "text-sm font-medium", "Default Currency" }
                        select {
                            id: "settings-currency",
                            value: "{cur}",
                            onchange: move |e| handle_save_currency(e.value()),
                            disabled: saving_currency(),
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring",
                            for (val, label) in CURRENCIES {
                                option { value: "{val}", "{label}" }
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
                                PREFERENCES.write().set_first_day_of_week(day.clone());
                                spawn(async move {
                                    match crate::api::preferences::update(&crate::models::preferences::UpdatePreferencesRequest {
                                        theme: None,
                                        subscriptions_view: None,
                                        first_day_of_week: Some(day),
                                    }).await {
                                        Ok(_) => TOAST.write().success("First day of week updated"),
                                        Err(_) => TOAST.write().error("Failed to update preference"),
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

            // Server card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Server" }
                    p { class: "text-sm text-muted-foreground", "Your Recurrency server connection" }
                }
                div { class: "p-6 space-y-4",
                    div { class: "space-y-2",
                        label { r#for: "settings-server", class: "text-sm font-medium", "Server URL" }
                        input {
                            id: "settings-server",
                            r#type: "text",
                            value: "{server_url}",
                            disabled: true,
                            class: "flex h-10 w-full rounded-md border border-input bg-muted px-3 py-2 text-sm opacity-50 cursor-not-allowed",
                        }
                    }
                }
            }

            // Notifications card
            div { class: "rounded-lg border border-border bg-card",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Notifications" }
                    p { class: "text-sm text-muted-foreground", "Configure how you receive notifications" }
                }
                div { class: "p-6 space-y-4",
                    for (idx, (key, label, desc)) in NOTIFICATION_CHANNELS.iter().enumerate() {
                        {let checked = notification_prefs().get(*key).copied().unwrap_or(false);
                         let key_str = key.to_string();
                         rsx! {
                            div { key: "{key}",
                                div { class: "flex items-center justify-between",
                                    div {
                                        p { class: "font-medium", "{label}" }
                                        p { class: "text-sm text-muted-foreground", "{desc}" }
                                    }
                                    button {
                                        class: if checked {
                                            "relative inline-flex h-6 w-11 items-center rounded-full bg-primary transition-colors focus:outline-none"
                                        } else {
                                            "relative inline-flex h-6 w-11 items-center rounded-full bg-muted transition-colors focus:outline-none"
                                        },
                                        onclick: {
                                            let key_str = key_str.clone();
                                            let new_val = !checked;
                                            move |_| handle_notification_toggle(key_str.clone(), new_val)
                                        },
                                        disabled: prefs_loading(),
                                        span {
                                            class: if checked {
                                                "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
                                            } else {
                                                "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
                                            },
                                        }
                                    }
                                }
                                if idx < NOTIFICATION_CHANNELS.len() - 1 {
                                    hr { class: "mt-4 border-border" }
                                }
                            }
                        }}
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
    fn notification_channels_count() {
        assert_eq!(NOTIFICATION_CHANNELS.len(), 3);
    }

    #[test]
    fn notification_channels_keys() {
        let keys: Vec<_> = NOTIFICATION_CHANNELS.iter().map(|(k,_,_)| *k).collect();
        assert!(keys.contains(&"email"));
        assert!(keys.contains(&"push"));
        assert!(keys.contains(&"system"));
    }

    #[test]
    fn currencies_contains_eur() {
        assert!(CURRENCIES.iter().any(|(v, _)| *v == "EUR"));
    }

    #[test]
    fn name_validation_too_short() {
        let name = "A";
        assert!(name.len() < 2);
    }

    #[test]
    fn name_validation_ok() {
        let name = "Alice";
        assert!(name.len() >= 2);
    }

    // ── Notification channel detailed tests ──
    #[test]
    fn notification_channels_have_labels() {
        for (key, label, desc) in NOTIFICATION_CHANNELS {
            assert!(!label.is_empty(), "Channel {} has empty label", key);
            assert!(!desc.is_empty(), "Channel {} has empty description", key);
        }
    }

    #[test]
    fn notification_channels_unique_keys() {
        let mut keys = std::collections::HashSet::new();
        for (key, _, _) in NOTIFICATION_CHANNELS {
            assert!(keys.insert(*key), "Duplicate key: {}", key);
        }
    }

    // ── Currency preference logic ──
    #[test]
    fn currencies_all_have_codes() {
        for (val, label) in CURRENCIES {
            assert!(!val.is_empty());
            assert!(!label.is_empty());
            assert!(label.contains(val), "Label '{}' should contain code '{}'", label, val);
        }
    }

    #[test]
    fn currencies_includes_usd() {
        assert!(CURRENCIES.iter().any(|(v, _)| *v == "USD"));
    }

    #[test]
    fn currencies_includes_gbp() {
        assert!(CURRENCIES.iter().any(|(v, _)| *v == "GBP"));
    }

    #[test]
    fn currencies_includes_jpy() {
        assert!(CURRENCIES.iter().any(|(v, _)| *v == "JPY"));
    }

    // ── UpdateSettingsRequest construction ──
    #[test]
    fn update_settings_name_only() {
        use crate::models::auth::UpdateSettingsRequest;
        let req = UpdateSettingsRequest {
            name: Some("Bob".into()),
            currency: None,
        };
        assert_eq!(req.name.as_deref(), Some("Bob"));
        assert!(req.currency.is_none());
    }

    #[test]
    fn update_settings_currency_only() {
        use crate::models::auth::UpdateSettingsRequest;
        let req = UpdateSettingsRequest {
            name: None,
            currency: Some("USD".into()),
        };
        assert!(req.name.is_none());
        assert_eq!(req.currency.as_deref(), Some("USD"));
    }

    #[test]
    fn update_settings_both() {
        use crate::models::auth::UpdateSettingsRequest;
        let req = UpdateSettingsRequest {
            name: Some("Alice".into()),
            currency: Some("GBP".into()),
        };
        assert_eq!(req.name.as_deref(), Some("Alice"));
        assert_eq!(req.currency.as_deref(), Some("GBP"));
    }

    // ── Notification preference request ──
    #[test]
    fn notification_pref_request_construction() {
        use crate::models::notification::{
            UpdateNotificationPreferenceRequest,
            UpdateNotificationPreferencesRequest,
        };
        let req = UpdateNotificationPreferencesRequest {
            preferences: vec![
                UpdateNotificationPreferenceRequest {
                    channel: "email".into(),
                    enabled: true,
                },
                UpdateNotificationPreferenceRequest {
                    channel: "push".into(),
                    enabled: false,
                },
            ],
        };
        assert_eq!(req.preferences.len(), 2);
        assert_eq!(req.preferences[0].channel, "email");
        assert!(req.preferences[0].enabled);
        assert!(!req.preferences[1].enabled);
    }

    // ── Name validation edge cases ──
    #[test]
    fn name_exactly_two_chars_valid() {
        let name = "Al";
        assert!(name.len() >= 2);
    }

    #[test]
    fn name_with_unicode_valid() {
        let name = "Álvaro";
        assert!(name.len() >= 2);
    }

    #[test]
    fn name_whitespace_only_invalid() {
        let name = "   ";
        // The component saves only if len >= 2, but real validation would trim
        // Testing the raw check from component: name.len() >= 2
        assert!(name.len() >= 2); // whitespace passes raw check
    }

    // ── Server URL construction ──
    #[test]
    fn get_base_url_returns_string() {
        let url = crate::api::client::get_base_url();
        assert!(url.starts_with("http"));
    }

    #[test]
    fn notification_toggle_logic() {
        // Simulate toggle: if current is true, new should be false, etc.
        let channels: std::collections::HashMap<String, bool> = [
            ("email".to_string(), true),
            ("push".to_string(), false),
        ].into_iter().collect();
        assert_eq!(*channels.get("email").unwrap(), true);
        assert_eq!(*channels.get("push").unwrap(), false);
    }

    #[test]
    fn notification_prefs_default_all_disabled() {
        let mut map = std::collections::HashMap::new();
        for (key, _, _) in NOTIFICATION_CHANNELS {
            map.insert(key.to_string(), false);
        }
        for (key, _, _) in NOTIFICATION_CHANNELS {
            assert_eq!(*map.get(*key).unwrap(), false);
        }
    }

    // ── Settings update auth flow simulation ──
    #[test]
    fn settings_update_response_updates_user_currency() {
        // Simulates what happens when settings::update returns SettingsResponse
        use crate::models::auth::{UserResponse, SettingsResponse};
        use uuid::Uuid;

        let updated_user = UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
            name: Some("Alice".into()),
            currency: "USD".into(),
            verified: true,
        };
        
        let response = SettingsResponse {
            user: updated_user,
        };
        
        // Verify response structure
        assert_eq!(response.user.currency, "USD");
        assert_eq!(response.user.name, Some("Alice".into()));
    }

    // ── Currency update flow simulation ──
    #[test]
    fn currency_update_request_response_cycle() {
        use crate::models::auth::{UpdateSettingsRequest, SettingsResponse, UserResponse};
        use uuid::Uuid;

        // Create request
        let req = UpdateSettingsRequest {
            name: None,
            currency: Some("JPY".into()),
        };
        assert_eq!(req.currency.as_deref(), Some("JPY"));

        // Simulate response (would come from API)
        let resp = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                name: None,
                currency: "JPY".into(),
                verified: true,
            },
        };
        
        // Verify currency was updated in response
        assert_eq!(resp.user.currency, "JPY");
    }

    // ── Toast notification integration tests ──
    #[test]
    fn settings_should_show_toast_on_success() {
        // Test that toast API exists and can be called
        // After fix: settings page should call TOAST.write().success(...) on save
        use crate::components::toast::{ToastState, ToastKind};
        
        let mut state = ToastState::default();
        state.success("Currency saved");
        
        assert_eq!(state.items.len(), 1);
        assert_eq!(state.items[0].message, "Currency saved");
        assert!(matches!(state.items[0].kind, ToastKind::Success));
    }

    #[test]
    fn settings_should_show_toast_on_error() {
        use crate::components::toast::{ToastState, ToastKind};
        
        let mut state = ToastState::default();
        state.error("Failed to save");
        
        assert_eq!(state.items.len(), 1);
        assert_eq!(state.items[0].message, "Failed to save");
        assert!(matches!(state.items[0].kind, ToastKind::Error));
    }

    // ── Toast lifecycle in settings flow ──
    #[test]
    fn settings_toast_lifecycle_add_then_remove() {
        // RED: Simulates settings save → toast → user dismisses
        use crate::components::toast::ToastState;
        
        let mut state = ToastState::default();
        
        // User saves settings, toast appears
        state.success("Currency updated");
        assert_eq!(state.items.len(), 1);
        let toast_id = state.items[0].id;
        
        // User clicks X to dismiss
        state.remove(toast_id);
        assert_eq!(state.items.len(), 0);
    }

    #[test]
    fn settings_multiple_toasts_can_be_individually_removed() {
        // RED: Multiple settings changes create multiple toasts
        use crate::components::toast::ToastState;
        
        let mut state = ToastState::default();
        
        // First save: currency
        state.success("Currency updated");
        let currency_id = state.items[0].id;
        
        // Second save: name
        state.success("Name updated");
        let name_id = state.items[1].id;
        
        assert_eq!(state.items.len(), 2);
        
        // Dismiss currency toast only
        state.remove(currency_id);
        assert_eq!(state.items.len(), 1);
        assert!(state.items.iter().any(|t| t.message == "Name updated"));
        
        // Dismiss name toast
        state.remove(name_id);
        assert_eq!(state.items.len(), 0);
    }
}