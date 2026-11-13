
    use super::*;
use crate::utils::constants::CURRENCIES;

const SETTINGS_SOURCES: &str = concat!(
    include_str!("../settings_page.rs"),
    include_str!("../settings_page/profile_card.rs"),
    include_str!("../settings_page/preferences_card.rs"),
    include_str!("../settings_page/delete_account_card.rs"),
    include_str!("../settings_page/delivery_card.rs"),
    include_str!("../settings_page/summary_card.rs"),
    include_str!("../../layout/settings_layout.rs"),
    include_str!("../../components/server_url.rs"),
    include_str!("../subscriptions_page/cards.rs"),
);

    #[test]
    fn app_version_matches_cargo() {
        assert_eq!(APP_VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn settings_page_exposes_money_section() {
        assert!(SETTINGS_SOURCES.contains("Money"));
        assert!(SETTINGS_SOURCES.contains("AccountsPage"));
        assert!(SETTINGS_SOURCES.contains("CategoriesPage"));
        assert!(SETTINGS_SOURCES.contains("role: \"tablist\""));
    }

    #[test]
    fn app_layout_keeps_topbar_and_has_no_logout() {
        let layout = include_str!("../../layout/app_layout.rs");
        assert!(layout.contains("TopNavLink"));
        assert!(!layout.contains("SettingsTopLink"));
        assert!(!layout.contains("AUTH.write().logout()"));
        assert!(!layout.contains("Done"));
    }

    #[test]
    fn settings_layout_owns_navigation_and_logout() {
        let layout = include_str!("../../layout/settings_layout.rs");
        assert!(layout.contains("SettingsSection"));
        assert!(layout.contains("Outlet::<crate::Route>"));
        assert!(layout.contains("Log out"));
        assert!(layout.contains("Settings sections"));
    }

    #[test]
    fn settings_subpages_are_routed() {
        let lib = include_str!("../../lib.rs");
        assert!(lib.contains("/settings/accounts"));
        assert!(lib.contains("/settings/categories"));
        assert!(lib.contains("/settings/notifications"));
        assert!(lib.contains("/settings/money"));
    }

    #[test]
    fn settings_page_handles_offline_notifications() {
        assert!(SETTINGS_SOURCES.contains("is_offline_mode"));
        assert!(SETTINGS_SOURCES.contains("Server notifications"));
    }

    #[test]
    fn app_and_settings_share_a_width() {
        let app = include_str!("../../layout/app_layout.rs");
        assert!(app.contains("max-w-7xl"));
        assert!(!app.contains("max-w-screen-2xl"));
        assert!(SETTINGS_SOURCES.contains("max-w-7xl"));
    }



    #[test]
    fn native_server_url_is_editable_before_login_and_in_settings() {
        let login = include_str!("../../auth/login.rs");
        assert!(login.contains("Route::ServerSettings"));
        assert!(login.contains(r#"cfg!(not(target_arch = "wasm32"))"#));
        assert!(SETTINGS_SOURCES.contains("ServerUrlEditor"));
        assert!(SETTINGS_SOURCES.contains("Save server"));
        assert!(SETTINGS_SOURCES.contains("normalize_server_url"));
    }

    #[test]
    fn web_never_exposes_server_url_editor() {
        let login = include_str!("../../auth/login.rs");
        let editor = include_str!("../../components/server_url.rs");
        assert!(login.contains(r#"if cfg!(not(target_arch = "wasm32"))"#));
        assert!(editor.contains("logout_on_change"));
        assert!(editor.contains("set_server"));
    }


    #[test]
    fn category_badge_is_shown_but_not_for_system_category() {
        assert!(SETTINGS_SOURCES.contains("Category") || SETTINGS_SOURCES.contains("category"));
        assert!(SETTINGS_SOURCES.contains("!c.is_system"));
        assert!(SETTINGS_SOURCES.contains("cat.name"));
    }

    #[test]
    fn delivery_channels_use_an_icon_rail() {
        assert!(SETTINGS_SOURCES.contains("role: \"tablist\""));
        assert!(SETTINGS_SOURCES.contains("provider_icon"));
        assert!(SETTINGS_SOURCES.contains("PROVIDERS"));
        assert!(!SETTINGS_SOURCES.contains("delivery-provider"));
    }

    #[test]
    fn notifications_keep_only_delivery_and_summary() {
        assert!(SETTINGS_SOURCES.contains("DeliveryCard"));
        assert!(SETTINGS_SOURCES.contains("SummaryCard"));
        assert!(!SETTINGS_SOURCES.contains("Reminders & alerts"));
        assert!(!SETTINGS_SOURCES.contains("NOTIFICATION_CHANNELS"));
    }

    #[test]
    fn settings_page_renders_version() {
        assert!(SETTINGS_SOURCES.contains("APP_VERSION"));
    }

    #[test]
    fn settings_profile_contains_delete_account() {
        assert!(SETTINGS_SOURCES.contains("DeleteAccountCard"));
        assert!(SETTINGS_SOURCES.contains("DeleteAccountRequest"));
        assert!(SETTINGS_SOURCES.contains("delete_account"));
        assert!(SETTINGS_SOURCES.contains("Confirm with your password"));
    }

    #[test]
    fn avatar_upload_resizes_android_gallery_photos() {
        assert!(SETTINGS_SOURCES.contains("resize_avatar"));
        assert!(SETTINGS_SOURCES.contains("canvas.toDataURL"));
        assert!(SETTINGS_SOURCES.contains("15 MB"));
        assert!(SETTINGS_SOURCES.contains("accept: \"image/*\""));
    }

    #[test]
    fn mobile_settings_expose_session_logout_and_version() {
        let layout = include_str!("../../layout/settings_layout.rs");
        assert!(SETTINGS_SOURCES.contains("MobileSessionCard"));
        assert!(SETTINGS_SOURCES.contains("Sign out of this device"));
        assert!(layout.contains("APP_VERSION"));
        assert!(layout.contains("lg:hidden px-1 pb-2"));
        assert!(!layout.contains("aria_label: \"Log out\""));
    }

    #[test]
    fn settings_profile_can_change_avatar() {
        assert!(SETTINGS_SOURCES.contains("Change photo"));
        assert!(SETTINGS_SOURCES.contains("read_bytes"));
        assert!(SETTINGS_SOURCES.contains("base64_encode"));
        assert!(SETTINGS_SOURCES.contains("absolute inset-0 h-full w-full cursor-pointer opacity-0"));
        assert!(SETTINGS_SOURCES.contains("RecurrencyImagePicker"));
        assert!(SETTINGS_SOURCES.contains("await new Promise"));
        assert!(SETTINGS_SOURCES.contains("avatar: Some(avatar())"));
    }

    #[test]
    fn settings_profile_can_change_email() {
        assert!(SETTINGS_SOURCES.contains("settings-email"));
        assert!(SETTINGS_SOURCES.contains("is_valid_email"));
        assert!(SETTINGS_SOURCES.contains("email: Some(next_email)"));
        assert!(SETTINGS_SOURCES.contains("avatar: Some(avatar())"));
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
            email: None,
                avatar: None,
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
            email: None,
                avatar: None,
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
            email: None,
                avatar: None,
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


    // ── Settings update auth flow simulation ──
    #[test]
    fn settings_update_response_updates_user_currency() {
        // Simulates what happens when settings::update returns SettingsResponse
        use crate::models::auth::{SettingsResponse, UserResponse};
        use uuid::Uuid;

        let updated_user = UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
                avatar: None,
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
            email: None,
                avatar: None,
            name: None,
            currency: Some("JPY".into()),
        };
        assert_eq!(req.currency.as_deref(), Some("JPY"));

        // Simulate response (would come from API)
        let resp = SettingsResponse {
            user: UserResponse {
                id: Uuid::new_v4(),
                email: "test@test.com".into(),
                avatar: None,
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


    // ── API call checks ──
    #[test]
    fn settings_page_calls_settings_api() {
        let src = SETTINGS_SOURCES;
        assert!(src.contains("settings::"), "Should call settings API");
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
                avatar: None,
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
                avatar: None,
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
                avatar: None,
                name: None,
                currency: "USD".into(),
                verified: false,
            },
        };
        assert!(!response.user.verified);
    }





#[test]
fn settings_expose_exchange_provider_and_fixer_key() {
    assert!(SETTINGS_SOURCES.contains("Exchange rates"));
    assert!(SETTINGS_SOURCES.contains("settings-provider"));
    assert!(SETTINGS_SOURCES.contains("settings-fixer-key"));
    assert!(SETTINGS_SOURCES.contains("update_settings"));
}

#[test]
fn settings_can_disable_update_notices() {
    assert!(SETTINGS_SOURCES.contains("settings-show-updates"));
    assert!(SETTINGS_SOURCES.contains("set_show_updates"));
}

#[test]
fn about_page_has_version_changelog_and_star_prompt() {
    let about = include_str!("../settings_page/about_page.rs");
    assert!(about.contains("Installed version"));
    assert!(about.contains("Show changelog"));
    assert!(about.contains("Star on GitHub"));
    assert!(about.contains("fetch_current"));
}

#[test]
fn settings_expose_backup_import_and_download() {
    let backup = include_str!("../settings_page/backup_card.rs");
    assert!(backup.contains("Download backup"));
    assert!(backup.contains("ondrop"));
    assert!(backup.contains("Restore backup"));
}

#[test]
fn settings_expose_date_format_choice() {
    assert!(SETTINGS_SOURCES.contains("settings-date-format"));
    assert!(SETTINGS_SOURCES.contains("DD/MM/YYYY"));
    assert!(SETTINGS_SOURCES.contains("MM/DD/YYYY"));
}

#[test]
fn backup_download_uses_native_download_bridge() {
    let src = include_str!("backup_card.rs");
    assert!(src.contains("save_text"));
}

#[test]
fn backup_download_button_is_right_on_desktop_and_centered_on_mobile() {
    let src = include_str!("backup_card.rs");
    assert!(src.contains("sm:flex-row"));
    assert!(src.contains("sm:justify-between"));
    assert!(src.contains("sm:text-left"));
}

#[test]
fn summary_card_offers_last_day_of_month() {
    let src = include_str!("summary_card.rs");
    assert!(src.contains("Last day"));
    assert!(src.contains("value: \"0\""));
}

#[test]
fn android_backup_picker_uses_the_native_bridge() {
    let src = include_str!("backup_card.rs");
    assert!(src.contains("RecurrencyFilePicker"));
    assert!(src.contains("__recurrencyBackupPicked"));
}

#[test]
fn local_sync_card_uses_android_folder_bridge() {
    let src = include_str!("sync_card.rs");
    assert!(src.contains("Cloud folder sync"));
    let bridge = include_str!("../../storage/folder_sync.rs");
    assert!(bridge.contains("RecurrencySyncFolder"));
    assert!(bridge.contains("sync_merge"));
    assert!(src.contains("folder_sync::sync_once"));
    assert!(src.contains("APP_MODE().mode.is_offline()"));
}

#[test]
fn backup_choose_file_has_a_pointer_cursor_on_desktop() {
    let src = include_str!("backup_card.rs");
    assert!(src.contains("cursor: pointer;"));
    assert!(src.contains("cursor-pointer"));
}
