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

/// Determine MIME type from file extension.
fn mime_from_path(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "text/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("map") => "application/json",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    }
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
    Some((file.contents(), mime_from_path(path).to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_embedded_files() {
        assert!(has_embedded(), "No embedded frontend files found. Run dx bundle first.");
    }

    #[test]
    fn get_index_html() {
        let (content, ct) = get_file("index.html").expect("index.html not found");
        assert!(content.len() > 0);
        assert_eq!(ct, "text/html");
    }

    #[test]
    fn get_main_js() {
        let result = get_file("assets/recurrency.js");
        // May or may not exist depending on build
        if let Some((content, ct)) = result {
            assert!(content.len() > 0);
            assert_eq!(ct, "text/javascript");
        }
    }
}
