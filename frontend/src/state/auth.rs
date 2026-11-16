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
#[cfg(not(target_arch = "wasm32"))]
const TOKEN_FILE: &str = "auth-token.txt";
#[cfg(not(target_arch = "wasm32"))]
const USER_FILE: &str = "auth-user.json";

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{TOKEN_FILE, USER_FILE};
    use crate::models::auth::UserResponse;
    use std::path::Path;

    const LEGACY_TEST_TOKEN: &str = "tdd-token";
    const LEGACY_TEST_EMAIL: &str = "tdd@test.com";

    pub(super) fn to_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
        serde_json::to_string(value).map_err(|error| error.to_string())
    }

    pub(super) fn write_session(
        dir: &Path,
        token: &str,
        user: &UserResponse,
    ) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
        std::fs::write(dir.join(TOKEN_FILE), token).map_err(|error| error.to_string())?;
        let user_json = to_json(user)?;
        std::fs::write(dir.join(USER_FILE), user_json).map_err(|error| error.to_string())
    }

    pub(super) fn read_session(dir: &Path) -> Option<(String, UserResponse)> {
        let token = std::fs::read_to_string(dir.join(TOKEN_FILE)).ok()?;
        if token.is_empty() {
            return None;
        }
        let user_json = std::fs::read_to_string(dir.join(USER_FILE)).ok()?;
        let user: UserResponse = serde_json::from_str(&user_json).ok()?;
        // Older test builds wrote their fixture directly into the real app
        // data directory. Remove that fixture instead of showing a fake
        // account after installing a release.
        if token == LEGACY_TEST_TOKEN && user.email == LEGACY_TEST_EMAIL {
            clear_session(dir);
            return None;
        }
        Some((token, user))
    }

    pub(super) fn clear_session(dir: &Path) {
        let _ = std::fs::remove_file(dir.join(TOKEN_FILE));
        let _ = std::fs::remove_file(dir.join(USER_FILE));
    }
}

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
        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            if self.token.is_some() && self.user.is_some() {
                let Some(dir) = native_app_dir() else {
                    eprintln!("[Auth] get_app_data_dir() returned None — cannot persist auth");
                    return;
                };
                if let Err(error) = self.persist_to_dir(&dir) {
                    eprintln!("[Auth] Failed to persist session: {error}");
                }
            }
        }
    }

    /// Write the current session to `dir`. No-op when signed out.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn persist_to_dir(&self, dir: &std::path::Path) -> Result<(), String> {
        match (&self.token, &self.user) {
            (Some(token), Some(user)) => native::write_session(dir, token, user),
            _ => Ok(()),
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
        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            if let Some(dir) = native_app_dir() {
                native::clear_session(&dir);
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
        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            if let Some(dir) = native_app_dir()
                && self.hydrate_from_dir(&dir)
            {
                return true;
            }
        }
        false
    }

    /// Restore a session from `dir`. Returns true when a valid session was read.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn hydrate_from_dir(&mut self, dir: &std::path::Path) -> bool {
        let Some((token, user)) = native::read_session(dir) else {
            return false;
        };
        self.token = Some(token);
        self.user = Some(user);
        self.is_authenticated = true;
        self.initialized = true;
        true
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
fn native_app_dir() -> Option<std::path::PathBuf> {
    get_app_data_dir().map(|dir| dir.join("recurrency"))
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
            is_admin: false,
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
            let dir = std::env::temp_dir().join(format!(
                "recurrency-auth-persist-{}",
                uuid::Uuid::new_v4()
            ));
            let _ = std::fs::remove_dir_all(&dir);

            let mut state = AuthState::default();
            state.set_authenticated("unique-token-xyz123".into(), sample_user(), false);
            state.persist_to_dir(&dir).unwrap();

            assert!(dir.join(TOKEN_FILE).exists(), "Token file should exist");
            assert!(dir.join(USER_FILE).exists(), "User file should exist");
            let token_content = std::fs::read_to_string(dir.join(TOKEN_FILE)).unwrap();
            assert_eq!(token_content, "unique-token-xyz123");

            let _ = std::fs::remove_dir_all(&dir);
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
            is_admin: false,
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
            is_admin: false,
        };
        state.set_authenticated("test-token".into(), user, false);
        state.logout();
        assert!(state.token.is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    mod native_session {
        use super::*;
        use serde::Serialize;
        use serde::ser::Error as _;

        struct Failing;
        impl Serialize for Failing {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(S::Error::custom("serialization failed"))
            }
        }

        fn temp_dir(label: &str) -> std::path::PathBuf {
            let dir = std::env::temp_dir()
                .join(format!("recurrency-auth-{label}-{}", uuid::Uuid::new_v4()));
            let _ = std::fs::remove_dir_all(&dir);
            dir
        }

        #[test]
        fn session_json_reports_serialization_errors() {
            assert!(super::super::native::to_json(&Failing).is_err());
        }

        #[test]
        fn session_round_trips_through_an_isolated_directory() {
            let dir = temp_dir("roundtrip");
            let user = sample_user();
            super::super::native::write_session(&dir, "token-123", &user).unwrap();

            let (token, restored) = super::super::native::read_session(&dir).unwrap();
            assert_eq!(token, "token-123");
            assert_eq!(restored, user);

            super::super::native::clear_session(&dir);
            assert!(super::super::native::read_session(&dir).is_none());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn read_session_returns_none_without_a_token_file() {
            let dir = temp_dir("missing-token");
            std::fs::create_dir_all(&dir).unwrap();
            assert!(super::super::native::read_session(&dir).is_none());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn read_session_returns_none_for_an_empty_token() {
            let dir = temp_dir("empty-token");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("auth-token.txt"), "").unwrap();
            assert!(super::super::native::read_session(&dir).is_none());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn read_session_returns_none_without_a_user_file() {
            let dir = temp_dir("missing-user");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("auth-token.txt"), "token").unwrap();
            assert!(super::super::native::read_session(&dir).is_none());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn read_session_returns_none_for_a_corrupt_user_file() {
            let dir = temp_dir("corrupt-user");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("auth-token.txt"), "token").unwrap();
            std::fs::write(dir.join("auth-user.json"), "{not json").unwrap();
            assert!(super::super::native::read_session(&dir).is_none());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn write_session_reports_directory_errors() {
            let base = temp_dir("write-dir");
            std::fs::create_dir_all(&base).unwrap();
            let blocker = base.join("blocker");
            std::fs::write(&blocker, "file").unwrap();
            let error =
                super::super::native::write_session(&blocker.join("nested"), "t", &sample_user())
                    .unwrap_err();
            assert!(!error.is_empty());
            let _ = std::fs::remove_dir_all(&base);
        }

        #[test]
        fn write_session_reports_token_write_errors() {
            let dir = temp_dir("write-token");
            std::fs::create_dir_all(dir.join("auth-token.txt")).unwrap();
            assert!(super::super::native::write_session(&dir, "t", &sample_user()).is_err());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn write_session_reports_user_write_errors() {
            let dir = temp_dir("write-user");
            std::fs::create_dir_all(dir.join("auth-user.json")).unwrap();
            assert!(super::super::native::write_session(&dir, "t", &sample_user()).is_err());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn auth_state_restores_a_persisted_session() {
            let dir = temp_dir("restore");
            let mut state = AuthState::default();
            state.set_authenticated("jwt-token".into(), sample_user(), false);
            state.persist_to_dir(&dir).unwrap();

            let mut restored = AuthState::default();
            assert!(restored.hydrate_from_dir(&dir));
            assert_eq!(restored.token.as_deref(), Some("jwt-token"));
            assert!(restored.is_authenticated);
            assert!(restored.initialized);

            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn persist_to_dir_is_a_noop_without_a_session() {
            let dir = temp_dir("signed-out");
            AuthState::default().persist_to_dir(&dir).unwrap();
            assert!(!dir.exists());
        }

        #[test]
        fn hydrate_removes_the_legacy_tdd_fixture() {
            let dir = temp_dir("legacy-tdd");
            let user = crate::models::auth::UserResponse {
                id: uuid::Uuid::new_v4(),
                email: "tdd@test.com".into(),
                avatar: None,
                name: Some("TDD Test".into()),
                currency: "EUR".into(),
                verified: false,
                is_admin: false,
            };
            super::super::native::write_session(&dir, "tdd-token", &user).unwrap();
            assert!(!AuthState::default().hydrate_from_dir(&dir));
            assert!(!dir.join(TOKEN_FILE).exists());
            assert!(!dir.join(USER_FILE).exists());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn hydrate_from_dir_returns_false_when_empty() {
            let dir = temp_dir("hydrate-empty");
            assert!(!AuthState::default().hydrate_from_dir(&dir));
        }
    }
}
