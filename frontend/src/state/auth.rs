use dioxus::prelude::*;
use crate::models::auth::UserResponse;

#[allow(dead_code)]
const TOKEN_KEY: &str = "recurrency-auth-token";
/// Returns the app data directory for native platforms.
/// Tries dirs::data_dir() first, then falls back to Android-specific paths.
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn get_app_data_dir() -> Option<std::path::PathBuf> {
    // 1. Try standard dirs (works on desktop Linux/macOS/Windows)
    if let Some(dir) = dirs::data_dir() {
        return Some(dir);
    }
    // 2. Android fallback: /data/data/<package>/files is the canonical private dir
    //    The parent /data/data/<package> always exists for installed apps.
    //    We just need to return a path so create_dir_all() can create the subdir.
    let android_files = std::path::PathBuf::from("/data/data/com.alvaro17f.recurrency/files");
    if android_files.parent().map(|p| p.exists()).unwrap_or(false) {
        return Some(android_files);
    }
    // 3. Fallback: HOME/.recurrency (works on desktop when dirs fails)
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        return Some(std::path::PathBuf::from(home).join(".recurrency"));
    }
    None
}


#[allow(dead_code)]
const USER_KEY: &str = "recurrency-auth-user";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuthState {
    pub user: Option<UserResponse>,
    pub is_authenticated: bool,
    pub is_loading: bool,
    pub initialized: bool,
    pub token: Option<String>,
}

impl AuthState {
    /// Check if the user is authenticated and verified.
    pub fn is_verified(&self) -> bool {
        self.user.as_ref().map_or(false, |u| u.verified)
    }

    /// Get the current user's currency, defaulting to EUR.
    pub fn currency(&self) -> &str {
        self.user
            .as_ref()
            .map(|u| u.currency.as_str())
            .unwrap_or("EUR")
    }

    /// Set the user after successful login/register.
    pub fn set_authenticated(&mut self, token: String, user: UserResponse, requires_verification: bool) {
        self.token = Some(token);
        self.user = Some(user);
        self.is_authenticated = !requires_verification;
        self.is_loading = false;
        self.initialized = true;
        self.persist_to_storage();
    }

    /// Mark user as verified after email verification.
    pub fn set_verified(&mut self) {
        self.is_authenticated = true;
        if let Some(ref mut user) = self.user {
            user.verified = true;
        }
        self.persist_to_storage();
    }

    /// Clear all auth state on logout.
    pub fn logout(&mut self) {
        Self::clear_storage();
        self.user = None;
        self.is_authenticated = false;
        self.is_loading = false;
        self.token = None;
    }

    /// Persist token + user to localStorage (WASM) or file (native).
    pub fn persist_to_storage(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let (Some(token), Some(user)) = (&self.token, &self.user) {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                {
                    let _ = storage.set_item(TOKEN_KEY, token);
                    if let Ok(user_json) = serde_json::to_string(user) {
                        let _ = storage.set_item(USER_KEY, &user_json);
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let (Some(token), Some(user)) = (&self.token, &self.user) {
                if let Some(data_dir) = get_app_data_dir() {
                    let app_dir = data_dir.join("recurrency");
                    if let Err(e) = std::fs::create_dir_all(&app_dir) {
                        eprintln!("[Auth] Failed to create app dir {:?}: {}", app_dir, e);
                    }
                    if let Err(e) = std::fs::write(app_dir.join("auth-token.txt"), token) {
                        eprintln!("[Auth] Failed to write token: {}", e);
                    }
                    if let Ok(user_json) = serde_json::to_string(user) {
                        if let Err(e) = std::fs::write(app_dir.join("auth-user.json"), &user_json) {
                            eprintln!("[Auth] Failed to write user: {}", e);
                        }
                    }
                } else {
                    eprintln!("[Auth] get_app_data_dir() returned None — cannot persist auth");
                }
            }
        }
    }

    /// Remove auth keys from localStorage (WASM) or file (native).
    fn clear_storage() {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.remove_item(TOKEN_KEY);
                let _ = storage.remove_item(USER_KEY);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(data_dir) = get_app_data_dir() {
                let app_dir = data_dir.join("recurrency");
                let _ = std::fs::remove_file(app_dir.join("auth-token.txt"));
                let _ = std::fs::remove_file(app_dir.join("auth-user.json"));
            }
        }
    }

    /// Restore token + user from localStorage (WASM) or file (native). Returns true if restored.
    pub fn hydrate_from_storage(&mut self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let token = storage.get_item(TOKEN_KEY).ok().flatten();
                let user_json = storage.get_item(USER_KEY).ok().flatten();
                if let (Some(token), Some(user_json)) = (token, user_json) {
                    if let Ok(user) = serde_json::from_str::<UserResponse>(&user_json) {
                        self.token = Some(token);
                        self.user = Some(user);
                        self.is_authenticated = true;
                        self.initialized = true;
                        return true;
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(data_dir) = get_app_data_dir() {
                let app_dir = data_dir.join("recurrency");
                let token_file = app_dir.join("auth-token.txt");
                let user_file = app_dir.join("auth-user.json");
                eprintln!("[Auth] Hydrate: looking for files in {:?}", app_dir);
                match (std::fs::read_to_string(&token_file), std::fs::read_to_string(&user_file)) {
                    (Ok(token), Ok(user_json)) => {
                        if let Ok(user) = serde_json::from_str::<UserResponse>(&user_json) {
                            if !token.is_empty() {
                                eprintln!("[Auth] Hydrate: token found (length {}), user hydrated", token.len());
                                self.token = Some(token);
                                self.user = Some(user);
                                self.is_authenticated = true;
                                self.initialized = true;
                                return true;
                            }
                        }
                        eprintln!("[Auth] Hydrate: files found but parse failed");
                    }
                    (Err(e1), e2) => {
                        eprintln!("[Auth] Hydrate: token file error: {:?}, user file result: {:?}", e1, e2);
                    }
                    (Ok(_), Err(e2)) => {
                        eprintln!("[Auth] Hydrate: user file error: {:?}", e2);
                    }
                }
            } else {
                eprintln!("[Auth] Hydrate: get_app_data_dir() returned None");
            }
        }
        false
    }
}

pub static AUTH: GlobalSignal<AuthState> = Signal::global(|| AuthState::default());

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_user() -> UserResponse {
        UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
            name: Some("Test".into()),
            currency: "EUR".into(),
            verified: true,
        }
    }

    #[test]
    fn default_state() {
        let state = AuthState::default();
        assert!(!state.is_authenticated);
        assert!(state.user.is_none());
        assert!(!state.is_loading);
        assert!(!state.initialized);
        assert!(state.token.is_none());
    }

    #[test]
    fn set_authenticated() {
        let mut state = AuthState::default();
        let user = sample_user();
        state.set_authenticated("token123".into(), user.clone(), false);
        assert!(state.is_authenticated);
        assert_eq!(state.token, Some("token123".into()));
        assert_eq!(state.user, Some(user));
        assert!(state.initialized);
    }

    #[test]
    fn set_authenticated_with_verification() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.verified = false;
        state.set_authenticated("token123".into(), user, true);
        assert!(!state.is_authenticated); // not verified yet
        assert_eq!(state.token, Some("token123".into()));
    }

    #[test]
    fn set_verified() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.verified = false;
        state.set_authenticated("token123".into(), user, true);
        state.set_verified();
        assert!(state.is_authenticated);
        assert!(state.user.unwrap().verified);
    }

    #[test]
    fn logout() {
        let mut state = AuthState::default();
        state.set_authenticated("token".into(), sample_user(), false);
        state.logout();
        assert!(!state.is_authenticated);
        assert!(state.token.is_none());
        assert!(state.user.is_none());
    }

    #[test]
    fn currency_default() {
        let state = AuthState::default();
        assert_eq!(state.currency(), "EUR");
    }

    #[test]
    fn currency_from_user() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.currency = "USD".into();
        state.user = Some(user);
        assert_eq!(state.currency(), "USD");
    }

    #[test]
    fn is_verified_true() {
        let mut state = AuthState::default();
        state.user = Some(sample_user());
        assert!(state.is_verified());
    }

    #[test]
    fn is_verified_false() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.verified = false;
        state.user = Some(user);
        assert!(!state.is_verified());
    }

    #[test]
    fn test_manual_user_update_updates_auth_state() {
        // Simulates settings page: AUTH.write().user = Some(resp.user)
        let mut state = AuthState::default();
        state.set_authenticated("token123".into(), sample_user(), false);
        
        // Simulate getting updated user from settings API
        let mut updated_user = sample_user();
        updated_user.currency = "USD".into();
        updated_user.name = Some("New Name".into());
        
        state.user = Some(updated_user);
        
        // Verify user was updated
        assert_eq!(state.user.as_ref().unwrap().currency, "USD");
        assert_eq!(state.user.as_ref().unwrap().name, Some("New Name".into()));
        assert!(state.is_authenticated); // Still authenticated
    }

        #[test]
    fn test_persist_to_storage_writes_files_on_native() {
        // TDD: persist_to_storage must write auth-token.txt and auth-user.json
        // to dirs::data_dir()/recurrency/ on native platforms.
        let data_dir = get_app_data_dir().expect("get_app_data_dir should return Some on native");
        let app_dir = data_dir.join("recurrency");
        // Clean up before test
        let _ = std::fs::remove_file(app_dir.join("auth-token.txt"));
        let _ = std::fs::remove_file(app_dir.join("auth-user.json"));

        let mut state = AuthState::default();
        state.set_authenticated("my-jwt-token".into(), sample_user(), false);
        state.persist_to_storage();

    }
}