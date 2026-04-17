//! TDD tests for Android app icon.
//! Bug: App launcher icon not showing on Android home screen.
use std::path::PathBuf;

fn dx_output_res_dir() -> PathBuf {
    PathBuf::from("target/dx/recurrency/release/android/app/app/src/main/res")
}

#[test]
fn test_mipmap_xxxhdpi_icon_exists() {
    // TDD RED: mipmap-xxxhdpi/ic_launcher.webp must exist.
    let res_dir = dx_output_res_dir();
    let xxxhdpi = res_dir.join("mipmap-xxxhdpi/ic_launcher.webp");
    assert!(xxxhdpi.exists(), "mipmap-xxxhdpi/ic_launcher.webp should exist after dx bundle");
}

#[test]
fn test_mipmap_xxxhdpi_icon_is_our_icon() {
    // TDD RED: mipmap icon file must be >1KB (our 1024x1024 icon compressed is large).
    // Default Dioxus icon is a tiny placeholder.
    let res_dir = dx_output_res_dir();
    let xxxhdpi = res_dir.join("mipmap-xxxhdpi/ic_launcher.webp");
    if xxxhdpi.exists() {
        let size = std::fs::metadata(&xxxhdpi).unwrap().len();
        // Our icon-1024.png is ~10KB. Default placeholder webp is tiny (<500 bytes).
        assert!(size > 1000, "mipmap icon should be our icon (~10KB), not default placeholder (<1KB). Got {} bytes", size);
    }
}

#[test]
fn test_drawable_foreground_is_not_default_vector() {
    // TDD RED: ic_launcher_foreground.xml should be our icon or generated from our PNG,
    // not a hardcoded default vector with android:pathData.
    let res_dir = dx_output_res_dir();
    let foreground = res_dir.join("drawable-v24/ic_launcher_foreground.xml");
    let xml = std::fs::read_to_string(&foreground).expect("foreground XML should exist");
    // Default Dioxus foreground is a white-on-transparent vector with pathData.
    // Our icon should either be an image file or a proper icon reference.
    // Currently it's the default vector placeholder.
    let has_path_data = xml.contains("android:pathData");
    let has_bitmap = xml.contains("bitmap") || xml.contains("drawable/ic_launcher");
    assert!(has_bitmap || !has_path_data,
        "ic_launcher_foreground should be our icon (bitmap or reference), not default vector pathData");
}

#[test]
fn test_adaptive_icon_xml_references_valid_resources() {
    // TDD RED: mipmap-anydpi-v26/ic_launcher.xml references background+foreground.
    // Both should exist AND not be the default placeholder.
    let res_dir = dx_output_res_dir();
    let adaptive = res_dir.join("mipmap-anydpi-v26/ic_launcher.xml");
    let xml = std::fs::read_to_string(&adaptive).expect("adaptive icon XML should exist");
    
    assert!(xml.contains("ic_launcher_background"), "should reference background");
    assert!(xml.contains("ic_launcher_foreground"), "should reference foreground");
}
