use crate::api::client::{api_get, ApiError};
use crate::models::logo::*;

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
}