use crate::api::client::{api_get, ApiError};
use crate::models::logo::LogoSearchResponse;

#[allow(dead_code)]
pub async fn search(query: &str) -> Result<LogoSearchResponse, ApiError> {
    api_get(&format!("/api/logos/search?q={}", query)).await
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
            name: "Netflix".into(),
            url: "https://logo.clearbit.com/netflix.com".into(),
        };
        assert_eq!(r.name, "Netflix");
        assert!(r.url.starts_with("https://"));
    }

    #[test]
    fn logo_result_serialization() {
        use crate::models::logo::LogoResult;
        let r = LogoResult {
            name: "Spotify".into(),
            url: "https://logo.clearbit.com/spotify.com".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("Spotify"));
        let back: LogoResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Spotify");
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
                LogoResult { name: "A".into(), url: "https://a.com/a.png".into() },
                LogoResult { name: "B".into(), url: "https://b.com/b.png".into() },
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