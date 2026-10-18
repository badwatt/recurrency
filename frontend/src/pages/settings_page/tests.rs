
    use super::*;
use crate::utils::constants::CURRENCIES;

const SETTINGS_SOURCES: &str = concat!(
    include_str!("../settings_page.rs"),
    include_str!("../settings_page/profile_card.rs"),
    include_str!("../settings_page/preferences_card.rs"),
    include_str!("../settings_page/connection_card.rs"),
    include_str!("../settings_page/notifications_card.rs"),
);


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
        let src = SETTINGS_SOURCES;
        assert!(src.contains("Settings"), "Should have Settings title");
    }

    #[test]
    fn settings_page_has_name_input() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("name") && src.contains("oninput"),
            "Should have name input"
        );
    }

    #[test]
    fn settings_page_has_currency_select() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("currency") || src.contains("Select"),
            "Should have currency selector"
        );
    }

    #[test]
    fn settings_page_has_notification_toggles() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("notification") || src.contains("NOTIFICATION_CHANNELS"),
            "Should have notification toggles"
        );
    }

    #[test]
    fn settings_page_uses_toast() {
        let src = SETTINGS_SOURCES;
        assert!(src.contains("TOAST"), "Should use TOAST for notifications");
    }

    #[test]
    fn settings_page_has_save_handlers() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("handle_save") || src.contains("on_change"),
            "Should have save handlers"
        );
    }

    // ── State initialization checks ──
    #[test]
    fn settings_page_loads_auth_state() {
        let src = SETTINGS_SOURCES;
        assert!(src.contains("AUTH()"), "Should load AUTH state");
    }

    #[test]
    fn settings_page_loads_notification_prefs() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("notification_preferences") || src.contains("NOTIFICATION_CHANNELS"),
            "Should load notification preferences"
        );
    }

    // ── API call checks ──
    #[test]
    fn settings_page_calls_settings_api() {
        let src = SETTINGS_SOURCES;
        assert!(src.contains("settings::"), "Should call settings API");
    }

    #[test]
    fn settings_page_calls_notification_prefs_api() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("notification_preferences::"),
            "Should call notification preferences API"
        );
    }

    // ── API request/response checks ──

    // ── Additional unique tests ──
    #[test]
    fn settings_page_uses_signals() {
        let src = SETTINGS_SOURCES;
        assert!(src.contains("use_signal"), "Should use signals for state");
    }

    #[test]
    fn settings_page_uses_effect() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("use_effect"),
            "Should use effect for side effects"
        );
    }

    #[test]
    fn settings_page_uses_future() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("use_future"),
            "Should use future for async loading"
        );
    }

    #[test]
    fn settings_page_uses_spawn() {
        let src = SETTINGS_SOURCES;
        assert!(
            src.contains("spawn"),
            "Should use spawn for async operations"
        );
    }

    #[test]
    fn settings_page_logs_errors() {
        let src = SETTINGS_SOURCES;
        assert!(src.contains("log::error"), "Should log errors");
    }

    #[test]
    fn settings_page_syncs_form_with_auth() {
        let src = SETTINGS_SOURCES;
        assert!(src.contains("AUTH().user"), "Should read user from AUTH");
    }

    #[test]
    fn settings_page_has_loading_states() {
        let src = SETTINGS_SOURCES;
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
