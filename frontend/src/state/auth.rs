use crate::models::auth::UserResponse;
use dioxus::prelude::*;

#[allow(dead_code)]
const TOKEN_KEY: &str = "recurrency-auth-token";
/// Returns the app data directory for native platforms. Android uses its
/// canonical private files directory; desktop uses dirs::data_dir().
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn get_app_data_dir() -> Option<std::path::PathBuf> {
    // Android's canonical private files directory is stable across launches.
    // Prefer it over dirs::data_dir(), which is not defined for Android.
    let android_files = std::path::PathBuf::from("/data/data/com.badwatt.recurrency/files");
    if android_files.parent().map(|p| p.exists()).unwrap_or(false) {
        return Some(android_files);
    }
    // Desktop: Linux/macOS/Windows data directory.
    if let Some(dir) = dirs::data_dir() {
        return Some(dir);
    }
    // Fallback: HOME/.recurrency (works on desktop when dirs fails)
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        return Some(std::path::PathBuf::from(home).join(".recurrency"));
    }
    None
}

#[allow(dead_code)]
const USER_KEY: &str = "recurrency-auth-user";

#[cfg(test)]
pub(crate) static AUTH_FILE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
        self.user.as_ref().is_some_and(|u| u.verified)
    }

    /// Get the current user's currency, defaulting to EUR.
    pub fn currency(&self) -> &str {
        self.user
            .as_ref()
            .map(|u| u.currency.as_str())
            .unwrap_or("EUR")
    }

    /// Set the user after successful login/register.
    pub fn set_authenticated(
        &mut self,
        token: String,
        user: UserResponse,
        requires_verification: bool,
    ) {
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
                    if let Ok(user_json) = serde_json::to_string(user)
                        && let Err(e) = std::fs::write(app_dir.join("auth-user.json"), &user_json) {
                            eprintln!("[Auth] Failed to write user: {}", e);
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
                match (
                    std::fs::read_to_string(&token_file),
                    std::fs::read_to_string(&user_file),
                ) {
                    (Ok(token), Ok(user_json)) => {
                        if let Ok(user) = serde_json::from_str::<UserResponse>(&user_json)
                            && !token.is_empty() {
                                eprintln!(
                                    "[Auth] Hydrate: token found (length {}), user hydrated",
                                    token.len()
                                );
                                self.token = Some(token);
                                self.user = Some(user);
                                self.is_authenticated = true;
                                self.initialized = true;
                                return true;
                            }
                        eprintln!("[Auth] Hydrate: files found but parse failed");
                    }
                    (Err(e1), e2) => {
                        eprintln!(
                            "[Auth] Hydrate: token file error: {:?}, user file result: {:?}",
                            e1, e2
                        );
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

pub static AUTH: GlobalSignal<AuthState> = Signal::global(AuthState::default);

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use uuid::Uuid;

    fn sample_user() -> UserResponse {
        UserResponse {
            id: Uuid::new_v4(),
            email: "test@test.com".into(),
                avatar: None,
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

    // ── AuthState additional tests ──
    #[test]
    fn auth_state_debug() {
        let state = AuthState::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("AuthState"));
    }

    #[test]
    fn auth_state_clone() {
        let state = AuthState::default();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn auth_state_set_authenticated_sets_loading_false() {
        let mut state = AuthState::default();
        state.is_loading = true;
        assert!(state.is_loading);
        state.set_authenticated("token".into(), sample_user(), false);
        assert!(!state.is_loading);
    }

    #[test]
    fn auth_state_is_verified_with_no_user() {
        let state = AuthState::default();
        assert!(!state.is_verified());
    }

    #[test]
    fn auth_state_is_verified_with_none_user() {
        let mut state = AuthState::default();
        state.user = None;
        assert!(!state.is_verified());
    }

    #[test]
    fn auth_state_currency_no_user() {
        let state = AuthState::default();
        assert_eq!(state.currency(), "EUR");
    }

    #[test]
    fn auth_state_is_authenticated_after_set_auth() {
        let mut state = AuthState::default();
        assert!(!state.is_authenticated);
        state.set_authenticated("tok".into(), sample_user(), false);
        assert!(state.is_authenticated);
    }

    #[test]
    fn auth_state_is_authenticated_after_set_verified() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.verified = false;
        state.set_authenticated("tok".into(), user, true);
        assert!(!state.is_authenticated);
        state.set_verified();
        assert!(state.is_authenticated);
    }

    #[test]
    fn auth_state_logout_clears_loading() {
        let mut state = AuthState::default();
        state.is_loading = true;
        state.set_authenticated("tok".into(), sample_user(), false);
        assert!(!state.is_loading);
        state.logout();
        assert!(!state.is_loading);
    }

    #[test]
    fn auth_state_set_verified_updates_user() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.verified = false;
        state.set_authenticated("tok".into(), user, true);
        assert!(!state.user.as_ref().unwrap().verified);
        state.set_verified();
        assert!(state.user.as_ref().unwrap().verified);
    }

    #[test]
    fn auth_state_set_authenticated_requires_verification_false() {
        let mut state = AuthState::default();
        state.set_authenticated("tok".into(), sample_user(), false);
        assert!(state.is_authenticated);
    }

    #[test]
    fn auth_state_set_authenticated_requires_verification_true() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.verified = false;
        state.set_authenticated("tok".into(), user, true);
        assert!(!state.is_authenticated);
        assert!(state.token.is_some());
        assert!(state.initialized);
    }

    #[test]
    fn auth_state_multiple_logouts() {
        let mut state = AuthState::default();
        state.set_authenticated("tok".into(), sample_user(), false);
        state.logout();
        assert!(!state.is_authenticated);
        // Second logout should be safe
        state.logout();
        assert!(!state.is_authenticated);
    }

    #[test]
    fn auth_state_multiple_verifications() {
        let mut state = AuthState::default();
        let mut user = sample_user();
        user.verified = false;
        state.set_authenticated("tok".into(), user, true);
        state.set_verified();
        // Second verification should be safe
        state.set_verified();
        assert!(state.is_authenticated);
    }

    // ── get_app_data_dir tests ──
    // Android path check: /data/data/com.badwatt.recurrency/files
    // This path only exists on Android devices, not on test machines.
    // Line 18-20 (android_files parent().map(...).unwrap_or(false)) is only hit on Android.
    // Line 22-25 (HOME/.recurrency fallback) is only hit when dirs::data_dir() returns None
    // AND the Android path doesn't exist.
    #[test]
    fn get_app_data_dir_returns_some_on_native() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let result = super::get_app_data_dir();
            // On native test platforms (Linux), dirs::data_dir() returns Some
            // (e.g., /home/user/.local/share or similar). So we get that path.
            assert!(
                result.is_some(),
                "get_app_data_dir should return Some on native"
            );
        }
    }

    #[test]
    fn android_path_parent_exists_check() {
        // Test the android path parent existence logic
        let android_files = std::path::PathBuf::from("/data/data/com.badwatt.recurrency/files");
        let parent = android_files.parent().expect("should have parent");
        // On non-Android (Linux), /data/data/ does NOT exist, so this returns None
        let exists = parent.exists();
        assert!(!exists, "Android path should not exist on Linux desktop");
    }

    // ── persist_to_storage tests (native) ──
    #[test]
    #[serial]
    fn persist_to_storage_writes_files_on_native() {
        let _guard = AUTH_FILE_TEST_LOCK.lock().unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let data_dir =
                get_app_data_dir().expect("get_app_data_dir should return Some on native");
            let app_dir = data_dir.join("recurrency");
            // Clean up before test - ensure fresh state
            let _ = std::fs::remove_file(app_dir.join("auth-token.txt"));
            let _ = std::fs::remove_file(app_dir.join("auth-user.json"));
            // Wait a bit for file system
            std::thread::sleep(std::time::Duration::from_millis(10));

            let mut state = AuthState::default();
            state.set_authenticated("unique-token-xyz123".into(), sample_user(), false);
            state.persist_to_storage();

            // Verify files exist
            assert!(
                app_dir.join("auth-token.txt").exists(),
                "Token file should exist"
            );
            assert!(
                app_dir.join("auth-user.json").exists(),
                "User file should exist"
            );

            // Verify content
            let token_content = std::fs::read_to_string(app_dir.join("auth-token.txt")).unwrap();
            assert_eq!(
                token_content, "unique-token-xyz123",
                "Token content should match"
            );

            // Clean up
            let _ = std::fs::remove_file(app_dir.join("auth-token.txt"));
            let _ = std::fs::remove_file(app_dir.join("auth-user.json"));
        }
    }

    #[test]
    #[serial]
    fn persist_to_storage_no_op_without_token() {
        let _guard = AUTH_FILE_TEST_LOCK.lock().unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let state = AuthState::default();
            // No token set, should not fail
            state.persist_to_storage();
            // Should not create any files
        }
    }

    // ── Additional AuthState tests ──
    #[test]
    fn auth_state_eq_default() {
        let state = AuthState::default();
        let state2 = AuthState::default();
        assert_eq!(state, state2);
    }

    #[test]
    fn auth_state_set_authenticated_sets_fields() {
        let mut state = AuthState::default();
        let user = crate::models::auth::UserResponse {
            id: uuid::Uuid::new_v4(),
            email: "test@test.com".into(),
                avatar: None,
            name: Some("Test".into()),
            currency: "EUR".into(),
            verified: true,
        };
        state.set_authenticated("test-token".into(), user, false);
        assert_eq!(state.token, Some("test-token".into()));
        assert!(!state.is_loading);
    }

    #[test]
    fn auth_state_logout_clears_token() {
        let mut state = AuthState::default();
        let user = crate::models::auth::UserResponse {
            id: uuid::Uuid::new_v4(),
            email: "test@test.com".into(),
                avatar: None,
            name: Some("Test".into()),
            currency: "EUR".into(),
            verified: true,
        };
        state.set_authenticated("test-token".into(), user, false);
        state.logout();
        assert!(state.token.is_none());
    }
}
