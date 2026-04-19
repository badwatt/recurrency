use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::transactions::models::{CreateTransactionRequest, TransactionResponse, UpdateTransactionRequest};
use crate::transactions::service::TransactionService;

use crate::shared::{AuthQuery, OptionalQuery};
use crate::shared::auth_extractor::extract_token_from_request;

#[derive(serde::Deserialize)]
pub struct DateRangeQuery {
    pub token: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

pub async fn create(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateTransactionRequest>,
) -> AppResult<(StatusCode, Json<TransactionResponse>)> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transaction = TransactionService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transactions = TransactionService::get_all(&pool, claims.sub).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<TransactionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transaction = TransactionService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(transaction.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateTransactionRequest>,
) -> AppResult<Json<TransactionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let transaction = TransactionService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(transaction.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    TransactionService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn by_date_range(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<DateRangeQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<TransactionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let from_date = query.as_ref()
        .and_then(|q| q.from_date.as_ref())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let to_date = query.as_ref()
        .and_then(|q| q.to_date.as_ref())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let transactions = TransactionService::get_by_date_range(&pool, claims.sub, from_date, to_date).await?;

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

    fn make_create_request(name: &str, amount: &str, currency: &str, date: &str) -> CreateTransactionRequest {
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
}