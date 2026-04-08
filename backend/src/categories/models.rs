use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    #[serde(default)]
    pub icon: serde_json::Value,
    #[serde(default)]
    pub color: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SetDefaultRequest {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            user_id: c.user_id,
            name: c.name,
            icon: c.icon,
            color: c.color,
            is_system: c.is_system,
            is_default: c.is_default,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}
