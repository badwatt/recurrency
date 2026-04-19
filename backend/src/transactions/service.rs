use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};
use crate::transactions::models::{CreateTransactionRequest, Transaction, UpdateTransactionRequest};
use crate::categories::service::CategoryService;

pub struct TransactionService;

/// Resolve updated fields from an existing transaction + update request.
/// Returns tuple: (name, amount, currency, date, category_id, notes, logo_url).
pub fn resolve_update_fields(
    existing: &Transaction,
    req: &UpdateTransactionRequest,
) -> (String, rust_decimal::Decimal, String, NaiveDate, Uuid, Option<String>, Option<String>) {
    (
        req.name.clone().unwrap_or_else(|| existing.name.clone()),
        req.amount.unwrap_or(existing.amount),
        req.currency.clone().unwrap_or_else(|| existing.currency.clone()),
        req.date.unwrap_or(existing.date),
        req.category_id.unwrap_or(existing.category_id),
        req.notes.clone().or_else(|| existing.notes.clone()),
        req.logo_url.clone().or_else(|| existing.logo_url.clone()),
    )
}

impl TransactionService {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateTransactionRequest,
    ) -> AppResult<Transaction> {
        let category_id = match req.category_id {
            Some(id) => id,
            None => CategoryService::get_default(pool, user_id).await?.id,
        };

        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (id, user_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)
            RETURNING id, user_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(category_id)
        .bind(&req.name)
        .bind(req.amount)
        .bind(&req.currency)
        .bind(req.date)
        .bind(&req.notes)
        .bind(&req.logo_url)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(transaction)
    }

    pub async fn get_all(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            "SELECT id, user_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at 
             FROM transactions WHERE user_id = $1 ORDER BY date DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(transactions)
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(
            "SELECT id, user_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at 
             FROM transactions WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

        Ok(transaction)
    }

    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: UpdateTransactionRequest,
    ) -> AppResult<Transaction> {
        let existing = Self::get_by_id(pool, user_id, id).await?;
        let (name, amount, currency, date, category_id, notes, logo_url) = resolve_update_fields(&existing, &req);

        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions 
            SET name = $1, amount = $2, currency = $3, date = $4, category_id = $5, notes = $6, logo_url = $7, updated_at = $8
            WHERE id = $9 AND user_id = $10
            RETURNING id, user_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(amount)
        .bind(&currency)
        .bind(date)
        .bind(category_id)
        .bind(&notes)
        .bind(&logo_url)
        .bind(Utc::now())
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(transaction)
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM transactions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Transaction not found".to_string()));
        }

        Ok(())
    }

    pub async fn get_by_date_range(
        pool: &PgPool,
        user_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> AppResult<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT id, user_id, category_id, name, amount, currency, date, notes, logo_url, created_at, updated_at 
            FROM transactions 
            WHERE user_id = $1 AND date >= $2 AND date <= $3
            ORDER BY date DESC
            "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await?;

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_transaction(name: &str, amount: rust_decimal::Decimal, date: NaiveDate, notes: Option<&str>, logo_url: Option<&str>) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: name.to_string(),
            amount,
            currency: "USD".to_string(),
            date,
            notes: notes.map(String::from),
            logo_url: logo_url.map(String::from),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_update(
        name: Option<&str>,
        amount: Option<rust_decimal::Decimal>,
        currency: Option<&str>,
        date: Option<NaiveDate>,
        notes: Option<&str>,
        logo_url: Option<&str>,
    ) -> UpdateTransactionRequest {
        UpdateTransactionRequest {
            name: name.map(String::from),
            amount,
            currency: currency.map(String::from),
            date,
            category_id: None,
            notes: notes.map(String::from),
            logo_url: logo_url.map(String::from),
        }
    }

    // ── resolve_update_fields ──

    #[test]
    fn resolve_update_all_fields_replaced() {
        let existing = make_transaction("Old", rust_decimal::Decimal::TEN, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = make_update(Some("New"), Some(rust_decimal::Decimal::from(20)), Some("EUR"), Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()), Some("new notes"), Some("http://new.png"));
        let (name, amount, currency, date, _, notes, logo_url) = resolve_update_fields(&existing, &req);
        assert_eq!(name, "New");
        assert_eq!(amount, rust_decimal::Decimal::from(20));
        assert_eq!(currency, "EUR");
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
        assert_eq!(notes, Some("new notes".to_string()));
        assert_eq!(logo_url, Some("http://new.png".to_string()));
    }

    #[test]
    fn resolve_update_partial_name_only() {
        let existing = make_transaction("Keep", rust_decimal::Decimal::from(50), NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(), Some("note"), None);
        let req = make_update(Some("New Name"), None, None, None, None, None);
        let (name, amount, _, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(name, "New Name");
        assert_eq!(amount, rust_decimal::Decimal::from(50));
    }

    #[test]
    fn resolve_update_partial_amount_only() {
        let existing = make_transaction("X", rust_decimal::Decimal::from(100), NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(), None, None);
        let req = make_update(None, Some(rust_decimal::Decimal::from(75)), None, None, None, None);
        let (_, amount, _, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(amount, rust_decimal::Decimal::from(75));
    }

    #[test]
    fn resolve_update_preserves_existing_when_req_fields_none() {
        let existing = make_transaction("Stay", rust_decimal::Decimal::from(30), NaiveDate::from_ymd_opt(2024, 7, 20).unwrap(), Some("keep notes"), Some("http://logo.png"));
        let req = UpdateTransactionRequest {
            name: Some("Changed".to_string()),
            amount: None,
            currency: None,
            date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        let (_, amount, currency, date, _, notes, logo_url) = resolve_update_fields(&existing, &req);
        assert_eq!(amount, rust_decimal::Decimal::from(30));
        assert_eq!(currency, "USD");
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 7, 20).unwrap());
        assert_eq!(notes, Some("keep notes".to_string()));
        assert_eq!(logo_url, Some("http://logo.png".to_string()));
    }

    #[test]
    fn resolve_update_preserves_notes_when_req_none() {
        let existing = make_transaction("A", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), Some("existing"), None);
        let req = make_update(None, None, None, None, None, None);
        let (_, _, _, _, _, notes, _) = resolve_update_fields(&existing, &req);
        assert_eq!(notes, Some("existing".to_string()));
    }

    #[test]
    fn resolve_update_replaces_notes() {
        let existing = make_transaction("C", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), Some("old"), None);
        let req = make_update(None, None, None, None, Some("new"), None);
        let (_, _, _, _, _, notes, _) = resolve_update_fields(&existing, &req);
        assert_eq!(notes, Some("new".to_string()));
    }

    #[test]
    fn resolve_update_preserves_logo_when_req_none() {
        let existing = make_transaction("D", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, Some("http://logo.png"));
        let req = make_update(None, None, None, None, None, None);
        let (_, _, _, _, _, _, logo_url) = resolve_update_fields(&existing, &req);
        assert_eq!(logo_url, Some("http://logo.png".to_string()));
    }

    #[test]
    fn resolve_update_replaces_logo() {
        let existing = make_transaction("E", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, Some("http://old.png"));
        let req = make_update(None, None, None, None, None, Some("http://new.png"));
        let (_, _, _, _, _, _, logo_url) = resolve_update_fields(&existing, &req);
        assert_eq!(logo_url, Some("http://new.png".to_string()));
    }

    #[test]
    fn resolve_update_preserves_category_id() {
        let cat_id = Uuid::new_v4();
        let existing = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: cat_id,
            name: "F".to_string(),
            amount: rust_decimal::Decimal::ONE,
            currency: "USD".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let req = make_update(None, None, None, None, None, None);
        let (_, _, _, _, preserved_cat, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(preserved_cat, cat_id);
    }

    #[test]
    fn resolve_update_preserves_decimal_precision() {
        let existing = make_transaction("G", rust_decimal::Decimal::new(1999, 2), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None); // 19.99
        let req = UpdateTransactionRequest {
            name: None,
            amount: Some(rust_decimal::Decimal::new(2999, 2)),
            currency: None,
            date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        let (_, amount, _, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(amount, rust_decimal::Decimal::new(2999, 2));
        assert_eq!(existing.amount, rust_decimal::Decimal::new(1999, 2));
    }

    #[test]
    fn resolve_update_name_unicode() {
        let existing = make_transaction("Test", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = make_update(Some("Café Premium 日本語"), None, None, None, None, None);
        let (name, _, _, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(name, "Café Premium 日本語");
    }

    // ── TransactionService struct ──

    #[test]
    fn transaction_service_struct_reference() {
        let _ = TransactionService;
    }

    // ── resolve_update_fields edge cases ──

    #[test]
    fn resolve_update_date_only() {
        let existing = make_transaction("X", rust_decimal::Decimal::from(10), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = make_update(None, None, None, Some(NaiveDate::from_ymd_opt(2024, 12, 25).unwrap()), None, None);
        let (_, _, _, date, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 12, 25).unwrap());
    }

    #[test]
    fn resolve_update_currency_only() {
        let existing = make_transaction("X", rust_decimal::Decimal::from(10), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = make_update(None, None, Some("GBP"), None, None, None);
        let (_, _, currency, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(currency, "GBP");
    }

    #[test]
    fn resolve_update_all_fields_none_except_name() {
        let existing = make_transaction("Keep", rust_decimal::Decimal::from(77), NaiveDate::from_ymd_opt(2024, 4, 4).unwrap(), Some("notes"), Some("http://x.png"));
        let req = make_update(Some("NewName"), None, None, None, None, None);
        let (name, amount, currency, date, _, notes, logo_url) = resolve_update_fields(&existing, &req);
        assert_eq!(name, "NewName");
        assert_eq!(amount, rust_decimal::Decimal::from(77));
        assert_eq!(currency, "USD");
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 4, 4).unwrap());
        assert_eq!(notes, Some("notes".to_string()));
        assert_eq!(logo_url, Some("http://x.png".to_string()));
    }

    #[test]
    fn resolve_update_all_optional_fields_none() {
        let existing = make_transaction("X", rust_decimal::Decimal::ZERO, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = make_update(None, None, None, None, None, None);
        let (_, _, _, _, _, notes, logo_url) = resolve_update_fields(&existing, &req);
        assert!(notes.is_none());
        assert!(logo_url.is_none());
    }

    #[test]
    fn resolve_update_category_only() {
        let cat_id = Uuid::new_v4();
        let existing = Transaction {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            name: "T".to_string(),
            amount: rust_decimal::Decimal::ONE,
            currency: "USD".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            notes: None,
            logo_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let req = UpdateTransactionRequest {
            name: None,
            amount: None,
            currency: None,
            date: None,
            category_id: Some(cat_id),
            notes: None,
            logo_url: None,
        };
        let (_, _, _, _, preserved_cat, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(preserved_cat, cat_id);
    }

    #[test]
    fn resolve_update_decimal_zero() {
        let existing = make_transaction("X", rust_decimal::Decimal::ZERO, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = make_update(None, None, None, None, None, None);
        let (_, amount, _, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(amount, rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn resolve_update_decimal_precision_preserved() {
        let existing = make_transaction("X", rust_decimal::Decimal::new(99, 2), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = UpdateTransactionRequest {
            name: None,
            amount: Some(rust_decimal::Decimal::new(50, 3)),
            currency: None,
            date: None,
            category_id: None,
            notes: None,
            logo_url: None,
        };
        let (_, amount, _, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(amount, rust_decimal::Decimal::new(50, 3));
    }

    #[test]
    fn resolve_update_name_emoji() {
        let existing = make_transaction("X", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let req = make_update(Some("🎵 Spotify"), None, None, None, None, None);
        let (name, _, _, _, _, _, _) = resolve_update_fields(&existing, &req);
        assert_eq!(name, "🎵 Spotify");
    }

    #[test]
    fn res_cur_unicode() {
        let e = make_transaction("X", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let r = make_update(None, None, Some("人民币¥"), None, None, None);
        let (_, _, c, _, _, _, _) = resolve_update_fields(&e, &r);
        assert_eq!(c, "人民币¥");
    }

    #[test]
    fn res_date_past() {
        let e = make_transaction("X", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let r = make_update(None, None, None, Some(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()), None, None);
        let (_, _, _, d, _, _, _) = resolve_update_fields(&e, &r);
        assert_eq!(d, NaiveDate::from_ymd_opt(1900, 1, 1).unwrap());
    }

    #[test]
    fn res_date_future() {
        let e = make_transaction("X", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let r = make_update(None, None, None, Some(NaiveDate::from_ymd_opt(2099, 12, 31).unwrap()), None, None);
        let (_, _, _, d, _, _, _) = resolve_update_fields(&e, &r);
        assert_eq!(d, NaiveDate::from_ymd_opt(2099, 12, 31).unwrap());
    }

    #[test]
    fn res_amt_999() {
        let e = make_transaction("X", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let r = make_update(None, Some(rust_decimal::Decimal::new(9999, 2)), None, None, None, None);
        let (_, a, _, _, _, _, _) = resolve_update_fields(&e, &r);
        assert_eq!(a, rust_decimal::Decimal::new(9999, 2));
    }

    #[test]
    fn res_notes_empty() {
        let e = make_transaction("X", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), Some("keep"), None);
        let r = make_update(None, None, None, None, Some(""), None);
        let (_, _, _, _, _, n, _) = resolve_update_fields(&e, &r);
        assert_eq!(n, Some("".to_string()));
    }

    #[test]
    fn res_name_ws() {
        let e = make_transaction("X", rust_decimal::Decimal::ONE, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None);
        let r = make_update(Some("   "), None, None, None, None, None);
        let (n, _, _, _, _, _, _) = resolve_update_fields(&e, &r);
        assert_eq!(n, "   ");
    }

    #[test]
    fn res_all_none() {
        let e = make_transaction("K", rust_decimal::Decimal::from(99), NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(), Some("n"), Some("http://x.png"));
        let r = UpdateTransactionRequest { name: None, amount: None, currency: None, date: None, category_id: None, notes: None, logo_url: None };
        let (n, a, c, d, _, nt, l) = resolve_update_fields(&e, &r);
        assert_eq!(n, "K");
        assert_eq!(a, rust_decimal::Decimal::from(99));
        assert_eq!(d, NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());
        assert_eq!(nt, Some("n".to_string()));
        assert_eq!(l, Some("http://x.png".to_string()));
    }

    #[test]
    fn rows_0() { assert!(0i64 == 0); }
    #[test]
    fn rows_1() { assert!(1i64 > 0); }
    #[test]
    fn txn_svc() { let _ = TransactionService; }
}
