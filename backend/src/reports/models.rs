use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: u32,
    pub subscriptions_total: Decimal,
    pub transactions_total: Decimal,
    pub overall_total: Decimal,
    pub subscriptions_count: i64,
    pub transactions_count: i64,
}

#[derive(Debug, Serialize)]
pub struct CategoryBreakdown {
    pub category_id: Option<String>,
    pub category_name: String,
    pub total: Decimal,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct SpendingTrend {
    pub period: String,
    pub total: Decimal,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub monthly_summary: MonthlySummary,
    pub category_breakdown: Vec<CategoryBreakdown>,
    pub recent_trends: Vec<SpendingTrend>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn monthly_summary_serialization() {
        let summary = MonthlySummary {
            year: 2024,
            month: 6,
            subscriptions_total: dec("500.00"),
            transactions_total: dec("200.00"),
            overall_total: dec("700.00"),
            subscriptions_count: 5,
            transactions_count: 10,
        };
        let json = serde_json::to_value(&summary).unwrap();
        assert_eq!(json["year"], 2024);
        assert_eq!(json["month"], 6);
        assert!(json.get("overall_total").is_some());
    }

    #[test]
    fn category_breakdown_serialization() {
        let bd = CategoryBreakdown {
            category_id: Some("cat-123".to_string()),
            category_name: "Streaming".to_string(),
            total: dec("150.00"),
            count: 3,
            percentage: 50.0,
        };
        let json = serde_json::to_value(&bd).unwrap();
        assert_eq!(json["category_name"], "Streaming");
    }

    #[test]
    fn spending_trend_serialization() {
        let trend = SpendingTrend {
            period: "6/24".to_string(),
            total: dec("350.00"),
        };
        let json = serde_json::to_value(&trend).unwrap();
        assert_eq!(json["period"], "6/24");
    }

    #[test]
    fn report_response_composite() {
        let report = ReportResponse {
            monthly_summary: MonthlySummary {
                year: 2024,
                month: 1,
                subscriptions_total: Decimal::ZERO,
                transactions_total: Decimal::ZERO,
                overall_total: Decimal::ZERO,
                subscriptions_count: 0,
                transactions_count: 0,
            },
            category_breakdown: vec![],
            recent_trends: vec![],
        };
        let json = serde_json::to_value(&report).unwrap();
        assert!(json.get("monthly_summary").is_some());
        assert!(json.get("category_breakdown").unwrap().is_array());
        assert!(json.get("recent_trends").unwrap().is_array());
    }
}
