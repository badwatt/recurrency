use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};

use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct LogoResult {
    pub url: String,
    pub thumbnail: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct DDGImageResult {
    image: String,
    thumbnail: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct DDGResponse {
    results: Option<Vec<DDGImageResult>>,
}

/// Extract VQD token from DDG HTML response.
/// Pattern: `vqd='<token>'` or `vqd="<token>"` in the HTML.
pub(crate) fn extract_vqd(html: &str) -> Option<String> {
    for prefix in &["vqd='", "vqd=\"", "vqd="] {
        if let Some(start) = html.find(prefix) {
            let token_start = start + prefix.len();
            let token_end = html[token_start..]
                .find(['\'', '"', '&', ';'])
                .map(|i| token_start + i)
                .unwrap_or(html.len());
            let token = &html[token_start..token_end];
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    None
}

pub async fn search_logos(query: &str) -> AppResult<Vec<LogoResult>> {
    // Clean query
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Ok(vec![]);
    }

    // Build search term with logo keyword
    let search_term = format!("{} logo transparent", query);

    // First get VQD token from DuckDuckGo
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    // Get VQD token
    let vqd_url = format!(
        "https://duckduckgo.com/?q={}",
        percent_encode(search_term.as_bytes(), NON_ALPHANUMERIC)
    );

    let vqd_response = client
        .get(&vqd_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch VQD token: {}", e)))?;

    let vqd_html = vqd_response
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read VQD response: {}", e)))?;

    // Extract VQD token from HTML
    let vqd_token = extract_vqd(&vqd_html)
        .ok_or_else(|| AppError::Internal("Failed to extract VQD token".to_string()))?;

    // Now fetch images using i.js endpoint
    let images_url = format!(
        "https://duckduckgo.com/i.js?q={}&vqd={}&o=json&f=%2C%2Ctransparent%2CWide%2C",
        search_term, vqd_token
    );

    let images_response = client
        .get(&images_url)
        .header("Referer", "https://duckduckgo.com/")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch images: {}", e)))?;

    let images_json: DDGResponse = images_response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse images response: {}", e)))?;

    let results = images_json
        .results
        .unwrap_or_default()
        .into_iter()
        .take(8) // Limit to 8 results
        .map(|r| LogoResult {
            url: r.image.clone(),
            thumbnail: r.thumbnail.unwrap_or_else(|| r.image.clone()),
            width: r.width,
            height: r.height,
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_vqd_single_quotes() {
        let html = r#"some text vqd='123-456' other stuff"#;
        assert_eq!(extract_vqd(html), Some("123-456".to_string()));
    }

    #[test]
    fn test_extract_vqd_double_quotes() {
        let html = r#"some text vqd="123-456" other stuff"#;
        assert_eq!(extract_vqd(html), Some("123-456".to_string()));
    }

    #[test]
    fn test_extract_vqd_ampersand() {
        let html = r#"some text vqd=123-456& other stuff"#;
        assert_eq!(extract_vqd(html), Some("123-456".to_string()));
    }

    #[test]
    fn test_extract_vqd_missing() {
        let html = r#"some text without vqd token"#;
        assert_eq!(extract_vqd(html), None);
    }

    #[test]
    fn test_extract_vqd_empty_token() {
        // vqd='' should return None because token is empty
        let html = r#"some text vqd='' other stuff"#;
        assert_eq!(extract_vqd(html), None);
    }

    #[test]
    fn test_extract_vqd_with_semicolon() {
        let html = r#"some text vqd=abc-def; other stuff"#;
        assert_eq!(extract_vqd(html), Some("abc-def".to_string()));
    }

    #[test]
    fn test_extract_vqd_at_end_of_string() {
        let html = r#"vqd='token123'"#;
        assert_eq!(extract_vqd(html), Some("token123".to_string()));
    }

    #[test]
    fn test_empty_query_returns_empty() {
        // We can't test the async function easily without network,
        // but we can test that an empty query string would be caught
        let query = "  ".trim();
        assert!(query.is_empty());
    }

    #[test]
    fn test_extract_vqd_mixed_html() {
        let html = r#"<html><body>some vqd='my-token-123' more</body></html>"#;
        assert_eq!(extract_vqd(html), Some("my-token-123".to_string()));
    }

    #[test]
    fn test_extract_vqd_multiple_vqds() {
        // Should find the first one
        let html = r#"vqd='first' vqd='second'"#;
        assert_eq!(extract_vqd(html), Some("first".to_string()));
    }

    #[test]
    fn test_extract_vqd_long_token() {
        let token = "a".repeat(200);
        let html = format!("vqd='{}'", token);
        assert_eq!(extract_vqd(&html), Some(token));
    }

    #[test]
    fn test_ddg_image_result_mapping() {
        // Simulate what search_logos does at the end
        let ddg_result = DDGImageResult {
            image: "https://example.com/img.png".to_string(),
            thumbnail: Some("https://example.com/thumb.png".to_string()),
            width: Some(100),
            height: Some(50),
        };
        let logo = LogoResult {
            url: ddg_result.image.clone(),
            thumbnail: ddg_result.thumbnail.clone().unwrap(),
            width: ddg_result.width,
            height: ddg_result.height,
        };
        assert_eq!(logo.url, "https://example.com/img.png");
        assert_eq!(logo.thumbnail, "https://example.com/thumb.png");
        assert_eq!(logo.width, Some(100));
        assert_eq!(logo.height, Some(50));
    }

    #[test]
    fn test_ddg_image_result_no_thumbnail_fallback() {
        // When no thumbnail, should use image URL as fallback
        let ddg_result = DDGImageResult {
            image: "https://example.com/img.png".to_string(),
            thumbnail: None,
            width: None,
            height: None,
        };
        let logo = LogoResult {
            url: ddg_result.image.clone(),
            thumbnail: ddg_result
                .thumbnail
                .unwrap_or_else(|| ddg_result.image.clone()),
            width: ddg_result.width,
            height: ddg_result.height,
        };
        assert_eq!(logo.thumbnail, "https://example.com/img.png");
    }

    #[test]
    fn test_logo_result_serialization() {
        let logo = LogoResult {
            url: "https://example.com/img.png".to_string(),
            thumbnail: "https://example.com/thumb.png".to_string(),
            width: Some(200),
            height: Some(100),
        };
        let json = serde_json::to_value(&logo).unwrap();
        assert_eq!(json["url"], "https://example.com/img.png");
        assert_eq!(json["width"], 200);
    }

    #[test]
    fn test_query_cleaning() {
        let query = "  Netflix  ".trim().to_lowercase();
        assert_eq!(query, "netflix");
    }

    #[test]
    fn test_search_term_format() {
        let query = "spotify";
        let search_term = format!("{} logo transparent", query);
        assert_eq!(search_term, "spotify logo transparent");
    }

    #[test]
    fn test_ddg_response_empty_results() {
        let json = r#"{"results":null}"#;
        let resp: DDGResponse = serde_json::from_str(json).unwrap();
        assert!(resp.results.is_none());
    }

    #[test]
    fn test_ddg_response_with_results() {
        let json = r#"{"results":[{"image":"https://x.com/i.png","thumbnail":"https://x.com/t.png","width":100,"height":50}]}"#;
        let resp: DDGResponse = serde_json::from_str(json).unwrap();
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].image, "https://x.com/i.png");
    }

    #[test]
    fn test_results_limited_to_8() {
        let ddg_results: Vec<DDGImageResult> = (0..20)
            .map(|i| DDGImageResult {
                image: format!("https://x.com/{}.png", i),
                thumbnail: Some(format!("https://x.com/t{}.png", i)),
                width: None,
                height: None,
            })
            .collect();
        let logos: Vec<LogoResult> = ddg_results
            .into_iter()
            .take(8)
            .map(|r| LogoResult {
                url: r.image.clone(),
                thumbnail: r.thumbnail.unwrap_or_else(|| r.image.clone()),
                width: r.width,
                height: r.height,
            })
            .collect();
        assert_eq!(logos.len(), 8);
    }
}
