use crate::api::client::{api_get, ApiError};
use crate::models::report::*;

pub async fn get_summary(year: i32, month: u32) -> Result<ReportResponse, ApiError> {
    api_get(&format!("/api/reports/summary?year={}&month={}", year, month)).await
}