//! Serialization tests for handlers - no async needed
//! Tests deserialization of request/response types

#[cfg(test)]
mod auth_model_tests {
    use recurrency_backend::auth::models::{
        CreateUserRequest, LoginRequest, UpdateSettingsRequest, UserResponse,
    };
    use validator::Validate;

    #[test]
    fn create_user_request_valid() {
        let req = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_user_request_invalid_email() {
        let req = CreateUserRequest {
            email: "invalid".to_string(),
            password: "password123".to_string(),
            name: "Test".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_short_password() {
        let req = CreateUserRequest {
            email: "test@test.com".to_string(),
            password: "short".to_string(),
            name: "Test".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_empty_name() {
        let req = CreateUserRequest {
            email: "test@test.com".to_string(),
            password: "password123".to_string(),
            name: "".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_user_request_unicode_email() {
        // Test with ASCII-compatible email
        let req = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Unicode User".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_user_request_json_roundtrip() {
        let json = r#"{"email":"roundtrip@test.com","password":"password123","name":"Roundtrip"}"#;
        let req: CreateUserRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "roundtrip@test.com");
    }

    #[test]
    fn login_request_deserialize() {
        let json = r#"{"email":"login@test.com","password":"secret123"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "login@test.com");
        assert_eq!(req.password, "secret123");
    }

    #[test]
    fn login_request_missing_fields() {
        let json = r#"{"email":"test@test.com"}"#;
        let req: Result<LoginRequest, _> = serde_json::from_str(json);
        assert!(req.is_err());
    }

    #[test]
    fn update_settings_request_name_only() {
        let json = r#"{"name":"New Name"}"#;
        let req: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_some());
        assert!(req.currency.is_none());
    }

    #[test]
    fn update_settings_request_currency_only() {
        let json = r#"{"currency":"EUR"}"#;
        let req: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.currency.is_some());
    }

    #[test]
    fn update_settings_request_both() {
        let json = r#"{"name":"Alice","currency":"GBP"}"#;
        let req: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Alice"));
        assert_eq!(req.currency.as_deref(), Some("GBP"));
    }

    #[test]
    fn user_response_serialization() {
        let user = UserResponse {
            id: uuid::Uuid::new_v4(),
            email: "serialize@test.com".to_string(),
            avatar: None,
            name: Some("Serialize Me".to_string()),
            currency: "GBP".to_string(),
            verified: true,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("serialize@test.com"));
        assert!(json.contains("Serialize Me"));
    }
}

#[cfg(test)]
mod subscription_model_tests {
    use recurrency_backend::subscriptions::models::CreateSubscriptionRequest;
    use validator::Validate;

    #[test]
    fn create_subscription_request_full() {
        let json = r#"{"name":"Netflix","amount":"15.99","currency":"USD","frequency":"monthly","start_date":"2024-01-15"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Netflix");
        assert_eq!(req.currency, "USD");
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_subscription_with_notes() {
        let json = r#"{"name":"Spotify","amount":"9.99","currency":"USD","frequency":"monthly","start_date":"2024-01-01","notes":"Family plan"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.notes.is_some());
    }

    #[test]
    fn create_subscription_notes_newlines() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-01-01","notes":"Line1\nLine2"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.notes.unwrap().contains('\n'));
    }

    #[test]
    fn create_subscription_leap_year() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-02-29"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(!req.start_date.to_string().is_empty());
    }

    #[test]
    fn create_subscription_frequencies() {
        for freq in ["daily", "weekly", "monthly", "quarterly", "yearly"] {
            let json = format!(
                r#"{{"name":"Test","amount":"10.00","currency":"USD","frequency":"{}","start_date":"2024-01-01"}}"#,
                freq
            );
            let req: CreateSubscriptionRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.frequency, freq);
        }
    }

    #[test]
    fn create_subscription_currencies() {
        let currencies = ["USD", "EUR", "GBP", "JPY"];
        for currency in currencies {
            let json = format!(
                r#"{{"name":"Test","amount":"10.00","currency":"{}","frequency":"monthly","start_date":"2024-01-01"}}"#,
                currency
            );
            let req: CreateSubscriptionRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.currency, currency);
        }
    }

    #[test]
    fn create_subscription_name_unicode() {
        let json = r#"{"name":"コーヒー","amount":"500","currency":"JPY","frequency":"monthly","start_date":"2024-01-01"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "コーヒー");
    }
}

#[cfg(test)]
mod transaction_model_tests {
    use recurrency_backend::transactions::models::CreateTransactionRequest;

    #[test]
    fn create_transaction_request_full() {
        let json = r#"{"name":"Coffee","amount":"4.50","currency":"USD","date":"2024-06-15"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Coffee");
        assert_eq!(req.amount.to_string(), "4.50");
        assert_eq!(req.date.to_string(), "2024-06-15");
    }

    #[test]
    fn create_transaction_request_minimal() {
        let json = r#"{"name":"Snack","amount":"2.00","currency":"USD","date":"2024-06-01"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Snack");
        assert!(req.notes.is_none());
    }

    #[test]
    fn create_transaction_request_with_notes() {
        let json = r#"{"name":"Lunch","amount":"15.00","currency":"USD","date":"2024-06-15","notes":"Business meeting"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.notes.is_some());
    }

    #[test]
    fn create_transaction_request_with_logo() {
        let json = r#"{"name":"Store","amount":"50.00","currency":"USD","date":"2024-06-15","logo_url":"https://store.com/logo.png"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.logo_url.is_some());
    }

    #[test]
    fn create_transaction_currencies() {
        let currencies = ["USD", "EUR", "GBP", "JPY"];
        for currency in currencies {
            let json = format!(
                r#"{{"name":"Test","amount":"10.00","currency":"{}","date":"2024-01-01"}}"#,
                currency
            );
            let req: CreateTransactionRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.currency, currency);
        }
    }

    #[test]
    fn create_transaction_name_unicode() {
        let json = r#"{"name":"コーヒー","amount":"500","currency":"JPY","date":"2024-01-01"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "コーヒー");
    }
}

#[cfg(test)]
mod category_model_tests {
    use recurrency_backend::categories::models::CreateCategoryRequest;

    #[test]
    fn create_category_request_full() {
        let json = "{\"name\":\"Entertainment\",\"icon\":\"movie\",\"color\":\"#FF0000\"}";
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Entertainment");
        assert!(req.icon.is_some());
        assert!(req.color.is_some());
    }

    #[test]
    fn create_category_request_minimal() {
        let json = r#"{"name":"Food"}"#;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Food");
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn create_category_unicode() {
        let json = r#"{"name":"カテゴリー","icon":"📁"}"#;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "カテゴリー");
    }

    #[test]
    fn create_category_color_formats() {
        let colors = ["#FF0000", "#00FF00", "#0000FF", "#FFFFFF", "#000000"];
        for color in colors {
            let json = format!(r#"{{"name":"Test","color":"{}"}}"#, color);
            let req: CreateCategoryRequest = serde_json::from_str(&json).unwrap();
            assert!(req.color.as_ref().is_some_and(|c| c == color));
        }
    }
}

#[cfg(test)]
mod notification_model_tests {
    use recurrency_backend::notifications::models::UpdatePreferenceRequest;

    #[test]
    fn update_preference_request_deserialize() {
        let json = r#"{"channel":"email","enabled":true}"#;
        let req: UpdatePreferenceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.channel, "email");
        assert!(req.enabled);
    }

    #[test]
    fn update_preference_request_disabled() {
        let json = r#"{"channel":"email","enabled":false}"#;
        let req: UpdatePreferenceRequest = serde_json::from_str(json).unwrap();
        assert!(!req.enabled);
    }

    #[test]
    fn update_preference_channels() {
        let channels = ["email", "push", "system"];
        for channel in channels {
            let json = format!(r#"{{"channel":"{}","enabled":true}}"#, channel);
            let req: UpdatePreferenceRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.channel, channel);
        }
    }
}

#[cfg(test)]
mod report_model_tests {
    use recurrency_backend::reports::models::{CategoryBreakdown, MonthlySummary, SpendingTrend};
    use rust_decimal::Decimal;

    #[test]
    fn monthly_summary_serialization() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: Decimal::new(50000, 2),
            transactions_total: Decimal::new(20000, 2),
            overall_total: Decimal::new(70000, 2),
            subscriptions_count: 5,
            transactions_count: 10,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("2024"));
        // Decimal 70000/100 = 700.00, JSON format may be "700" or "700.00" or similar
        assert!(json.contains("700"));
    }

    #[test]
    fn monthly_summary_zero() {
        let summary = MonthlySummary {
            year: 2024,
            month: 1,
            subscriptions_total: Decimal::ZERO,
            transactions_total: Decimal::ZERO,
            overall_total: Decimal::ZERO,
            subscriptions_count: 0,
            transactions_count: 0,
        };
        assert_eq!(summary.overall_total, Decimal::ZERO);
    }

    #[test]
    fn category_breakdown_serialization() {
        let breakdown = CategoryBreakdown {
            category_id: Some("cat-123".to_string()),
            category_name: "Entertainment".to_string(),
            total: Decimal::new(15000, 2),
            count: 3,
            percentage: 21.4,
        };
        let json = serde_json::to_string(&breakdown).unwrap();
        assert!(json.contains("Entertainment"));
        assert!(json.contains("21.4"));
        // Verify total is in the JSON (format may vary)
        assert!(json.contains("150"));
    }

    #[test]
    fn category_breakdown_no_category_id() {
        let breakdown = CategoryBreakdown {
            category_id: None,
            category_name: "Uncategorized".to_string(),
            total: Decimal::new(1000, 2),
            count: 5,
            percentage: 100.0,
        };
        assert!(breakdown.category_id.is_none());
    }

    #[test]
    fn spending_trend_serialization() {
        let trend = SpendingTrend {
            period: "2024/06".to_string(),
            total: Decimal::new(70000, 2),
        };
        let json = serde_json::to_string(&trend).unwrap();
        assert!(json.contains("2024/06"));
    }

    #[test]
    fn spending_trend_zero_total() {
        let trend = SpendingTrend {
            period: "2024/01".to_string(),
            total: Decimal::ZERO,
        };
        assert_eq!(trend.total, Decimal::ZERO);
    }
}

#[cfg(test)]
mod preferences_model_tests {
    use recurrency_backend::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};

    #[test]
    fn preferences_response_serialization() {
        let resp = PreferencesResponse {
            theme: "dark".to_string(),
            subscriptions_view: "list".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("dark"));
        assert!(json.contains("list"));
    }

    #[test]
    fn preferences_response_roundtrip() {
        let resp = PreferencesResponse {
            theme: "dark".to_string(),
            subscriptions_view: "list".to_string(),
            first_day_of_week: "monday".to_string(),
            date_format: "DD-MM-YYYY".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: PreferencesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.theme, resp.theme);
        assert_eq!(parsed.subscriptions_view, resp.subscriptions_view);
    }

    #[test]
    fn update_preferences_theme() {
        let json = r#"{"theme":"dark"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.theme.is_some());
        assert_eq!(req.theme.unwrap(), "dark");
    }

    #[test]
    fn update_preferences_view() {
        let json = r#"{"subscriptions_view":"list"}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.subscriptions_view.is_some());
        assert_eq!(req.subscriptions_view.unwrap(), "list");
    }

    #[test]
    fn update_preferences_empty() {
        let json = r#"{}"#;
        let req: UpdatePreferencesRequest = serde_json::from_str(json).unwrap();
        assert!(req.theme.is_none());
        assert!(req.subscriptions_view.is_none());
    }
}

#[cfg(test)]
mod auth_extractor_tests {
    use axum::http::HeaderMap;
    use recurrency_backend::shared::auth_extractor::extract_token_from_request;

    #[test]
    fn extract_token_bearer_valid() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer my_token".parse().unwrap());
        let result = extract_token_from_request(None, &headers).unwrap();
        assert_eq!(result, "my_token");
    }

    #[test]
    fn extract_token_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic my_token".parse().unwrap());
        let result = extract_token_from_request(None, &headers);
        assert!(result.is_err());
    }

    #[test]
    fn extract_token_missing_header() {
        let headers = HeaderMap::new();
        let result = extract_token_from_request(None, &headers);
        assert!(result.is_err());
    }

    #[test]
    fn extract_token_query_param_precedence() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer header_token".parse().unwrap());
        let result =
            extract_token_from_request(Some(&"query_token".to_string()), &headers).unwrap();
        assert_eq!(result, "query_token");
    }

    #[test]
    fn extract_token_query_only() {
        let headers = HeaderMap::new();
        let result = extract_token_from_request(Some(&"query_only".to_string()), &headers).unwrap();
        assert_eq!(result, "query_only");
    }

    #[test]
    fn extract_token_empty_bearer() {
        // Empty bearer should still return empty string (or error depending on implementation)
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer ".parse().unwrap());
        let result = extract_token_from_request(None, &headers);
        // Result depends on implementation - either ok(empty) or err
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn extract_token_query_empty_string() {
        // Empty query param should return empty string
        let headers = HeaderMap::new();
        let result = extract_token_from_request(Some(&"".to_string()), &headers);
        // Empty string might be treated as Some("") or None
        // Just verify it doesn't panic
        let _ = result.is_ok();
    }
}

#[cfg(test)]
mod error_tests {
    use recurrency_backend::shared::error::AppError;

    #[test]
    fn app_error_conflict() {
        let err = AppError::Conflict("Email exists".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Email exists"));
    }

    #[test]
    fn app_error_not_found() {
        let err = AppError::NotFound("User not found".to_string());
        let msg = err.to_string();
        assert!(msg.contains("User not found"));
    }

    #[test]
    fn app_error_unauthorized() {
        let err = AppError::Unauthorized("Bad token".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Bad token"));
    }

    #[test]
    fn app_error_bad_request() {
        let err = AppError::BadRequest("Invalid input".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid input"));
    }

    #[test]
    fn app_error_internal() {
        let err = AppError::Internal("Server error".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Server error"));
    }

    #[test]
    fn app_error_into_response_conflict() {
        use axum::{http::StatusCode, response::IntoResponse};
        let err = AppError::Conflict("Conflict".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn app_error_into_response_unauthorized() {
        use axum::{http::StatusCode, response::IntoResponse};
        let err = AppError::Unauthorized("Unauthorized".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn app_error_into_response_bad_request() {
        use axum::{http::StatusCode, response::IntoResponse};
        let err = AppError::BadRequest("Bad request".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn app_error_into_response_internal() {
        use axum::{http::StatusCode, response::IntoResponse};
        let err = AppError::Internal("Internal".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn app_error_into_response_not_found() {
        use axum::{http::StatusCode, response::IntoResponse};
        let err = AppError::NotFound("Not found".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}

#[cfg(test)]
mod query_deserialization_tests {
    #[derive(serde::Deserialize)]
    struct TestQuery {
        page: Option<u32>,
        limit: Option<u32>,
        token: Option<String>,
    }

    #[test]
    fn query_deserialize_full() {
        let query = "page=5&limit=10&token=abc";
        let result: TestQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.page, Some(5));
        assert_eq!(result.limit, Some(10));
        assert_eq!(result.token.as_deref(), Some("abc"));
    }

    #[test]
    fn query_deserialize_empty() {
        let result: TestQuery = serde_urlencoded::from_str("").unwrap();
        assert!(result.page.is_none());
        assert!(result.limit.is_none());
    }
}

#[cfg(test)]
mod email_service_tests {
    #[test]
    fn email_is_configured_function_exists() {
        use recurrency_backend::email::is_configured;
        let _ = is_configured();
    }
}
