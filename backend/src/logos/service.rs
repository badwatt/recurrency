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
fn extract_vqd(html: &str) -> Option<String> {
    for prefix in &["vqd='", "vqd=\"", "vqd="] {
        if let Some(start) = html.find(prefix) {
            let token_start = start + prefix.len();
            let token_end = html[token_start..]
                .find(|c: char| c == '\'' || c == '"' || c == '&' || c == ';')
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
    let vqd_url = format!("https://duckduckgo.com/?q={}", search_term);

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
}