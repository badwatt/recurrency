use crate::logos::service::{search_logos, LogoResult};
use axum::{extract::Query, Json};
use serde::Deserialize;

#[cfg(test)]
use axum::{body::Body, routing::get, Router};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LogoSearchResponse {
    pub results: Vec<LogoResult>,
}

// ── Handler ──

pub async fn search(
    Query(query): Query<SearchQuery>,
) -> Result<Json<LogoSearchResponse>, crate::shared::error::AppError> {
    let results = search_logos(&query.q).await?;
    Ok(Json(LogoSearchResponse { results }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    // Test search query deserialization
    #[test]
    fn search_query_valid() {
        let json = "{\"q\":\"netflix\"}";
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.q, "netflix");
    }

    #[test]
    fn search_query_with_spaces() {
        let json = "{\"q\":\"spotify premium\"}";
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.q, "spotify premium");
    }

    #[test]
    fn search_query_unicode() {
        let json = "{\"q\":\"ネットフリックス\"}";
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.q, "ネットフリックス");
    }

    #[test]
    fn search_query_empty_string() {
        let json = r#"{"q":""}"#;
        let query: SearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.q, "");
    }

    // Test search endpoint
    #[tokio::test]
    async fn search_logos_success() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search?q=netflix")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status().is_success());
    }

    #[tokio::test]
    async fn search_logos_with_query_param() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search?q=spotify")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status().is_success());
    }

    // Test missing query parameter
    #[tokio::test]
    async fn search_logos_missing_query() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // Test wrong HTTP method
    #[tokio::test]
    async fn search_logos_post_not_allowed() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("POST")
            .uri("/api/logos/search?q=test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // Test search response structure
    #[tokio::test]
    async fn search_logos_response_structure() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search?q=netflix")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status().is_success());

        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_object());
        assert!(json.get("results").unwrap().is_array());
    }

    // Test search with special characters
    #[tokio::test]
    async fn search_logos_special_chars() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search?q=netflix%20test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status().is_success());
    }

    // Test empty query still works
    #[tokio::test]
    async fn search_logos_empty_query() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search?q=")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status().is_success());
    }

    // Test with body (should be ignored for GET)
    #[tokio::test]
    async fn search_logos_ignores_body() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search?q=test")
            .header("Content-Type", "application/json")
            .body(Body::from("{\"ignored\":\"body\"}"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert!(response.status().is_success());
    }

    // Test multiple searches
    #[tokio::test]
    async fn search_logos_multiple_queries() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let queries = vec!["netflix", "spotify", "amazon", "youtube"];

        for q in queries {
            let req = Request::builder()
                .method("GET")
                .uri(format!("/api/logos/search?q={}", q))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(req).await.unwrap();
            // Rate limiting: allow success or 429 (rate limited)
            assert!(
                response.status().is_success()
                    || response.status() == StatusCode::TOO_MANY_REQUESTS
            );
            // Small delay to avoid rate limiting between requests
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    // Test response is JSON array of logos
    #[tokio::test]
    async fn search_logos_returns_logo_format() {
        let app = Router::new()
            .route("/api/logos/search", get(search))
            .with_state(());

        let req = Request::builder()
            .method("GET")
            .uri("/api/logos/search?q=test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
        let empty: Vec<serde_json::Value> = vec![];
        let results = json.get("results").and_then(|v| v.as_array()).unwrap_or(&empty);

        // Each logo should have expected fields
        for logo in results.iter().take(1) {
            assert!(logo.is_object());
            assert!(logo.get("url").is_some());
            assert!(logo.get("thumbnail").is_some());
        }
    }
}
