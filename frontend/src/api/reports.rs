use crate::api::client::{api_get, ApiError};
use crate::models::report::*;

pub async fn get_summary(year: i32, month: u32) -> Result<ReportResponse, ApiError> {
    api_get(&format!("/api/reports/summary?year={}&month={}", year, month)).await
}

#[cfg(test)]
mod tests {
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
}