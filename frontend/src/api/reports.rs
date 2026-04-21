use crate::api::client::{api_get, ApiError};
use crate::models::report::*;

pub async fn get_summary(year: i32, month: u32) -> Result<ReportResponse, ApiError> {
    api_get(&format!(
        "/api/reports/summary?year={}&month={}",
        year, month
    ))
    .await
}

#[cfg(test)]
mod tests {
    use crate::models::report::*;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[test]
    fn reports_summary_url() {
        let path = format!("/api/reports/summary?year={}&month={}", 2024, 3);
        assert_eq!(path, "/api/reports/summary?year=2024&month=3");
    }

    #[test]
    fn reports_url_various_dates() {
        let (y, m) = (2023, 1);
        let path = format!("/api/reports/summary?year={}&month={}", y, m);
        assert!(path.contains("year=2023"));
        assert!(path.contains("month=1"));
    }

    #[test]
    fn report_response_fields() {
        let resp = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(100.0),
                transactions_total: dec!(50.0),
                overall_total: dec!(150.0),
                subscriptions_count: 5,
                transactions_count: 3,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        assert_eq!(resp.monthly_summary.year, 2024);
        assert_eq!(resp.monthly_summary.month, 6);
        assert_eq!(resp.monthly_summary.subscriptions_total, dec!(100.0));
        assert_eq!(resp.monthly_summary.transactions_total, dec!(50.0));
        assert_eq!(resp.monthly_summary.overall_total, dec!(150.0));
    }

    #[test]
    fn report_response_with_categories() {
        let resp = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(100.0),
                transactions_total: dec!(50.0),
                overall_total: dec!(150.0),
                subscriptions_count: 5,
                transactions_count: 3,
            },
            category_breakdown: vec![
                CategoryBreakdown {
                    category_id: Some(Uuid::new_v4().to_string()),
                    category_name: "Food".into(),
                    total: dec!(30.0),
                    count: 2,
                    percentage: 50.0,
                },
                CategoryBreakdown {
                    category_id: Some(Uuid::new_v4().to_string()),
                    category_name: "Transport".into(),
                    total: dec!(30.0),
                    count: 1,
                    percentage: 50.0,
                },
            ],
            recent_trends: vec![],
        };
        assert_eq!(resp.category_breakdown.len(), 2);
        assert_eq!(resp.category_breakdown[0].category_name, "Food");
        assert_eq!(resp.category_breakdown[0].percentage, 50.0);
    }

    #[test]
    fn category_breakdown_serialization() {
        let cat = CategoryBreakdown {
            category_id: Some(Uuid::new_v4().to_string()),
            category_name: "Test".into(),
            total: dec!(25.5),
            count: 1,
            percentage: 33.3,
        };
        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("Test"));
        let back: CategoryBreakdown = serde_json::from_str(&json).unwrap();
        assert_eq!(back.category_name, "Test");
    }

    #[test]
    fn spending_trend_serialization() {
        let trend = SpendingTrend {
            period: "2024-06".into(),
            total: dec!(150.0),
        };
        let json = serde_json::to_string(&trend).unwrap();
        let back: SpendingTrend = serde_json::from_str(&json).unwrap();
        assert_eq!(back.period, "2024-06");
    }

    #[test]
    fn report_response_with_trends() {
        let resp = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(100.0),
                transactions_total: dec!(50.0),
                overall_total: dec!(150.0),
                subscriptions_count: 5,
                transactions_count: 3,
            },
            category_breakdown: vec![],
            recent_trends: vec![SpendingTrend {
                period: "2024-05".into(),
                total: dec!(140.0),
            }],
        };
        assert!(!resp.recent_trends.is_empty());
    }

    #[test]
    fn report_response_zero_totals() {
        let resp = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 1,
                subscriptions_total: dec!(0),
                transactions_total: dec!(0),
                overall_total: dec!(0),
                subscriptions_count: 0,
                transactions_count: 0,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        assert_eq!(resp.monthly_summary.overall_total, dec!(0));
    }

    #[test]
    fn monthly_summary_counts() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(80.0),
            transactions_total: dec!(40.0),
            overall_total: dec!(120.0),
            subscriptions_count: 8,
            transactions_count: 5,
        };
        assert_eq!(summary.subscriptions_count, 8);
        assert_eq!(summary.transactions_count, 5);
    }

    #[test]
    fn summary_url_year_month_boundaries() {
        for (y, m) in [(2020, 1), (2030, 12), (2024, 6)] {
            let path = format!("/api/reports/summary?year={}&month={}", y, m);
            assert!(path.contains(&format!("year={}", y)));
            assert!(path.contains(&format!("month={}", m)));
        }
    }
}
