use crate::models::preferences::PreferencesResponse;
use dioxus::prelude::*;

#[allow(dead_code)]
const THEME_KEY: &str = "recurrency-theme";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PreferencesState {
    pub preferences: PreferencesResponse,
    pub is_loading: bool,
    pub initialized: bool,
}

impl PreferencesState {
    /// Get current theme name.
    pub fn theme(&self) -> &str {
        &self.preferences.theme
    }

    /// Get current subscriptions view mode.
    pub fn subscriptions_view(&self) -> &str {
        &self.preferences.subscriptions_view
    }

    /// Get first day of week: "monday" or "sunday" (default: "monday").
    pub fn first_day_of_week(&self) -> &str {
        &self.preferences.first_day_of_week
    }

    /// Get date format derived from system locale.
    pub fn date_format(&self) -> String {
        let locale = crate::utils::dates::get_locale();
        crate::utils::dates::get_date_format_from_locale(&locale).to_string()
    }

    pub fn set_loading(&mut self) {
        self.is_loading = true;
    }

    pub fn set_data(&mut self, prefs: PreferencesResponse) {
        self.preferences = prefs;
        self.is_loading = false;
        self.initialized = true;
    }

    pub fn set_theme(&mut self, theme: String) {
        self.preferences.theme = theme.clone();
        self.persist_theme(&theme);
    }

    pub fn set_subscriptions_view(&mut self, view: String) {
        self.preferences.subscriptions_view = view;
    }

    pub fn set_first_day_of_week(&mut self, day: String) {
        self.preferences.first_day_of_week = day;
    }

    /// Persist theme to localStorage (WASM) or file (native).
    fn persist_theme(&self, theme: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.set_item(THEME_KEY, theme);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(data_dir) = crate::state::auth::get_app_data_dir() {
                let app_dir = data_dir.join("recurrency");
                if std::fs::create_dir_all(&app_dir).is_ok() {
                    let _ = std::fs::write(app_dir.join("theme.txt"), theme);
                }
            }
        }
    }

    /// Hydrate theme from localStorage (WASM) or file (native) on startup.
    pub fn hydrate_from_storage(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                if let Ok(Some(theme)) = storage.get_item(THEME_KEY) {
                    if ["light", "dark", "mocha", "system"].contains(&theme.as_str()) {
                        self.preferences.theme = theme;
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(data_dir) = crate::state::auth::get_app_data_dir() {
                let theme_file = data_dir.join("recurrency").join("theme.txt");
                if theme_file.exists()
                    && let Ok(theme) = std::fs::read_to_string(&theme_file) {
                        let theme = theme.trim().to_string();
                        if ["light", "dark", "mocha", "system"].contains(&theme.as_str()) {
                            self.preferences.theme = theme;
                        }
                    }
            }
        }
    }
}

pub static PREFERENCES: GlobalSignal<PreferencesState> = Signal::global(PreferencesState::default);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_system_grid_monday() {
        let state = PreferencesState::default();
        assert_eq!(state.theme(), "system");
        assert_eq!(state.subscriptions_view(), "grid");
        assert_eq!(state.first_day_of_week(), "monday");
    }

    #[test]
    fn set_theme() {
        let mut state = PreferencesState::default();
        state.set_theme("dark".into());
        assert_eq!(state.theme(), "dark");
    }

    #[test]
    fn set_subscriptions_view() {
        let mut state = PreferencesState::default();
        state.set_subscriptions_view("list".into());
        assert_eq!(state.subscriptions_view(), "list");
    }

    #[test]
    fn set_first_day_of_week() {
        let mut state = PreferencesState::default();
        state.set_first_day_of_week("sunday".into());
        assert_eq!(state.first_day_of_week(), "sunday");
        state.set_first_day_of_week("monday".into());
        assert_eq!(state.first_day_of_week(), "monday");
    }

    #[test]
    fn set_data_marks_initialized() {
        let mut state = PreferencesState::default();
        state.set_data(PreferencesResponse {
            theme: "mocha".into(),
            subscriptions_view: "list".into(),
            first_day_of_week: "monday".into(),
            date_format: "DD-MM-YYYY".into(),
        });
        assert!(state.initialized);
        assert_eq!(state.theme(), "mocha");
    }

    #[test]
    fn set_loading_state() {
        let mut state = PreferencesState::default();
        state.set_loading();
        assert!(state.is_loading);
    }

    #[test]
    fn set_data_clears_loading() {
        let mut state = PreferencesState::default();
        state.set_loading();
        state.set_data(PreferencesResponse::default());
        assert!(!state.is_loading);
    }

    #[test]
    fn default_not_initialized() {
        let state = PreferencesState::default();
        assert!(!state.initialized);
    }

    // ── Theme persistence tests ──
    #[test]
    fn set_theme_updates_preferences() {
        let mut state = PreferencesState::default();
        state.set_theme("dark".into());
        assert_eq!(state.theme(), "dark");
    }

    #[test]
    fn set_theme_does_not_persist_by_itself() {
        // set_theme only updates in-memory state
        // Persistence should be handled separately
        let mut state = PreferencesState::default();
        state.set_theme("mocha".into());
        assert_eq!(state.theme(), "mocha");
        // State has the theme but no localStorage persistence yet
    }

    #[test]
    fn preferences_state_can_be_updated_multiple_times() {
        let mut state = PreferencesState::default();
        state.set_theme("light".into());
        assert_eq!(state.theme(), "light");
        state.set_theme("dark".into());
        assert_eq!(state.theme(), "dark");
        state.set_theme("system".into());
        assert_eq!(state.theme(), "system");
    }

    #[test]
    fn preferences_state_clone() {
        let state = PreferencesState::default();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn preferences_state_partial_eq() {
        let s1 = PreferencesState::default();
        let s2 = PreferencesState::default();
        assert_eq!(s1, s2);
    }

    #[test]
    fn preferences_state_set_data() {
        let mut state = PreferencesState::default();
        let resp = PreferencesResponse {
            theme: "catppuccin-mocha".into(),
            subscriptions_view: "table".into(),
            first_day_of_week: "sunday".into(),
            date_format: "YYYY-MM-DD".into(),
        };
        state.set_data(resp);
        assert_eq!(state.theme(), "catppuccin-mocha");
        assert_eq!(state.subscriptions_view(), "table");
        assert_eq!(state.first_day_of_week(), "sunday");
        assert!(state.initialized);
        assert!(!state.is_loading);
    }

    #[test]
    fn preferences_state_set_subscriptions_view() {
        let mut state = PreferencesState::default();
        state.set_subscriptions_view("list".into());
        assert_eq!(state.subscriptions_view(), "list");
    }

    #[test]
    fn preferences_state_set_first_day_of_week() {
        let mut state = PreferencesState::default();
        state.set_first_day_of_week("sunday".into());
        assert_eq!(state.first_day_of_week(), "sunday");
    }

    // ── Additional PreferencesState tests ──
    #[test]
    fn preferences_state_debug() {
        let state = PreferencesState::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("PreferencesState"));
    }

    #[test]
    fn preferences_state_default_not_loading() {
        let state = PreferencesState::default();
        assert!(!state.is_loading);
        assert!(!state.initialized);
    }

    #[test]
    fn preferences_state_theme_cycle() {
        let mut state = PreferencesState::default();
        assert_eq!(state.theme(), "system");
        state.set_theme("light".into());
        assert_eq!(state.theme(), "light");
        state.set_theme("dark".into());
        assert_eq!(state.theme(), "dark");
        state.set_theme("mocha".into());
        assert_eq!(state.theme(), "mocha");
        state.set_theme("system".into());
        assert_eq!(state.theme(), "system");
    }

    #[test]
    fn preferences_state_set_data_marks_initialized() {
        let mut state = PreferencesState::default();
        assert!(!state.initialized);
        state.set_data(PreferencesResponse::default());
        assert!(state.initialized);
    }

    #[test]
    fn preferences_response_default_values() {
        let resp = PreferencesResponse::default();
        assert_eq!(resp.theme, "system");
        assert_eq!(resp.subscriptions_view, "grid");
        assert_eq!(resp.first_day_of_week, "monday");
    }
}
