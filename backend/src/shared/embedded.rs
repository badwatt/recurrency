//! Embedded frontend detection and initialization.
//! Provides functions to check if frontend files are available.

use include_dir::include_dir;

/// Embedded dist/ directory from frontend/
/// This macro expands at compile time to include all files in frontend/dist/
static DIST_DIR: include_dir::Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");

/// Check if embedded dist has files
pub fn has_embedded() -> bool {
    DIST_DIR.get_file("public/index.html").is_some()
}

/// Initialize embedded frontend system.
/// Returns true if dist/ was found and embedded.
pub fn init() -> bool {
    let build_ui = std::env::var("BUILD_UI").unwrap_or_else(|_| "true".into());
    if build_ui != "true" {
        tracing::info!("BUILD_UI=false, frontend serving disabled");
        return false;
    }

    if !has_embedded() {
        tracing::warn!("No embedded frontend files. Run `dx bundle --platform web` first or set BUILD_UI=false.");
        return false;
    }

    tracing::info!("Frontend embedded: {} files available", count_files());
    true
}

/// Count files in dist/
fn count_files() -> usize {
    DIST_DIR.files().count()
}

/// Get file content and mime type by path from embedded dist.
/// Returns (content_bytes, mime_type) if found.
/// dx bundle outputs to dist/public/, so files are under public/ in include_dir.
pub fn get_file(path: &str) -> Option<(&'static [u8], String)> {
    let file_path = if path.is_empty() || path == "/" || path == "index.html" {
        "public/index.html"
    } else {
        &format!("public/{}", path)
    };
    let file = DIST_DIR.get_file(file_path)?;
    let ct = mime_guess::from_path(path).first_or_octet_stream();
    Some((file.contents(), ct.to_string()))
}
