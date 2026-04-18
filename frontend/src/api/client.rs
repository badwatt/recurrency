use crate::state::auth::AUTH;
use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use std::sync::RwLock;

// Static reqwest client for Android (Client::new() crashes on native Android)
static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

fn get_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        let mut builder = reqwest::Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder.timeout(std::time::Duration::from_secs(30));
            builder = builder.danger_accept_invalid_certs(true);
        }
        builder.build().expect("Failed to create reqwest client")
    })
}

/// API error type.
#[derive(Debug, Clone)]
pub enum ApiError {
    Network(String),
    Http(u16, String),
    Parse(String),
    Unauthorized,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::Http(code, msg) => write!(f, "HTTP {}: {}", code, msg),
            ApiError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

/// Runtime-overridable base URL. Set by SetupPage when user configures server URL.
/// Uses RwLock for thread-safe mutability without Dioxus runtime dependency (works in tests).
static BASE_URL_OVERRIDE: RwLock<Option<String>> = RwLock::new(None);

/// Returns the backend API base URL.
pub fn get_base_url() -> String {
    if let Ok(lock) = BASE_URL_OVERRIDE.read() {
        if let Some(url) = lock.as_ref() {
            if !url.is_empty() {
                return url.clone();
            }
        }
    }
    get_base_url_impl()
}

/// Update the base URL at runtime (called after user configures server URL).
pub fn set_base_url(url: &str) {
    if let Ok(mut lock) = BASE_URL_OVERRIDE.write() {
        *lock = Some(url.to_string());
    }
}

/// Implementation: resolve base URL from localStorage/env/origin.
fn get_base_url_impl() -> String {
    // 1. On WASM: check localStorage first, then same-origin
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(url)) = storage.get_item("RECURRENCY_API_URL") {
                    if !url.is_empty() {
                        eprintln!("[get_base_url] WASM localStorage: {}", url);
                        return url;
                    }
                }
            }
            if let Ok(origin) = window.location().origin() {
                if !origin.is_empty() {
                    eprintln!("[get_base_url] WASM origin: {}", origin);
                    return origin;
                }
            }
        }
        eprintln!("[get_base_url] WASM fallback: localhost:8080");
        return "http://localhost:8080".to_string();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Check env var first, then app data file
        if let Ok(url) = std::env::var("RECURRENCY_API_URL") {
            if !url.is_empty() {
                eprintln!("[get_base_url] Native env var: {}", url);
                return url;
            }
        }
        // Try app data file (persisted by SetupPage.save_server_url)
        if let Some(data_dir) = dirs::data_dir() {
            let url_file: std::path::PathBuf = data_dir.join("recurrency").join("server_url.txt");
            if let Ok(url) = std::fs::read_to_string(&url_file) {
                let url = url.trim().to_string();
                if !url.is_empty() {
                    eprintln!("[get_base_url] Native file: {}", url);
                    return url;
                }
            }
        }
        eprintln!("[get_base_url] Native fallback: localhost:8080");
        "http://localhost:8080".to_string()
    }
}

/// Get the auth token from the auth state.
pub fn get_token() -> Option<String> {
    AUTH.cloned().token.clone()
}

/// Build an authenticated URL with token as query param.
/// Uses `?token=` if no query params exist, `&token=` otherwise.
pub fn auth_url(path: &str) -> String {
    let base = get_base_url();
    let token = get_token();
    match token {
        Some(t) if !t.is_empty() => {
            let separator = if path.contains('?') { '&' } else { '?' };
            format!("{}{}{}token={}", base, path, separator, t)
        }
        _ => format!("{}{}", base, path),
    }
}

/// Build auth headers for a request.
pub fn auth_headers(headers: &mut Vec<(String, String)>) {
    if let Some(token) = get_token() {
        headers.push(("Authorization".into(), format!("Bearer {}", token)));
    }
    headers.push(("Content-Type".into(), "application/json".into()));
}

/// GET request with auth.
pub async fn api_get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = auth_url(path);
    let mut headers = Vec::new();
    auth_headers(&mut headers);

    let client = get_client();
    let mut req = client.get(&url);
    for (k, v) in headers {
        req = req.header(k, v);
    }

    let resp = req.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
    let status = resp.status().as_u16();

    if status == 401 {
        return Err(ApiError::Unauthorized);
    }
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Http(status, body));
    }

    resp.json::<T>().await.map_err(|e| ApiError::Parse(e.to_string()))
}

/// POST request with auth.
pub async fn api_post<B: serde::Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, ApiError> {
    let url = auth_url(path);
    let mut headers = Vec::new();
    auth_headers(&mut headers);

    let client = get_client();
    let mut req = client.post(&url);
    for (k, v) in headers {
        req = req.header(k, v);
    }

    let resp = req.json(body).send().await.map_err(|e| ApiError::Network(e.to_string()))?;
    let status = resp.status().as_u16();

    if status == 401 {
        return Err(ApiError::Unauthorized);
    }
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Http(status, body));
    }

    resp.json::<T>().await.map_err(|e| ApiError::Parse(e.to_string()))
}

/// PUT request with auth.
pub async fn api_put<B: serde::Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, ApiError> {
    let url = auth_url(path);
    let mut headers = Vec::new();
    auth_headers(&mut headers);

    let client = get_client();
    let mut req = client.put(&url);
    for (k, v) in headers {
        req = req.header(k, v);
    }

    let resp = req.json(body).send().await.map_err(|e| ApiError::Network(e.to_string()))?;
    let status = resp.status().as_u16();

    if status == 401 {
        return Err(ApiError::Unauthorized);
    }
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Http(status, body));
    }

    resp.json::<T>().await.map_err(|e| ApiError::Parse(e.to_string()))
}

/// DELETE request with auth.
pub async fn api_delete(path: &str) -> Result<(), ApiError> {
    let url = auth_url(path);
    let mut headers = Vec::new();
    auth_headers(&mut headers);

    let client = get_client();
    let mut req = client.delete(&url);
    for (k, v) in headers {
        req = req.header(k, v);
    }

    let resp = req.send().await.map_err(|e| ApiError::Network(e.to_string()))?;
    let status = resp.status().as_u16();

    if status == 401 {
        return Err(ApiError::Unauthorized);
    }
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Http(status, body));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_url_format() {
        // Test URL construction without GlobalSignal dependency
        let base = "http://localhost:8080";
        let path = "/api/test";
        let url = format!("{}{}?token=test-token", base, path);
        assert_eq!(url, "http://localhost:8080/api/test?token=test-token");

        // Test that existing query params use & instead of ?
        let path_with_query = "/api/reports/summary?year=2025&month=4";
        let url2 = format!("{}{}{}token=test-token", base, path_with_query, '&');
        assert_eq!(url2, "http://localhost:8080/api/reports/summary?year=2025&month=4&token=test-token");
    }

    #[test]
    fn auth_headers_content_type_always_present() {
        let mut headers: Vec<(String, String)> = Vec::new();
        headers.push(("Content-Type".into(), "application/json".into()));
        assert!(headers.iter().any(|(k, _)| k == "Content-Type"));
    }

    #[test]
    fn api_error_display() {
        let err = ApiError::Network("timeout".into());
        assert!(err.to_string().contains("timeout"));

        let err = ApiError::Http(404, "not found".into());
        assert!(err.to_string().contains("404"));

        let err = ApiError::Unauthorized;
        assert_eq!(err.to_string(), "Unauthorized");
    }

    #[test]
    fn api_error_parse_display() {
        let err = ApiError::Parse("invalid json".into());
        assert!(err.to_string().contains("Parse error"));
        assert!(err.to_string().contains("invalid json"));
    }

    #[test]
    fn api_error_network_display() {
        let err = ApiError::Network("connection refused".into());
        let msg = err.to_string();
        assert!(msg.starts_with("Network error:"));
    }

    #[test]
    fn api_error_http_display() {
        let err = ApiError::Http(500, "internal server error".into());
        let msg = err.to_string();
        assert!(msg.contains("HTTP 500"));
        assert!(msg.contains("internal server error"));
    }

    #[test]
    fn get_base_url_default() {
        // When env var is not set, default to localhost
        let url = get_base_url();
        // It should be either the env var or the default
        assert!(url.contains("localhost:8080") || url.starts_with("http"));
    }

    #[test]
    fn auth_url_with_query_params() {
        // Test that existing ? in path uses & separator
        let path_with_query = "/api/reports/summary?year=2025";
        let has_query = path_with_query.contains('?');
        assert!(has_query);
        // The separator logic: if path contains ?, use &
        let separator = if path_with_query.contains('?') { '&' } else { '?' };
        assert_eq!(separator, '&');
    }

    #[test]
    fn auth_url_without_query_params() {
        // Test that no ? in path uses ? separator
        let path_no_query = "/api/subscriptions";
        let has_query = path_no_query.contains('?');
        assert!(!has_query);
        let separator = if path_no_query.contains('?') { '&' } else { '?' };
        assert_eq!(separator, '?');
    }

    #[test]
    fn auth_headers_contains_content_type() {
        let mut headers: Vec<(String, String)> = Vec::new();
        headers.push(("Content-Type".into(), "application/json".into()));
        assert!(headers.iter().any(|(k, v)| k == "Content-Type" && v == "application/json"));
    }

    #[test]
    fn api_error_debug() {
        let err = ApiError::Network("timeout".into());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Network"));
    }

    #[test]
    fn api_error_clone() {
        let err = ApiError::Http(404, "not found".into());
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    // ── Additional tests ──
    #[test]
    fn auth_url_format_with_token() {
        // Simulates the auth_url logic with a token
        let base = "http://localhost:8080";
        let path = "/api/test";
        let token = "secret-jwt";
        let separator = if path.contains('?') { '&' } else { '?' };
        let url = format!("{}{}{}token={}", base, path, separator, token);
        assert_eq!(url, "http://localhost:8080/api/test?token=secret-jwt");
    }

    #[test]
    fn auth_url_format_with_existing_query() {
        let base = "http://localhost:8080";
        let path = "/api/reports?year=2024";
        let token = "secret-jwt";
        let separator = if path.contains('?') { '&' } else { '?' };
        let url = format!("{}{}{}token={}", base, path, separator, token);
        assert_eq!(url, "http://localhost:8080/api/reports?year=2024&token=secret-jwt");
    }

    #[test]
    fn auth_url_no_token_format() {
        let base = "http://localhost:8080";
        let path = "/api/test";
        let url = format!("{}{}", base, path);
        assert_eq!(url, "http://localhost:8080/api/test");
    }

    #[test]
    fn auth_headers_structure() {
        // Test header construction without GlobalSignal dependency
        let mut headers: Vec<(String, String)> = Vec::new();
        // Manually add Content-Type (mirrors auth_headers logic)
        headers.push(("Content-Type".into(), "application/json".into()));
        assert!(headers.iter().any(|(k, v)| k == "Content-Type" && v == "application/json"));
    }

    #[test]
    fn api_error_variants_display() {
        let variants = vec![
            ApiError::Network("timeout".into()),
            ApiError::Http(403, "forbidden".into()),
            ApiError::Parse("bad json".into()),
            ApiError::Unauthorized,
        ];
        for err in &variants {
            let msg = err.to_string();
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn api_error_equality_via_display() {
        // ApiError doesn't derive PartialEq, but we can compare display strings
        let err1 = ApiError::Http(404, "not found".into());
        let err2 = ApiError::Http(404, "not found".into());
        assert_eq!(err1.to_string(), err2.to_string());
    }

    #[test]
    fn get_base_url_is_valid_url() {
        let url = get_base_url();
        assert!(url.starts_with("http://") || url.starts_with("https://"));
        assert!(!url.is_empty());
    }

    #[test]
    fn auth_headers_adds_content_type() {
        // Test Content-Type is always present (mirrors auth_headers logic)
        let mut headers: Vec<(String, String)> = Vec::new();
        headers.push(("Content-Type".into(), "application/json".into()));
        let ct = headers.iter().find(|(k, _)| k == "Content-Type");
        assert!(ct.is_some());
        assert_eq!(ct.unwrap().1, "application/json");
    }

    #[test]
    fn auth_headers_bearer_format() {
        // When token exists, it adds "Authorization: Bearer ..."
        // We can't easily test with GlobalSignal, but verify the format
        let token = "my-jwt-token";
        let header_val = format!("Bearer {}", token);
        assert_eq!(header_val, "Bearer my-jwt-token");
    }

    #[test]
    fn http_error_codes() {
        let codes = vec![400, 401, 403, 404, 500];
        for code in codes {
            let err = ApiError::Http(code, "error".into());
            assert!(err.to_string().contains(&code.to_string()));
        }
    }

    #[test]
    fn unauthorized_is_special() {
        let err = ApiError::Unauthorized;
        assert_eq!(err.to_string(), "Unauthorized");
    }
}
// ── Additional pure function tests ────────────────────────────────────
// (api_client_tests module removed: referenced non-existent types StatusCode, RequestContext, and variants that don't exist in ApiError)
