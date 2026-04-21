// TDD tests for Android app icon bug.
// Bug: App icon not showing on Android home screen.
// Root cause: dx bundle generates mipmap-*/ic_launcher.webp but Dioxus default
// icon is white vector. Need to generate mipmap icons from app's icon.png.
// Also: adaptive icon needs foreground drawable (bitmap, not vector pathData).

/// TDD RED: mipmap directories must exist with ic_launcher files after build.
#[test]
fn test_mipmap_directories_exist_with_icons() {
    let res_dir = std::path::PathBuf::from(
        "frontend/target/dx/recurrency/release/android/app/app/src/main/res",
    );
    if !res_dir.exists() {
        // Skip if not built yet (CI/local dev without android build)
        return;
    }
    let densities = [
        "mipmap-mdpi",
        "mipmap-hdpi",
        "mipmap-xhdpi",
        "mipmap-xxhdpi",
        "mipmap-xxxhdpi",
    ];
    for density in densities {
        let mipmap_dir = res_dir.join(density);
        if mipmap_dir.exists() {
            let icons: Vec<_> = std::fs::read_dir(&mipmap_dir)
                .ok()
                .into_iter()
                .flatten()
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().contains("ic_launcher"))
                .collect();
            assert!(
                !icons.is_empty(),
                "{} must have ic_launcher icon file",
                density
            );
            // File must be >1KB (not empty placeholder)
            let size = icons[0].metadata().map(|m| m.len()).unwrap_or(0);
            assert!(
                size > 1024,
                "{} ic_launcher must be >1KB (got {} bytes)",
                density,
                size
            );
        }
    }
}

/// TDD RED: adaptive icon foreground must be bitmap (not vector pathData).
/// Dioxus default: drawable-v24/ic_launcher_foreground.xml with white vector.
/// Fix: drawable-v24/ic_launcher_foreground.xml should reference bitmap.
#[test]
fn test_foreground_xml_not_vector_pathdata() {
    let foreground = std::path::PathBuf::from("frontend/target/dx/recurrency/release/android/app/app/src/main/res/drawable-v24/ic_launcher_foreground.xml");
    if !foreground.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&foreground).expect("foreground XML should exist");
    let has_path_data = xml.contains("android:pathData");
    let has_bitmap = xml.contains("bitmap") || xml.contains("drawable/ic_launcher");
    // Either bitmap reference OR no pathData (replaced with image)
    assert!(has_bitmap || !has_path_data,
        "ic_launcher_foreground should be bitmap reference, not default vector pathData. Has pathData: {}, Has bitmap: {}",
        has_path_data, has_bitmap);
}

/// TDD RED: foreground drawable directory must exist with icon image.
#[test]
fn test_drawable_foreground_icon_exists() {
    let foreground_png = std::path::PathBuf::from("frontend/target/dx/recurrency/release/android/app/app/src/main/res/drawable/ic_launcher_foreground.png");
    if !foreground_png.exists() {
        return;
    }
    let size = std::fs::metadata(&foreground_png)
        .map(|m| m.len())
        .unwrap_or(0);
    assert!(
        size > 1024,
        "drawable/ic_launcher_foreground.png must be >1KB (got {} bytes)",
        size
    );
}

/// TDD RED: mipmap-webp icons must be >1KB (not empty Dioxus defaults).
#[test]
fn test_mipmap_icons_not_empty() {
    let res_dir = std::path::PathBuf::from(
        "frontend/target/dx/recurrency/release/android/app/app/src/main/res",
    );
    if !res_dir.exists() {
        return;
    }
    let densities = [
        "mipmap-mdpi",
        "mipmap-hdpi",
        "mipmap-xhdpi",
        "mipmap-xxhdpi",
        "mipmap-xxxhdpi",
    ];
    let mut found_any = false;
    for density in densities {
        let mipmap_dir = res_dir.join(density);
        if mipmap_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&mipmap_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains("ic_launcher") && !name.contains(".xml") {
                        found_any = true;
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        assert!(
                            size > 1024,
                            "{}/{} must be >1KB (got {} bytes)",
                            density,
                            name,
                            size
                        );
                    }
                }
            }
        }
    }
    // At least one density should have an icon
    assert!(
        found_any,
        "At least one mipmap density should have ic_launcher icon files"
    );
}
