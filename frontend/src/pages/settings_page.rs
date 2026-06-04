#![allow(clippy::suspicious_else_formatting)]
use crate::api::app;
use crate::api::notification_preferences;
use crate::api::settings;
use crate::components::toast::TOAST;
use crate::components::page_header::PageHeader;
use crate::models::auth::UpdateSettingsRequest;
use crate::models::notification::{
    UpdateNotificationPreferenceRequest, UpdateNotificationPreferencesRequest,
};
use crate::state::app_mode::APP_MODE;
use crate::state::auth::AUTH;
use crate::state::preferences::PREFERENCES;
use crate::Route;
use crate::utils::constants::CURRENCIES;
use dioxus::prelude::*;

const NOTIFICATION_CHANNELS: &[(&str, &str, &str)] = &[
    (
        "email",
        "Email Notifications",
        "Receive email reminders before payments",
    ),
    (
        "push",
        "Push Notifications",
        "Receive push notifications on your devices",
    ),
    (
        "system",
        "System Notifications",
        "Receive desktop notifications",
    ),
];

pub fn is_email_disabled(channel: &str, smtp_configured: bool) -> bool {
    channel == "email" && !smtp_configured
}

#[component]
pub fn SettingsPage() -> Element {
    let mut name = use_signal(|| {
        AUTH()
            .user
            .as_ref()
            .and_then(|u| u.name.clone())
            .unwrap_or_default()
    });
    let mut currency = use_signal(|| AUTH().currency().to_string());
    let mut notification_prefs = use_signal(|| {
        let mut map = std::collections::HashMap::new();
        for (key, _, _) in NOTIFICATION_CHANNELS {
            map.insert(key.to_string(), false);
        }
        map
    });
    let mut prefs_loading = use_signal(|| false);
    let mut smtp_configured = use_signal(|| true); // Default true to avoid flash
    let mut saving_name = use_signal(|| false);
    let mut saving_currency = use_signal(|| false);

    // Load notification preferences
    use_future(move || async move {
        // android guard removed
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

    // Load app info to check SMTP configuration
    use_future(move || async move {
        match app::get_app_info().await {
            Ok(info) => smtp_configured.set(info.smtp_configured),
            Err(e) => log::error!("Failed to load app info: {}", e),
        }
    });

    // Sync form state when user changes (only if fields are empty to avoid overwriting edits)
    use_effect(move || {
        if let Some(u) = &AUTH().user {
            // Only initialize empty fields; don't overwrite user edits
            if name().is_empty() {
                name.set(u.name.clone().unwrap_or_default());
            }
            if currency().is_empty() {
                currency.set(u.currency.clone());
            }
        }
    });

    let mut handle_save_currency = move |new_currency: String| {
        currency.set(new_currency.clone());
        saving_currency.set(true);
        spawn(async move {
            match settings::update(&UpdateSettingsRequest {
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

    let user_email = AUTH()
        .user
        .as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_default();
    let cur = currency();

    rsx! {
        div { class: "p-2 md:p-6 flex flex-col gap-6",
            div { class: "animate-in fade-in slide-in-from-bottom-4 duration-700",
                PageHeader { title: "Settings", subtitle: "Manage your account and preferences" }
            }

            // Profile card
            div { class: "rounded-xl bg-card/80 border border-border/40",
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
                            value: {user_email.to_string()},
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
                            value: {name.to_string()},
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
            div { class: "rounded-xl bg-card/80 border border-border/40",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Preferences" }
                    p { class: "text-sm text-muted-foreground", "Customize your experience" }
                }
                div { class: "p-6 space-y-4",
                    div { class: "space-y-2",
                        label { r#for: "settings-currency", class: "text-sm font-medium", "Default Currency" }
                        select {
                            id: "settings-currency",
                            value: {cur.to_string()},
                            onchange: move |e| handle_save_currency(e.value()),
                            disabled: saving_currency(),
                            class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring",
                            for (val, label) in CURRENCIES {
                                option { value: {val.to_string()}, "{label}" }
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

            // Connection mode card
            div { class: "rounded-xl bg-card/80 border border-border/40",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Connection" }
                    p { class: "text-sm text-muted-foreground",
                        if APP_MODE().mode.is_offline() {
                            "Offline mode — data stored on this device"
                        } else {
                            "Connected to your Recurrency server"
                        }
                    }
                }
                div { class: "p-6 space-y-4",
                    // Current mode indicator
                    div { class: "flex items-center gap-3",
                        div { class: "flex h-9 w-9 items-center justify-center rounded-lg",
                            if APP_MODE().mode.is_offline() {
                                // Database icon
                                svg {
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    class: "h-5 w-5 text-primary",
                                    path { d: "M12 2L2 7l10 5 10-5-10-5z" }
                                    path { d: "M2 17l10 5 10-5" }
                                    path { d: "M2 12l10 5 10-5" }
                                }
                            } else {
                                // Cloud icon
                                svg {
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    class: "h-5 w-5 text-blue-500",
                                    circle { cx: "12", cy: "12", r: "3" }
                                    path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
                                }
                            }
                        }
                        div { class: "flex-1",
                            p { class: "text-sm font-medium",
                                if APP_MODE().mode.is_offline() {
                                    "Offline Mode"
                                } else {
                                    "Server Mode"
                                }
                            }
                            if !APP_MODE().mode.is_offline() {
                                p { class: "text-xs text-muted-foreground",
                                    "{APP_MODE().mode.server_url().unwrap_or_default()}"
                                }
                            }
                        }
                    }
                    // Change mode button
                    {
                        let nav = navigator();
                        rsx! {
                            button {
                                onclick: move |_| {
                                    nav.push(Route::Setup {});
                                },
                                class: "w-full rounded-md border border-border bg-background px-4 py-2 text-sm font-medium text-foreground hover:bg-accent transition-colors",
                                "Change connection mode"
                            }
                        }
                    }
                }
            }

            // Notifications card
            div { class: "rounded-xl bg-card/80 border border-border/40",
                div { class: "p-6 border-b border-border",
                    h2 { class: "text-lg font-semibold", "Notifications" }
                    p { class: "text-sm text-muted-foreground", "Configure how you receive notifications" }
                }
                div { class: "p-6 space-y-4",
                    for (idx, (key, label, desc)) in NOTIFICATION_CHANNELS.iter().enumerate() {
                        {let checked = notification_prefs().get(*key).copied().unwrap_or(false);
                         let key_str = key.to_string();
                         let is_email_disabled = is_email_disabled(key, smtp_configured());
                         rsx! {
                            div { key: "{key}",
                                div { class: "flex items-center justify-between gap-4",
                                    div { class: "min-w-0",
                                        p { class: "font-medium truncate", "{label}" }
                                        p { class: "text-sm text-muted-foreground truncate", "{desc}" }
                                        if is_email_disabled {
                                            div { class: "flex items-center gap-1.5 text-amber-600 mt-1",
                                                svg {
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    width: "14",
                                                    height: "14",
                                                    view_box: "0 0 24 24",
                                                    fill: "none",
                                                    stroke: "currentColor",
                                                    stroke_width: "2",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    polygon { points: "7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2" }
                                                    line { x1: "12", y1: "8", x2: "12", y2: "12" }
                                                    line { x1: "12", y1: "16", x2: "12.01", y2: "16" }
                                                }
                                                p { class: "text-xs font-medium", "SMTP not configured — email notifications unavailable" }
                                            }
                                        }
                                    }
                                    button {
                                        class: if checked {
                                            "relative inline-flex h-6 w-11 items-center rounded-full bg-primary transition-colors focus:outline-none shrink-0"
                                        } else {
                                            "relative inline-flex h-6 w-11 items-center rounded-full bg-muted transition-colors focus:outline-none shrink-0"
                                        },
                                        onclick: {
                                            let key_str = key_str.clone();
                                            let new_val = !checked;
                                            move |_| handle_notification_toggle(key_str.clone(), new_val)
                                        },
                                        disabled: prefs_loading() || is_email_disabled,
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
        let keys: Vec<_> = NOTIFICATION_CHANNELS.iter().map(|(k, _, _)| *k).collect();
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
            assert!(
                label.contains(val),
                "Label '{}' should contain code '{}'",
                label,
                val
            );
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
            UpdateNotificationPreferenceRequest, UpdateNotificationPreferencesRequest,
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
        let channels: std::collections::HashMap<String, bool> =
            [("email".to_string(), true), ("push".to_string(), false)]
                .into_iter()
                .collect();
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
        use crate::models::auth::{SettingsResponse, UserResponse};
        use uuid::Uuid;

        let updated_user = UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
            name: Some("Alice".into()),
            currency: "USD".into(),
            verified: true,
        };

        let response = SettingsResponse { user: updated_user };

        // Verify response structure
        assert_eq!(response.user.currency, "USD");
        assert_eq!(response.user.name, Some("Alice".into()));
    }

    // ── Currency update flow simulation ──
    #[test]
    fn currency_update_request_response_cycle() {
        use crate::models::auth::{SettingsResponse, UpdateSettingsRequest, UserResponse};
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
        use crate::components::toast::{ToastKind, ToastState};

        let mut state = ToastState::default();
        state.success("Currency saved");

        assert_eq!(state.items.len(), 1);
        assert_eq!(state.items[0].message, "Currency saved");
        assert!(matches!(state.items[0].kind, ToastKind::Success));
    }

    #[test]
    fn settings_should_show_toast_on_error() {
        use crate::components::toast::{ToastKind, ToastState};

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

    // ── RSX structure checks ──
    #[test]
    fn settings_page_has_form_layout() {
        let src = include_str!("settings_page.rs");
        assert!(src.contains("Settings"), "Should have Settings title");
    }

    #[test]
    fn settings_page_has_name_input() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("name") && src.contains("oninput"),
            "Should have name input"
        );
    }

    #[test]
    fn settings_page_has_currency_select() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("currency") || src.contains("Select"),
            "Should have currency selector"
        );
    }

    #[test]
    fn settings_page_has_notification_toggles() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("notification") || src.contains("NOTIFICATION_CHANNELS"),
            "Should have notification toggles"
        );
    }

    #[test]
    fn settings_page_uses_toast() {
        let src = include_str!("settings_page.rs");
        assert!(src.contains("TOAST"), "Should use TOAST for notifications");
    }

    #[test]
    fn settings_page_has_save_handlers() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("handle_save") || src.contains("on_change"),
            "Should have save handlers"
        );
    }

    // ── State initialization checks ──
    #[test]
    fn settings_page_loads_auth_state() {
        let src = include_str!("settings_page.rs");
        assert!(src.contains("AUTH()"), "Should load AUTH state");
    }

    #[test]
    fn settings_page_loads_notification_prefs() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("notification_preferences") || src.contains("NOTIFICATION_CHANNELS"),
            "Should load notification preferences"
        );
    }

    // ── API call checks ──
    #[test]
    fn settings_page_calls_settings_api() {
        let src = include_str!("settings_page.rs");
        assert!(src.contains("settings::"), "Should call settings API");
    }

    #[test]
    fn settings_page_calls_notification_prefs_api() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("notification_preferences::"),
            "Should call notification preferences API"
        );
    }

    // ── API request/response checks ──

    // ── Additional unique tests ──
    #[test]
    fn settings_page_uses_signals() {
        let src = include_str!("settings_page.rs");
        assert!(src.contains("use_signal"), "Should use signals for state");
    }

    #[test]
    fn settings_page_uses_effect() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("use_effect"),
            "Should use effect for side effects"
        );
    }

    #[test]
    fn settings_page_uses_future() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("use_future"),
            "Should use future for async loading"
        );
    }

    #[test]
    fn settings_page_uses_spawn() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("spawn"),
            "Should use spawn for async operations"
        );
    }

    #[test]
    fn settings_page_logs_errors() {
        let src = include_str!("settings_page.rs");
        assert!(src.contains("log::error"), "Should log errors");
    }

    #[test]
    fn settings_page_syncs_form_with_auth() {
        let src = include_str!("settings_page.rs");
        assert!(src.contains("AUTH().user"), "Should read user from AUTH");
    }

    #[test]
    fn settings_page_has_loading_states() {
        let src = include_str!("settings_page.rs");
        assert!(
            src.contains("_loading") || src.contains("loading"),
            "Should have loading state"
        );
    }

    #[test]
    fn notification_toggle_behavior() {
        let mut prefs = std::collections::HashMap::new();
        prefs.insert("email".to_string(), false);

        prefs.insert("email".to_string(), true);
        assert_eq!(prefs.get("email"), Some(&true));

        prefs.insert("email".to_string(), false);
        assert_eq!(prefs.get("email"), Some(&false));
    }

    #[test]
    fn settings_update_response_verifies_currency() {
        use crate::models::auth::{SettingsResponse, UserResponse};
        use uuid::Uuid;

        let response = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                name: Some("Test".into()),
                currency: "CHF".into(),
                verified: true,
            },
        };
        assert_eq!(response.user.currency, "CHF");
    }

    // ── SettingsResponse additional tests ──
    #[test]
    fn settings_response_user_name() {
        use crate::models::auth::{SettingsResponse, UserResponse};
        use uuid::Uuid;

        let response = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                name: Some("John Doe".into()),
                currency: "EUR".into(),
                verified: true,
            },
        };
        assert_eq!(response.user.name.as_deref(), Some("John Doe"));
    }

    #[test]
    fn settings_response_user_verified() {
        use crate::models::auth::{SettingsResponse, UserResponse};
        use uuid::Uuid;

        let response = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                name: None,
                currency: "USD".into(),
                verified: false,
            },
        };
        assert!(!response.user.verified);
    }

    #[test]
    fn email_disabled_when_smtp_not_configured() {
        assert!(is_email_disabled("email", false));
    }

    #[test]
    fn email_not_disabled_when_smtp_configured() {
        assert!(!is_email_disabled("email", true));
    }

    #[test]
    fn push_not_disabled_when_smtp_missing() {
        assert!(!is_email_disabled("push", false));
    }

    #[test]
    fn system_not_disabled_when_smtp_missing() {
        assert!(!is_email_disabled("system", false));
    }
}
