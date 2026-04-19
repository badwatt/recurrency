use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use chrono::Datelike;
use sqlx::PgPool;

use crate::auth::service::AuthService;
use crate::reports::models::ReportResponse;
use crate::reports::service::ReportService;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::OptionalQuery;

#[derive(serde::Deserialize)]
pub struct ReportQuery {
    pub token: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn get_report(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<ReportQuery>,
    headers: HeaderMap,
) -> AppResult<Json<ReportResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let today = chrono::Utc::now().date_naive();
    let year = query.as_ref().and_then(|q| q.year).unwrap_or(today.year());
    let month = query.as_ref().and_then(|q| q.month).unwrap_or(today.month());

    let report = ReportService::get_full_report(&pool, claims.sub, year, month).await;

    Ok(Json(report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::models::{CategoryBreakdown, MonthlySummary, ReportResponse, SpendingTrend};
    use rust_decimal::Decimal;

    #[test]
    fn report_query_deserialize_with_params() {
        let query = "year=2024&month=6&token=abc";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert_eq!(result.month, Some(6));
        assert_eq!(result.token.as_deref(), Some("abc"));
    }

    #[test]
    fn report_query_deserialize_empty() {
        let query = "";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.year.is_none());
        assert!(result.month.is_none());
        assert!(result.token.is_none());
    }

    #[test]
    fn report_query_defaults_to_current_date() {
        let today = chrono::Utc::now().date_naive();
        let year = None::<i32>.unwrap_or(today.year());
        let month = None::<u32>.unwrap_or(today.month());
        assert_eq!(year, today.year());
        assert_eq!(month, today.month());
    }

    #[test]
    fn report_query_year_only() {
        let query = "year=2024";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert!(result.month.is_none());
    }

    #[test]
    fn report_query_month_only() {
        let query = "month=12";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.year.is_none());
        assert_eq!(result.month, Some(12));
    }

    #[test]
    fn report_query_token_only() {
        let query = "token=only-token";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("only-token"));
    }

    #[test]
    fn report_query_empty_token() {
        let query = "token=";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.as_ref().unwrap().is_empty());
    }

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
        assert!(json.contains("6"));
        assert!(json.contains("500"));
        assert!(json.contains("200"));
        assert!(json.contains("700"));
    }

    #[test]
    fn category_breakdown_serialization() {
        let breakdown = CategoryBreakdown {
            category_id: Some("cat-123".to_string()),
            category_name: "Streaming".to_string(),
            total: Decimal::new(5999, 2),
            count: 3,
            percentage: 60.0,
        };
        let json = serde_json::to_string(&breakdown).unwrap();
        assert!(json.contains("Streaming"));
        assert!(json.contains("59.99"));
        assert!(json.contains("60"));
    }

    #[test]
    fn category_breakdown_without_category_id() {
        let breakdown = CategoryBreakdown {
            category_id: None,
            category_name: "Uncategorized".to_string(),
            total: Decimal::new(1000, 2),
            count: 2,
            percentage: 10.0,
        };
        let json = serde_json::to_value(&breakdown).unwrap();
        assert!(json.get("category_id").is_none() || json["category_id"].is_null());
    }

    #[test]
    fn spending_trend_serialization() {
        let trend = SpendingTrend {
            period: "6/24".to_string(),
            total: Decimal::new(100000, 2),
        };
        let json = serde_json::to_string(&trend).unwrap();
        assert!(json.contains("6/24"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn report_response_serialization() {
        use rust_decimal::Decimal;
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: Decimal::new(50000, 2),
                transactions_total: Decimal::new(20000, 2),
                overall_total: Decimal::new(70000, 2),
                subscriptions_count: 5,
                transactions_count: 10,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("monthly_summary"));
        assert!(json.contains("category_breakdown"));
        assert!(json.contains("recent_trends"));
    }

    #[test]
    fn report_query_all_months() {
        for month in 1..=12 {
            let query = format!("month={}", month);
            let result: ReportQuery = serde_urlencoded::from_str(&query).unwrap();
            assert_eq!(result.month, Some(month as u32));
        }
    }

    #[test]
    fn report_query_various_years() {
        for year in &[2020, 2024, 2030, 2100] {
            let query = format!("year={}", year);
            let result: ReportQuery = serde_urlencoded::from_str(&query).unwrap();
            assert_eq!(result.year, Some(*year));
        }
    }

    #[test]
    fn monthly_summary_zero_totals() {
        let summary = MonthlySummary {
            year: 2024,
            month: 1,
            subscriptions_total: Decimal::ZERO,
            transactions_total: Decimal::ZERO,
            overall_total: Decimal::ZERO,
            subscriptions_count: 0,
            transactions_count: 0,
        };
        assert_eq!(summary.subscriptions_count, 0);
        assert_eq!(summary.overall_total, Decimal::ZERO);
    }

    #[test]
    fn category_breakdown_100_percent() {
        let breakdown = CategoryBreakdown {
            category_id: Some("cat-1".to_string()),
            category_name: "Only".to_string(),
            total: Decimal::new(10000, 2),
            count: 1,
            percentage: 100.0,
        };
        assert_eq!(breakdown.percentage, 100.0);
    }

    #[test]
    fn spending_trend_different_periods() {
        let periods = vec!["1/24", "6/23", "12/23", "1/25"];
        for period in periods {
            let trend = SpendingTrend {
                period: period.to_string(),
                total: Decimal::new(50000, 2),
            };
            assert_eq!(trend.period, period);
        }
    }
}