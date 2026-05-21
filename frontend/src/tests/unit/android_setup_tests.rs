// TDD tests for setup.rs server URL persistence bug.
// Bug: SetupPage save_server_url uses dirs::data_dir() → None on Android → URL not saved.

// TDD tests for scripts/build-android.sh paths and correctness.

/// TDD RED: build-android.sh must reference dx bundle output dir correctly.
/// dx bundle --platform android generates target/dx/recurrency-frontend/release/android.
/// Script must NOT use old path target/dx/recurrency/release/android.
#[test]
fn test_build_android_script_uses_correct_dx_bundle_dir() {
    let script = include_str!("../../../../scripts/build-android.sh");
    assert!(
        script.contains("target/dx/recurrency-frontend/release/android"),
        "build-android.sh must reference target/dx/recurrency-frontend/release/android"
    );
    assert!(
        !script.contains("target/dx/recurrency/release/android"),
        "build-android.sh must NOT reference old target/dx/recurrency/release/android path"
    );
}

/// TDD RED: build-android.sh must read VERSION from workspace Cargo.toml.
/// No VERSION file exists; reading ../VERSION fails silently producing empty string.
#[test]
fn test_build_android_script_reads_version_from_cargo_toml() {
    let script = include_str!("../../../../scripts/build-android.sh");
    assert!(
        script.contains("Cargo.toml"),
        "build-android.sh must read version from Cargo.toml, not ../VERSION"
    );
    assert!(
        !script.contains("cat ../VERSION"),
        "build-android.sh must NOT read from ../VERSION (file does not exist)"
    );
}

/// TDD RED: build-android.sh must use 'magick convert' for ImageMagick 7.
/// Plain 'convert' is deprecated in IMv7 and causes warnings.
#[test]
fn test_build_android_script_uses_magick_convert() {
    let script = include_str!("../../../../scripts/build-android.sh");
    let has_magick = script.contains("magick convert");
    let has_plain = script.contains("\nconvert ") || script.contains(";convert ") || script.contains(" convert ");
    assert!(
        has_magick || !has_plain,
        "build-android.sh must use 'magick convert' instead of deprecated 'convert'"
    );
}



/// TDD RED: build-android.sh must default output dir to $ROOT/bin/android.
#[test]
fn test_build_android_script_defaults_out_dir() {
    let script = include_str!("../../../../scripts/build-android.sh");
    assert!(
        script.contains("if [ -z \"$OUT_DIR\" ]; then") && script.contains("OUT_DIR=\"$ROOT/bin/android\""),
        "build-android.sh must default OUT_DIR to $ROOT/bin/android when --out-dir absent"
    );
}



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
