use axum::{
    extract::Query,
    response::Json,
};
use serde::Deserialize;

use crate::logos::service;
use crate::shared::error::AppResult;

#[derive(Debug, Deserialize)]
pub struct LogoSearchQuery {
    pub q: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LogoSearchResponse {
    pub results: Vec<service::LogoResult>,
}

pub async fn search(
    Query(params): Query<LogoSearchQuery>,
) -> AppResult<Json<LogoSearchResponse>> {
    let results = service::search_logos(&params.q).await?;
    
    Ok(Json(LogoSearchResponse { results }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_search_query_deserialize() {
        let query = "q=netflix";
        let result: LogoSearchQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.q, "netflix");
    }

    #[test]
    fn logo_search_query_empty_q() {
        let query = "q=";
        let result: LogoSearchQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.q.is_empty());
    }

    #[test]
    fn logo_search_response_serialization() {
        let resp = LogoSearchResponse {
            results: vec![],
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("results").unwrap().is_array());
    }
}
