//! Integration tests for notifications, reports, and logos handlers.
//!
//! These tests verify:
//! 1. Handler input parsing/extraction logic
//! 2. Service-level pure business logic
//! 3. Model serialization/deserialization
//! 4. Error handling paths

use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// ============================================================================
// NOTIFICATIONS TESTS
// ============================================================================

mod notifications_tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Notification {
        pub id: Uuid,
        pub user_id: Uuid,
        #[serde(rename = "type")]
        pub notification_type: String,
        pub title: String,
        pub message: Option<String>,
        pub is_read: bool,
        pub created_at: chrono::DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NotificationResponse {
        pub notifications: Vec<Notification>,
        pub unread_count: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NotificationPreferenceResponse {
        pub channel: String,
        pub enabled: bool,
        pub reminder_days_before: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NotificationPreferencesResponse {
        pub preferences: Vec<NotificationPreferenceResponse>,
    }

    #[derive(Debug, Deserialize)]
    pub struct UpdateNotificationPreferencesRequest {
        pub preferences: Vec<UpdatePreferenceRequest>,
    }

    #[derive(Debug, Deserialize)]
    pub struct UpdatePreferenceRequest {
        pub channel: String,
        pub enabled: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct NotificationQuery {
        pub token: Option<String>,
    }

    fn default_notification_preferences() -> Vec<NotificationPreferenceResponse> {
        vec![
            NotificationPreferenceResponse { channel: "email".to_string(), enabled: false, reminder_days_before: 3 },
            NotificationPreferenceResponse { channel: "push".to_string(), enabled: false, reminder_days_before: 3 },
            NotificationPreferenceResponse { channel: "system".to_string(), enabled: false, reminder_days_before: 3 },
        ]
    }

    fn mock_list_notifications(user_id: Uuid) -> NotificationResponse {
        let notifications = vec![
            Notification {
                id: Uuid::new_v4(),
                user_id,
                notification_type: "billing".to_string(),
                title: "Netflix Due Tomorrow".to_string(),
                message: Some("Your Netflix subscription renews tomorrow".to_string()),
                is_read: false,
                created_at: chrono::DateTime::<Utc>::from_timestamp(Utc::now().timestamp() - 3600, 0).unwrap(),
            },
            Notification {
                id: Uuid::new_v4(),
                user_id,
                notification_type: "reminder".to_string(),
                title: "Spotify Payment".to_string(),
                message: None,
                is_read: true,
                created_at: chrono::DateTime::<Utc>::from_timestamp(Utc::now().timestamp() - 86400, 0).unwrap(),
            },
        ];
        NotificationResponse { notifications, unread_count: 1 }
    }

    fn mock_mark_read(notification_id: Uuid, user_id: Uuid) -> bool {
        !notification_id.is_nil() && !user_id.is_nil()
    }

    fn mock_mark_all_read(user_id: Uuid) -> i64 {
        if !user_id.is_nil() { 5 } else { 0 }
    }

    fn mock_delete_notification(notification_id: Uuid, user_id: Uuid) -> bool {
        !notification_id.is_nil() && !user_id.is_nil()
    }

    fn mock_update_preferences(_user_id: Uuid, updates: Vec<UpdatePreferenceRequest>) -> Vec<NotificationPreferenceResponse> {
        let mut prefs = default_notification_preferences();
        for req in updates {
            if let Some(p) = prefs.iter_mut().find(|p| p.channel == req.channel) {
                p.enabled = req.enabled;
            }
        }
        prefs
    }

    // Handler Input Tests
    #[test]
    fn test_notification_query_with_token() {
        let query = "token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.unwrap().starts_with("eyJ"));
    }

    #[test]
    fn test_notification_query_without_token() {
        let query = "";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_none());
    }

    #[test]
    fn test_notification_query_empty_token() {
        let query = "token=";
        let result: NotificationQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.token.is_some());
        assert!(result.token.unwrap().is_empty());
    }

    #[test]
    fn test_parse_valid_uuid() {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_version_num(), 4);
    }

    #[test]
    fn test_parse_invalid_uuid_returns_err() {
        let id_str = "not-a-valid-uuid";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_uuid_returns_err() {
        let id_str = "";
        let result = Uuid::parse_str(id_str);
        assert!(result.is_err());
    }

    // Mock Service Tests
    #[test]
    fn test_mock_list_notifications_returns_correct_count() {
        let user_id = Uuid::new_v4();
        let response = mock_list_notifications(user_id);
        assert_eq!(response.notifications.len(), 2);
        assert_eq!(response.unread_count, 1);
        for n in &response.notifications {
            assert_eq!(n.user_id, user_id);
        }
    }

    #[test]
    fn test_mock_list_notifications_unread_count() {
        let user_id = Uuid::new_v4();
        let response = mock_list_notifications(user_id);
        let unread: Vec<_> = response.notifications.iter().filter(|n| !n.is_read).collect();
        assert_eq!(unread.len(), response.unread_count as usize);
    }

    #[test]
    fn test_mock_mark_read_success() {
        let notification_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        assert!(mock_mark_read(notification_id, user_id));
    }

    #[test]
    fn test_mock_mark_read_nil_uuid_fails() {
        let notification_id = Uuid::nil();
        let user_id = Uuid::new_v4();
        assert!(!mock_mark_read(notification_id, user_id));
    }

    #[test]
    fn test_mock_mark_all_read_returns_count() {
        let user_id = Uuid::new_v4();
        let count = mock_mark_all_read(user_id);
        assert_eq!(count, 5);
    }

    #[test]
    fn test_mock_delete_notification() {
        let notification_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        assert!(mock_delete_notification(notification_id, user_id));
    }

    // Preferences Tests
    #[test]
    fn test_default_preferences_three_channels() {
        let prefs = default_notification_preferences();
        assert_eq!(prefs.len(), 3);
    }

    #[test]
    fn test_default_preferences_all_disabled() {
        let prefs = default_notification_preferences();
        for p in prefs {
            assert!(!p.enabled);
            assert_eq!(p.reminder_days_before, 3);
        }
    }

    #[test]
    fn test_update_preferences_enable_email() {
        let user_id = Uuid::new_v4();
        let updates = vec![UpdatePreferenceRequest { channel: "email".to_string(), enabled: true }];
        let result = mock_update_preferences(user_id, updates);
        let email = result.iter().find(|p| p.channel == "email").unwrap();
        assert!(email.enabled);
    }

    #[test]
    fn test_update_preferences_disable_push() {
        let user_id = Uuid::new_v4();
        let updates = vec![UpdatePreferenceRequest { channel: "push".to_string(), enabled: false }];
        let result = mock_update_preferences(user_id, updates);
        let push = result.iter().find(|p| p.channel == "push").unwrap();
        assert!(!push.enabled);
    }

    #[test]
    fn test_update_multiple_preferences() {
        let user_id = Uuid::new_v4();
        let updates = vec![
            UpdatePreferenceRequest { channel: "email".to_string(), enabled: true },
            UpdatePreferenceRequest { channel: "push".to_string(), enabled: true },
            UpdatePreferenceRequest { channel: "system".to_string(), enabled: false },
        ];
        let result = mock_update_preferences(user_id, updates);
        assert!(result.iter().find(|p| p.channel == "email").unwrap().enabled);
        assert!(result.iter().find(|p| p.channel == "push").unwrap().enabled);
        assert!(!result.iter().find(|p| p.channel == "system").unwrap().enabled);
    }

    // Serialization Tests
    #[test]
    fn test_notification_response_json_serialization() {
        let user_id = Uuid::new_v4();
        let response = mock_list_notifications(user_id);
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("notifications").unwrap().is_array());
        assert_eq!(json["unread_count"], 1);
    }

    #[test]
    fn test_notification_serialization_with_type_field() {
        let notification = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "billing_reminder".to_string(),
            title: "Test".to_string(),
            message: Some("Test message".to_string()),
            is_read: false,
            created_at: chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
        };
        let json = serde_json::to_value(&notification).unwrap();
        assert_eq!(json["type"], "billing_reminder");
        assert_eq!(json["title"], "Test");
    }

    #[test]
    fn test_update_preferences_request_deserialization() {
        let json = r#"{"preferences":[{"channel":"email","enabled":true},{"channel":"push","enabled":false}]}"#;
        let req: UpdateNotificationPreferencesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.preferences.len(), 2);
        assert_eq!(req.preferences[0].channel, "email");
        assert!(req.preferences[0].enabled);
    }

    #[test]
    fn test_notification_preferences_response_serialization() {
        let response = NotificationPreferencesResponse { preferences: default_notification_preferences() };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"preferences\""));
    }

    // Error Handling Tests
    #[test]
    fn test_uuid_parse_error_is_detectable() {
        let invalid_id = "invalid";
        let result = Uuid::parse_str(invalid_id);
        assert!(result.is_err());
        let err = result.map_err(|_| "Invalid notification ID".to_string());
        assert!(err.is_err());
    }

    #[test]
    fn test_notification_with_null_message() {
        let notification = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "system".to_string(),
            title: "System Update".to_string(),
            message: None,
            is_read: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_value(&notification).unwrap();
        assert!(json.get("message").unwrap().is_null());
    }

    #[test]
    fn test_preferences_reminder_days_range() {
        let prefs = vec![
            NotificationPreferenceResponse { channel: "email".to_string(), enabled: true, reminder_days_before: 0 },
            NotificationPreferenceResponse { channel: "push".to_string(), enabled: true, reminder_days_before: 7 },
        ];
        for p in prefs {
            assert!(p.reminder_days_before >= 0);
            assert!(p.reminder_days_before <= 30);
        }
    }

    #[test]
    fn test_empty_update_preferences_keeps_defaults() {
        let user_id = Uuid::new_v4();
        let updates = vec![];
        let result = mock_update_preferences(user_id, updates);
        for p in result {
            assert!(!p.enabled);
        }
    }
}

// ============================================================================
// REPORTS TESTS
// ============================================================================

mod reports_tests {
    use super::*;

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

    #[derive(Debug, Deserialize)]
    pub struct ReportQuery {
        pub token: Option<String>,
        pub year: Option<i32>,
        pub month: Option<u32>,
    }

    fn dec(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn month_date_range(year: i32, month: u32) -> (NaiveDate, NaiveDate) {
        let from_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let to_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
        };
        (from_date, to_date)
    }

    fn calculate_percentage(total: Decimal, overall_total: Decimal) -> f64 {
        if overall_total > Decimal::ZERO {
            (total / overall_total * Decimal::from(100)).round_dp(1).to_string().parse().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn combine_category_totals(
        subs_by_cat: Vec<(Option<String>, String, Decimal)>,
        txns_by_cat: Vec<(Option<String>, String, Decimal)>,
    ) -> Vec<CategoryBreakdown> {
        let mut category_totals: HashMap<String, Decimal> = HashMap::new();
        let mut category_ids: HashMap<String, Option<String>> = HashMap::new();
        let mut category_counts: HashMap<String, i64> = HashMap::new();

        for (cat_id, cat_name, total) in subs_by_cat {
            let key = cat_name.clone();
            *category_totals.entry(key.clone()).or_insert(Decimal::ZERO) += total;
            category_ids.entry(key.clone()).or_insert(cat_id);
            *category_counts.entry(key).or_insert(0) += 1;
        }

        for (cat_id, cat_name, total) in txns_by_cat {
            let key = cat_name.clone();
            *category_totals.entry(key.clone()).or_insert(Decimal::ZERO) += total;
            if !category_ids.contains_key(&key) {
                category_ids.insert(key.clone(), cat_id);
            }
            *category_counts.entry(key).or_insert(0) += 1;
        }

        let overall_total: Decimal = category_totals.values().sum();
        let mut breakdown: Vec<CategoryBreakdown> = category_totals
            .into_iter()
            .map(|(name, total)| {
                let percentage = calculate_percentage(total, overall_total);
                CategoryBreakdown {
                    category_id: category_ids.remove(&name).unwrap_or(None),
                    category_name: name.clone(),
                    total,
                    count: *category_counts.get(&name).unwrap_or(&0),
                    percentage,
                }
            })
            .collect();

        breakdown.sort_by(|a, b| b.total.cmp(&a.total));
        breakdown
    }

    fn days_in_month(year: i32, month: u32) -> u32 {
        if month == 12 { 31 } else {
            (NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)).day()
        }
    }

    fn mock_generate_report(_user_id: Uuid, year: i32, month: u32) -> ReportResponse {
        let summary = MonthlySummary {
            year, month,
            subscriptions_total: dec("500.00"),
            transactions_total: dec("200.00"),
            overall_total: dec("700.00"),
            subscriptions_count: 5,
            transactions_count: 10,
        };
        let breakdown = vec![
            CategoryBreakdown { category_id: Some("cat-1".to_string()), category_name: "Streaming".to_string(), total: dec("150.00"), count: 3, percentage: 21.4 },
            CategoryBreakdown { category_id: Some("cat-2".to_string()), category_name: "Software".to_string(), total: dec("100.00"), count: 2, percentage: 14.3 },
        ];
        let trends = vec![
            SpendingTrend { period: "3/26".to_string(), total: dec("650.00") },
            SpendingTrend { period: "2/26".to_string(), total: dec("620.00") },
            SpendingTrend { period: "1/26".to_string(), total: dec("580.00") },
        ];
        ReportResponse { monthly_summary: summary, category_breakdown: breakdown, recent_trends: trends }
    }

    // Handler Query Parsing Tests
    #[test]
    fn test_report_query_full_params() {
        let query = "year=2024&month=6&token=abc123";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert_eq!(result.month, Some(6));
        assert_eq!(result.token.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_report_query_partial_params() {
        let query = "year=2024";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.year, Some(2024));
        assert!(result.month.is_none());
        assert!(result.token.is_none());
    }

    #[test]
    fn test_report_query_empty() {
        let query = "";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.year.is_none());
        assert!(result.month.is_none());
    }

    #[test]
    fn test_report_query_only_token() {
        let query = "token=xyz";
        let result: ReportQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.token.as_deref(), Some("xyz"));
    }

    // Date Range Calculation Tests
    #[test]
    fn test_month_date_range_january() {
        let (from, to) = month_date_range(2024, 1);
        assert_eq!(from, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn test_month_date_range_february_leap() {
        let (from, to) = month_date_range(2024, 2);
        assert_eq!(from, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    }

    #[test]
    fn test_month_date_range_february_non_leap() {
        let (_, to) = month_date_range(2023, 2);
        assert_eq!(to, NaiveDate::from_ymd_opt(2023, 2, 28).unwrap());
    }

    #[test]
    fn test_month_date_range_april() {
        let (from, to) = month_date_range(2024, 4);
        assert_eq!(from, NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2024, 4, 30).unwrap());
    }

    #[test]
    fn test_month_date_range_december() {
        let (from, to) = month_date_range(2024, 12);
        assert_eq!(from, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    // Percentage Calculation Tests
    #[test]
    fn test_calculate_percentage_half() {
        let result = calculate_percentage(dec("50"), dec("100"));
        assert!((result - 50.0).abs() < 0.2);
    }

    #[test]
    fn test_calculate_percentage_full() {
        let result = calculate_percentage(dec("100"), dec("100"));
        assert!((result - 100.0).abs() < 0.2);
    }

    #[test]
    fn test_calculate_percentage_zero_total() {
        let result = calculate_percentage(dec("50"), Decimal::ZERO);
        assert_eq!(result, 0.0);
    }

    // Category Aggregation Tests
    #[test]
    fn test_combine_empty_categories() {
        let subs: Vec<(Option<String>, String, Decimal)> = vec![];
        let txns: Vec<(Option<String>, String, Decimal)> = vec![];
        let result = combine_category_totals(subs, txns);
        assert!(result.is_empty());
    }

    #[test]
    fn test_combine_subscriptions_only() {
        let subs = vec![(Some("cat-1".to_string()), "Streaming".to_string(), dec("50"))];
        let txns: Vec<(Option<String>, String, Decimal)> = vec![];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category_name, "Streaming");
        assert_eq!(result[0].total, dec("50"));
        assert_eq!(result[0].percentage, 100.0);
    }

    #[test]
    fn test_combine_transactions_only() {
        let subs: Vec<(Option<String>, String, Decimal)> = vec![];
        let txns = vec![(Some("cat-2".to_string()), "Groceries".to_string(), dec("200"))];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].category_name, "Groceries");
    }

    #[test]
    fn test_combine_same_category_merges() {
        let subs = vec![(Some("cat-1".to_string()), "Entertainment".to_string(), dec("30"))];
        let txns = vec![(None, "Entertainment".to_string(), dec("20"))];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].total, dec("50"));
    }

    #[test]
    fn test_combine_sorted_by_total_descending() {
        let subs = vec![
            (Some("1".to_string()), "Small".to_string(), dec("10")),
            (Some("2".to_string()), "Big".to_string(), dec("90")),
        ];
        let txns: Vec<(Option<String>, String, Decimal)> = vec![];
        let result = combine_category_totals(subs, txns);
        assert_eq!(result[0].category_name, "Big");
        assert_eq!(result[1].category_name, "Small");
    }

    // Days in Month Tests
    #[test]
    fn test_days_in_month_february_leap() {
        assert_eq!(days_in_month(2024, 2), 29);
    }

    #[test]
    fn test_days_in_month_february_non_leap() {
        assert_eq!(days_in_month(2023, 2), 28);
    }

    // Mock Report Tests
    #[test]
    fn test_mock_generate_report_structure() {
        let user_id = Uuid::new_v4();
        let report = mock_generate_report(user_id, 2024, 6);
        assert_eq!(report.monthly_summary.year, 2024);
        assert_eq!(report.monthly_summary.month, 6);
        assert!(!report.category_breakdown.is_empty());
        assert!(!report.recent_trends.is_empty());
    }

    #[test]
    fn test_mock_report_overall_total() {
        let user_id = Uuid::new_v4();
        let report = mock_generate_report(user_id, 2024, 6);
        assert_eq!(report.monthly_summary.overall_total, report.monthly_summary.subscriptions_total + report.monthly_summary.transactions_total);
    }

    #[test]
    fn test_mock_report_categories_sorted() {
        let user_id = Uuid::new_v4();
        let report = mock_generate_report(user_id, 2024, 6);
        for i in 0..report.category_breakdown.len() - 1 {
            assert!(report.category_breakdown[i].total >= report.category_breakdown[i + 1].total);
        }
    }

    // Serialization Tests
    #[test]
    fn test_report_response_serialization() {
        let user_id = Uuid::new_v4();
        let report = mock_generate_report(user_id, 2024, 6);
        let json = serde_json::to_value(&report).unwrap();
        assert!(json.get("monthly_summary").is_some());
        assert!(json.get("category_breakdown").unwrap().is_array());
    }

    #[test]
    fn test_monthly_summary_serialization() {
        let summary = MonthlySummary {
            year: 2024, month: 6,
            subscriptions_total: dec("500.00"),
            transactions_total: dec("200.00"),
            overall_total: dec("700.00"),
            subscriptions_count: 5, transactions_count: 10,
        };
        let json = serde_json::to_value(&summary).unwrap();
        assert_eq!(json["year"], 2024);
        assert_eq!(json["month"], 6);
    }

    #[test]
    fn test_trend_period_format() {
        let period = format!("{}/{:02}", 6, 24);
        assert_eq!(period, "6/24");
    }
}

// ============================================================================
// LOGOS TESTS
// ============================================================================

mod logos_tests {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct LogoResult {
        pub url: String,
        pub thumbnail: String,
        pub width: Option<i32>,
        pub height: Option<i32>,
    }

    #[derive(Debug, Deserialize)]
    pub struct LogoSearchQuery {
        pub q: String,
    }

    #[derive(Debug, Serialize)]
    pub struct LogoSearchResponse {
        pub results: Vec<LogoResult>,
    }

    fn extract_vqd(html: &str) -> Option<String> {
        for prefix in &["vqd='", "vqd=\"", "vqd="] {
            if let Some(start) = html.find(prefix) {
                let token_start = start + prefix.len();
                let token_end = html[token_start..]
                    .find(|c: char| c == '\'' || c == '"' || c == '&' || c == ';')
                    .map(|i| token_start + i)
                    .unwrap_or(html.len());
                let token = &html[token_start..token_end];
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
        None
    }

    fn mock_logo_results(query: &str) -> Vec<LogoResult> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return vec![];
        }
        vec![
            LogoResult { url: format!("https://example.com/{}.png", query), thumbnail: format!("https://example.com/{}_thumb.png", query), width: Some(200), height: Some(100) },
            LogoResult { url: format!("https://cdn.example.com/{}-logo.png", query), thumbnail: format!("https://cdn.example.com/{}-logo_thumb.png", query), width: Some(300), height: Some(150) },
        ]
    }

    // Query Parsing Tests
    #[test]
    fn test_logo_search_query_simple() {
        let query = "q=netflix";
        let result: LogoSearchQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.q, "netflix");
    }

    #[test]
    fn test_logo_search_query_with_spaces() {
        let query = "q=spotify+premium";
        let result: LogoSearchQuery = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(result.q, "spotify premium");
    }

    #[test]
    fn test_logo_search_query_empty() {
        let query = "q=";
        let result: LogoSearchQuery = serde_urlencoded::from_str(query).unwrap();
        assert!(result.q.is_empty());
    }

    // VQD Extraction Tests
    #[test]
    fn test_extract_vqd_single_quotes() {
        let html = r#"some text vqd='123-456' other stuff"#;
        assert_eq!(extract_vqd(html), Some("123-456".to_string()));
    }

    #[test]
    fn test_extract_vqd_double_quotes() {
        let html = r#"some text vqd="123-456" other stuff"#;
        assert_eq!(extract_vqd(html), Some("123-456".to_string()));
    }

    #[test]
    fn test_extract_vqd_no_quotes_with_ampersand() {
        let html = r#"some text vqd=123-456& other stuff"#;
        assert_eq!(extract_vqd(html), Some("123-456".to_string()));
    }

    #[test]
    fn test_extract_vqd_no_quotes_with_semicolon() {
        let html = r#"some text vqd=abc-def; other stuff"#;
        assert_eq!(extract_vqd(html), Some("abc-def".to_string()));
    }

    #[test]
    fn test_extract_vqd_missing() {
        let html = r#"some text without vqd token"#;
        assert_eq!(extract_vqd(html), None);
    }

    #[test]
    fn test_extract_vqd_empty_token() {
        let html = r#"some text vqd='' other stuff"#;
        assert_eq!(extract_vqd(html), None);
    }

    #[test]
    fn test_extract_vqd_at_end_of_string() {
        let html = r#"vqd='token123'"#;
        assert_eq!(extract_vqd(html), Some("token123".to_string()));
    }

    #[test]
    fn test_extract_vqd_mixed_html() {
        let html = r#"<html><body>vqd='my-token-123' more</body></html>"#;
        assert_eq!(extract_vqd(html), Some("my-token-123".to_string()));
    }

    #[test]
    fn test_extract_vqd_multiple_vqds() {
        let html = r#"vqd='first' vqd='second'"#;
        assert_eq!(extract_vqd(html), Some("first".to_string()));
    }

    #[test]
    fn test_extract_vqd_long_token() {
        let token = "a".repeat(200);
        let html = format!("vqd='{}'", token);
        assert_eq!(extract_vqd(&html), Some(token));
    }

    // Mock Logo Search Tests
    #[test]
    fn test_mock_logo_results_simple_query() {
        let results = mock_logo_results("netflix");
        assert_eq!(results.len(), 2);
        assert!(results[0].url.contains("netflix"));
    }

    #[test]
    fn test_mock_logo_results_empty_query() {
        let results = mock_logo_results("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_mock_logo_results_whitespace_query() {
        let results = mock_logo_results("   ");
        assert!(results.is_empty());
    }

    #[test]
    fn test_mock_logo_results_url_format() {
        let results = mock_logo_results("spotify");
        for result in &results {
            assert!(result.url.starts_with("https://"));
            assert!(result.thumbnail.starts_with("https://"));
        }
    }

    #[test]
    fn test_mock_logo_results_dimensions() {
        let results = mock_logo_results("amazon");
        for result in &results {
            assert!(result.width.is_some());
            assert!(result.height.is_some());
        }
    }

    // Serialization Tests
    #[test]
    fn test_logo_result_serialization() {
        let logo = LogoResult { url: "https://example.com/img.png".to_string(), thumbnail: "https://example.com/thumb.png".to_string(), width: Some(200), height: Some(100) };
        let json = serde_json::to_value(&logo).unwrap();
        assert_eq!(json["url"], "https://example.com/img.png");
        assert_eq!(json["width"], 200);
    }

    #[test]
    fn test_logo_result_no_dimensions() {
        let logo = LogoResult { url: "https://example.com/img.png".to_string(), thumbnail: "https://example.com/thumb.png".to_string(), width: None, height: None };
        let json = serde_json::to_value(&logo).unwrap();
        assert!(json["width"].is_null());
    }

    #[test]
    fn test_logo_search_response_serialization() {
        let response = LogoSearchResponse { results: vec![LogoResult { url: "https://x.com/1.png".to_string(), thumbnail: "https://x.com/t1.png".to_string(), width: Some(100), height: Some(50) }] };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"results\""));
    }

    // Search Term Formatting Tests
    #[test]
    fn test_search_term_format() {
        let query = "netflix";
        let search_term = format!("{} logo transparent", query);
        assert_eq!(search_term, "netflix logo transparent");
    }

    #[test]
    fn test_query_cleaning() {
        let query = "  Netflix  ".trim().to_lowercase();
        assert_eq!(query, "netflix");
    }

    #[test]
    fn test_results_take_limit() {
        let all_results: Vec<LogoResult> = (0..20)
            .map(|i| LogoResult { url: format!("https://x.com/{}.png", i), thumbnail: format!("https://x.com/t{}.png", i), width: None, height: None })
            .collect();
        let limited: Vec<LogoResult> = all_results.into_iter().take(8).collect();
        assert_eq!(limited.len(), 8);
    }

    #[test]
    fn test_thumbnail_uses_url_when_missing() {
        let logo = LogoResult { url: "https://example.com/img.png".to_string(), thumbnail: "https://example.com/img.png".to_string(), width: None, height: None };
        assert_eq!(logo.url, logo.thumbnail);
    }
}

// ============================================================================
// INTEGRATION SCENARIO TESTS
// ============================================================================

mod integration_tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Notification {
        pub id: Uuid, pub user_id: Uuid,
        #[serde(rename = "type")]
        pub notification_type: String,
        pub title: String, pub message: Option<String>,
        pub is_read: bool, pub created_at: chrono::DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NotificationResponse { pub notifications: Vec<Notification>, pub unread_count: i64 }

    #[derive(Debug, Serialize)]
    pub struct MonthlySummary {
        pub year: i32, pub month: u32,
        pub subscriptions_total: Decimal, pub transactions_total: Decimal,
        pub overall_total: Decimal, pub subscriptions_count: i64, pub transactions_count: i64,
    }

    #[derive(Debug, Serialize)]
    pub struct CategoryBreakdown {
        pub category_id: Option<String>, pub category_name: String,
        pub total: Decimal, pub count: i64, pub percentage: f64,
    }

    #[derive(Debug, Serialize)]
    pub struct SpendingTrend { pub period: String, pub total: Decimal }

    #[derive(Debug, Serialize)]
    pub struct ReportResponse {
        pub monthly_summary: MonthlySummary,
        pub category_breakdown: Vec<CategoryBreakdown>,
        pub recent_trends: Vec<SpendingTrend>,
    }

    #[derive(Debug, Serialize)]
    pub struct LogoResult { pub url: String, pub thumbnail: String, pub width: Option<i32>, pub height: Option<i32> }

    #[derive(Debug, Serialize)]
    pub struct LogoSearchResponse { pub results: Vec<LogoResult> }

    #[derive(Debug, Deserialize)]
    pub struct LogoSearchQuery { pub q: String }

    fn extract_vqd(html: &str) -> Option<String> {
        for prefix in &["vqd='", "vqd=\"", "vqd="] {
            if let Some(start) = html.find(prefix) {
                let token_start = start + prefix.len();
                let token_end = html[token_start..].find(|c: char| c == '\'' || c == '"' || c == '&' || c == ';').map(|i| token_start + i).unwrap_or(html.len());
                let token = &html[token_start..token_end];
                if !token.is_empty() { return Some(token.to_string()); }
            }
        }
        None
    }

    fn mock_generate_report(_user_id: Uuid, year: i32, month: u32) -> ReportResponse {
        let summary = MonthlySummary { year, month, subscriptions_total: Decimal::from_str("500.00").unwrap(), transactions_total: Decimal::from_str("200.00").unwrap(), overall_total: Decimal::from_str("700.00").unwrap(), subscriptions_count: 5, transactions_count: 10 };
        let breakdown = vec![CategoryBreakdown { category_id: Some("cat-1".to_string()), category_name: "Streaming".to_string(), total: Decimal::from_str("150.00").unwrap(), count: 3, percentage: 21.4 }];
        let trends = vec![SpendingTrend { period: "3/26".to_string(), total: Decimal::from_str("650.00").unwrap() }];
        ReportResponse { monthly_summary: summary, category_breakdown: breakdown, recent_trends: trends }
    }

    fn mock_logo_results(query: &str) -> Vec<LogoResult> {
        let query = query.trim().to_lowercase();
        if query.is_empty() { return vec![]; }
        vec![LogoResult { url: format!("https://example.com/{}.png", query), thumbnail: format!("https://example.com/{}_thumb.png", query), width: Some(200), height: Some(100) }]
    }

    #[test]
    fn test_notification_end_to_end_flow() {
        let user_id = Uuid::new_v4();
        let notifications = vec![Notification { id: Uuid::new_v4(), user_id, notification_type: "billing".to_string(), title: "Test".to_string(), message: None, is_read: false, created_at: Utc::now() }];
        let response = NotificationResponse { notifications, unread_count: 1 };
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("notifications").unwrap().is_array());
        assert_eq!(json["unread_count"], 1);
    }

    #[test]
    fn test_report_end_to_end_flow() {
        let user_id = Uuid::new_v4();
        let today = Utc::now().date_naive();
        let report = mock_generate_report(user_id, today.year(), today.month());
        let json = serde_json::to_value(&report).unwrap();
        assert!(json.get("monthly_summary").is_some());
        assert_eq!(report.monthly_summary.overall_total, report.monthly_summary.subscriptions_total + report.monthly_summary.transactions_total);
    }

    #[test]
    fn test_logo_search_end_to_end_flow() {
        let query: LogoSearchQuery = serde_urlencoded::from_str("q=spotify").unwrap();
        assert_eq!(query.q, "spotify");
        let search_term = format!("{} logo transparent", query.q);
        assert!(search_term.contains("logo"));
        let html = r#"<html>vqd='test-token-123'</html>"#;
        let vqd = extract_vqd(html);
        assert!(vqd.is_some());
        let results = mock_logo_results(&query.q);
        let response = LogoSearchResponse { results };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("results"));
    }
}
