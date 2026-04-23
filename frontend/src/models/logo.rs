use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogoResult {
    pub url: String,
    pub thumbnail: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogoSearchResponse {
    pub results: Vec<LogoResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_search_roundtrip() {
        let resp = LogoSearchResponse {
            results: vec![LogoResult {
                url: "https://logo.com/netflix.png".into(),
                thumbnail: "https://logo.com/netflix_thumb.png".into(),
                width: Some(200),
                height: Some(100),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: LogoSearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn logo_result_fields() {
        let result = LogoResult {
            url: "https://logo.com/spotify.png".into(),
            thumbnail: "https://logo.com/spotify_thumb.png".into(),
            width: Some(200),
            height: Some(100),
        };
        assert_eq!(result.url, "https://logo.com/spotify.png");
        assert_eq!(result.thumbnail, "https://logo.com/spotify_thumb.png");
        assert_eq!(result.width, Some(200));
        assert_eq!(result.height, Some(100));
    }

    #[test]
    fn empty_search_response() {
        let resp = LogoSearchResponse { results: vec![] };
        let json = serde_json::to_string(&resp).unwrap();
        let back: LogoSearchResponse = serde_json::from_str(&json).unwrap();
        assert!(back.results.is_empty());
    }

    #[test]
    fn multiple_results() {
        let resp = LogoSearchResponse {
            results: vec![
                LogoResult {
                    url: "https://logo.com/n.png".into(),
                    thumbnail: "https://logo.com/n_thumb.png".into(),
                    width: None,
                    height: None,
                },
                LogoResult {
                    url: "https://logo.com/s.png".into(),
                    thumbnail: "https://logo.com/s_thumb.png".into(),
                    width: None,
                    height: None,
                },
                LogoResult {
                    url: "https://logo.com/y.png".into(),
                    thumbnail: "https://logo.com/y_thumb.png".into(),
                    width: None,
                    height: None,
                },
            ],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: LogoSearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.results.len(), 3);
        assert_eq!(back.results[1].url, "https://logo.com/s.png");
    }

    #[test]
    fn logo_result_url_and_thumbnail() {
        let result = LogoResult {
            url: "https://logo.com/n.png".into(),
            thumbnail: "https://logo.com/n_thumb.png".into(),
            width: None,
            height: None,
        };
        assert!(result.url.starts_with("https://"));
        assert!(result.thumbnail.starts_with("https://"));
    }

    #[test]
    fn logo_search_response_empty() {
        let resp = LogoSearchResponse { results: vec![] };
        assert!(resp.results.is_empty());
    }
}
