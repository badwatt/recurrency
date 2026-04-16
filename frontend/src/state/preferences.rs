use dioxus::prelude::*;
use crate::models::preferences::PreferencesResponse;

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

    /// Persist theme to localStorage.
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
    }

    /// Hydrate theme from localStorage on startup.
    pub fn hydrate_from_storage(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                if let Ok(Some(theme)) = storage.get_item(THEME_KEY) {
                    // Only update if theme is valid
                    if ["light", "dark", "mocha", "system"].contains(&theme.as_str()) {
                        self.preferences.theme = theme;
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // No localStorage available outside wasm32
        }
    }
}

pub static PREFERENCES: GlobalSignal<PreferencesState> = Signal::global(|| PreferencesState::default());

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
}