use axum::http::HeaderMap;

/// Extract auth token from query param (?token=) or Authorization header.
/// Query param takes priority.
pub fn extract_token_from_request(
    query_token: Option<&String>,
    headers: &HeaderMap,
) -> Result<String, crate::shared::error::AppError> {
    // 1. Query param takes priority
    if let Some(token) = query_token
        && !token.is_empty() {
            return Ok(token.clone());
        }

    // 2. Fallback to Authorization: Bearer <token>
    if let Some(auth_header) = headers.get("authorization")
        && let Ok(auth_str) = auth_header.to_str()
            && let Some(token) = auth_str.strip_prefix("Bearer ")
                && !token.is_empty() {
                    return Ok(token.to_string());
                }

    Err(crate::shared::error::AppError::Unauthorized(
        "Missing authentication token".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn query_param_takes_priority() {
        let token = Some(&"query-token".to_string());
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer header-token".parse().unwrap());
        let result = extract_token_from_request(token, &headers).unwrap();
        assert_eq!(result, "query-token");
    }

    #[test]
    fn bearer_header_fallback() {
        let token: Option<&String> = None;
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer my-jwt-token".parse().unwrap());
        let result = extract_token_from_request(token, &headers).unwrap();
        assert_eq!(result, "my-jwt-token");
    }

    #[test]
    fn no_token_returns_error() {
        let token: Option<&String> = None;
        let headers = HeaderMap::new();
        let result = extract_token_from_request(token, &headers);
        assert!(result.is_err());
    }

    #[test]
    fn empty_query_token_falls_back_to_header() {
        let token = Some(&"".to_string());
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer fallback".parse().unwrap());
        let result = extract_token_from_request(token, &headers).unwrap();
        assert_eq!(result, "fallback");
    }

    #[test]
    fn empty_bearer_token_fails() {
        let token: Option<&String> = None;
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer ".parse().unwrap()); // just spaces after Bearer
        let result = extract_token_from_request(token, &headers);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_auth_header_format_fails() {
        let token: Option<&String> = None;
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Basic abc123".parse().unwrap());
        let result = extract_token_from_request(token, &headers);
        assert!(result.is_err());
    }

    #[test]
    fn query_token_with_empty_header() {
        let token = Some(&"from-query".to_string());
        let headers = HeaderMap::new();
        let result = extract_token_from_request(token, &headers).unwrap();
        assert_eq!(result, "from-query");
    }

    #[test]
    fn non_utf8_auth_header_falls_through_to_error() {
        // HeaderValue with invalid UTF-8 bytes -> to_str() returns Err -> falls through.
        let token: Option<&String> = None;
        let mut headers = HeaderMap::new();
        // HeaderValue::from_bytes creates a value that may not be valid UTF-8.
        // For visible ASCII, from_str will succeed but non-ASCII bytes fail to_str.
        // Use a header with raw bytes that aren't valid UTF-8.
        let non_utf8 = axum::http::HeaderValue::from_bytes(b"\xff\xfeBearer token").unwrap();
        headers.insert("authorization", non_utf8);
        let result = extract_token_from_request(token, &headers);
        assert!(result.is_err());
    }
}
