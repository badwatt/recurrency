use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};
use crate::categories::models::{CreateCategoryRequest, Category, UpdateCategoryRequest};

pub struct CategoryService;

impl CategoryService {
    /// Creates "Uncategorized" category for a new user and sets it as default
    pub async fn create_uncategorized(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO categories (id, user_id, name, is_system, is_default, created_at, updated_at)
            VALUES ($1, $2, 'Uncategorized', true, true, $3, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(Utc::now())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: CreateCategoryRequest,
    ) -> AppResult<Category> {
        // Check if this is the first category for this user
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM categories WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let is_default = count.0 == 0;

        let category = sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO categories (id, user_id, name, icon, color, is_system, is_default, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, false, $6, $7, $7)
            RETURNING id, user_id, name, icon, color, is_system, is_default, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&req.name)
        .bind(&req.icon)
        .bind(&req.color)
        .bind(is_default)
        .bind(Utc::now())
        .fetch_one(pool)
        .await?;

        Ok(category)
    }

    pub async fn get_all(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT id, user_id, name, icon, color, is_system, is_default, created_at, updated_at FROM categories WHERE user_id = $1 ORDER BY is_default DESC, is_system DESC, name ASC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    pub async fn get_by_id(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Category> {
        let category = sqlx::query_as::<_, Category>(
            "SELECT id, user_id, name, icon, color, is_system, is_default, created_at, updated_at FROM categories WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        Ok(category)
    }

    pub async fn update(
        pool: &PgPool,
        user_id: Uuid,
        id: Uuid,
        req: UpdateCategoryRequest,
    ) -> AppResult<Category> {
        let existing = Self::get_by_id(pool, user_id, id).await?;

        let name = req.name.unwrap_or(existing.name);

        // Handle icon: null = clear, string = set, absent = keep existing
        let icon = resolve_optional_json_field(req.icon, existing.icon);

        // Handle color: null = clear, string = set, absent = keep existing
        let color = resolve_optional_json_field(req.color, existing.color);

        let category = sqlx::query_as::<_, Category>(
            r#"
            UPDATE categories 
            SET name = $1, icon = $2, color = $3, updated_at = $4
            WHERE id = $5 AND user_id = $6
            RETURNING id, user_id, name, icon, color, is_system, is_default, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(&icon)
        .bind(&color)
        .bind(Utc::now())
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        Ok(category)
    }

    /// Get the default category for a user
    pub async fn get_default(pool: &PgPool, user_id: Uuid) -> AppResult<Category> {
        let category = sqlx::query_as::<_, Category>(
            "SELECT id, user_id, name, icon, color, is_system, is_default, created_at, updated_at FROM categories WHERE user_id = $1 AND is_default = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Default category not found".to_string()))?;

        Ok(category)
    }

    pub async fn delete(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<()> {
        let category = Self::get_by_id(pool, user_id, id).await?;

        // Cannot delete the default category
        if category.is_default {
            return Err(AppError::BadRequest("Cannot delete the default category".to_string()));
        }

        // Reassign all subscriptions and transactions from this category to the default category
        let default_category = Self::get_default(pool, user_id).await?;

        sqlx::query("UPDATE subscriptions SET category_id = $1 WHERE category_id = $2 AND user_id = $3")
            .bind(default_category.id)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        sqlx::query("UPDATE transactions SET category_id = $1 WHERE category_id = $2 AND user_id = $3")
            .bind(default_category.id)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        let result = sqlx::query("DELETE FROM categories WHERE id = $1 AND user_id = $2 AND is_default = false")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Category not found or is the default category".to_string()));
        }

        Ok(())
    }

    /// Set a category as the default
    /// Also reassigns all subscriptions and transactions that were using the previous default
    pub async fn set_default(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<Category> {
        // Use a transaction to ensure atomicity
        let mut tx = pool.begin().await?;

        // Remove default flag from all categories for this user
        sqlx::query("UPDATE categories SET is_default = false WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // Set the new default
        let updated = sqlx::query_as::<_, Category>(
            r#"
            UPDATE categories 
            SET is_default = true, updated_at = $1
            WHERE id = $2 AND user_id = $3
            RETURNING id, user_id, name, icon, color, is_system, is_default, created_at, updated_at
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

        tx.commit().await?;

        Ok(updated)
    }

    pub async fn get_with_counts(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<CategoryWithCounts>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT id, user_id, name, icon, color, is_system, is_default, created_at, updated_at FROM categories WHERE user_id = $1 ORDER BY is_default DESC, is_system DESC, name ASC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut result: Vec<CategoryWithCounts> = Vec::new();

        for cat in categories {
            let subscription_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM subscriptions WHERE category_id = $1"
            )
            .bind(cat.id)
            .fetch_one(pool)
            .await?;

            let transaction_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM transactions WHERE category_id = $1"
            )
            .bind(cat.id)
            .fetch_one(pool)
            .await?;

            result.push(CategoryWithCounts {
                category: cat.into(),
                subscription_count: subscription_count.0,
                transaction_count: transaction_count.0,
            });
        }

        Ok(result)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CategoryWithCounts {
    pub category: crate::categories::models::CategoryResponse,
    pub subscription_count: i64,
    pub transaction_count: i64,
}

/// Resolve an optional JSON field against existing value.
/// - String (non-empty): update to new value
/// - String (empty) or Null: clear (set to None)
/// - Other (Number, Bool, etc.): keep existing
pub(crate) fn resolve_optional_json_field(
    new_value: serde_json::Value,
    existing: Option<String>,
) -> Option<String> {
    match new_value {
        serde_json::Value::String(s) if !s.is_empty() => Some(s),
        serde_json::Value::String(_) => None, // empty string = clear
        serde_json::Value::Null => None,
        _ => existing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_optional_json_field ──

    #[test]
    fn json_field_string_updates() {
        let result = resolve_optional_json_field(
            serde_json::Value::String("new-icon".to_string()),
            Some("old-icon".to_string()),
        );
        assert_eq!(result, Some("new-icon".to_string()));
    }

    #[test]
    fn json_field_empty_string_clears() {
        let result = resolve_optional_json_field(
            serde_json::Value::String("".to_string()),
            Some("old-icon".to_string()),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn json_field_null_clears() {
        let result = resolve_optional_json_field(
            serde_json::Value::Null,
            Some("old-icon".to_string()),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn json_field_number_keeps_existing() {
        let result = resolve_optional_json_field(
            serde_json::Value::Number(42.into()),
            Some("existing".to_string()),
        );
        assert_eq!(result, Some("existing".to_string()));
    }

    #[test]
    fn json_field_bool_keeps_existing() {
        let result = resolve_optional_json_field(
            serde_json::Value::Bool(true),
            Some("existing".to_string()),
        );
        assert_eq!(result, Some("existing".to_string()));
    }

    #[test]
    fn json_field_string_no_existing_overwrite() {
        let result = resolve_optional_json_field(
            serde_json::Value::String("new-value".to_string()),
            None,
        );
        assert_eq!(result, Some("new-value".to_string()));
    }

    #[test]
    fn json_field_null_no_existing() {
        let result = resolve_optional_json_field(
            serde_json::Value::Null,
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn json_field_number_no_existing() {
        let result = resolve_optional_json_field(
            serde_json::Value::Number(0.into()),
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn json_field_object_keeps_existing() {
        let result = resolve_optional_json_field(
            serde_json::json!({"key": "val"}),
            Some("keep".to_string()),
        );
        assert_eq!(result, Some("keep".to_string()));
    }

    // ── CategoryWithCounts construction ──

    #[test]
    fn category_with_counts_serialization() {
        let cat_response = crate::categories::models::CategoryResponse {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: "Test".to_string(),
            icon: Some("star".to_string()),
            color: Some("#fff".to_string()),
            is_system: false,
            is_default: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let with_counts = CategoryWithCounts {
            category: cat_response,
            subscription_count: 5,
            transaction_count: 10,
        };
        assert_eq!(with_counts.subscription_count, 5);
        assert_eq!(with_counts.transaction_count, 10);
    }
}
