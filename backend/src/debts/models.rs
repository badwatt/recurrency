use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Debt {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub currency: String,
    pub total_amount: Decimal,
    pub remaining_amount: Decimal,
    pub debt_type: String,
    pub monthly_payment: Option<Decimal>,
    pub next_payment_date: Option<NaiveDate>,
    pub subscription_id: Option<Uuid>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtPayment {
    pub id: Uuid,
    pub debt_id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub paid_on: NaiveDate,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDebtRequest {
    pub name: String,
    pub total_amount: Decimal,
    pub currency: String,
    #[serde(default)]
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    /// "manual" or "automatic"
    #[serde(default = "manual_type")]
    pub debt_type: String,
    #[serde(default)]
    pub monthly_payment: Option<Decimal>,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateDebtRequest {
    pub name: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub clear_notes: bool,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordDebtPaymentRequest {
    pub amount: Decimal,
    pub paid_on: NaiveDate,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DebtResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub currency: String,
    pub total_amount: Decimal,
    pub remaining_amount: Decimal,
    pub paid_amount: Decimal,
    pub debt_type: String,
    pub monthly_payment: Option<Decimal>,
    pub next_payment_date: Option<NaiveDate>,
    pub subscription_id: Option<Uuid>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Debt> for DebtResponse {
    fn from(debt: Debt) -> Self {
        Self {
            paid_amount: debt.total_amount - debt.remaining_amount,
            id: debt.id,
            user_id: debt.user_id,
            account_id: debt.account_id,
            category_id: debt.category_id,
            name: debt.name,
            currency: debt.currency,
            total_amount: debt.total_amount,
            remaining_amount: debt.remaining_amount,
            debt_type: debt.debt_type,
            monthly_payment: debt.monthly_payment,
            next_payment_date: debt.next_payment_date,
            subscription_id: debt.subscription_id,
            is_active: debt.is_active,
            notes: debt.notes,
            logo_url: debt.logo_url,
            created_at: debt.created_at,
            updated_at: debt.updated_at,
        }
    }
}

fn manual_type() -> String {
    "manual".to_string()
}

pub fn validate_debt_type(value: &str) -> bool {
    matches!(value, "manual" | "automatic")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debt_type_validation() {
        assert!(validate_debt_type("manual"));
        assert!(validate_debt_type("automatic"));
        assert!(!validate_debt_type("credit"));
    }
}
