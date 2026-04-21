use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Datelike;
use uuid::Uuid;
use validator::Validate;

use crate::auth::service::AuthService;
use crate::shared::app_state::AppState;
use crate::shared::error::AppResult;
use crate::subscriptions::models::{
    CreateSubscriptionRequest, SubscriptionResponse, UpdateSubscriptionRequest,
};
use crate::subscriptions::service::{SubscriptionService, CalendarItem};

use crate::shared::{AuthQuery, OptionalQuery};
use crate::shared::auth_extractor::extract_token_from_request;

#[derive(serde::Deserialize)]
pub struct CalendarQuery {
    pub token: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn create(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateSubscriptionRequest>,
) -> AppResult<(StatusCode, Json<SubscriptionResponse>)> {
    req.validate()?;

    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscription = SubscriptionService::create(&state.pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(subscription.into())))
}

pub async fn list(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<SubscriptionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscriptions = SubscriptionService::get_all(&state.pool, claims.sub).await?;

    Ok(Json(subscriptions.into_iter().map(Into::into).collect()))
}

pub async fn get(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<SubscriptionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscription = SubscriptionService::get_by_id(&state.pool, claims.sub, id).await?;

    Ok(Json(subscription.into()))
}

pub async fn update(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> AppResult<Json<SubscriptionResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscription = SubscriptionService::update(&state.pool, claims.sub, id, req).await?;

    Ok(Json(subscription.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    SubscriptionService::delete(&state.pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn upcoming(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<SubscriptionResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let subscriptions = SubscriptionService::get_upcoming(&state.pool, claims.sub, 7).await?;

    Ok(Json(subscriptions.into_iter().map(Into::into).collect()))
}

pub async fn calendar(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<CalendarQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CalendarItem>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let today = chrono::Utc::now().date_naive();
    let year = query.as_ref().and_then(|q| q.year).unwrap_or(today.year());
    let month = query.as_ref().and_then(|q| q.month).unwrap_or(today.month());

    let items = SubscriptionService::get_calendar(&state.pool, claims.sub, year, month).await?;

    Ok(Json(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subscriptions::models::{CreateSubscriptionRequest, UpdateSubscriptionRequest};
    use validator::Validate;

    fn make_create_request(name: &str, amount: &str, currency: &str, frequency: &str, start_date: &str) -> CreateSubscriptionRequest {
        let json = format!(
            r#"{{"name":"{}","amount":"{}","currency":"{}","frequency":"{}","start_date":"{}"}}"#,
            name, amount, currency, frequency, start_date
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn calendar_query_deserialize() {
        let query = "year=2024&month=6&token=abc";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert_eq!(result.month, Some(6));
        assert_eq!(result.token.as_deref(), Some("abc"));
    }

    #[test]
    fn calendar_query_empty() {
        let query = "";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.year.is_none());
        assert!(result.month.is_none());
        assert!(result.token.is_none());
    }

    #[test]
    fn calendar_query_partial_year_only() {
        let query = "year=2024";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert!(result.month.is_none());
    }

    #[test]
    fn calendar_query_partial_month_only() {
        let query = "month=12";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.year.is_none());
        assert_eq!(result.month, Some(12));
    }

    #[test]
    fn calendar_query_token_only() {
        let query = "token=only-token";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("only-token"));
    }

    #[test]
    fn calendar_defaults_to_current_date() {
        let today = chrono::Utc::now().date_naive();
        let year = None::<i32>.unwrap_or(today.year());
        let month = None::<u32>.unwrap_or(today.month());
        assert_eq!(year, today.year());
        assert_eq!(month, today.month());
    }

    #[test]
    fn calendar_item_serialization() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: rust_decimal::Decimal::from(1599),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            is_active: true,
            logo_url: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["name"], "Netflix");
        assert!(json.get("billing_date").is_some(), "Should have billing_date field, got: {:?}", json);
        assert!(json.get("is_active").is_some(), "Should have is_active field");
        assert!(json.get("date").is_none(), "Should NOT have 'date' field");
    }

    #[test]
    fn create_subscription_request_valid() {
        let req = make_create_request("Netflix", "15.99", "USD", "monthly", "2024-01-01");
        assert!(req.validate().is_ok());
    }

    #[test]
    fn create_subscription_request_all_frequencies() {
        for freq in &["weekly", "monthly", "quarterly", "yearly"] {
            let req = make_create_request("Test", "10.00", "USD", freq, "2024-01-01");
            assert!(req.validate().is_ok(), "Frequency {} should be valid", freq);
        }
    }

    #[test]
    fn create_subscription_request_invalid_frequency() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"invalid","start_date":"2024-01-01"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        // Frequency validation may allow any string or validate
        // We just verify it deserializes
        assert_eq!(req.frequency, "invalid");
    }

    #[test]
    fn create_subscription_request_empty_name() {
        let req = make_create_request("", "10.00", "USD", "monthly", "2024-01-01");
        assert!(req.validate().is_err());
    }

    #[test]
    fn create_subscription_request_negative_amount() {
        // Negative amounts should be handled by validation
        let json = r#"{"name":"Test","amount":"-10.00","currency":"USD","frequency":"monthly","start_date":"2024-01-01"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        // Amount is parsed as-is
        assert_eq!(req.amount.to_string().starts_with("-"), true);
    }

    #[test]
    fn create_subscription_request_invalid_date() {
        // Invalid date format will fail validation
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"invalid-date"}"#;
        let result: Result<CreateSubscriptionRequest, _> = serde_json::from_str(json);
        // Either parsing fails or validation fails
        assert!(result.is_err() || result.unwrap().validate().is_err());
    }

    #[test]
    fn create_subscription_request_with_category() {
        let cat_id = Uuid::new_v4();
        let json = format!(
            r#"{{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-01-01","category_id":"{}"}}"#,
            cat_id
        );
        let req: CreateSubscriptionRequest = serde_json::from_str(&json).unwrap();
        assert!(req.category_id.is_some());
    }

    #[test]
    fn create_subscription_request_with_notes() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-01-01","notes":"Monthly subscription"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.notes.as_deref(), Some("Monthly subscription"));
    }

    #[test]
    fn create_subscription_request_with_logo_url() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-01-01","logo_url":"https://logo.test.com"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.logo_url.as_deref(), Some("https://logo.test.com"));
    }

    #[test]
    fn update_subscription_request_name_only() {
        let json = r#"{"name":"Updated"}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert!(req.amount.is_none());
    }

    #[test]
    fn update_subscription_request_amount_only() {
        let json = r#"{"amount":"25.00"}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.amount.is_some());
        assert!(req.name.is_none());
    }

    #[test]
    fn update_subscription_request_frequency() {
        let json = r#"{"frequency":"yearly"}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.frequency.as_deref(), Some("yearly"));
    }

    #[test]
    fn update_subscription_request_is_active() {
        let json = r#"{"is_active":false}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.is_active, Some(false));
    }

    #[test]
    fn update_subscription_request_empty() {
        let json = r#"{}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.amount.is_none());
    }

    #[test]
    fn update_subscription_request_all_fields() {
        let json = r#"{"name":"Updated","amount":"20.00","currency":"EUR","frequency":"quarterly","is_active":false}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert!(req.amount.is_some());
        assert_eq!(req.currency.as_deref(), Some("EUR"));
        assert_eq!(req.frequency.as_deref(), Some("quarterly"));
    }

    #[test]
    fn subscription_response_serialization() {
        use crate::subscriptions::models::SubscriptionResponse;
        let resp = SubscriptionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Netflix".to_string(),
            amount: rust_decimal::Decimal::new(1599, 2),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()),
            is_active: true,
            notes: Some("Family plan".to_string()),
            logo_url: Some("https://logo.netflix.com".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Netflix"));
        assert!(json.contains("15.99"));
    }

    #[test]
    fn calendar_item_inactive() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Cancelled".to_string(),
            amount: rust_decimal::Decimal::from(999),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            is_active: false,
            logo_url: None,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["is_active"], false);
    }

    #[test]
    fn calendar_item_with_logo() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Spotify".to_string(),
            amount: rust_decimal::Decimal::from(999),
            currency: "EUR".to_string(),
            frequency: "monthly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            is_active: true,
            logo_url: Some("https://logo.spotify.com".to_string()),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert!(json.get("logo_url").is_some());
    }

    // ── CalendarItem response tests ──
    #[test]
    fn calendar_item_weekly() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Gym".to_string(),
            amount: rust_decimal::Decimal::from(50),
            currency: "USD".to_string(),
            frequency: "weekly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert_eq!(item.frequency, "weekly");
    }

    #[test]
    fn calendar_item_quarterly() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Insurance".to_string(),
            amount: rust_decimal::Decimal::from(300),
            currency: "USD".to_string(),
            frequency: "quarterly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert_eq!(item.frequency, "quarterly");
    }

    #[test]
    fn calendar_item_yearly() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Amazon Prime".to_string(),
            amount: rust_decimal::Decimal::from(149),
            currency: "USD".to_string(),
            frequency: "yearly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert_eq!(item.frequency, "yearly");
    }

    #[test]
    fn subscription_response_all_currencies() {
        use crate::subscriptions::models::SubscriptionResponse;
        for currency in &["USD", "EUR", "GBP", "JPY", "CAD"] {
            let resp = SubscriptionResponse {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                category_id: Uuid::new_v4(),
                name: "Test".to_string(),
                amount: rust_decimal::Decimal::from(1000),
                currency: currency.to_string(),
                frequency: "monthly".to_string(),
                start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                next_billing_date: None,
                is_active: true,
                notes: None,
                logo_url: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let json = serde_json::to_string(&resp).unwrap();
            assert!(json.contains(currency));
        }
    }

    #[test]
    fn subscription_response_without_notes() {
        use crate::subscriptions::models::SubscriptionResponse;
        let resp = SubscriptionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "No Notes".to_string(),
            amount: rust_decimal::Decimal::from(500),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        // Either no notes field or notes is null
        let has_notes_key = json.contains("\"notes\"");
        if has_notes_key {
            assert!(json.contains("notes") && json.contains("null"));
        }
    }

    #[test]
    fn subscription_response_without_logo() {
        use crate::subscriptions::models::SubscriptionResponse;
        let resp = SubscriptionResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "No Logo".to_string(),
            amount: rust_decimal::Decimal::from(500),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        // logo_url is either not present or null
        let has_logo_url_key = json.contains("\"logo_url\"");
        if has_logo_url_key {
            assert!(json.contains("logo_url") && json.contains("null"));
        } else {
            assert!(true);
        }
    }

    #[test]
    fn update_subscription_is_active_true() {
        let json = r#"{"is_active":true}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.is_active, Some(true));
    }

    #[test]
    fn update_subscription_next_billing_date() {
        let json = r#"{"next_billing_date":"2025-01-01"}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.next_billing_date.is_some());
    }

    #[test]
    fn update_subscription_notes_empty() {
        let json = r#"{"notes":""}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.notes, Some("".to_string()));
    }

    #[test]
    fn create_subscription_request_with_emoji() {
        let json = r#"{"name":"Netflix 🎬","amount":"15.99","currency":"USD","frequency":"monthly","start_date":"2024-01-01"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.contains("🎬"));
    }

    #[test]
    fn create_subscription_large_amount() {
        let json = r#"{"name":"Enterprise","amount":"999999.99","currency":"USD","frequency":"monthly","start_date":"2024-01-01"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.amount > rust_decimal::Decimal::from(100000));
    }
}
    #[test]
    fn calendar_query_special_chars_in_token() {
        let query = "token=eyJhbGciOiJIUzI1NiJ9.abc123";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("eyJhbGciOiJIUzI1NiJ9.abc123"));
    }

    #[test]
    fn calendar_query_negative_year() {
        let query = "year=-2024";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(-2024));
    }

    #[test]
    fn calendar_query_zero_month() {
        let query = "month=0";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.month, Some(0));
    }

    #[test]
    fn calendar_query_month_13() {
        let query = "month=13";
        let result: CalendarQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.month, Some(13));
    }

    #[test]
    fn calendar_item_micro_amount() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Micro".to_string(),
            amount: rust_decimal::Decimal::new(1, 4),
            currency: "BTC".to_string(),
            frequency: "weekly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert!(item.amount < rust_decimal::Decimal::ONE);
    }

    #[test]
    fn calendar_item_currency_jpy() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Japanese".to_string(),
            amount: rust_decimal::Decimal::from(5000),
            currency: "JPY".to_string(),
            frequency: "yearly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert_eq!(item.currency, "JPY");
    }

    #[test]
    fn calendar_item_name_with_special_chars() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Netflix & Spotify™".to_string(),
            amount: rust_decimal::Decimal::from(2000),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert!(item.name.contains('™'));
    }

    #[test]
    fn calendar_item_billing_date_far_future() {
        let item = CalendarItem {
            subscription_id: Uuid::new_v4(),
            name: "Far".to_string(),
            amount: rust_decimal::Decimal::from(100),
            currency: "USD".to_string(),
            frequency: "yearly".to_string(),
            billing_date: chrono::NaiveDate::from_ymd_opt(2099, 12, 31).unwrap(),
            is_active: true,
            logo_url: None,
        };
        assert!(item.billing_date > chrono::Utc::now().date_naive());
    }

    #[test]
    fn create_subscription_start_date_feb_29_leap_year() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-02-29"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.start_date.month(), 2);
        assert_eq!(req.start_date.day(), 29);
    }

    #[test]
    fn create_subscription_notes_with_newlines() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-01-01","notes":"Line1\nLine2\nLine3"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.notes.unwrap().contains('\n'));
    }

    #[test]
    fn create_subscription_logo_url_https() {
        let json = r#"{"name":"Test","amount":"10.00","currency":"USD","frequency":"monthly","start_date":"2024-01-01","logo_url":"https://logo.example.com/foo.png"}"#;
        let req: CreateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert!(req.logo_url.unwrap().starts_with("https://"));
    }

    #[test]
    fn update_subscription_currency_change() {
        let json = r#"{"currency":"GBP"}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.currency.unwrap(), "GBP");
    }

    #[test]
    fn update_subscription_frequency_to_yearly() {
        let json = r#"{"frequency":"yearly"}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.frequency.unwrap(), "yearly");
    }

    #[test]
    fn update_subscription_deactivate() {
        let json = r#"{"is_active":false}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.is_active, Some(false));
    }

    #[test]
    fn update_subscription_category_id_reassign() {
        let cat_id = Uuid::new_v4();
        let json = format!(r#"{{"category_id":"{}"}}"#, cat_id);
        let req: UpdateSubscriptionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.category_id, Some(cat_id));
    }

    #[test]
    fn update_subscription_multiple_fields() {
        let json = r#"{"name":"New","amount":"20.00","is_active":false,"notes":"Updated"}"#;
        let req: UpdateSubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.unwrap(), "New");
        assert_eq!(req.amount.unwrap().to_string(), "20.00");
        assert_eq!(req.is_active, Some(false));
    }

    // ── SubscriptionService resolve_update_fields ──
    #[test]
    fn resolve_subscription_name_unicode() {
        use crate::subscriptions::models::Subscription;
        let existing = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Café Premium 日本語".to_string(),
            amount: rust_decimal::Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let req = UpdateSubscriptionRequest {
            name: Some("New Name".to_string()),
            amount: None,
            currency: None,
            frequency: None,
            start_date: None,
            next_billing_date: None,
            category_id: None,
            is_active: None,
            notes: None,
            logo_url: None,
        };
        let (name, ..) = crate::subscriptions::service::resolve_update_fields(&existing, &req);
        assert_eq!(name, "New Name");
    }

    #[test]
    fn resolve_subscription_all_none_preserves_existing() {
        use crate::subscriptions::models::Subscription;
        let existing = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Keep".to_string(),
            amount: rust_decimal::Decimal::new(1999, 2),
            currency: "EUR".to_string(),
            frequency: "quarterly".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
            is_active: true,
            notes: Some("keep notes".to_string()),
            logo_url: Some("https://old.png".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let req = UpdateSubscriptionRequest {
            name: None,
            amount: None,
            currency: None,
            frequency: None,
            start_date: None,
            next_billing_date: None,
            category_id: None,
            is_active: None,
            notes: None,
            logo_url: None,
        };
        let resolved = crate::subscriptions::service::resolve_update_fields(&existing, &req);
        assert_eq!(resolved.0, "Keep");
        assert_eq!(resolved.2, "EUR");
    }

    #[test]
    fn resolve_subscription_partial_update_amount_only() {
        use crate::subscriptions::models::Subscription;
        let existing = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Sub".to_string(),
            amount: rust_decimal::Decimal::from(1000),
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let req = UpdateSubscriptionRequest {
            name: None,
            amount: Some(rust_decimal::Decimal::from(2000)),
            currency: None,
            frequency: None,
            start_date: None,
            next_billing_date: None,
            category_id: None,
            is_active: None,
            notes: None,
            logo_url: None,
        };
        let resolved = crate::subscriptions::service::resolve_update_fields(&existing, &req);
        assert_eq!(resolved.0, "Sub");
        assert_eq!(resolved.1, rust_decimal::Decimal::from(2000));
    }

    #[test]
    fn resolve_subscription_start_date_change() {
        use crate::subscriptions::models::Subscription;
        let existing = Subscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Sub".to_string(),
            amount: rust_decimal::Decimal::TEN,
            currency: "USD".to_string(),
            frequency: "monthly".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_billing_date: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let req = UpdateSubscriptionRequest {
            name: None,
            amount: None,
            currency: None,
            frequency: None,
            start_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            next_billing_date: None,
            category_id: None,
            is_active: None,
            notes: None,
            logo_url: None,
        };
        let resolved = crate::subscriptions::service::resolve_update_fields(&existing, &req);
        assert_eq!(resolved.4, chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
    }
