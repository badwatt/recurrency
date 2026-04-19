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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_to_response_preserves_fields() {
        let uid = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let cat = Category {
            id: uid,
            user_id,
            name: "Streaming".to_string(),
            icon: Some("tv".to_string()),
            color: Some("#ff0000".to_string()),
            is_system: false,
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = CategoryResponse::from(cat);
        assert_eq!(resp.id, uid);
        assert_eq!(resp.name, "Streaming");
        assert_eq!(resp.icon, Some("tv".to_string()));
        assert_eq!(resp.color, Some("#ff0000".to_string()));
        assert!(resp.is_default);
        assert!(!resp.is_system);
    }

    #[test]
    fn create_category_request_fields() {
        let req = CreateCategoryRequest {
            name: "Food".to_string(),
            icon: Some("utensils".to_string()),
            color: Some("#00ff00".to_string()),
        };
        assert_eq!(req.name, "Food");
        assert_eq!(req.icon.as_deref(), Some("utensils"));
    }

    #[test]
    fn update_category_request_destructure() {
        let req = UpdateCategoryRequest {
            name: Some("Updated".to_string()),
            icon: serde_json::Value::String("icon".to_string()),
            color: serde_json::Value::Null,
        };
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert!(req.icon.is_string());
        assert!(req.color.is_null());
    }

    #[test]
    fn update_category_request_default_values() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: serde_json::Value::Null,   // serde default
            color: serde_json::Value::Null,  // serde default
        };
        assert!(req.name.is_none());
        assert!(req.icon.is_null());
        assert!(req.color.is_null());
    }

    #[test]
    fn set_default_request() {
        let id = Uuid::new_v4();
        let req = SetDefaultRequest { id };
        assert_eq!(req.id, id);
    }
}
