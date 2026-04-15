use axum::http::HeaderMap;

/// Extract auth token from query param (?token=) or Authorization header.
/// Query param takes priority.
pub fn extract_token_from_request(query_token: Option<&String>, headers: &HeaderMap) -> Result<String, crate::shared::error::AppError> {
    // 1. Query param takes priority
    if let Some(token) = query_token {
        if !token.is_empty() {
            return Ok(token.clone());
        }
    }

    // 2. Fallback to Authorization: Bearer <token>
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Ok(token.to_string());
                }
            }
        }
    }

    Err(crate::shared::error::AppError::Unauthorized("Missing authentication token".into()))
}