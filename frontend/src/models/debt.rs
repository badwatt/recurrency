use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Debt {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(default)]
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

impl Debt {
    pub fn progress(&self) -> f64 {
        if self.total_amount <= Decimal::ZERO {
            return 0.0;
        }
        let paid: f64 = (self.total_amount - self.remaining_amount)
            .try_into()
            .unwrap_or(0.0);
        let total: f64 = self.total_amount.try_into().unwrap_or(1.0);
        (paid / total * 100.0).clamp(0.0, 100.0)
    }

    pub fn is_automatic(&self) -> bool {
        self.debt_type == "automatic"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateDebtRequest {
    pub name: String,
    pub total_amount: Decimal,
    pub currency: String,
    #[serde(default)]
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub debt_type: String,
    pub monthly_payment: Option<Decimal>,
    pub start_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateDebtRequest {
    pub name: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub clear_notes: bool,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordDebtPaymentRequest {
    pub amount: Decimal,
    pub paid_on: NaiveDate,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DebtPayment {
    pub id: Uuid,
    pub debt_id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub paid_on: NaiveDate,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn debt(total: Decimal, remaining: Decimal) -> Debt {
        Debt {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "Mortgage".into(),
            currency: "EUR".into(),
            total_amount: total,
            remaining_amount: remaining,
            paid_amount: total - remaining,
            debt_type: "automatic".into(),
            monthly_payment: Some(Decimal::from(100)),
            next_payment_date: None,
            subscription_id: Some(Uuid::new_v4()),
            is_active: remaining > Decimal::ZERO,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn progress_is_percentage() {
        let value = debt(Decimal::from(1000), Decimal::from(250));
        assert!((value.progress() - 75.0).abs() < 0.01);
    }

    #[test]
    fn type_detection() {
        assert!(debt(Decimal::from(10), Decimal::from(5)).is_automatic());
    }
}
