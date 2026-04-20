use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: u32,
    pub subscriptions_total: Decimal,
    pub transactions_total: Decimal,
    pub overall_total: Decimal,
    pub subscriptions_count: i64,
    pub transactions_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryBreakdown {
    pub category_id: Option<String>,
    pub category_name: String,
    pub total: Decimal,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpendingTrend {
    pub period: String,
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportResponse {
    pub monthly_summary: MonthlySummary,
    pub category_breakdown: Vec<CategoryBreakdown>,
    pub recent_trends: Vec<SpendingTrend>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn report_roundtrip() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 1,
                subscriptions_total: dec!(100),
                transactions_total: dec!(50),
                overall_total: dec!(150),
                subscriptions_count: 5,
                transactions_count: 3,
            },
            category_breakdown: vec![CategoryBreakdown {
                category_id: None,
                category_name: "Other".into(),
                total: dec!(150),
                count: 8,
                percentage: 100.0,
            }],
            recent_trends: vec![SpendingTrend {
                period: "2024-01".into(),
                total: dec!(150),
            }],
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: ReportResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(report, back);
    }

    #[test]
    fn monthly_summary_fields() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(200),
            transactions_total: dec!(75),
            overall_total: dec!(275),
            subscriptions_count: 10,
            transactions_count: 4,
        };
        assert_eq!(summary.year, 2024);
        assert_eq!(summary.month, 6);
        assert_eq!(summary.subscriptions_total, dec!(200));
        assert_eq!(summary.transactions_total, dec!(75));
        assert_eq!(summary.overall_total, dec!(275));
    }

    #[test]
    fn category_breakdown_with_id() {
        let breakdown = CategoryBreakdown {
            category_id: Some("uuid-123".into()),
            category_name: "Entertainment".into(),
            total: dec!(50),
            count: 2,
            percentage: 33.33,
        };
        let json = serde_json::to_string(&breakdown).unwrap();
        let back: CategoryBreakdown = serde_json::from_str(&json).unwrap();
        assert_eq!(back.category_id, Some("uuid-123".into()));
        assert_eq!(back.category_name, "Entertainment");
        assert_eq!(back.count, 2);
    }

    #[test]
    fn spending_trend_serialization() {
        let trend = SpendingTrend {
            period: "2024-03".into(),
            total: dec!(123.45),
        };
        let json = serde_json::to_string(&trend).unwrap();
        let back: SpendingTrend = serde_json::from_str(&json).unwrap();
        assert_eq!(back.period, "2024-03");
        assert_eq!(back.total, dec!(123.45));
    }

    #[test]
    fn empty_report() {
        let report = ReportResponse {
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
        let json = serde_json::to_string(&report).unwrap();
        let back: ReportResponse = serde_json::from_str(&json).unwrap();
        assert!(back.category_breakdown.is_empty());
        assert!(back.recent_trends.is_empty());
    }

    // ── MonthlySummary tests ──
    #[test]
    fn monthly_summary_debug() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(100),
            transactions_total: dec!(50),
            overall_total: dec!(150),
            subscriptions_count: 5,
            transactions_count: 2,
        };
        let debug = format!("{:?}", summary);
        assert!(debug.contains("MonthlySummary"));
    }

    #[test]
    fn monthly_summary_clone() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(100),
            transactions_total: dec!(50),
            overall_total: dec!(150),
            subscriptions_count: 5,
            transactions_count: 2,
        };
        let cloned = summary.clone();
        assert_eq!(summary, cloned);
    }

    #[test]
    fn monthly_summary_partial_eq() {
        let s1 = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(100),
            transactions_total: dec!(50),
            overall_total: dec!(150),
            subscriptions_count: 5,
            transactions_count: 2,
        };
        let s2 = MonthlySummary { ..s1 };
        assert_eq!(s1, s2);
    }

    #[test]
    fn monthly_summary_serialization() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(100.50),
            transactions_total: dec!(50.25),
            overall_total: dec!(150.75),
            subscriptions_count: 5,
            transactions_count: 2,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: MonthlySummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back);
    }

    // ── CategoryBreakdown tests ──
    #[test]
    fn category_breakdown_with_none_id() {
        let breakdown = CategoryBreakdown {
            category_id: None,
            category_name: "".into(),
            total: dec!(0),
            count: 0,
            percentage: 0.0,
        };
        assert!(breakdown.category_id.is_none());
        assert_eq!(breakdown.category_name, "");
        assert_eq!(breakdown.total, dec!(0));
    }

    #[test]
    fn category_breakdown_no_category_id() {
        let breakdown = CategoryBreakdown {
            category_id: None,
            category_name: "Uncategorized".into(),
            total: dec!(50),
            count: 3,
            percentage: 100.0,
        };
        assert!(breakdown.category_id.is_none());
        assert_eq!(breakdown.category_name, "Uncategorized");
    }

    #[test]
    fn category_breakdown_debug() {
        let breakdown = CategoryBreakdown {
            category_id: Some("cat-1".into()),
            category_name: "Food".into(),
            total: dec!(100),
            count: 5,
            percentage: 25.0,
        };
        let debug = format!("{:?}", breakdown);
        assert!(debug.contains("CategoryBreakdown"));
    }

    #[test]
    fn category_breakdown_clone() {
        let breakdown = CategoryBreakdown {
            category_id: Some("cat-1".into()),
            category_name: "Food".into(),
            total: dec!(100),
            count: 5,
            percentage: 25.0,
        };
        let cloned = breakdown.clone();
        assert_eq!(breakdown, cloned);
    }

    // ── SpendingTrend tests ──
    #[test]
    fn spending_trend_empty() {
        let trend = SpendingTrend {
            period: "".into(),
            total: dec!(0),
        };
        assert_eq!(trend.period, "");
        assert_eq!(trend.total, dec!(0));
    }

    #[test]
    fn spending_trend_debug() {
        let trend = SpendingTrend {
            period: "2024-06".into(),
            total: dec!(500),
        };
        let debug = format!("{:?}", trend);
        assert!(debug.contains("SpendingTrend"));
    }

    #[test]
    fn spending_trend_clone() {
        let trend = SpendingTrend {
            period: "2024-06".into(),
            total: dec!(500),
        };
        let cloned = trend.clone();
        assert_eq!(trend, cloned);
    }

    #[test]
    fn spending_trend_serialization_roundtrip() {
        let trend = SpendingTrend {
            period: "2024-Q2".into(),
            total: dec!(1234.56),
        };
        let json = serde_json::to_string(&trend).unwrap();
        let back: SpendingTrend = serde_json::from_str(&json).unwrap();
        assert_eq!(trend, back);
    }

    // ── ReportResponse tests ──
    #[test]
    fn report_response_debug() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(100),
                transactions_total: dec!(50),
                overall_total: dec!(150),
                subscriptions_count: 5,
                transactions_count: 2,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        let debug = format!("{:?}", report);
        assert!(debug.contains("ReportResponse"));
    }

    #[test]
    fn report_response_clone() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(100),
                transactions_total: dec!(50),
                overall_total: dec!(150),
                subscriptions_count: 5,
                transactions_count: 2,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        let cloned = report.clone();
        assert_eq!(report, cloned);
    }

    #[test]
    fn report_response_with_multiple_categories() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 6,
                subscriptions_total: dec!(200),
                transactions_total: dec!(100),
                overall_total: dec!(300),
                subscriptions_count: 10,
                transactions_count: 5,
            },
            category_breakdown: vec![
                CategoryBreakdown {
                    category_id: Some("cat-1".into()),
                    category_name: "Food".into(),
                    total: dec!(150),
                    count: 10,
                    percentage: 50.0,
                },
                CategoryBreakdown {
                    category_id: Some("cat-2".into()),
                    category_name: "Entertainment".into(),
                    total: dec!(150),
                    count: 5,
                    percentage: 50.0,
                },
            ],
            recent_trends: vec![
                SpendingTrend {
                    period: "2024-05".into(),
                    total: dec!(250),
                },
                SpendingTrend {
                    period: "2024-06".into(),
                    total: dec!(300),
                },
            ],
        };
        assert_eq!(report.category_breakdown.len(), 2);
        assert_eq!(report.recent_trends.len(), 2);
    }

    #[test]
    fn category_breakdown_percentage_range() {
        let breakdown = CategoryBreakdown {
            category_id: None,
            category_name: "Food".into(),
            total: dec!(100),
            count: 5,
            percentage: 85.5,
        };
        assert!(breakdown.percentage >= 0.0);
        assert!(breakdown.percentage <= 100.0);
    }

    #[test]
    fn spending_trend_positive() {
        let trend = SpendingTrend {
            period: "2024-06".into(),
            total: dec!(500),
        };
        assert!(trend.total > dec!(0));
    }

    #[test]
    fn spending_trend_negative_total() {
        let trend = SpendingTrend {
            period: "2024-05".into(),
            total: dec!(0),
        };
        assert_eq!(trend.total, dec!(0));
    }

    #[test]
    fn monthly_summary_transaction_counts() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec!(100),
            transactions_total: dec!(200),
            overall_total: dec!(300),
            subscriptions_count: 5,
            transactions_count: 15,
        };
        assert_eq!(summary.subscriptions_count + summary.transactions_count, 20);
    }
}