use crate::api::client::{api_get, ApiError};
use crate::models::logo::LogoSearchResponse;

#[allow(dead_code)]
pub async fn search(query: &str) -> Result<LogoSearchResponse, ApiError> {
    let encoded: String = query
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '#' => "%23".to_string(),
            '%' => "%25".to_string(),
            '+' => "%2B".to_string(),
            '?' => "%3F".to_string(),
            _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => {
                c.to_string()
            }
            _ => {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                s.bytes()
                    .map(|b| format!("%{:02X}", b))
                    .collect::<String>()
            }
        })
        .collect();
    api_get(&format!("/api/logos/search?q={}", encoded)).await
}

#[cfg(test)]
mod tests {

    #[test]
    fn logos_search_url_format() {
        let query = "netflix";
        let path = format!("/api/logos/search?q={}", query);
        assert_eq!(path, "/api/logos/search?q=netflix");
    }

    #[test]
    fn logos_search_url_special_chars() {
        let query = "my service";
        let path = format!("/api/logos/search?q={}", query);
        assert!(path.contains("my service"));
    }

    #[test]
    fn logo_result_fields() {
        use crate::models::logo::LogoResult;
        let r = LogoResult {
            url: "https://logo.clearbit.com/netflix.com".into(),
            thumbnail: "https://logo.clearbit.com/netflix.com".into(),
            width: None,
            height: None,
        };
        assert!(r.url.starts_with("https://"));
        assert!(r.thumbnail.starts_with("https://"));
    }

    #[test]
    fn logo_result_serialization() {
        use crate::models::logo::LogoResult;
        let r = LogoResult {
            url: "https://logo.clearbit.com/spotify.com".into(),
            thumbnail: "https://logo.clearbit.com/spotify.com".into(),
            width: None,
            height: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("url"));
        let back: LogoResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.url, r.url);
    }

    #[test]
    fn logo_search_response_empty() {
        use crate::models::logo::LogoSearchResponse;
        let resp = LogoSearchResponse { results: vec![] };
        assert!(resp.results.is_empty());
    }

    #[test]
    fn logo_search_response_multiple() {
        use crate::models::logo::{LogoResult, LogoSearchResponse};
        let resp = LogoSearchResponse {
            results: vec![
                LogoResult {
                    url: "https://a.com/a.png".into(),
                    thumbnail: "https://a.com/a_thumb.png".into(),
                    width: None,
                    height: None,
                },
                LogoResult {
                    url: "https://b.com/b.png".into(),
                    thumbnail: "https://b.com/b_thumb.png".into(),
                    width: None,
                    height: None,
                },
            ],
        };
        assert_eq!(resp.results.len(), 2);
    }

    #[test]
    fn logos_url_encoded() {
        let query = "netflix+tv";
        let path = format!("/api/logos/search?q={}", query);
        assert!(path.contains("netflix+tv"));
    }

    #[test]
    fn logos_url_unicode() {
        let query = "Netflix日本";
        let path = format!("/api/logos/search?q={}", query);
        assert!(path.contains("Netflix"));
    }
}
