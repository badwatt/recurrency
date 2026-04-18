use crate::api::client::{api_get, api_post, api_put, api_delete, ApiError};
use crate::models::category::*;

pub async fn list() -> Result<Vec<Category>, ApiError> {
    api_get("/api/categories").await
}

pub async fn create(req: &CreateCategoryRequest) -> Result<Category, ApiError> {
    api_post("/api/categories", req).await
}

pub async fn update(id: &uuid::Uuid, req: &UpdateCategoryRequest) -> Result<Category, ApiError> {
    api_put(&format!("/api/categories/{}", id), req).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/categories/{}", id)).await
}

pub async fn set_default(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_put("/api/categories/default", &serde_json::json!({ "id": id })).await
}

#[cfg(test)]
mod tests {
    use crate::models::category::*;
    use uuid::Uuid;
    use chrono::DateTime;

    #[test]
    fn category_api_paths() {
        let base = "/api/categories";
        assert!(base.starts_with("/api/"));

        let id = uuid::Uuid::new_v4();
        let detail = format!("/api/categories/{}", id);
        assert!(detail.starts_with("/api/categories/"));
    }

    #[test]
    fn set_default_path() {
        let path = "/api/categories/default";
        assert!(path.contains("default"));
    }

    #[test]
    fn set_default_request_body() {
        let id = uuid::Uuid::new_v4();
        let body = serde_json::json!({ "id": id });
        assert_eq!(body["id"], id.to_string());
    }

    #[test]
    fn category_model_fields() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Food".into(),
            icon: Some("Coffee".into()),
            color: Some("#ef4444".into()),
            is_system: false,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        assert_eq!(cat.name, "Food");
        assert_eq!(cat.icon.as_deref(), Some("Coffee"));
        assert_eq!(cat.color.as_deref(), Some("#ef4444"));
        assert!(!cat.is_system);
        assert!(!cat.is_default);
    }

    #[test]
    fn category_system() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Uncategorized".into(),
            icon: None,
            color: None,
            is_system: true,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        assert!(cat.is_system);
        assert!(cat.icon.is_none());
        assert!(cat.color.is_none());
    }

    #[test]
    fn category_default() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "General".into(),
            icon: Some("Tag".into()),
            color: Some("#6b7280".into()),
            is_system: false,
            is_default: true,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        assert!(cat.is_default);
        assert!(!cat.is_system);
    }

    #[test]
    fn create_category_request() {
        let req = CreateCategoryRequest {
            name: "Transport".into(),
            icon: Some("Car".into()),
            color: Some("#3b82f6".into()),
        };
        assert_eq!(req.name, "Transport");
        assert_eq!(req.icon.as_deref(), Some("Car"));
        assert_eq!(req.color.as_deref(), Some("#3b82f6"));
    }

    #[test]
    fn create_category_request_minimal() {
        let req = CreateCategoryRequest {
            name: "Other".into(),
            icon: None,
            color: None,
        };
        assert_eq!(req.name, "Other");
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn update_category_request_partial() {
        let req = UpdateCategoryRequest {
            name: Some("New Name".into()),
            icon: None,
            color: None,
        };
        assert_eq!(req.name.as_deref(), Some("New Name"));
        assert!(req.icon.is_none());
    }

    #[test]
    fn update_category_request_all_none() {
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
    fn category_serialization_roundtrip() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test".into(),
            icon: Some("Star".into()),
            color: Some("#ff0000".into()),
            is_system: false,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&cat).unwrap();
        let back: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Test");
    }

    #[test]
    fn create_request_serialization() {
        let req = CreateCategoryRequest {
            name: "Food".into(),
            icon: Some("Coffee".into()),
            color: Some("#ef4444".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Food"));
        assert!(json.contains("Coffee"));
    }

    #[test]
    fn category_uuid_uniqueness() {
        let ids: std::collections::HashSet<_> = (0..100).map(|_| Uuid::new_v4()).collect();
        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn update_category_name_only() {
        let req = UpdateCategoryRequest {
            name: Some("Changed".into()),
            icon: None,
            color: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Changed"));
    }

    #[test]
    fn update_category_color_only() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: None,
            color: Some("#0000ff".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("#0000ff"));
    }
}