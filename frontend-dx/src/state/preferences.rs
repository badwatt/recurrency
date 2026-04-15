use dioxus::prelude::*;
use crate::models::preferences::PreferencesResponse;

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

    pub fn set_loading(&mut self) {
        self.is_loading = true;
    }

    pub fn set_data(&mut self, prefs: PreferencesResponse) {
        self.preferences = prefs;
        self.is_loading = false;
        self.initialized = true;
    }

    pub fn set_theme(&mut self, theme: String) {
        self.preferences.theme = theme;
    }

    pub fn set_subscriptions_view(&mut self, view: String) {
        self.preferences.subscriptions_view = view;
    }
}

pub static PREFERENCES: GlobalSignal<PreferencesState> = Signal::global(|| PreferencesState::default());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_system_grid() {
        let state = PreferencesState::default();
        assert_eq!(state.theme(), "system");
        assert_eq!(state.subscriptions_view(), "grid");
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
    fn set_data_marks_initialized() {
        let mut state = PreferencesState::default();
        state.set_data(PreferencesResponse {
            theme: "mocha".into(),
            subscriptions_view: "list".into(),
        });
        assert!(state.initialized);
        assert_eq!(state.theme(), "mocha");
    }
}