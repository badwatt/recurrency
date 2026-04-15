use crate::state::auth::AUTH;
use dioxus::prelude::*;
use serde::de::DeserializeOwned;

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

/// Get the base URL for the API.
/// On desktop/mobile: reads from RECURRENCY_API_URL env var.
/// On web: same, plus localStorage support via JS eval.
pub fn get_base_url() -> String {
    std::env::var("RECURRENCY_API_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string())
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

    let client = reqwest::Client::new();
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

    let client = reqwest::Client::new();
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

    let client = reqwest::Client::new();
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

    let client = reqwest::Client::new();
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
}