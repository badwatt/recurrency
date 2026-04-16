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

    #[test]
    fn category_with_icon_and_color() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Entertainment".into(),
            icon: Some("film".into()),
            color: Some("#8b5cf6".into()),
            is_system: true,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        assert!(cat.is_system);
        assert!(!cat.is_default);
        assert_eq!(cat.icon.as_deref(), Some("film"));
        assert_eq!(cat.color.as_deref(), Some("#8b5cf6"));
    }

    #[test]
    fn update_category_request_default() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: None,
            color: None,
        };
        assert!(req.name.is_none());
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn update_category_request_partial() {
        let req = UpdateCategoryRequest {
            name: Some("New Name".into()),
            icon: None,
            color: Some("#ff0000".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: UpdateCategoryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, Some("New Name".into()));
        assert!(back.icon.is_none());
        assert_eq!(back.color, Some("#ff0000".into()));
    }

    #[test]
    fn create_category_with_all_fields() {
        let req = CreateCategoryRequest {
            name: "Travel".into(),
            icon: Some("plane".into()),
            color: Some("#00ff00".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: CreateCategoryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }
}