use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use crate::shared::app_state::AppState;
use crate::shared::error::{AppError, AppResult};
use crate::transactions::models::{
    CreateTransactionRequest, TransactionResponse, UpdateTransactionRequest,
};
use crate::transactions::service::TransactionService;

use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::{AuthQuery, OptionalQuery};

#[derive(serde::Deserialize)]
pub struct DateRangeQuery {
    pub token: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateTransactionRequest>,
) -> AppResult<(StatusCode, Json<TransactionResponse>)> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;

    let transaction = TransactionService::create(&state.pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub async fn list(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;

    let transactions = TransactionService::get_all(&state.pool, claims.sub).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<TransactionResponse>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;

    let transaction = TransactionService::get_by_id(&state.pool, claims.sub, id).await?;

    Ok(Json(transaction.into()))
}

pub async fn update(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateTransactionRequest>,
) -> AppResult<Json<TransactionResponse>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;

    let transaction = TransactionService::update(&state.pool, claims.sub, id, req).await?;

    Ok(Json(transaction.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;

    TransactionService::delete(&state.pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn by_date_range(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<DateRangeQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = state.auth_service.verify_token(&extract_token_from_request(
        query.as_ref().and_then(|q| q.token.as_ref()),
        &headers,
    )?)
    .await?;

    let from_date = match query.as_ref().and_then(|q| q.from_date.as_deref()) {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest(format!("Invalid from_date: {}", d)))?,
        None => chrono::Utc::now().date_naive(),
    };

    let to_date = match query.as_ref().and_then(|q| q.to_date.as_deref()) {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest(format!("Invalid to_date: {}", d)))?,
        None => chrono::Utc::now().date_naive(),
    };

    let transactions =
        TransactionService::get_by_date_range(&state.pool, claims.sub, from_date, to_date).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}

/// Parse a date string in YYYY-MM-DD format, returning None on failure.
#[allow(dead_code)]
pub(crate) fn parse_date(date_str: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transactions::models::{CreateTransactionRequest, UpdateTransactionRequest};
    use chrono::Datelike;

    fn make_create_request(
        name: &str,
        amount: &str,
        currency: &str,
        date: &str,
    ) -> CreateTransactionRequest {
        let json = format!(
            r#"{{"name":"{}","amount":"{}","currency":"{}","date":"{}"}}"#,
            name, amount, currency, date
        );
        serde_json::from_str(&json).unwrap()
    }

    // ── parse_date ──

    #[test]
    fn parse_date_valid() {
        let result = parse_date("2024-06-15");
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.year(), 2024);
        assert_eq!(d.month(), 6);
        assert_eq!(d.day(), 15);
    }

    #[test]
    fn parse_date_invalid_format() {
        assert!(parse_date("06/15/2024").is_none());
        assert!(parse_date("2024-13-01").is_none());
        assert!(parse_date("not-a-date").is_none());
        assert!(parse_date("").is_none());
    }

    #[test]
    fn parse_date_edge_cases() {
        assert!(parse_date("2024-02-29").is_some()); // leap year
        assert!(parse_date("2023-02-29").is_none()); // not a leap year
        assert!(parse_date("2024-12-31").is_some());
        assert!(parse_date("2024-01-01").is_some());
    }

    // ── DateRangeQuery deserialization ──

    #[test]
    fn date_range_query_deserialize() {
        let query = "from_date=2024-01-01&to_date=2024-12-31";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.from_date.as_deref(), Some("2024-01-01"));
        assert_eq!(result.to_date.as_deref(), Some("2024-12-31"));
    }

    #[test]
    fn date_range_query_empty() {
        let query = "";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.from_date.is_none());
        assert!(result.to_date.is_none());
    }

    #[test]
    fn date_range_query_partial() {
        let query = "from_date=2024-06-01";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.from_date.as_deref(), Some("2024-06-01"));
        assert!(result.to_date.is_none());
    }

    #[test]
    fn date_range_query_to_date_only() {
        let query = "to_date=2024-12-31";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.from_date.is_none());
        assert_eq!(result.to_date.as_deref(), Some("2024-12-31"));
    }

    #[test]
    fn date_range_query_with_token() {
        let query = "from_date=2024-01-01&to_date=2024-12-31&token=mytoken";
        let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("mytoken"));
    }

    // ── CreateTransactionRequest tests ──

    #[test]
    fn create_transaction_request_valid() {
        let req = make_create_request("Coffee", "4.50", "USD", "2024-06-15");
        assert_eq!(req.name, "Coffee");
        assert_eq!(req.currency, "USD");
    }

    #[test]
    fn create_transaction_request_empty_name() {
        let req = make_create_request("", "10.00", "USD", "2024-06-15");
        assert_eq!(req.name, "");
    }

    #[test]
    fn create_transaction_request_with_category() {
        let cat_id = Uuid::new_v4();
        let json = format!(
            r#"{{"name":"Test","amount":"10.00","currency":"USD","date":"2024-06-15","category_id":"{}"}}"#,
            cat_id
        );
        let req: CreateTransactionRequest = serde_json::from_str(&json).unwrap();
        assert!(req.category_id.is_some());
    }

    #[test]
    fn create_transaction_request_with_notes() {
        let json = r#"{"name":"Lunch","amount":"15.00","currency":"USD","date":"2024-06-15","notes":"Business lunch"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.notes.as_deref(), Some("Business lunch"));
    }

    #[test]
    fn create_transaction_request_with_logo_url() {
        let json = r#"{"name":"Coffee","amount":"4.50","currency":"USD","date":"2024-06-15","logo_url":"https://logo.cafe.com"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.logo_url.as_deref(), Some("https://logo.cafe.com"));
    }

    #[test]
    fn create_transaction_request_unicode() {
        let json = r#"{"name":"咖啡 ☕","amount":"25.00","currency":"CNY","date":"2024-06-15"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "咖啡 ☕");
    }

    #[test]
    fn create_transaction_request_all_currencies() {
        for currency in &["USD", "EUR", "GBP", "JPY", "CAD", "AUD"] {
            let req = make_create_request("Test", "10.00", currency, "2024-06-15");
            assert_eq!(req.currency, *currency);
        }
    }

    // ── UpdateTransactionRequest tests ──

    #[test]
    fn update_transaction_request_name_only() {
        let json = r#"{"name":"Updated"}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert!(req.amount.is_none());
    }

    #[test]
    fn update_transaction_request_amount_only() {
        let json = r#"{"amount":"50.00"}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.amount.is_some());
        assert!(req.name.is_none());
    }

    #[test]
    fn update_transaction_request_date() {
        let json = r#"{"date":"2025-12-25"}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.date.is_some());
    }

    #[test]
    fn update_transaction_request_all_fields() {
        let json = r#"{"name":"Updated","amount":"75.00","currency":"EUR","date":"2025-12-25"}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert!(req.amount.is_some());
        assert_eq!(req.currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn update_transaction_request_empty() {
        let json = r#"{}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
    }

    #[test]
    fn update_transaction_request_category() {
        let cat_id = Uuid::new_v4();
        let json = format!(r#"{{"category_id":"{}"}}"#, cat_id);
        let req: UpdateTransactionRequest = serde_json::from_str(&json).unwrap();
        assert!(req.category_id.is_some());
    }

    // ── TransactionResponse tests ──

    #[test]
    fn transaction_response_serialization() {
        let resp = TransactionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Lunch".to_string(),
            amount: rust_decimal::Decimal::new(1599, 2),
            currency: "USD".to_string(),
            date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: Some("Business lunch".to_string()),
            logo_url: Some("https://logo.restaurant.com".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Lunch"));
        assert!(json.contains("15.99"));
        assert!(json.contains("Business lunch"));
    }

    #[test]
    fn transaction_response_without_optional_fields() {
        let resp = TransactionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Coffee".to_string(),
            amount: rust_decimal::Decimal::new(499, 2),
            currency: "EUR".to_string(),
            date: chrono::NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Coffee"));
        assert!(json.contains("4.99"));
    }

    // ── Date parsing edge cases ──

    #[test]
    fn parse_date_all_months() {
        for month in 1..=12 {
            let date_str = format!("2024-{:02}-15", month);
            let result = parse_date(&date_str);
            assert!(result.is_some(), "Date {} should parse", date_str);
            let d = result.unwrap();
            assert_eq!(d.month(), month as u32);
        }
    }

    #[test]
    fn parse_date_various_years() {
        for year in &[2020, 2024, 2030, 2100] {
            let date_str = format!("{}-06-15", year);
            let result = parse_date(&date_str);
            assert!(result.is_some());
            assert_eq!(result.unwrap().year(), *year);
        }
    }

    // ── Additional edge case tests ──
    #[test]
    fn create_transaction_name_special_chars_edge() {
        let json = r##"{"name":"Lunch & Dinner #1","amount":"25.50","currency":"USD","date":"2024-06-15"}"##;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Lunch & Dinner #1");
    }

    #[test]
    fn create_transaction_name_unicode_edge() {
        let json = r#"{"name":"café ☕","amount":"5.00","currency":"EUR","date":"2024-06-15"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.contains("café"));
    }

    #[test]
    fn create_transaction_all_currencies_edge() {
        for currency in &["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF"] {
            let json = format!(
                r#"{{"name":"Test","amount":"10.00","currency":"{}","date":"2024-06-15"}}"#,
                currency
            );
            let req: CreateTransactionRequest = serde_json::from_str(&json).unwrap();
            assert_eq!(req.currency, *currency);
        }
    }

    #[test]
    fn create_transaction_large_amount_edge() {
        let json =
            r#"{"name":"Big Purchase","amount":"999999.99","currency":"USD","date":"2024-06-15"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.amount > rust_decimal::Decimal::from(100000));
    }

    #[test]
    fn create_transaction_small_amount_edge() {
        let json = r#"{"name":"Tiny","amount":"0.01","currency":"USD","date":"2024-06-15"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.amount < rust_decimal::Decimal::from(1));
    }

    #[test]
    fn create_transaction_negative_amount_edge() {
        let json = r#"{"name":"Test","amount":"-10.00","currency":"USD","date":"2024-06-15"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.amount < rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn create_transaction_with_category_edge() {
        let cat_id = Uuid::new_v4();
        let json = serde_json::json!({
            "name": "Test",
            "amount": "10.00",
            "currency": "USD",
            "date": "2024-06-15",
            "category_id": cat_id.to_string()
        })
        .to_string();
        let req: CreateTransactionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.category_id, Some(cat_id));
    }

    #[test]
    fn create_transaction_with_logo_edge() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","date":"2024-06-15","logo_url":"https://example.com/logo.png"}"#;
        let req: CreateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.logo_url,
            Some("https://example.com/logo.png".to_string())
        );
    }

    #[test]
    fn update_transaction_partial_amount_edge() {
        let json = r#"{"amount":"99.99"}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.amount.is_some());
        assert!(req.name.is_none());
        assert!(req.date.is_none());
    }

    #[test]
    fn update_transaction_partial_name_edge() {
        let json = r#"{"name":"New Name"}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("New Name"));
        assert!(req.amount.is_none());
    }

    #[test]
    fn update_transaction_partial_date_edge() {
        let json = r#"{"date":"2025-01-01"}"#;
        let req: UpdateTransactionRequest = serde_json::from_str(json).unwrap();
        assert!(req.date.is_some());
        assert!(req.name.is_none());
    }

    #[test]
    fn update_transaction_all_fields_edge() {
        let json = serde_json::json!({
            "name": "Updated",
            "amount": "50.00",
            "currency": "EUR",
            "date": "2025-01-01",
            "notes": "Updated notes",
            "category_id": Uuid::new_v4().to_string()
        })
        .to_string();
        let req: UpdateTransactionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert!(req.amount.is_some());
        assert_eq!(req.currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn transaction_response_large_amount_edge() {
        let resp = TransactionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Expensive".to_string(),
            amount: rust_decimal::Decimal::new(99999999, 2),
            currency: "USD".to_string(),
            date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("999999.99") || json.contains("99999999"));
    }

    #[test]
    fn transaction_response_zero_amount_edge() {
        let resp = TransactionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Free".to_string(),
            amount: rust_decimal::Decimal::ZERO,
            currency: "USD".to_string(),
            date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(resp.amount, rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn transaction_response_long_notes_edge() {
        let long_notes = "A".repeat(1000);
        let resp = TransactionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Test".to_string(),
            amount: rust_decimal::Decimal::new(100, 2),
            currency: "USD".to_string(),
            date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: Some(long_notes.clone()),
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(resp.notes.unwrap().len(), 1000);
    }

    #[test]
    fn transaction_response_long_logo_edge() {
        let long_url = format!("https://example.com/{}", "a".repeat(200));
        let resp = TransactionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Test".to_string(),
            amount: rust_decimal::Decimal::new(100, 2),
            currency: "USD".to_string(),
            date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            notes: None,
            logo_url: Some(long_url.clone()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(resp.logo_url.unwrap().len(), long_url.len());
    }

    #[test]
    fn date_range_query_token_edge() {
        let json = serde_json::json!({"token": "abc123"}).to_string();
        let query: DateRangeQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(query.token, Some("abc123".to_string()));
    }

    #[test]
    fn date_range_query_empty_edge() {
        let json = "{}";
        let query: DateRangeQuery = serde_json::from_str(json).unwrap();
        assert!(query.token.is_none());
    }

    #[test]
    fn date_range_query_large_token_edge() {
        let long_token = "a".repeat(500);
        let json = serde_json::json!({"token": long_token}).to_string();
        let query: DateRangeQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(query.token.unwrap().len(), 500);
    }
}
// ── DateRangeQuery from_date/to_date ──
#[test]
fn date_range_query_from_to_date() {
    let query = "from_date=2024-01-01&to_date=2024-06-30";
    let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
    assert_eq!(result.from_date, Some("2024-01-01".to_string()));
    assert_eq!(result.to_date, Some("2024-06-30".to_string()));
}

#[test]
fn date_range_query_all_params() {
    let query = "token=abc&from_date=2024-01-01&to_date=2024-12-31";
    let result: DateRangeQuery = serde_urlencoded::from_str(query).unwrap();
    assert!(result.token.is_some());
    assert!(result.from_date.is_some());
    assert!(result.to_date.is_some());
}

// ── TransactionResponse edge cases ──
#[test]
fn transaction_response_zero_amount() {
    use crate::transactions::models::TransactionResponse;
    let resp = TransactionResponse {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        category_id: Uuid::new_v4(),
        name: "Zero".to_string(),
        amount: rust_decimal::Decimal::ZERO,
        currency: "USD".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        notes: None,
        logo_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert_eq!(resp.amount, rust_decimal::Decimal::ZERO);
}

#[test]
fn transaction_response_negative_amount() {
    use crate::transactions::models::TransactionResponse;
    let resp = TransactionResponse {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        category_id: Uuid::new_v4(),
        name: "Refund".to_string(),
        amount: rust_decimal::Decimal::new(-5000, 2),
        currency: "USD".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        notes: None,
        logo_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert!(resp.amount < rust_decimal::Decimal::ZERO);
}
