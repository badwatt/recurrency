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
}