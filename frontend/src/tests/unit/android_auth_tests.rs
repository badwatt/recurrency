//! TDD tests for Android auth persistence.
//! Bug: Auth state lost after app restart on Android.
//! Root cause: dirs::data_dir() returns None on Android (no jni feature).
//! Fix: add jni feature to dirs crate.

use serial_test::serial;

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
#[serial]
fn test_set_authenticated_writes_auth_files() {
    let _guard = crate::state::auth::AUTH_FILE_TEST_LOCK.lock().unwrap();
    let auth_dir = std::env::temp_dir().join(format!(
        "recurrency-android-auth-{}",
        uuid::Uuid::new_v4()
    ));
    let _ = std::fs::remove_dir_all(&auth_dir);

    let mut state = crate::state::auth::AuthState::default();
    let user = crate::models::auth::UserResponse {
        id: uuid::Uuid::new_v4(),
        email: "tdd@test.com".into(),
        avatar: None,
        name: Some("TDD Test".into()),
        currency: "EUR".into(),
        verified: false,
    };
    state.set_authenticated("tdd-token".into(), user, false);
    state.persist_to_dir(&auth_dir).unwrap();

    assert!(
        auth_dir.join("auth-token.txt").exists() && auth_dir.join("auth-user.json").exists(),
        "Auth files must exist after persist_to_dir"
    );
    let _ = std::fs::remove_dir_all(&auth_dir);
}
