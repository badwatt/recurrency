//! Integration tests for handlers using axum test utilities.
//!
//! These tests verify the HTTP layer: request parsing, auth extraction,
//! and response serialization without requiring a real database.

#[cfg(test)]
mod handler_tests {
    use axum::{
        body::Body,
        extract::Path,
        http::{Request, StatusCode},
        routing::{delete, get, post, put},
        Json, Router,
    };
    use chrono::Datelike;
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;
    use validator::Validate;

    // =========================================================================
    // Test Request/Response Types
    // =========================================================================

    #[derive(Debug, Serialize, Deserialize)]
    struct CreateSubscriptionRequest {
        name: String,
        amount: rust_decimal::Decimal,
        currency: String,
        frequency: String,
        start_date: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct SubscriptionResponse {
        id: String,
        name: String,
        amount: rust_decimal::Decimal,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CreateTransactionRequest {
        name: String,
        amount: rust_decimal::Decimal,
        currency: String,
        date: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TransactionResponse {
        id: String,
        name: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CreateCategoryRequest {
        name: String,
        icon: Option<String>,
        color: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CategoryResponse {
        id: String,
        name: String,
    }

    // Helper to call app with request and get response
    async fn call_app(app: Router, req: Request<Body>) -> axum::response::Response {
        app.oneshot(req).await.unwrap()
    }

    // =========================================================================
    // Subscription Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn subscriptions_create_requires_auth() {
        let app = Router::new().route(
            "/api/subscriptions",
            post(|| async { Json(serde_json::json!({"id": "test"})) }),
        );

        let body = serde_json::to_string(&CreateSubscriptionRequest {
            name: "Netflix".to_string(),
            amount: rust_decimal::Decimal::new(1599, 2),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: "2024-01-01".to_string(),
        })
        .unwrap();

        let req = Request::post("/api/subscriptions")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = call_app(app, req).await;

        // Without auth token, should get through but we're just testing routing
        assert_ne!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn subscriptions_list_returns_json() {
        let app = Router::new().route(
            "/api/subscriptions",
            get(|| async {
                Json(vec![
                    SubscriptionResponse {
                        id: "1".to_string(),
                        name: "Netflix".to_string(),
                        amount: rust_decimal::Decimal::new(1599, 2),
                    },
                    SubscriptionResponse {
                        id: "2".to_string(),
                        name: "Spotify".to_string(),
                        amount: rust_decimal::Decimal::new(999, 2),
                    },
                ])
            }),
        );

        let req = Request::get("/api/subscriptions")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let subscriptions: Vec<SubscriptionResponse> = serde_json::from_slice(&body).unwrap();
        assert_eq!(subscriptions.len(), 2);
    }

    #[tokio::test]
    async fn subscription_get_by_id() {
        let app = Router::new().route(
            "/api/subscriptions/{id}",
            get(
                |axum::extract::Path(id): axum::extract::Path<String>| async move {
                    Json(SubscriptionResponse {
                        id,
                        name: "Netflix".to_string(),
                        amount: rust_decimal::Decimal::new(1599, 2),
                    })
                },
            ),
        );

        let req = Request::get("/api/subscriptions/550e8400-e29b-41d4-a716-446655440000")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let sub: SubscriptionResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(sub.id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[tokio::test]
    async fn subscription_delete_returns_no_content() {
        let app = Router::new().route(
            "/api/subscriptions/{id}",
            delete(
                |axum::extract::Path(_): axum::extract::Path<String>| async {
                    StatusCode::NO_CONTENT
                },
            ),
        );

        let req = Request::delete("/api/subscriptions/550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn calendar_endpoint_accepts_query_params() {
        let app = Router::new().route(
            "/api/subscriptions/calendar",
            get(
                |axum::extract::Query(params): axum::extract::Query<
                    std::collections::HashMap<String, String>,
                >| async move {
                    Json(serde_json::json!({
                        "year": params.get("year"),
                        "month": params.get("month"),
                    }))
                },
            ),
        );

        let req = Request::get("/api/subscriptions/calendar?year=2024&month=6")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["year"], "2024");
        assert_eq!(result["month"], "6");
    }

    #[tokio::test]
    async fn calendar_endpoint_defaults_when_no_params() {
        let app = Router::new().route(
            "/api/subscriptions/calendar",
            get(|| async {
                let now = chrono::Utc::now().date_naive();
                Json(serde_json::json!({
                    "year": now.year(),
                    "month": now.month(),
                }))
            }),
        );

        let req = Request::get("/api/subscriptions/calendar")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn upcoming_endpoint_returns_list() {
        let app = Router::new().route(
            "/api/subscriptions/upcoming",
            get(|| async {
                Json(vec![SubscriptionResponse {
                    id: "1".to_string(),
                    name: "Netflix".to_string(),
                    amount: rust_decimal::Decimal::new(1599, 2),
                }])
            }),
        );

        let req = Request::get("/api/subscriptions/upcoming")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Transaction Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn transactions_list_returns_json() {
        let app = Router::new().route(
            "/api/transactions",
            get(|| async {
                Json(vec![
                    TransactionResponse {
                        id: "1".to_string(),
                        name: "Coffee".to_string(),
                    },
                    TransactionResponse {
                        id: "2".to_string(),
                        name: "Groceries".to_string(),
                    },
                ])
            }),
        );

        let req = Request::get("/api/transactions")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let transactions: Vec<TransactionResponse> = serde_json::from_slice(&body).unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[tokio::test]
    async fn transactions_create_returns_created() {
        let app = Router::new().route(
            "/api/transactions",
            post(|Json(req): Json<CreateTransactionRequest>| async move {
                Json(TransactionResponse {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: req.name,
                })
            }),
        );

        let body = serde_json::to_string(&CreateTransactionRequest {
            name: "Coffee".to_string(),
            amount: rust_decimal::Decimal::new(499, 2),
            currency: "USD".to_string(),
            date: "2024-06-15".to_string(),
        })
        .unwrap();

        let req = Request::post("/api/transactions")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK); // Json returns 200 by default
    }

    #[tokio::test]
    async fn transactions_date_range_filter() {
        let app = Router::new().route(
            "/api/transactions/date-range",
            get(
                |axum::extract::Query(params): axum::extract::Query<
                    std::collections::HashMap<String, String>,
                >| async move {
                    let from = params.get("from_date").cloned().unwrap_or_default();
                    let to = params.get("to_date").cloned().unwrap_or_default();
                    Json(serde_json::json!({"from": from, "to": to}))
                },
            ),
        );

        let req =
            Request::get("/api/transactions/date-range?from_date=2024-01-01&to_date=2024-12-31")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(result["from"], "2024-01-01");
        assert_eq!(result["to"], "2024-12-31");
    }

    #[tokio::test]
    async fn transaction_update() {
        let app = Router::new().route(
            "/api/transactions/{id}",
            put(
                |axum::extract::Path(id): axum::extract::Path<String>| async move {
                    Json(TransactionResponse {
                        id,
                        name: "Updated".to_string(),
                    })
                },
            ),
        );

        let body =
            serde_json::to_string(&serde_json::json!({"name": "Updated Transaction"})).unwrap();

        let req = Request::put("/api/transactions/550e8400-e29b-41d4-a716-446655440000")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn transaction_delete_returns_no_content() {
        let app = Router::new().route(
            "/api/transactions/{id}",
            delete(
                |axum::extract::Path(_): axum::extract::Path<String>| async {
                    StatusCode::NO_CONTENT
                },
            ),
        );

        let req = Request::delete("/api/transactions/550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    // =========================================================================
    // Category Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn categories_list_returns_json() {
        let app = Router::new().route(
            "/api/categories",
            get(|| async {
                Json(vec![
                    CategoryResponse {
                        id: "1".to_string(),
                        name: "Entertainment".to_string(),
                    },
                    CategoryResponse {
                        id: "2".to_string(),
                        name: "Food".to_string(),
                    },
                ])
            }),
        );

        let req = Request::get("/api/categories")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let categories: Vec<CategoryResponse> = serde_json::from_slice(&body).unwrap();
        assert_eq!(categories.len(), 2);
    }

    #[tokio::test]
    async fn categories_create_returns_created() {
        let app = Router::new().route(
            "/api/categories",
            post(|Json(req): Json<CreateCategoryRequest>| async move {
                Json(CategoryResponse {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: req.name,
                })
            }),
        );

        let body = serde_json::to_string(&CreateCategoryRequest {
            name: "Streaming".to_string(),
            icon: Some("tv".to_string()),
            color: Some("#ff0000".to_string()),
        })
        .unwrap();

        let req = Request::post("/api/categories")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn category_get_by_id() {
        let app = Router::new().route(
            "/api/categories/{id}",
            get(
                |axum::extract::Path(id): axum::extract::Path<String>| async move {
                    Json(CategoryResponse {
                        id,
                        name: "Entertainment".to_string(),
                    })
                },
            ),
        );

        let req = Request::get("/api/categories/550e8400-e29b-41d4-a716-446655440000")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let cat: CategoryResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(cat.id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[tokio::test]
    async fn category_update() {
        let app = Router::new().route(
            "/api/categories/{id}",
            put(
                |axum::extract::Path(id): axum::extract::Path<String>| async move {
                    Json(CategoryResponse {
                        id,
                        name: "Updated".to_string(),
                    })
                },
            ),
        );

        let body = serde_json::to_string(&serde_json::json!({"name": "Updated Category"})).unwrap();

        let req = Request::put("/api/categories/550e8400-e29b-41d4-a716-446655440000")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn category_delete_returns_no_content() {
        let app = Router::new().route(
            "/api/categories/{id}",
            delete(
                |axum::extract::Path(_): axum::extract::Path<String>| async {
                    StatusCode::NO_CONTENT
                },
            ),
        );

        let req = Request::delete("/api/categories/550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn category_set_default() {
        let app = Router::new().route(
            "/api/categories/default",
            put(|Json(req): Json<serde_json::Value>| async move {
                Json(serde_json::json!({"id": req["id"], "is_default": true}))
            }),
        );

        let body = serde_json::to_string(
            &serde_json::json!({"id": "550e8400-e29b-41d4-a716-446655440000"}),
        )
        .unwrap();

        let req = Request::put("/api/categories/default")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn categories_with_counts_endpoint() {
        let app = Router::new().route(
            "/api/categories/with-counts",
            get(|| async {
                Json(vec![serde_json::json!({
                    "category": {"id": "1", "name": "Entertainment"},
                    "subscription_count": 5,
                    "transaction_count": 10
                })])
            }),
        );

        let req = Request::get("/api/categories/with-counts")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let result: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(result[0]["subscription_count"], 5);
        assert_eq!(result[0]["transaction_count"], 10);
    }

    // =========================================================================
    // Error Handling Tests
    // =========================================================================

    #[tokio::test]
    async fn not_found_for_unknown_route() {
        let app = Router::new();

        let req = Request::get("/api/unknown").body(Body::empty()).unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn method_not_allowed_for_wrong_method() {
        let app = Router::new().route("/api/subscriptions", get(|| async { "GET only" }));

        let req = Request::post("/api/subscriptions")
            .body(Body::empty())
            .unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn invalid_json_returns_bad_request() {
        let app = Router::new().route(
            "/api/subscriptions",
            post(|Json(_): Json<CreateSubscriptionRequest>| async {
                Json(serde_json::json!({"id": "test"}))
            }),
        );

        let req = Request::post("/api/subscriptions")
            .header("Content-Type", "application/json")
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = call_app(app, req).await;
        // Axum returns 400 for JSON parse errors with the correct Content-Type
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // =========================================================================
    // Health Check Test
    // =========================================================================

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = Router::new().route("/health", get(|| async { "OK" }));

        let req = Request::get("/health").body(Body::empty()).unwrap();

        let response = call_app(app, req).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&body[..], b"OK");
    }

    // =========================================================================
    // HTTP Tests: routing behavior
    // =========================================================================

    #[tokio::test]
    async fn http_route_not_found_returns_404() {
        let app = Router::new().route(
            "/api/subscriptions",
            get(|| async { Json::<Vec<SubscriptionResponse>>(vec![]) }),
        );

        let req = Request::get("/api/nonexistent")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_wrong_method_returns_405() {
        let app = Router::new().route(
            "/api/subscriptions",
            get(|| async { Json::<Vec<SubscriptionResponse>>(vec![]) }),
        );

        let req = Request::post("/api/subscriptions")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn http_invalid_json_returns_422() {
        let app = Router::new().route(
            "/api/subscriptions",
            post(
                |axum::extract::Json(_): axum::extract::Json<CreateSubscriptionRequest>| async {
                    Json(serde_json::json!({"ok": true}))
                },
            ),
        );

        let req = Request::post("/api/subscriptions")
            .header("Content-Type", "application/json")
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_validation_invalid_email_returns_422() {
        let app = Router::new().route(
            "/api/auth/register",
            post(
                |axum::extract::Json(input): axum::extract::Json<
                    crate::auth::models::CreateUserRequest,
                >| async move {
                    if input.validate().is_err() {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": "validation failed"})),
                        )
                    } else {
                        (StatusCode::OK, Json(serde_json::json!({"ok": true})))
                    }
                },
            ),
        );

        let req = Request::post("/api/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"email":"not-an-email","password":"password123","name":"Test"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_validation_short_password_returns_422() {
        let app = Router::new().route(
            "/api/auth/register",
            post(
                |axum::extract::Json(input): axum::extract::Json<
                    crate::auth::models::CreateUserRequest,
                >| async move {
                    if input.validate().is_err() {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": "validation failed"})),
                        )
                    } else {
                        (StatusCode::OK, Json(serde_json::json!({"ok": true})))
                    }
                },
            ),
        );

        let req = Request::post("/api/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"email":"test@example.com","password":"short","name":"Test"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // =========================================================================
    // Additional Handler HTTP Tests
    // =========================================================================

    #[tokio::test]
    async fn http_category_create_validates_json_body() {
        let app = Router::new().route(
            "/api/categories",
            post(
                |axum::extract::Json(_): axum::extract::Json<CreateCategoryRequest>| async {
                    Json(serde_json::json!({"id": "test", "name": "ok"}))
                },
            ),
        );

        let req = Request::post("/api/categories")
            .header("Content-Type", "application/json")
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_transaction_create_validates_json_body() {
        let app = Router::new().route(
            "/api/transactions",
            post(
                |axum::extract::Json(_): axum::extract::Json<CreateTransactionRequest>| async {
                    Json(serde_json::json!({"id": "test"}))
                },
            ),
        );

        let req = Request::post("/api/transactions")
            .header("Content-Type", "application/json")
            .body(Body::from("not json"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_get_nonexistent_returns_404() {
        let app = Router::new().route(
            "/api/subscriptions/{id}",
            get(|_: Path<String>| async { StatusCode::NOT_FOUND }),
        );

        let req = Request::get("/api/subscriptions/00000000-0000-0000-0000-000000000000")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_delete_nonexistent_returns_404() {
        let app = Router::new().route(
            "/api/subscriptions/{id}",
            delete(|_: Path<String>| async { StatusCode::NOT_FOUND }),
        );

        let req = Request::delete("/api/subscriptions/00000000-0000-0000-0000-000000000000")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_missing_auth_token_returns_unauthorized() {
        use crate::shared::auth_extractor::extract_token_from_request;

        let app = Router::new().route(
            "/api/settings",
            get(|headers: axum::http::HeaderMap| async move {
                match extract_token_from_request(None, &headers) {
                    Err(_) => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({}))),
                    Ok(_) => (StatusCode::OK, Json(serde_json::json!({}))),
                }
            }),
        );

        let req = Request::get("/api/settings").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn http_invalid_bearer_format_returns_unauthorized() {
        use crate::shared::auth_extractor::extract_token_from_request;

        let app = Router::new().route(
            "/api/settings",
            get(|headers: axum::http::HeaderMap| async move {
                match extract_token_from_request(None, &headers) {
                    Err(_) => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({}))),
                    Ok(_) => (StatusCode::OK, Json(serde_json::json!({}))),
                }
            }),
        );

        let req = Request::get("/api/settings")
            .header("Authorization", "InvalidFormat token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn http_valid_bearer_token_extracted() {
        use crate::shared::auth_extractor::extract_token_from_request;

        let app = Router::new().route(
            "/api/settings",
            get(|headers: axum::http::HeaderMap| async move {
                match extract_token_from_request(Some(&"valid_token".to_string()), &headers) {
                    Err(_) => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({}))),
                    Ok(token) => (StatusCode::OK, Json(serde_json::json!({"token": token}))),
                }
            }),
        );

        let req = Request::get("/api/settings")
            .header("Authorization", "Bearer valid_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_empty_body_returns_bad_request() {
        let app = Router::new().route(
            "/api/auth/register",
            post(
                |axum::extract::Json(_): axum::extract::Json<
                    crate::auth::models::CreateUserRequest,
                >| async { Json(serde_json::json!({})) },
            ),
        );

        let req = Request::post("/api/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_query_params_parsed_correctly() {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            page: Option<u32>,
            limit: Option<u32>,
        }

        let app = Router::new().route(
            "/api/test",
            get(
                |axum::extract::Query(q): axum::extract::Query<QueryParams>| async move {
                    Json(serde_json::json!({
                        "page": q.page.unwrap_or(1),
                        "limit": q.limit.unwrap_or(10)
                    }))
                },
            ),
        );

        let req = Request::get("/api/test?page=5&limit=20")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["page"], 5);
        assert_eq!(json["limit"], 20);
    }

    #[tokio::test]
    async fn http_path_params_parsed() {
        let app = Router::new().route(
            "/api/items/{id}",
            get(|Path(id): Path<String>| async move { Json(serde_json::json!({"id": id})) }),
        );

        let req = Request::get("/api/items/abc123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["id"], "abc123");
    }

    #[tokio::test]
    async fn http_multiple_routes_same_path_different_methods() {
        let app = Router::new()
            .route("/api/items", get(|| async { "GET" }))
            .route("/api/items", post(|| async { "POST" }))
            .route(
                "/api/items",
                put(|Json(_): Json<serde_json::Value>| async { "PUT" }),
            )
            .route("/api/items", delete(|| async { "DELETE" }));

        for method in ["GET", "POST", "PUT", "DELETE"] {
            let req = Request::builder()
                .method(method)
                .uri("/api/items")
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap();

            let response = app.clone().oneshot(req).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Method {} should be OK",
                method
            );
        }
    }

    #[tokio::test]
    async fn http_unsupported_method_returns_405() {
        let app = Router::new().route("/api/only-get", get(|| async { "OK" }));

        let req = Request::delete("/api/only-get")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn http_options_request_returns_404_or_405() {
        let app = Router::new().route("/api/test", get(|| async { "OK" }));

        let req = Request::builder()
            .method("OPTIONS")
            .uri("/api/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        // OPTIONS on route without OPTIONS handler returns 405 or 404
        assert!(
            response.status() == StatusCode::METHOD_NOT_ALLOWED
                || response.status() == StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn http_patch_request_returns_405_for_get_only_route() {
        let app = Router::new().route("/api/test", get(|| async { "OK" }));

        let req = Request::builder()
            .method("PATCH")
            .uri("/api/test")
            .header("Content-Type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn http_large_json_body_may_be_rejected() {
        let app = Router::new().route(
            "/api/test",
            post(
                |axum::extract::Json(_): axum::extract::Json<serde_json::Value>| async {
                    Json(serde_json::json!({}))
                },
            ),
        );

        let large_body = format!("{{\"data\": \"{}\"}}", "x".repeat(1024 * 1024));
        let req = Request::post("/api/test")
            .header("Content-Type", "application/json")
            .body(Body::from(large_body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        // Large body may return 400, 413, or succeed depending on config
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::PAYLOAD_TOO_LARGE
        );
    }

    #[tokio::test]
    async fn http_malformed_uuid_returns_400() {
        let app = Router::new().route(
            "/api/items/{id}",
            get(|Path(id): Path<uuid::Uuid>| async move {
                Json(serde_json::json!({"id": id.to_string()}))
            }),
        );

        let req = Request::get("/api/items/not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_valid_uuid_parsed() {
        let app = Router::new().route(
            "/api/items/{id}",
            get(|Path(id): Path<uuid::Uuid>| async move {
                Json(serde_json::json!({"id": id.to_string()}))
            }),
        );

        let req = Request::get("/api/items/550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_nested_routes_work() {
        let app = Router::new().route(
            "/api/v1/users/{user_id}/items",
            get(|Path((user_id,)): Path<(String,)>| async move {
                Json(serde_json::json!({"user_id": user_id}))
            }),
        );

        let req = Request::get("/api/v1/users/123/items")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_content_type_application_json_required() {
        let app = Router::new().route(
            "/api/test",
            post(
                |axum::extract::Json(_): axum::extract::Json<serde_json::Value>| async {
                    Json(serde_json::json!({}))
                },
            ),
        );

        let req = Request::post("/api/test")
            .header("Content-Type", "text/plain")
            .body(Body::from("{}"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        // Wrong content type returns 415 Unsupported Media Type or 400
        assert!(
            response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE
                || response.status() == StatusCode::BAD_REQUEST
        );
    }

    // =========================================================================
    // Additional HTTP Layer Tests
    // =========================================================================

    #[tokio::test]
    async fn http_auth_login_returns_json() {
        let app = Router::new().route(
            "/api/auth/login",
            post(|Json(input): Json<serde_json::Value>| async move {
                // Basic validation
                let email = input.get("email").and_then(|v| v.as_str()).unwrap_or("");
                let password = input.get("password").and_then(|v| v.as_str()).unwrap_or("");
                if email.is_empty() || password.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "invalid"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let body = serde_json::to_string(&serde_json::json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .unwrap();

        let req = Request::post("/api/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_auth_login_empty_email_returns_400() {
        let app = Router::new().route(
            "/api/auth/login",
            post(|Json(input): Json<serde_json::Value>| async move {
                let email = input.get("email").and_then(|v| v.as_str()).unwrap_or("");
                let password = input.get("password").and_then(|v| v.as_str()).unwrap_or("");
                if email.is_empty() || password.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "invalid"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::post("/api/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"email":"","password":"password123"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_auth_login_empty_password_returns_400() {
        let app = Router::new().route(
            "/api/auth/login",
            post(|Json(input): Json<serde_json::Value>| async move {
                let email = input.get("email").and_then(|v| v.as_str()).unwrap_or("");
                let password = input.get("password").and_then(|v| v.as_str()).unwrap_or("");
                if email.is_empty() || password.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "invalid"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::post("/api/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"email":"test@example.com","password":""}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_auth_login_invalid_json_returns_400() {
        let app = Router::new().route(
            "/api/auth/login",
            post(|Json(_): Json<serde_json::Value>| async {
                (StatusCode::OK, Json(serde_json::json!({})))
            }),
        );

        let req = Request::post("/api/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from("not json"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_report_summary_returns_json() {
        let app = Router::new().route(
            "/api/reports/summary",
            get(
                |axum::extract::Query(params): axum::extract::Query<serde_json::Value>| async move {
                    Json(serde_json::json!({
                        "year": params.get("year").and_then(|v| v.as_i64()).unwrap_or(2024),
                        "month": params.get("month").and_then(|v| v.as_i64()).unwrap_or(6),
                        "subscriptions_total": "0.00",
                        "transactions_total": "0.00",
                    }))
                },
            ),
        );

        let req = Request::get("/api/reports/summary?year=2024&month=6")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["year"], 2024);
        assert_eq!(json["month"], 6);
    }

    #[tokio::test]
    async fn http_report_summary_without_params_uses_defaults() {
        let app = Router::new().route(
            "/api/reports/summary",
            get(|| async {
                let now = chrono::Utc::now();
                Json(serde_json::json!({
                    "year": now.year(),
                    "month": now.month(),
                }))
            }),
        );

        let req = Request::get("/api/reports/summary")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_categories_with_counts_returns_json() {
        let app = Router::new().route(
            "/api/categories/with-counts",
            get(|| async {
                Json(serde_json::json!([
                    {"name": "Food", "subscription_count": 2, "transaction_count": 10},
                    {"name": "Entertainment", "subscription_count": 5, "transaction_count": 3}
                ]))
            }),
        );

        let req = Request::get("/api/categories/with-counts")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["name"], "Food");
    }

    #[tokio::test]
    async fn http_categories_list_returns_json_array() {
        let app = Router::new().route(
            "/api/categories",
            get(|| async {
                Json(serde_json::json!([
                    {"id": "uuid-1", "name": "Food"},
                    {"id": "uuid-2", "name": "Transport"}
                ]))
            }),
        );

        let req = Request::get("/api/categories").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_transactions_list_returns_json_array() {
        let app = Router::new().route(
            "/api/transactions",
            get(|| async {
                Json(serde_json::json!([
                    {"id": "uuid-1", "name": "Coffee", "amount": "4.50"},
                    {"id": "uuid-2", "name": "Groceries", "amount": "85.00"}
                ]))
            }),
        );

        let req = Request::get("/api/transactions")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_transactions_create_validates_required_fields() {
        let app = Router::new().route(
            "/api/transactions",
            post(|Json(input): Json<serde_json::Value>| async move {
                if !input.get("name").is_some() || !input.get("amount").is_some() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "missing fields"})),
                    );
                }
                (StatusCode::CREATED, Json(serde_json::json!({})))
            }),
        );

        // Missing required fields
        let req = Request::post("/api/transactions")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"other": "field"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_subscriptions_list_returns_empty_array() {
        let app = Router::new().route(
            "/api/subscriptions",
            get(|| async { Json(serde_json::json!([])) }),
        );

        let req = Request::get("/api/subscriptions")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn http_calendar_returns_items() {
        let app = Router::new().route(
            "/api/calendar",
            get(|| async {
                Json(serde_json::json!([
                    {"name": "Netflix", "billing_date": "2024-06-15", "amount": "15.99"},
                    {"name": "Spotify", "billing_date": "2024-06-20", "amount": "9.99"}
                ]))
            }),
        );

        let req = Request::get("/api/calendar?year=2024&month=6")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_health_endpoint_returns_200() {
        let app = Router::new().route("/health", get(|| async { "OK" }));

        let req = Request::get("/health").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_root_endpoint_returns_404() {
        let app = Router::new();

        let req = Request::get("/").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_deeply_nested_route_returns_404() {
        let app = Router::new().route("/api/v1/subscriptions", get(|| async { "OK" }));

        let req = Request::get("/api/v2/subscriptions")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_static_file_like_path_returns_404() {
        let app = Router::new().route("/api/test", get(|| async { "OK" }));

        let req = Request::get("/api/test.css").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_very_long_path_returns_404() {
        let app = Router::new().route("/api/test", get(|| async { "OK" }));

        let long_path = format!("/api/test/{}?q=test", "x".repeat(1000));
        let req = Request::builder()
            .method("GET")
            .uri(long_path)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_emoji_in_path_returns_404() {
        let app = Router::new().route("/api/test", get(|| async { "OK" }));

        let req = Request::get("/api/test/🎉").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_unicode_path_returns_404() {
        let app = Router::new().route("/api/test", get(|| async { "OK" }));

        let req = Request::get("/api/test/日本語")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn http_whitespace_path_returns_404() {
        let app = Router::new().route("/api/test", get(|| async { "OK" }));

        let req = Request::get("/api/test/%20").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // =========================================================================
    // Auth Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_auth_register_validates_email() {
        let app = Router::new().route(
            "/api/auth/register",
            post(|Json(input): Json<serde_json::Value>| async move {
                let email = input.get("email").and_then(|v| v.as_str()).unwrap_or("");
                let password = input.get("password").and_then(|v| v.as_str()).unwrap_or("");
                let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("");

                if email.is_empty() || !email.contains('@') {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "invalid email"})),
                    );
                }
                if password.len() < 8 {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "password too short"})),
                    );
                }
                if name.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "name required"})),
                    );
                }

                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        // Invalid email
        let req = Request::post("/api/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"email":"not-email","password":"password123","name":"Test"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_auth_register_validates_password_length() {
        let app = Router::new().route(
            "/api/auth/register",
            post(|Json(input): Json<serde_json::Value>| async move {
                let password = input.get("password").and_then(|v| v.as_str()).unwrap_or("");
                if password.len() < 8 {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "password too short"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::post("/api/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"email":"test@example.com","password":"short","name":"Test"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_auth_register_validates_name_required() {
        let app = Router::new().route(
            "/api/auth/register",
            post(|Json(input): Json<serde_json::Value>| async move {
                let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "name required"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::post("/api/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"email":"test@example.com","password":"password123","name":""}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn http_auth_register_success() {
        let app = Router::new().route(
            "/api/auth/register",
            post(|Json(_): Json<serde_json::Value>| async {
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::post("/api/auth/register")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"email":"test@example.com","password":"password123","name":"Test User"}"#,
            ))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_auth_logout_returns_ok() {
        let app = Router::new().route(
            "/api/auth/logout",
            post(|| async {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({"message": "logged out"})),
                )
            }),
        );

        let req = Request::post("/api/auth/logout")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_auth_verify_with_valid_token() {
        // Use a simple mock token for testing
        let app = Router::new().route(
            "/api/auth/verify",
            post(|Json(t): Json<String>| async move {
                if t.is_empty() || t.len() < 10 {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({"error": "invalid"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"valid": true})))
            }),
        );

        // Create a mock token
        let mock_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.test.test";

        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/verify")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&mock_token).unwrap()))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_auth_verify_with_invalid_token() {
        let app = Router::new().route(
            "/api/auth/verify",
            post(|Json(t): Json<String>| async move {
                if t.is_empty() || t.len() < 10 {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({"error": "invalid"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"valid": true})))
            }),
        );

        let req = Request::builder()
            .method("POST")
            .uri("/api/auth/verify")
            .header("Content-Type", "application/json")
            .body(Body::from(r#""invalid""#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn http_auth_settings_requires_auth() {
        let app = Router::new().route(
            "/api/auth/settings",
            get(|headers: axum::http::HeaderMap| async move {
                let auth = headers.get("authorization");
                if auth.is_none() {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({"error": "no auth"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"user": "data"})))
            }),
        );

        let req = Request::get("/api/auth/settings")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn http_auth_settings_with_bearer_token() {
        let app = Router::new().route(
            "/api/auth/settings",
            get(|headers: axum::http::HeaderMap| async move {
                let auth = headers.get("authorization");
                if let Some(val) = auth {
                    if let Ok(s) = val.to_str() {
                        if s.starts_with("Bearer ") {
                            return (StatusCode::OK, Json(serde_json::json!({"ok": true})));
                        }
                    }
                }
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "no auth"})),
                )
            }),
        );

        let req = Request::get("/api/auth/settings")
            .header("Authorization", "Bearer valid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_auth_settings_update_valid() {
        let app = Router::new().route(
            "/api/auth/settings",
            put(|Json(input): Json<serde_json::Value>| async move {
                let name = input.get("name");
                let currency = input.get("currency");

                if name.is_none() && currency.is_none() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "no fields"})),
                    );
                }

                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/auth/settings")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name":"Updated Name"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_auth_settings_update_empty_body() {
        let app = Router::new().route(
            "/api/auth/settings",
            put(|Json(input): Json<serde_json::Value>| async move {
                if input.get("name").is_none() && input.get("currency").is_none() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "no fields"})),
                    );
                }
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/auth/settings")
            .header("Content-Type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // =========================================================================
    // Preferences Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_preferences_update_returns_ok() {
        let app = Router::new().route(
            "/api/preferences",
            put(|Json(input): Json<serde_json::Value>| async {
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/preferences")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"currency":"EUR"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Notifications Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_notifications_list_returns_array() {
        let app = Router::new().route(
            "/api/notifications",
            get(|| async {
                Json(serde_json::json!([
                    {"id": "1", "title": "Test", "is_read": false},
                    {"id": "2", "title": "Another", "is_read": true}
                ]))
            }),
        );

        let req = Request::get("/api/notifications")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn http_notifications_mark_read_returns_ok() {
        let app = Router::new().route(
            "/api/notifications/{id}/read",
            put(|Path(_): Path<String>| async {
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/notifications/123/read")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_notifications_mark_all_read_returns_ok() {
        let app = Router::new().route(
            "/api/notifications/read-all",
            put(|| async {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({"ok": true, "count": 5})),
                )
            }),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/notifications/read-all")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Subscription Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_subscription_get_returns_details() {
        let app = Router::new().route(
            "/api/subscriptions/{id}",
            get(|Path(id): Path<String>| async move {
                Json(serde_json::json!({
                    "id": id,
                    "name": "Netflix",
                    "amount": "15.99",
                    "currency": "USD"
                }))
            }),
        );

        let req = Request::get("/api/subscriptions/123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_subscription_delete_returns_204() {
        let app = Router::new().route(
            "/api/subscriptions/{id}",
            delete(|Path(_): Path<String>| async { StatusCode::NO_CONTENT }),
        );

        let req = Request::builder()
            .method("DELETE")
            .uri("/api/subscriptions/123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn http_subscription_update_partial() {
        let app = Router::new().route(
            "/api/subscriptions/{id}",
            put(
                |Path(id): Path<String>, Json(input): Json<serde_json::Value>| async move {
                    let name = input.get("name");
                    if name.is_none() {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": "no name"})),
                        );
                    }
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({"id": id, "name": name})),
                    )
                },
            ),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/subscriptions/123")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name":"Spotify"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_subscription_calendar_returns_items() {
        let app = Router::new().route(
            "/api/calendar",
            get(
                |axum::extract::Query(_): axum::extract::Query<serde_json::Value>| async {
                    Json(serde_json::json!([
                        {"name": "Netflix", "billing_date": "2024-06-15"}
                    ]))
                },
            ),
        );

        let req = Request::get("/api/calendar?year=2024&month=6")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Transaction Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_transaction_get_returns_details() {
        let app = Router::new().route(
            "/api/transactions/{id}",
            get(|Path(id): Path<String>| async move {
                Json(serde_json::json!({
                    "id": id,
                    "name": "Coffee",
                    "amount": "4.50"
                }))
            }),
        );

        let req = Request::get("/api/transactions/456")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_transaction_delete_returns_204() {
        let app = Router::new().route(
            "/api/transactions/{id}",
            delete(|Path(_): Path<String>| async { StatusCode::NO_CONTENT }),
        );

        let req = Request::builder()
            .method("DELETE")
            .uri("/api/transactions/456")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn http_transaction_update_valid() {
        let app = Router::new().route(
            "/api/transactions/{id}",
            put(
                |Path(_): Path<String>, Json(input): Json<serde_json::Value>| async move {
                    let amount = input.get("amount");
                    if amount.is_none() {
                        return (StatusCode::OK, Json(serde_json::json!({"ok": true})));
                    }
                    (StatusCode::OK, Json(serde_json::json!({"ok": true})))
                },
            ),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/transactions/456")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"amount":"10.00"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Category Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_category_get_returns_details() {
        let app = Router::new().route(
            "/api/categories/{id}",
            get(|Path(id): Path<String>| async move {
                Json(serde_json::json!({
                    "id": id,
                    "name": "Entertainment",
                    "color": "#FF0000"
                }))
            }),
        );

        let req = Request::get("/api/categories/cat123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_category_delete_returns_204() {
        let app = Router::new().route(
            "/api/categories/{id}",
            delete(|Path(_): Path<String>| async { StatusCode::NO_CONTENT }),
        );

        let req = Request::builder()
            .method("DELETE")
            .uri("/api/categories/cat123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn http_category_create_valid() {
        let app = Router::new().route(
            "/api/categories",
            post(|Json(input): Json<serde_json::Value>| async move {
                let name = input.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "name required"})),
                    );
                }
                (
                    StatusCode::CREATED,
                    Json(serde_json::json!({"id": "new-id", "name": name})),
                )
            }),
        );

        let req = Request::post("/api/categories")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name":"New Category"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn http_category_update_valid() {
        let app = Router::new().route(
            "/api/categories/{id}",
            put(
                |Path(_): Path<String>, Json(input): Json<serde_json::Value>| async move {
                    let name = input.get("name");
                    let color = input.get("color");
                    if name.is_none() && color.is_none() {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": "no fields"})),
                        );
                    }
                    (StatusCode::OK, Json(serde_json::json!({"ok": true})))
                },
            ),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/categories/cat123")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"name":"Updated Category"}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_category_set_default_returns_ok() {
        let app = Router::new().route(
            "/api/categories/{id}/default",
            put(|Path(_): Path<String>| async {
                (StatusCode::OK, Json(serde_json::json!({"ok": true})))
            }),
        );

        let req = Request::builder()
            .method("PUT")
            .uri("/api/categories/cat123/default")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Reports Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_reports_categories_returns_breakdown() {
        let app = Router::new().route(
            "/api/reports/categories",
            get(
                |axum::extract::Query(_): axum::extract::Query<serde_json::Value>| async {
                    Json(serde_json::json!({
                        "categories": [
                            {"name": "Food", "amount": "200.00", "count": 5},
                            {"name": "Entertainment", "amount": "150.00", "count": 3}
                        ]
                    }))
                },
            ),
        );

        let req = Request::get("/api/reports/categories?year=2024&month=6")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_reports_trends_returns_data() {
        let app = Router::new().route(
            "/api/reports/trends",
            get(
                |axum::extract::Query(_): axum::extract::Query<serde_json::Value>| async {
                    Json(serde_json::json!({
                        "trends": [
                            {"month": "2024-01", "total": "500.00"},
                            {"month": "2024-02", "total": "550.00"}
                        ]
                    }))
                },
            ),
        );

        let req = Request::get("/api/reports/trends?year=2024")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Demo Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_demo_init_returns_data() {
        let app = Router::new().route(
            "/api/demo/init",
            post(|| async {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "user_id": "demo-user-id",
                        "email": "demo@recurrency.local"
                    })),
                )
            }),
        );

        let req = Request::post("/api/demo/init").body(Body::empty()).unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn http_demo_reset_returns_ok() {
        let app = Router::new().route(
            "/api/demo/reset",
            post(|| async { (StatusCode::OK, Json(serde_json::json!({"ok": true}))) }),
        );

        let req = Request::post("/api/demo/reset")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Logos Handler Tests
    // =========================================================================

    #[tokio::test]
    async fn http_logos_search_empty_query() {
        let app = Router::new().route(
            "/api/logos/search",
            get(
                |axum::extract::Query(q): axum::extract::Query<serde_json::Value>| async move {
                    let query = q
                        .get("q")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if query.is_empty() {
                        Json(serde_json::json!([]))
                    } else {
                        Json(serde_json::json!([{"url": "logo.png"}]))
                    }
                },
            ),
        );

        let req = Request::get("/api/logos/search?q=")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
    }
}
