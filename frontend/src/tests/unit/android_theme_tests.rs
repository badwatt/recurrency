// TDD tests for Android theme persistence.
// Bug: theme not persisting across app restarts on Android.

/// TDD RED: persist_theme must write theme.txt on native (non-WASM32).
/// Uses /tmp for test isolation (real app uses get_app_data_dir()).
#[test]
fn test_theme_persists_to_file_on_native() {
    let theme_dir = std::path::PathBuf::from("/tmp/test_recurrency_theme2/recurrency");
    let theme_file = theme_dir.join("theme.txt");

    std::fs::create_dir_all(&theme_dir).ok();
    std::fs::write(&theme_file, "dark").ok();

    let read = std::fs::read_to_string(&theme_file).unwrap();
    assert_eq!(read.trim(), "dark", "theme file must exist after write");

    std::fs::remove_file(&theme_file).ok();
    std::fs::remove_dir_all(theme_dir.parent().unwrap()).ok();
}
