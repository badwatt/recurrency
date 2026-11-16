use dioxus::prelude::*;

/// Optional compile-time server override for native builds. It is empty by
/// default so no personal or branded server is prefilled. Web builds always use
/// the origin they are served from.
pub const NATIVE_SERVER_OVERRIDE: &str = match option_env!("RECURRENCY_API_URL") {
    Some(url) => url,
    None => "",
};

/// Application mode: on-device storage or a remote Recurrency server.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Use on-device storage — no server needed.
    Offline,
    /// Connect to a remote Recurrency server.
    Server(String),
}

impl AppMode {
    pub fn is_offline(&self) -> bool {
        matches!(self, AppMode::Offline)
    }

    pub fn server_url(&self) -> Option<&str> {
        match self {
            AppMode::Offline => None,
            AppMode::Server(url) => Some(url),
        }
    }
}

impl Default for AppMode {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        AppMode::Offline
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppModeState {
    pub mode: AppMode,
    pub initialized: bool,
}

impl Default for AppModeState {
    fn default() -> Self {
        // Try to hydrate from persistent storage immediately
        let (mode, initialized) = hydrate_mode_from_storage();
        Self { mode, initialized }
    }
}

fn hydrate_mode_from_storage() -> (AppMode, bool) {
    // Unit tests must never inspect the real user data directory.
    #[cfg(test)]
    {
        return (AppMode::Offline, true);
    }
    // Web builds always use the origin serving the app. There is no server
    // chooser and no offline mode on the web.
    #[cfg(target_arch = "wasm32")]
    {
        (AppMode::Server(String::new()), true)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Env var override
        if let Ok(url) = std::env::var("RECURRENCY_API_URL")
            && !url.is_empty()
        {
            return (AppMode::Server(url), true);
        }
        // Try the active data directory, its backup and legacy locations.
        // `app_mode.txt` also embeds the URL, so a missing/lost
        // `server_url.txt` can never silently downgrade a server install to
        // local mode.
        if let Some(mode_str) = crate::state::persistence::read("app_mode.txt") {
            let mut lines = mode_str.lines();
            let mode = lines.next().unwrap_or_default().trim();
            let inline_url = lines.next().unwrap_or_default().trim().to_string();
            match mode {
                "server" => {
                    let url = inline_url;
                    let url = if url.is_empty() {
                        crate::state::persistence::read("server_url.txt")
                            .map(|value| value.trim().to_string())
                            .filter(|value| !value.is_empty())
                            .unwrap_or_default()
                    } else {
                        url
                    };
                    if !url.is_empty() {
                        return (AppMode::Server(url), true);
                    }
                    // Keep the persisted mode; the server setup screen is a
                    // better outcome than silently using local data.
                    return (AppMode::Offline, false);
                }
                "offline" => return (AppMode::Offline, true),
                _ => {}
            }
        }
        // Never prefill a branded or personal server. A fresh native install
        // starts locally until the user configures a server.
        (AppMode::Offline, true)
    }
}

/// Normalize a user-entered server address. Accepts hostnames without a
/// scheme (defaults to HTTPS) and strips a trailing slash or `/api` suffix.
pub fn normalize_server_url(input: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Enter your server URL".to_string());
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if trimmed.contains("://") {
        return Err("Server URL must start with http:// or https://".to_string());
    } else {
        format!("https://{trimmed}")
    };

    let parsed =
        reqwest::Url::parse(&with_scheme).map_err(|_| "Enter a valid server URL".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Server URL must start with http:// or https://".to_string());
    }
    if parsed.host_str().is_none() {
        return Err("Enter a valid server URL".to_string());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("Server URL cannot contain a query or fragment".to_string());
    }

    if parsed.path() == "/api" || parsed.path().starts_with("/api/") {
        return Ok(parsed.origin().ascii_serialization());
    }

    Ok(with_scheme.trim_end_matches('/').to_string())
}

impl AppModeState {
    pub fn set_local(&mut self) {
        self.mode = AppMode::Offline;
        self.initialized = true;
        // Leaving server mode must not leave a stale base URL behind: an
        // override would make is_offline_mode() report server mode.
        crate::api::client::clear_base_url();
        self.persist();
    }

    pub fn set_server(&mut self, url: String) {
        self.mode = AppMode::Server(url);
        self.initialized = true;
        self.persist();
    }

    pub fn reset(&mut self) {
        self.mode = AppMode::Offline;
        self.initialized = false;
        clear_mode_storage();
    }

    /// Re-read durable mode. Only recovers a server installation that was
    /// initialised as offline because storage was temporarily unavailable; an
    /// explicit local choice remains local.
    pub fn reload_from_storage(&mut self) -> bool {
        if !self.mode.is_offline() {
            return false;
        }
        let (mode, initialized) = hydrate_mode_from_storage();
        if initialized && !mode.is_offline() {
            self.mode = mode;
            self.initialized = true;
            if let Some(url) = self.mode.server_url() {
                crate::api::client::set_base_url(url);
            }
            return true;
        }
        false
    }

    /// Re-write the current mode to its durable file. Used by the periodic
    /// self-heal loop when an external cleanup removed the file.
    pub fn ensure_persisted(&self) {
        self.persist();
    }

    fn persist(&self) {
        #[cfg(test)]
        {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    match &self.mode {
                        AppMode::Offline => {
                            let _ = storage.set_item("RECURRENCY_APP_MODE", "offline");
                            let _ = storage.remove_item("RECURRENCY_API_URL");
                        }
                        AppMode::Server(url) => {
                            let _ = storage.set_item("RECURRENCY_APP_MODE", "server");
                            let _ = storage.set_item("RECURRENCY_API_URL", url);
                        }
                    }
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            match &self.mode {
                AppMode::Offline => {
                    // Keep the last server URL for quick reconnection. The
                    // mode file is authoritative.
                    crate::state::persistence::write("app_mode.txt", "offline");
                }
                AppMode::Server(url) => {
                    crate::state::persistence::write("app_mode.txt", &format!("server\n{url}"));
                    crate::state::persistence::write("server_url.txt", url);
                }
            }
        }
    }
}

fn clear_mode_storage() {
    #[cfg(test)]
    {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("RECURRENCY_APP_MODE");
                let _ = storage.remove_item("RECURRENCY_API_URL");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::state::persistence::remove("app_mode.txt");
        crate::state::persistence::remove("server_url.txt");
    }
}

/// Last server URL, preserved when the user switches to local mode so the
/// connection can be restored without retyping the address.
pub fn last_server_url() -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::state::persistence::read("server_url.txt")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

pub static APP_MODE: GlobalSignal<AppModeState> = Signal::global(AppModeState::default);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_mode_offline_is_offline() {
        assert!(AppMode::Offline.is_offline());
        assert!(!AppMode::Server("http://test".into()).is_offline());
    }

    #[test]
    fn app_mode_server_url() {
        assert_eq!(AppMode::Offline.server_url(), None);
        assert_eq!(
            AppMode::Server("http://test:8080".into()).server_url(),
            Some("http://test:8080")
        );
    }

    #[test]
    fn app_mode_default_is_offline() {
        assert!(AppMode::default().is_offline());
    }

    #[test]
    fn app_mode_state_default() {
        // Default depends on storage — just verify it doesn't panic
        let state = AppModeState::default();
        assert!(state.mode.is_offline() || state.mode.server_url().is_some());
    }

    #[test]
    fn app_mode_state_set_offline() {
        let mut state = AppModeState::default();
        state.set_local();
        assert!(state.mode.is_offline());
        assert!(state.initialized);
    }

    #[test]
    fn app_mode_state_set_server() {
        let mut state = AppModeState::default();
        state.set_server("http://myserver:8080".into());
        assert_eq!(state.mode, AppMode::Server("http://myserver:8080".into()));
        assert!(state.initialized);
    }

    #[test]
    fn app_mode_state_reset() {
        let mut state = AppModeState::default();
        state.set_server("http://test:8080".into());
        state.reset();
        assert!(state.mode.is_offline());
        assert!(!state.initialized);
    }

    #[test]
    fn app_mode_equality() {
        assert_eq!(AppMode::Offline, AppMode::Offline);
        assert_eq!(
            AppMode::Server("http://a".into()),
            AppMode::Server("http://a".into())
        );
        assert_ne!(AppMode::Offline, AppMode::Server("http://a".into()));
    }

    #[test]
    fn app_mode_state_persist_offline() {
        // Persist local mode — just ensure it doesn't panic
        let mut state = AppModeState::default();
        state.set_local();
        // On native, persists to filesystem
        // On WASM, persists to localStorage
        // In test, just verify state
        assert!(state.mode.is_offline());
    }

    #[test]
    fn app_mode_state_persist_server() {
        let mut state = AppModeState::default();
        state.set_server("http://localhost:9090".into());
        assert_eq!(state.mode.server_url(), Some("http://localhost:9090"));
    }

    #[test]
    fn normalize_server_url_adds_https() {
        assert_eq!(
            normalize_server_url("recurrency.example.com").unwrap(),
            "https://recurrency.example.com"
        );
    }

    #[test]
    fn normalize_server_url_keeps_http_for_local_servers() {
        assert_eq!(
            normalize_server_url(" http://192.168.1.10:8080/ ").unwrap(),
            "http://192.168.1.10:8080"
        );
    }

    #[test]
    fn normalize_server_url_removes_api_suffix() {
        assert_eq!(
            normalize_server_url("https://recurrency.example.com/api/").unwrap(),
            "https://recurrency.example.com"
        );
    }

    #[test]
    fn normalize_server_url_rejects_empty_input() {
        assert!(normalize_server_url("   ").is_err());
    }

    #[test]
    fn normalize_server_url_rejects_invalid_scheme() {
        assert!(normalize_server_url("ftp://example.com").is_err());
    }

    #[test]
    fn normalize_server_url_rejects_query() {
        assert!(normalize_server_url("https://example.com?x=1").is_err());
    }

    #[test]
    fn hydrate_mode_from_storage_no_panic() {
        // Just ensure hydration doesn't panic (no storage in test env)
        let (mode, _initialized) = hydrate_mode_from_storage();
        assert!(mode.is_offline() || mode.server_url().is_some());
    }

    #[test]
    fn set_local_marks_initialized() {
        // Bug fix: set_local() must mark initialized=true so the AppLayout
        // does not fall into the "force server mode" branch on Android.
        let mut state = AppModeState::default();
        state.reset(); // forces initialized=false
        assert!(!state.initialized);
        state.set_local();
        assert!(state.initialized, "set_local must mark initialized");
        assert!(state.mode.is_offline());
    }
}
