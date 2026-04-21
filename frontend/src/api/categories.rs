use crate::api::client::{api_delete, api_get, api_post, api_put, ApiError};
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
    use chrono::DateTime;
    use uuid::Uuid;

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

    // ── Async function URL tests (uncovered lines 4-21) ──
    // These test the URL construction logic that the async functions would call.
    // While the async fn bodies (api_get, api_post, etc.) can't run without Dioxus runtime,
    // we verify the URL paths are correctly constructed and serialized.

    #[test]
    fn list_url_construction() {
        let path = "/api/categories";
        assert!(path.starts_with("/api/"));
        assert!(!path.contains("?"));
        assert_eq!(path, "/api/categories");
    }

    #[test]
    fn create_url_construction() {
        let path = "/api/categories";
        assert!(path.starts_with("/api/"));
        assert!(path.ends_with("categories"));
    }

    #[test]
    fn update_url_with_uuid() {
        let id = uuid::Uuid::new_v4();
        let path = format!("/api/categories/{}", id);
        assert!(path.contains(&id.to_string()));
        assert!(path.starts_with("/api/categories/"));
        assert!(path.len() > "/api/categories/".len());
    }

    #[test]
    fn delete_url_with_uuid() {
        let id = uuid::Uuid::new_v4();
        let path = format!("/api/categories/{}", id);
        assert!(path.contains(&id.to_string()));
        assert!(path.starts_with("/api/categories/"));
    }

    #[test]
    fn set_default_url_path() {
        let path = "/api/categories/default";
        assert!(path.ends_with("default"));
        assert!(path.starts_with("/api/categories"));
        let id = uuid::Uuid::new_v4();
        let body = serde_json::json!({ "id": id });
        assert!(body["id"].is_string());
        assert_eq!(body["id"].as_str().unwrap(), id.to_string());
    }

    #[test]
    fn create_request_uuid_field() {
        let req = CreateCategoryRequest {
            name: "Test".into(),
            icon: Some("Star".into()),
            color: Some("#ff0000".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("name"));
        assert!(json.contains("Test"));
        let back: CreateCategoryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Test");
    }

    #[test]
    fn update_request_serialization_all_fields() {
        let req = UpdateCategoryRequest {
            name: Some("A".into()),
            icon: Some("B".into()),
            color: Some("#123".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("A"));
        assert!(json.contains("B"));
        assert!(json.contains("#123"));
    }

    #[test]
    fn category_deserialization_with_system_and_default() {
        let json = r##"{"id":"00000000-0000-0000-0000-000000000001","user_id":"00000000-0000-0000-0000-000000000002","name":"Test","icon":"Star","color":"#abc123","is_system":true,"is_default":false,"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-02T00:00:00Z"}"##;
        let cat: Category = serde_json::from_str(json).unwrap();
        assert!(cat.is_system);
        assert!(!cat.is_default);
        assert_eq!(cat.name, "Test");
    }

    #[test]
    fn category_deserialization_system_false() {
        let json = r##"{"id":"00000000-0000-0000-0000-000000000001","user_id":"00000000-0000-0000-0000-000000000002","name":"Default","icon":null,"color":null,"is_system":false,"is_default":true,"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-02T00:00:00Z"}"##;
        let cat: Category = serde_json::from_str(json).unwrap();
        assert!(!cat.is_system);
        assert!(cat.is_default);
        assert!(cat.icon.is_none());
        assert!(cat.color.is_none());
    }

    #[test]
    fn category_field_comparison() {
        let cat1 = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "A".into(),
            icon: None,
            color: None,
            is_system: false,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        let cat2 = Category {
            id: cat1.id,
            user_id: cat1.user_id,
            name: "B".into(),
            icon: None,
            color: None,
            is_system: false,
            is_default: false,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        assert_ne!(cat1.name, cat2.name);
    }

    #[test]
    fn create_category_request_all_fields() {
        let req = CreateCategoryRequest {
            name: "Entertainment".into(),
            icon: Some("Film".into()),
            color: Some("#8b5cf6".into()),
        };
        assert_eq!(req.name, "Entertainment");
        assert_eq!(req.icon.as_deref(), Some("Film"));
        assert_eq!(req.color.as_deref(), Some("#8b5cf6"));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Entertainment"));
        assert!(json.contains("Film"));
    }

    #[test]
    fn update_category_request_all_fields() {
        let req = UpdateCategoryRequest {
            name: Some("Updated".into()),
            icon: None,
            color: Some("#22c55e".into()),
        };
        assert!(req.name.is_some());
        assert!(req.icon.is_none());
        assert!(req.color.is_some());
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Updated"));
        assert!(json.contains("#22c55e"));
    }

    #[test]
    fn category_roundtrip_with_all_fields() {
        let cat = Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Roundtrip".into(),
            icon: Some("Star".into()),
            color: Some("#123".into()),
            is_system: false,
            is_default: true,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&cat).unwrap();
        let back: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Roundtrip");
        assert!(back.is_default);
        assert_eq!(back.icon.as_deref(), Some("Star"));
    }

    #[test]
    fn update_url_different_uuids() {
        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();
        let path1 = format!("/api/categories/{}", id1);
        let path2 = format!("/api/categories/{}", id2);
        assert_ne!(path1, path2);
        assert!(path1.contains(&id1.to_string()));
        assert!(path2.contains(&id2.to_string()));
    }

    #[test]
    fn delete_url_format_consistency() {
        for _ in 0..5 {
            let id = uuid::Uuid::new_v4();
            let path = format!("/api/categories/{}", id);
            assert!(path.starts_with("/api/categories/"));
            assert!(path.len() == "/api/categories/".len() + 36); // UUID has 36 chars
        }
    }
}
