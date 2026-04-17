
//! TDD tests for Android auth persistence.
//! Bug: Auth state lost after app restart on Android.
//! Root cause: dirs::data_dir() returns None on Android (no jni feature).
//! Fix: add jni feature to dirs crate.

use std::path::PathBuf;

/// Returns the native auth persistence dir used by the app.
fn native_auth_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("recurrency")
}

/// TDD RED: persist_to_storage must write auth files on Android.
/// Without jni feature, dirs::data_dir() returns None → no files → login on restart.
#[test]
fn test_dirs_data_dir_returns_some_on_native() {
    // TDD RED: dirs::data_dir() must return Some(path) on native platforms.
    // Without jni feature: returns None on Android → auth broken.
    // With jni feature: returns proper Android files dir.
    let data_dir = dirs::data_dir();
    assert!(
        data_dir.is_some(),
        "dirs::data_dir() must return Some on native. Got None — add jni feature to dirs crate.         Auth will NOT persist without this fix."
    );
}

/// TDD RED: After set_authenticated, auth files must exist.
#[test]
fn test_set_authenticated_writes_auth_files() {
    // Clean up first
    let auth_dir = native_auth_dir();
    let _ = std::fs::remove_file(auth_dir.join("auth-token.txt"));
    let _ = std::fs::remove_file(auth_dir.join("auth-user.json"));

    let mut state = crate::state::auth::AuthState::default();
    let user = crate::models::auth::UserResponse {
        id: uuid::Uuid::new_v4(),
        email: "tdd@test.com".into(),
        name: Some("TDD Test".into()),
        currency: "EUR".into(),
        verified: false,
    };
    state.set_authenticated("tdd-token".into(), user, false);

    let token_path = auth_dir.join("auth-token.txt");
    let user_path = auth_dir.join("auth-user.json");

    assert!(
        token_path.exists() && user_path.exists(),
        "Auth files must exist after set_authenticated.         dirs::data_dir()={:?}. Fix: add jni feature to dirs crate.",
        dirs::data_dir()
    );
}
