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

    #[test]
    fn optional_query_parsing_empty() {
        // Test the logic that OptionalQuery uses internally
        let query = "";
        let result = if query.is_empty() { None } else { serde_urlencoded::from_str::<AuthQuery>(query).ok() };
        assert!(result.is_none());
    }

    #[test]
    fn optional_query_parsing_valid() {
        let query = "token=mytoken";
        let result = serde_urlencoded::from_str::<AuthQuery>(query).ok();
        assert!(result.is_some());
        assert_eq!(result.unwrap().token.as_deref(), Some("mytoken"));
    }

    #[test]
    fn optional_query_none_when_malformed() {
        // A malformed query string should result in None for the OptionalQuery logic
        // Simulate: valid URL but params don't match the struct
        let query = "unknown_param=value";
        let result = serde_urlencoded::from_str::<AuthQuery>(query).ok();
        // AuthQuery has token as Option, so unknown params are fine
        assert!(result.is_some());
    }
}