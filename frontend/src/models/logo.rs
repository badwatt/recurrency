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
}