use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub name: Option<String>,
    pub amount: Option<Decimal>,
    pub currency: Option<String>,
    pub date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub amount: Decimal,
    pub currency: String,
    pub date: NaiveDate,
    pub notes: Option<String>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Transaction> for TransactionResponse {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id,
            user_id: t.user_id,
            category_id: t.category_id,
            name: t.name,
            amount: t.amount,
            currency: t.currency,
            date: t.date,
            notes: t.notes,
            logo_url: t.logo_url,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}
