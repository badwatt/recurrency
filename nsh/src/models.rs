//! API response models shared by the native screens.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_default: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
    pub logo_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub logo_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
