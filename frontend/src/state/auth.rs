use dioxus::prelude::*;
use crate::models::auth::UserResponse;

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
    }

    /// Mark user as verified after email verification.
    pub fn set_verified(&mut self) {
        self.is_authenticated = true;
        if let Some(ref mut user) = self.user {
            user.verified = true;
        }
    }

    /// Clear all auth state on logout.
    pub fn logout(&mut self) {
        self.user = None;
        self.is_authenticated = false;
        self.is_loading = false;
        self.token = None;
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
}