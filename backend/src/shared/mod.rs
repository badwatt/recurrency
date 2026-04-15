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