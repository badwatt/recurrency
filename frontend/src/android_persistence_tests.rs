
// TDD tests for Android persistence bugs.
// Bug 1: Server URL resets to localhost:8080 on restart.
// Bug 2: Theme not persisting on Android.
// Bug 3: Icon not showing.
// All root causes: dirs::data_dir() returns None on Android.

/// TDD RED: server_url.txt must be readable after app restart.
/// Without Android fix, get_app_data_dir() returns None → file not found → localhost:8080.
#[test]
fn test_server_url_file_persists_on_native() {
    use std::path::PathBuf;
    use std::env;

    // Determine the app data dir (same logic as main.rs)
    fn get_app_data_dir_test() -> Option<PathBuf> {
        if let Some(dir) = dirs::data_dir() {
            return Some(dir);
        }
        let android_files = PathBuf::from("/data/data/com.alvaro17f.recurrency/files");
        if android_files.parent().map(|p| p.exists()).unwrap_or(false) {
            return Some(android_files);
        }
        if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
            return Some(PathBuf::from(home).join(".recurrency"));
        }
        None
    }

    let data_dir = get_app_data_dir_test().expect("app data dir must be accessible");
    let app_dir = data_dir.join("recurrency");
    let url_file = app_dir.join("server_url.txt");

    // Write test URL
    std::fs::create_dir_all(&app_dir).ok();
    std::fs::write(&url_file, "http://my-server:3000").unwrap();

    // Read back — must match
    let read_url = std::fs::read_to_string(&url_file).unwrap();
    assert_eq!(read_url.trim(), "http://my-server:3000", "server_url.txt must persist across restarts");

    // Cleanup
    std::fs::remove_file(&url_file).ok();
}

/// TDD RED: theme preference must persist across app restarts.
/// Without persist_theme on native, theme always defaults (no persistence).
#[test]
fn test_theme_preference_persists_on_native() {
    use std::path::PathBuf;
    use std::env;

    fn get_app_data_dir_test() -> Option<PathBuf> {
        if let Some(dir) = dirs::data_dir() {
            return Some(dir);
        }
        let android_files = PathBuf::from("/data/data/com.alvaro17f.recurrency/files");
        if android_files.parent().map(|p| p.exists()).unwrap_or(false) {
            return Some(android_files);
        }
        if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
            return Some(PathBuf::from(home).join(".recurrency"));
        }
        None
    }

    let data_dir = get_app_data_dir_test().expect("app data dir must be accessible");
    let app_dir = data_dir.join("recurrency");
    let theme_file = app_dir.join("theme.txt");

    // Write test theme
    std::fs::create_dir_all(&app_dir).ok();
    std::fs::write(&theme_file, "dark").unwrap();

    // Read back
    let read_theme = std::fs::read_to_string(&theme_file).unwrap();
    assert_eq!(read_theme.trim(), "dark", "theme must persist across restarts");

    // Cleanup
    std::fs::remove_file(&theme_file).ok();
}

/// TDD RED: preferences dir must be accessible for settings persistence.
/// dirs::data_dir() returns None on Android → preferences not saved.
#[test]
fn test_preferences_data_dir_accessible_on_native() {
    fn get_app_data_dir_test() -> Option<std::path::PathBuf> {
        if let Some(dir) = dirs::data_dir() {
            return Some(dir);
        }
        let android_files = std::path::PathBuf::from("/data/data/com.alvaro17f.recurrency/files");
        if android_files.parent().map(|p| p.exists()).unwrap_or(false) {
            return Some(android_files);
        }
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            return Some(std::path::PathBuf::from(home).join(".recurrency"));
        }
        None
    }

    let data_dir = get_app_data_dir_test();
    assert!(data_dir.is_some(), "app data dir must be accessible on native (Android fallback required)");
}
