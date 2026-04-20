use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogoResult {
    pub name: String,
    pub url: String,
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
            results: vec![
                LogoResult {
                    name: "Netflix".into(),
                    url: "https://logo.com/netflix.png".into(),
                },
            ],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: LogoSearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn logo_result_fields() {
        let result = LogoResult {
            name: "Spotify".into(),
            url: "https://logo.com/spotify.png".into(),
        };
        assert_eq!(result.name, "Spotify");
        assert_eq!(result.url, "https://logo.com/spotify.png");
    }

    #[test]
    fn empty_search_response() {
        let resp = LogoSearchResponse {
            results: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: LogoSearchResponse = serde_json::from_str(&json).unwrap();
        assert!(back.results.is_empty());
    }

    #[test]
    fn multiple_results() {
        let resp = LogoSearchResponse {
            results: vec![
                LogoResult { name: "Netflix".into(), url: "https://logo.com/n.png".into() },
                LogoResult { name: "Spotify".into(), url: "https://logo.com/s.png".into() },
                LogoResult { name: "YouTube".into(), url: "https://logo.com/y.png".into() },
            ],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: LogoSearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.results.len(), 3);
        assert_eq!(back.results[1].name, "Spotify");
    }

    #[test]
    fn logo_result_name_and_url() {
        let result = LogoResult {
            name: "Netflix".into(),
            url: "https://logo.com/n.png".into(),
        };
        assert_eq!(result.name, "Netflix");
        assert!(result.url.starts_with("https://"));
    }

    #[test]
    fn logo_search_response_empty() {
        let resp = LogoSearchResponse { results: vec![] };
        assert!(resp.results.is_empty());
    }
}