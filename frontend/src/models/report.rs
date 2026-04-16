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
}