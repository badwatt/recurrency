//! Embedded SPA frontend for the standalone all-in-one binary.

use axum::{
    body::Body,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};

static WEB: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist/public");

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "wasm" => "application/wasm",
        "woff2" => "font/woff2",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

pub fn enabled() -> bool {
    std::env::var("ENABLE_WEB")
        .map(|value| value.to_lowercase() != "false")
        .unwrap_or(true)
}

pub async fn serve(uri: Uri) -> Response {
    if uri.path().starts_with("/api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let requested = uri.path().trim_start_matches('/');
    let requested = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };
    let file = WEB.get_file(requested).or_else(|| {
        if !requested.contains('.') {
            WEB.get_file("index.html")
        } else {
            None
        }
    });
    let Some(file) = file else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let path = file.path().to_string_lossy();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type(&path))
        .body(Body::from(file.contents().to_vec()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
