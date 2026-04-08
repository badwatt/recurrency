use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};
use crate::transactions::models::{CreateTransactionRequest, Transaction, UpdateTransactionRequest};

pub struct TransactionService;

impl TransactionService {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateTransactionRequest,
    ) -> AppResult<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (id, user_id, category_id, name, amount, currency, date, notes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
            RETURNING id, user_id, category_id, name, amount, currency, date, notes, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(req.category_id)
        .bind(&req.name)
        .bind(req.amount)
        .bind(&req.currency)
        .bind(req.date)
        .bind(&req.notes)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(transaction)
    }

    pub async fn get_all(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            "SELECT id, user_id, category_id, name, amount, currency, date, notes, created_at, updated_at 
             FROM transactions WHERE user_id = $1 ORDER BY date DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(transactions)
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(
            "SELECT id, user_id, category_id, name, amount, currency, date, notes, created_at, updated_at 
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

        let name = req.name.unwrap_or(existing.name);
        let amount = req.amount.unwrap_or(existing.amount);
        let currency = req.currency.unwrap_or(existing.currency);
        let date = req.date.unwrap_or(existing.date);
        let category_id = req.category_id.or(existing.category_id);
        let notes = req.notes.or(existing.notes);

        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions 
            SET name = $1, amount = $2, currency = $3, date = $4, category_id = $5, notes = $6, updated_at = $7
            WHERE id = $8 AND user_id = $9
            RETURNING id, user_id, category_id, name, amount, currency, date, notes, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(amount)
        .bind(&currency)
        .bind(date)
        .bind(category_id)
        .bind(&notes)
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
        from_date: chrono::NaiveDate,
        to_date: chrono::NaiveDate,
    ) -> AppResult<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT id, user_id, category_id, name, amount, currency, date, notes, created_at, updated_at 
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
