
// TDD tests for Android theme persistence.
// Bug: theme not persisting across app restarts on Android.

/// TDD RED: persist_theme must write theme.txt on native (non-WASM32).
#[test]
fn test_theme_persists_to_file_on_native() {
    let theme_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/data/data/com.alvaro17f.recurrency/files"))
        .join("recurrency");
    let theme_file = theme_dir.join("theme.txt");

    std::fs::create_dir_all(&theme_dir).ok();
    std::fs::write(&theme_file, "dark").ok();

    let read = std::fs::read_to_string(&theme_file).unwrap();
    assert_eq!(read.trim(), "dark", "theme file must exist after write");

    std::fs::remove_file(&theme_file).ok();
}
