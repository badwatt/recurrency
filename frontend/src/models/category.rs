use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_serialization_roundtrip() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Entertainment".into(),
            icon: Some("film".into()),
            color: Some("#8b5cf6".into()),
            is_system: false,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&cat).unwrap();
        let back: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, back);
    }

    #[test]
    fn create_category_minimal() {
        let req = CreateCategoryRequest {
            name: "Food".into(),
            icon: None,
            color: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Food"));
    }
}