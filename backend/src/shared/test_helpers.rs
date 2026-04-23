//! Test helpers for handler integration tests using axum test utilities.
//!
//! This module provides utilities to test HTTP handlers with mocked services.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde::de::DeserializeOwned;
use tower::util::ServiceExt;

/// Make a JSON POST request to the test app.
pub async fn post_json<T: serde::Serialize, R: DeserializeOwned>(
    app: Router,
    uri: &str,
    body: &T,
    auth_token: Option<&str>,
) -> (StatusCode, R) {
    let json_body = serde_json::to_string(body).unwrap();
    let mut req = Request::post(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(json_body))
        .unwrap();

    if let Some(token) = auth_token {
        req.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    let response = app.oneshot(req).await.unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let response_body: R = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
        panic!(
            "Failed to parse response: {:?}",
            String::from_utf8_lossy(&body_bytes)
        )
    });

    (status, response_body)
}

/// Make a JSON PUT request to the test app.
pub async fn put_json<T: serde::Serialize, R: DeserializeOwned>(
    app: Router,
    uri: &str,
    body: &T,
    auth_token: Option<&str>,
) -> (StatusCode, R) {
    let json_body = serde_json::to_string(body).unwrap();
    let mut req = Request::put(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(json_body))
        .unwrap();

    if let Some(token) = auth_token {
        req.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    let response = app.oneshot(req).await.unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let response_body: R = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
        panic!(
            "Failed to parse response: {:?}",
            String::from_utf8_lossy(&body_bytes)
        )
    });

    (status, response_body)
}

/// Make a DELETE request to the test app.
pub async fn delete_request(app: Router, uri: &str, auth_token: Option<&str>) -> StatusCode {
    let mut req = Request::delete(uri)
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();

    if let Some(token) = auth_token {
        req.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    let response = app.oneshot(req).await.unwrap();

    response.status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;

    #[tokio::test]
    async fn test_get_health() {
        // Test that a simple GET request returns a response.
        // Uses oneshot directly since response is plain text (not JSON).
        let app = Router::new().route("/health", get(|| async { "OK" }));
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&body_bytes[..], b"OK");
    }

    #[tokio::test]
    async fn test_post_accepts_json_body() {
        use axum::routing::post;

        // Simple test: verify mock handler receives and parses request
        let app = Router::new().route(
            "/api/subscriptions",
            post(
                |axum::Json(body): axum::Json<serde_json::Value>| async move {
                    axum::Json(serde_json::json!({"received": body["name"]}))
                },
            ),
        );

        let (status, body): (StatusCode, serde_json::Value) = post_json(
            app,
            "/api/subscriptions",
            &serde_json::json!({"name": "test"}),
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["received"], "test");
    }

    #[tokio::test]
    async fn test_bearer_token_extraction() {
        use axum::extract::Request;

        // Create a simple endpoint that extracts and returns the token
        async fn handler(request: Request) -> String {
            let auth_header = request.headers().get("authorization");
            if let Some(h) = auth_header {
                h.to_str().unwrap_or("invalid").to_string()
            } else {
                "no header".to_string()
            }
        }

        let app = Router::new().route("/test", get(handler));

        let req = Request::get("/test")
            .header("Authorization", "Bearer my-token-123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        assert!(body_str.contains("my-token-123"));
    }

    #[tokio::test]
    async fn helper_put_json_works() {
        use crate::shared::test_helpers::put_json;
        use axum::routing::put;

        async fn handler(
            axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
        ) -> impl axum::response::IntoResponse {
            axum::Json(payload)
        }

        let app = Router::new().route("/test", put(handler));
        let payload = serde_json::json!({"key": "value", "num": 42});

        let (status, body): (StatusCode, serde_json::Value) =
            put_json(app, "/test", &payload, None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, payload);
    }

    #[tokio::test]
    async fn helper_put_json_error() {
        use crate::shared::test_helpers::put_json;
        use axum::routing::put;

        async fn handler() -> impl axum::response::IntoResponse {
            (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"error": "bad"})),
            )
        }

        let app = Router::new().route("/test", put(handler));
        let payload = serde_json::json!({"invalid": true});

        let (status, _body) = put_json::<_, serde_json::Value>(app, "/test", &payload, None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn helper_put_json_with_auth() {
        use crate::shared::test_helpers::put_json;
        use axum::routing::put;

        async fn handler(
            axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
        ) -> impl axum::response::IntoResponse {
            axum::Json(payload)
        }

        let app = Router::new().route("/test", put(handler));
        let payload = serde_json::json!({"name": "test"});

        let (status, body): (StatusCode, serde_json::Value) =
            put_json(app, "/test", &payload, Some("Bearer token123")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, payload);
    }

    #[tokio::test]
    async fn helper_delete_request_works() {
        use crate::shared::test_helpers::delete_request;
        use axum::routing::delete;

        async fn handler() -> impl axum::response::IntoResponse {
            axum::Json(serde_json::json!({"deleted": true}))
        }

        let app = Router::new().route("/test", delete(handler));

        let status = delete_request(app, "/test", None).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn helper_delete_request_not_found() {
        use crate::shared::test_helpers::delete_request;
        use axum::routing::delete;

        async fn handler() -> impl axum::response::IntoResponse {
            (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({"error": "not found"})),
            )
        }

        let app = Router::new().route("/test", delete(handler));
        let status = delete_request(app, "/test", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn helper_delete_request_with_auth() {
        use crate::shared::test_helpers::delete_request;
        use axum::routing::delete;

        async fn handler() -> impl axum::response::IntoResponse {
            axum::Json(serde_json::json!({"deleted": true}))
        }

        let app = Router::new().route("/test", delete(handler));
        let status = delete_request(app, "/test", Some("Bearer mytoken")).await;
        assert_eq!(status, StatusCode::OK);
    }
}
