//! Release/update discovery used by the web client.
//!
//! GitHub releases are visible as soon as the release object is created, while
//! Docker images and platform assets are uploaded afterwards. The web update
//! banner must only appear once the matching GHCR image tag exists.

use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::shared::error::{AppError, AppResult};

const GHCR_TOKEN_URL: &str =
    "https://ghcr.io/token?scope=repository:badwatt/recurrency:pull&service=ghcr.io";
const GHCR_MANIFEST_PREFIX: &str = "https://ghcr.io/v2/badwatt/recurrency/manifests/";
const OCI_MANIFEST_ACCEPT: &str = "application/vnd.oci.image.index.v1+json,application/vnd.docker.distribution.manifest.list.v2+json";
const HTTP_TIMEOUT_SECONDS: u64 = 10;
const MAX_TAG_CHARS: usize = 64;

#[derive(Debug, Default, Deserialize)]
pub struct UpdateImageQuery {
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateImageResponse {
    pub image_available: bool,
}

fn normalize_tag(value: &str) -> Option<String> {
    let tag = value.trim().trim_start_matches('v');
    if tag.is_empty()
        || tag.len() > MAX_TAG_CHARS
        || !tag
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
    {
        return None;
    }
    Some(tag.to_string())
}

fn manifest_url(tag: &str) -> String {
    format!("{GHCR_MANIFEST_PREFIX}{tag}")
}

async fn image_available(tag: &str) -> AppResult<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECONDS))
        .build()
        .map_err(|error| AppError::Internal(format!("Could not create update client: {error}")))?;
    let token_response = client.get(GHCR_TOKEN_URL).send().await.map_err(|error| {
        AppError::Internal(format!("Could not check the update image: {error}"))
    })?;
    if !token_response.status().is_success() {
        return Ok(false);
    }
    let token_json: Value = token_response
        .json()
        .await
        .map_err(|error| AppError::Internal(format!("Invalid registry token: {error}")))?;
    let Some(token) = token_json.get("token").and_then(Value::as_str) else {
        return Ok(false);
    };
    let response = client
        .get(manifest_url(tag))
        .bearer_auth(token)
        .header("Accept", OCI_MANIFEST_ACCEPT)
        .send()
        .await
        .map_err(|error| {
            AppError::Internal(format!("Could not check the update image: {error}"))
        })?;
    Ok(response.status().is_success())
}

pub async fn image(Query(query): Query<UpdateImageQuery>) -> AppResult<Json<UpdateImageResponse>> {
    let tag = query
        .tag
        .as_deref()
        .and_then(normalize_tag)
        .ok_or_else(|| AppError::BadRequest("A valid release tag is required".to_string()))?;
    Ok(Json(UpdateImageResponse {
        image_available: image_available(&tag).await?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_normalized_and_validated() {
        assert_eq!(normalize_tag("v0.55.4"), Some("0.55.4".to_string()));
        assert_eq!(normalize_tag(" 0.55.4 "), Some("0.55.4".to_string()));
        assert!(normalize_tag("").is_none());
        assert!(normalize_tag("../latest").is_none());
        assert!(normalize_tag("a b").is_none());
    }

    #[test]
    fn manifest_url_uses_the_normalized_tag() {
        assert_eq!(
            manifest_url("0.55.4"),
            "https://ghcr.io/v2/badwatt/recurrency/manifests/0.55.4"
        );
    }

    #[test]
    fn tags_have_a_bounded_length_and_safe_characters() {
        assert!(normalize_tag(&"a".repeat(MAX_TAG_CHARS)).is_some());
        assert!(normalize_tag(&"a".repeat(MAX_TAG_CHARS + 1)).is_none());
        assert!(normalize_tag("v1_2.3-rc_1").is_some());
        assert!(normalize_tag("v1/2").is_none());
        assert!(normalize_tag("v1:2").is_none());
    }

    #[tokio::test]
    async fn image_route_rejects_missing_and_invalid_tags() {
        let missing = image(Query(UpdateImageQuery { tag: None })).await;
        assert!(matches!(missing, Err(AppError::BadRequest(_))));
        let invalid = image(Query(UpdateImageQuery {
            tag: Some("../latest".to_string()),
        }))
        .await;
        assert!(matches!(invalid, Err(AppError::BadRequest(_))));
    }
}
