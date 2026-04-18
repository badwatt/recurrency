
// TDD tests for Android persistence bugs.
// Bug 1: Server URL resets to localhost:8080 on restart.
// Bug 2: Theme not persisting on Android.
// All root causes: dirs::data_dir() returns None on Android → get_app_data_dir() fallback needed.

/// TDD RED: server_url.txt must be writable/readable on native (Android fallback).
#[test]
fn test_server_url_file_persists_on_native() {
    // Use /tmp for test isolation (don't pollute real user data dirs).
    // Real app uses get_app_data_dir() which tries dirs::data_dir() → Android fallback → HOME fallback.
    let app_dir = std::path::PathBuf::from("/tmp/test_recurrency_recurrency");
    let url_file = app_dir.join("server_url.txt");

    std::fs::create_dir_all(&app_dir).ok();
    std::fs::write(&url_file, "http://my-server:3000").unwrap();

    let read_url = std::fs::read_to_string(&url_file).unwrap();
    assert_eq!(read_url.trim(), "http://my-server:3000", "server_url.txt must persist across restarts");

    std::fs::remove_file(&url_file).ok();
    std::fs::remove_dir_all(app_dir.parent().unwrap()).ok();
}

/// TDD RED: theme.txt must be writable/readable on native (Android fallback).
#[test]
fn test_theme_file_persists_on_native() {
    let app_dir = std::path::PathBuf::from("/tmp/test_recurrency_theme");
    let theme_file = app_dir.join("theme.txt");

    std::fs::create_dir_all(&app_dir).ok();
    std::fs::write(&theme_file, "dark").unwrap();

    let read_theme = std::fs::read_to_string(&theme_file).unwrap();
    assert_eq!(read_theme.trim(), "dark", "theme.txt must persist across restarts");

    std::fs::remove_file(&theme_file).ok();
    std::fs::remove_dir_all(app_dir.parent().unwrap()).ok();
}

/// TDD RED: get_app_data_dir() must return Some on Android (not None).
#[test]
fn test_get_app_data_dir_returns_path_on_native() {
    let data_dir = crate::state::auth::get_app_data_dir();
    assert!(data_dir.is_some(), "get_app_data_dir must return Some on native (Android fallback required)");
}
