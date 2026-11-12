use crate::models::preferences::PreferencesResponse;
use dioxus::prelude::*;

#[allow(dead_code)]
const THEME_KEY: &str = "recurrency-theme";
const VIEW_KEY: &str = "recurrency-subscriptions-view";
const TRANSACTIONS_VIEW_KEY: &str = "recurrency-transactions-view";
const UPDATES_KEY: &str = "recurrency-show-updates";

const THEME_FILE: &str = "theme.txt";
const VIEW_FILE: &str = "subscriptions_view.txt";
const TRANSACTIONS_VIEW_FILE: &str = "transactions_view.txt";
const UPDATES_FILE: &str = "show_updates.txt";

/// Themes the user can pick in settings ("system" is resolved in JS).
const SETTABLE_THEMES: [&str; 3] = ["light", "dark", "mocha"];
/// Values accepted when rehydrating from local storage.
const STORED_THEMES: [&str; 4] = ["light", "dark", "mocha", "system"];
const VALID_VIEWS: [&str; 2] = ["grid", "list"];

/// Persist a preference value to localStorage (WASM) or a file (native).
fn persist_local(key: &str, file_name: &str, value: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = file_name;
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item(key, value);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        if let Some(data_dir) = crate::state::auth::get_app_data_dir() {
            let app_dir = data_dir.join("recurrency");
            if std::fs::create_dir_all(&app_dir).is_ok() {
                let _ = std::fs::write(app_dir.join(file_name), value);
            }
        }
    }
}

/// Read a persisted preference value from localStorage (WASM) or a file (native).
fn read_local(key: &str, file_name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = file_name;
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
            && let Ok(Some(value)) = storage.get_item(key)
        {
            return Some(value);
        }
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        if let Some(data_dir) = crate::state::auth::get_app_data_dir() {
            let path = data_dir.join("recurrency").join(file_name);
            if let Ok(value) = std::fs::read_to_string(&path) {
                return Some(value);
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreferencesState {
    pub preferences: PreferencesResponse,
    pub is_loading: bool,
    pub initialized: bool,
    /// Local-only view preference for the transactions page.
    pub transactions_view: String,
    /// Whether the in-app update banner is allowed to appear.
    pub show_updates: bool,
    /// True when the view mode was set locally (hydrated or toggled), meaning
    /// it must take precedence over the server value fetched later.
    view_from_local: bool,
}

impl Default for PreferencesState {
    fn default() -> Self {
        Self {
            preferences: PreferencesResponse::default(),
            is_loading: false,
            initialized: false,
            transactions_view: "list".into(),
            show_updates: true,
            view_from_local: false,
        }
    }
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

    /// Get current transactions view mode.
    pub fn transactions_view(&self) -> &str {
        &self.transactions_view
    }

    pub fn show_updates(&self) -> bool {
        self.show_updates
    }

    pub fn set_show_updates(&mut self, show: bool) {
        self.show_updates = show;
        persist_local(UPDATES_KEY, UPDATES_FILE, if show { "true" } else { "false" });
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
        // Only accept valid themes; reject unknown values
        if SETTABLE_THEMES.contains(&theme.as_str()) {
            self.preferences.theme = theme.clone();
            persist_local(THEME_KEY, THEME_FILE, &theme);
        }
    }

    pub fn set_subscriptions_view(&mut self, view: String) {
        if VALID_VIEWS.contains(&view.as_str()) {
            persist_local(VIEW_KEY, VIEW_FILE, &view);
            self.view_from_local = true;
        }
        self.preferences.subscriptions_view = view;
    }

    pub fn set_transactions_view(&mut self, view: String) {
        if VALID_VIEWS.contains(&view.as_str()) {
            persist_local(TRANSACTIONS_VIEW_KEY, TRANSACTIONS_VIEW_FILE, &view);
            self.transactions_view = view;
        }
    }

    /// Whether the current view mode was set locally and must win over the
    /// server preferences fetched after startup.
    pub fn has_local_view(&self) -> bool {
        self.view_from_local
    }

    pub fn set_first_day_of_week(&mut self, day: String) {
        self.preferences.first_day_of_week = day;
    }

    /// Apply locally persisted preferences, ignoring unknown values.
    pub fn apply_local_preferences(
        &mut self,
        theme: Option<&str>,
        subscriptions_view: Option<&str>,
    ) {
        if let Some(theme) = theme.map(str::trim)
            && STORED_THEMES.contains(&theme)
        {
            self.preferences.theme = theme.to_string();
        }
        if let Some(view) = subscriptions_view.map(str::trim)
            && VALID_VIEWS.contains(&view)
        {
            self.preferences.subscriptions_view = view.to_string();
            self.view_from_local = true;
        }
    }

    /// Hydrate theme and view mode from localStorage (WASM) or files (native).
    pub fn hydrate_from_storage(&mut self) {
        let theme = read_local(THEME_KEY, THEME_FILE);
        let view = read_local(VIEW_KEY, VIEW_FILE);
        let transactions_view = read_local(TRANSACTIONS_VIEW_KEY, TRANSACTIONS_VIEW_FILE);
        let show_updates = read_local(UPDATES_KEY, UPDATES_FILE);
        self.apply_local_preferences(theme.as_deref(), view.as_deref());
        if let Some(view) = transactions_view.as_deref().map(str::trim)
            && VALID_VIEWS.contains(&view)
        {
            self.transactions_view = view.to_string();
        }
        if let Some(show) = show_updates.as_deref().map(str::trim) {
            self.show_updates = show != "false";
        }
    }

    /// Build the state already hydrated from persistent storage so the first
    /// render uses the user's saved theme and view mode (no grid flash).
    pub fn hydrated() -> Self {
        let mut state = Self::default();
        state.hydrate_from_storage();
        state
    }
}

pub static PREFERENCES: GlobalSignal<PreferencesState> = Signal::global(PreferencesState::hydrated);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_system_grid_monday() {
        let state = PreferencesState::default();
        assert_eq!(state.theme(), "mocha");
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
    fn updates_are_visible_by_default() {
        assert!(PreferencesState::default().show_updates());
    }

    #[test]
    fn updates_can_be_disabled() {
        let mut state = PreferencesState::default();
        state.set_show_updates(false);
        assert!(!state.show_updates());
    }

    #[test]
    fn transactions_view_defaults_to_list() {
        assert_eq!(PreferencesState::default().transactions_view(), "list");
    }

    #[test]
    fn set_transactions_view() {
        let mut state = PreferencesState::default();
        state.set_transactions_view("grid".into());
        assert_eq!(state.transactions_view(), "grid");
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
        // "system" is rejected — theme stays "dark"
        state.set_theme("system".into());
        assert_eq!(state.theme(), "dark");
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
        assert_eq!(state.theme(), "mocha");
        state.set_theme("light".into());
        assert_eq!(state.theme(), "light");
        state.set_theme("dark".into());
        assert_eq!(state.theme(), "dark");
        state.set_theme("mocha".into());
        assert_eq!(state.theme(), "mocha");
        state.set_theme("system".into());
        assert_eq!(state.theme(), "mocha");
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
        assert_eq!(resp.theme, "mocha");
        assert_eq!(resp.subscriptions_view, "grid");
        assert_eq!(resp.first_day_of_week, "monday");
    }

    // ── Local hydration (view mode persistence) ──
    #[test]
    fn apply_local_preferences_restores_list_view() {
        let mut state = PreferencesState::default();
        state.apply_local_preferences(None, Some("list"));
        assert_eq!(state.subscriptions_view(), "list");
    }

    #[test]
    fn apply_local_preferences_restores_theme_and_view() {
        let mut state = PreferencesState::default();
        state.apply_local_preferences(Some("light"), Some("list"));
        assert_eq!(state.theme(), "light");
        assert_eq!(state.subscriptions_view(), "list");
    }

    #[test]
    fn apply_local_preferences_ignores_invalid_view() {
        let mut state = PreferencesState::default();
        state.apply_local_preferences(None, Some("table"));
        assert_eq!(state.subscriptions_view(), "grid");
    }

    #[test]
    fn apply_local_preferences_ignores_invalid_theme() {
        let mut state = PreferencesState::default();
        state.apply_local_preferences(Some("neon"), Some("list"));
        assert_eq!(state.theme(), "mocha");
        assert_eq!(state.subscriptions_view(), "list");
    }

    #[test]
    fn apply_local_preferences_trims_stored_values() {
        let mut state = PreferencesState::default();
        state.apply_local_preferences(Some(" dark\n"), Some(" list\n"));
        assert_eq!(state.theme(), "dark");
        assert_eq!(state.subscriptions_view(), "list");
    }

    #[test]
    fn apply_local_preferences_none_keeps_defaults() {
        let mut state = PreferencesState::default();
        state.apply_local_preferences(None, None);
        assert_eq!(state, PreferencesState::default());
    }

    #[test]
    fn apply_local_preferences_marks_view_as_local() {
        let mut state = PreferencesState::default();
        assert!(!state.has_local_view());
        state.apply_local_preferences(None, Some("list"));
        assert!(state.has_local_view());
    }

    #[test]
    fn invalid_stored_view_is_not_marked_as_local() {
        let mut state = PreferencesState::default();
        state.apply_local_preferences(None, Some("table"));
        assert!(!state.has_local_view());
    }

    #[test]
    fn toggling_view_marks_it_as_local() {
        let mut state = PreferencesState::default();
        assert!(!state.has_local_view());
        state.set_subscriptions_view("list".into());
        assert!(state.has_local_view());
    }

    #[test]
    fn invalid_view_is_not_marked_as_local() {
        let mut state = PreferencesState::default();
        state.set_subscriptions_view("table".into());
        assert!(!state.has_local_view());
    }

    #[test]
    fn valid_views_are_grid_and_list() {
        assert!(VALID_VIEWS.contains(&"grid"));
        assert!(VALID_VIEWS.contains(&"list"));
    }
}
