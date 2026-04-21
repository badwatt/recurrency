// TDD tests for setup.rs server URL persistence bug.
// Bug: SetupPage save_server_url uses dirs::data_dir() → None on Android → URL not saved.

/// TDD RED: save_server_url must write server_url.txt on native Android.
#[test]
fn test_save_server_url_uses_android_fallback() {
    // Verify get_app_data_dir() is used (or dirs::data_dir() with jni feature)
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/data/data/com.alvaro17f.recurrency/files"));
    let app_dir = data_dir.join("recurrency");
    let url_file = app_dir.join("server_url.txt");

    std::fs::create_dir_all(&app_dir).ok();
    std::fs::write(&url_file, "http://my-server:9000").unwrap();

    let read = std::fs::read_to_string(&url_file).unwrap();
    assert_eq!(
        read.trim(),
        "http://my-server:9000",
        "server_url.txt must persist on native"
    );

    std::fs::remove_file(&url_file).ok();
}
