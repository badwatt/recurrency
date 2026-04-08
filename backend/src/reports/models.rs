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
