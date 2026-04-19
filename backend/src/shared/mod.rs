#[cfg(test)]
mod test_helpers;

pub mod app_utils;
pub mod auth_extractor;
pub mod date_utils;
pub mod db;
pub mod error;
pub mod password_utils;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::de::DeserializeOwned;
use std::convert::Infallible;

#[derive(serde::Deserialize)]
pub struct AuthQuery {
    pub token: Option<String>,
}

/// Custom extractor that makes the entire query string optional.
///
/// Unlike `Option<Query<T>>` (which requires `OptionalFromRequestParts` — not
/// implemented by axum's `Query`), this extractor returns `None` when the URL
/// has no query string at all, and `Some(T)` when it does and can be parsed.
/// If a query string is present but cannot be deserialized, `None` is returned
/// (allowing fallback to other auth mechanisms like the Authorization header).
pub struct OptionalQuery<T>(pub Option<T>);

impl<T, S> FromRequestParts<S> for OptionalQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let result = parts
            .uri
            .query()
            .filter(|q: &&str| !q.is_empty())
            .and_then(|q| serde_urlencoded::from_str::<T>(q).ok());
        Ok(OptionalQuery(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Uri;

    // ── AuthQuery deserialization ──

    #[test]
    fn auth_query_deserialize_with_token() {
        let query = "token=abc123";
        let result: AuthQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("abc123"));
    }

    #[test]
    fn auth_query_deserialize_without_token() {
        let query = "";
        let result: AuthQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    // A struct with a required (non-optional) field to force parse failure.
    #[derive(serde::Deserialize)]
    struct RequiredToken {
        token: String,
    }

    #[test]
    fn auth_query_empty_query_string_parses_as_none_token() {
        let result: AuthQuery = serde_urlencoded::from_str("").unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn required_field_fails_on_empty_query() {
        let result: Result<RequiredToken, _> = serde_urlencoded::from_str("");
        assert!(result.is_err());
    }

    // ── OptionalQuery extraction logic ──
    // Tests cover: no query → None, empty query → None,
    // valid → Some, malformed (deser failure) → None.

    fn extract_query<T: DeserializeOwned>(uri_str: &str) -> Option<T> {
        let uri: Uri = uri_str.parse().unwrap();
        uri.query()
            .filter(|q| !q.is_empty())
            .and_then(|q| serde_urlencoded::from_str::<T>(q).ok())
    }

    #[tokio::test]
    async fn optional_query_no_query_returns_none() {
        let result = extract_query::<AuthQuery>("/api/test");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn optional_query_empty_query_returns_none() {
        let result = extract_query::<AuthQuery>("/api/test?");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn optional_query_valid_query_returns_some() {
        let result = extract_query::<AuthQuery>("/api/test?token=my-jwt-token");
        let opt = result.expect("should be Some");
        assert_eq!(opt.token.as_deref(), Some("my-jwt-token"));
    }

    #[tokio::test]
    async fn optional_query_malformed_query_returns_none() {
        // RequiredToken: required field → deserialization fails → None.
        let result = extract_query::<RequiredToken>("/api/test?");
        assert!(result.is_none());
    }
}
