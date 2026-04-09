use regex::Regex;
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
        urlencoding::encode(&search_term)
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

    // Extract VQD token using regex
    let vqd_regex = Regex::new(r"vqd=([\d-]+)&")
        .map_err(|e| AppError::Internal(format!("Invalid regex: {}", e)))?;

    let vqd_token = vqd_regex
        .captures(&vqd_html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| AppError::Internal("Failed to extract VQD token".to_string()))?;

    // Now fetch images using i.js endpoint
    let images_url = format!(
        "https://duckduckgo.com/i.js?q={}&vqd={}&o=json&f=%2C%2Ctransparent%2CWide%2C",
        urlencoding::encode(&search_term),
        vqd_token
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
    fn test_vqd_regex() {
        let html = r#"some text vqd=123-456& other stuff"#;
        let re = Regex::new(r"vqd=([\d-]+)&").unwrap();
        let caps = re.captures(html).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "123-456");
    }
}
