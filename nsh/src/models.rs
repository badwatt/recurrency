//! API response models shared by the native screens.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_default: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AnnualDate {
    pub month: u8,
    pub day: u8,
    #[serde(default)]
    pub amount: Option<Decimal>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Recurrence {
    pub interval: u16,
    /// "day", "week", "month" or "year".
    pub unit: String,
    /// Days of the month. 0 means the last day of the month.
    #[serde(default)]
    pub days: Vec<u8>,
    /// ISO weekdays, Monday = 1.
    #[serde(default)]
    pub weekdays: Vec<u8>,
    /// Explicit annual dates, optionally with a per-date amount.
    #[serde(default)]
    pub dates: Vec<AnnualDate>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Subscription {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub recurrence: Recurrence,
    pub start_date: NaiveDate,
    pub next_billing_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_active: bool,
    #[serde(default)]
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    #[serde(default)]
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Rates {
    pub base: String,
    #[serde(default)]
    pub rates: HashMap<String, Decimal>,
}

impl Rates {
    pub fn convert(&self, amount: Decimal, from: &str) -> Decimal {
        if self.base.is_empty() || from.eq_ignore_ascii_case(&self.base) {
            return amount;
        }
        let Some(rate) = self.rates.get(&from.to_uppercase()) else {
            return amount;
        };
        if *rate <= Decimal::ZERO {
            return amount;
        }
        amount / *rate
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Debt {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub currency: String,
    pub total_amount: Decimal,
    pub remaining_amount: Decimal,
    #[serde(default)]
    pub paid_amount: Decimal,
    pub debt_type: String,
    pub monthly_payment: Option<Decimal>,
    #[serde(default)]
    pub payment_days: Vec<u8>,
    pub start_date: NaiveDate,
    pub next_payment_date: Option<NaiveDate>,
    pub subscription_id: Option<Uuid>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DebtPayment {
    pub id: Uuid,
    pub debt_id: Uuid,
    pub amount: Decimal,
    pub paid_on: NaiveDate,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CalendarItem {
    pub subscription_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub frequency: String,
    pub billing_date: NaiveDate,
    pub is_active: bool,
    pub logo_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: u32,
    pub subscriptions_total: Decimal,
    pub transactions_total: Decimal,
    pub overall_total: Decimal,
    pub subscriptions_count: i64,
    pub transactions_count: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CategoryBreakdown {
    pub category_id: Option<String>,
    pub category_name: String,
    pub total: Decimal,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SpendingTrend {
    pub period: String,
    pub total: Decimal,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ReportResponse {
    pub monthly_summary: MonthlySummary,
    pub category_breakdown: Vec<CategoryBreakdown>,
    pub recent_trends: Vec<SpendingTrend>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct NotificationPreference {
    pub channel: String,
    pub enabled: bool,
    pub reminder_days_before: i32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct NotificationPreferences {
    #[serde(default)]
    pub preferences: Vec<NotificationPreference>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct AiSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub configured: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AccountWithCounts {
    pub account: Account,
    pub subscription_count: i64,
    pub transaction_count: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CategoryWithCounts {
    pub category: Category,
    pub subscription_count: i64,
    pub transaction_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AdminUser {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub currency: String,
    pub verified: bool,
    pub is_admin: bool,
}
