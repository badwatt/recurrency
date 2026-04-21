use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::shared::app_state::AppState;
use crate::shared::error::AppResult;
use crate::categories::models::{CreateCategoryRequest, CategoryResponse, UpdateCategoryRequest, SetDefaultRequest};
use crate::categories::service::{CategoryService, CategoryWithCounts};

use crate::shared::{AuthQuery, OptionalQuery};
use crate::shared::auth_extractor::extract_token_from_request;

pub async fn create(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateCategoryRequest>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::create(&state.pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn list(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CategoryResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let categories = CategoryService::get_all(&state.pool, claims.sub).await?;

    Ok(Json(categories.into_iter().map(Into::into).collect()))
}

pub async fn list_with_counts(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CategoryWithCounts>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let categories = CategoryService::get_with_counts(&state.pool, claims.sub).await?;

    Ok(Json(categories))
}

pub async fn get(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::get_by_id(&state.pool, claims.sub, id).await?;

    Ok(Json(category.into()))
}

pub async fn update(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateCategoryRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::update(&state.pool, claims.sub, id, req).await?;

    Ok(Json(category.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    CategoryService::delete(&state.pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<SetDefaultRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::set_default(&state.pool, claims.sub, req.id).await?;

    Ok(Json(category.into()))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::categories::service::resolve_optional_json_field;

    #[test]
    fn set_default_request_deserialize() {
        let json = r#"{"id":"550e8400-e29b-41d4-a716-446655440000"}"#;
        let req: SetDefaultRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id, Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
    }

    #[test]
    fn set_default_request_invalid_uuid() {
        let json = r#"{"id":"not-a-uuid"}"#;
        let result: Result<SetDefaultRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn create_category_request_deserialize() {
        let json = "{\"name\":\"Streaming\",\"icon\":\"tv\",\"color\":\"#ff0000\"}";
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Streaming");
        assert_eq!(req.icon.as_deref(), Some("tv"));
        assert_eq!(req.color.as_deref(), Some("#ff0000"));
    }

    #[test]
    fn create_category_request_minimal() {
        let json = r#"{"name":"Utilities"}"#;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Utilities");
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn create_category_request_empty_name() {
        let json = r#"{"name":""}"#;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "");
    }

    #[test]
    fn create_category_request_unicode() {
        let json = "{\"name\":\"娱乐\",\"icon\":\"🎮\",\"color\":\"#中文\"}";
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "娱乐");
        assert_eq!(req.icon.as_deref(), Some("🎮"));
    }

    #[test]
    fn create_category_request_empty_icon_color() {
        let json = r#"{"name":"Test","icon":"","color":""}"#;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.icon.as_deref(), Some(""));
        assert_eq!(req.color.as_deref(), Some(""));
    }

    #[test]
    fn update_category_request_deserialize_all_fields() {
        let json = "{\"name\":\"Updated\",\"icon\":\"star\",\"color\":\"#00FF00\"}";
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Updated"));
        assert!(req.icon.is_string());
        assert!(req.color.is_string());
    }

    #[test]
    fn update_category_request_deserialize_null_fields() {
        let json = r#"{"name":null,"icon":null,"color":null}"#;
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.icon.is_null());
        assert!(req.color.is_null());
    }

    #[test]
    fn update_category_request_empty_json() {
        let json = r#"{}"#;
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.icon.is_null());
        assert!(req.color.is_null());
    }

    #[test]
    fn update_category_request_name_only() {
        let json = r#"{"name":"NewName"}"#;
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("NewName"));
        assert!(req.icon.is_null());
    }

    #[test]
    fn update_category_icon_resolution() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: serde_json::Value::String("new-icon".to_string()),
            color: serde_json::Value::Null,
        };
        let icon = resolve_optional_json_field(req.icon, Some("old".to_string()));
        assert_eq!(icon, Some("new-icon".to_string()));
    }

    #[test]
    fn update_category_icon_null_fallback() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: serde_json::Value::Null,
            color: serde_json::Value::Null,
        };
        let icon = resolve_optional_json_field(req.icon, Some("old".to_string()));
        assert_eq!(icon, None);
    }

    #[test]
    fn update_category_icon_not_null_fallback() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: serde_json::Value::Null,
            color: serde_json::Value::Null,
        };
        let icon = resolve_optional_json_field(req.icon, None);
        assert_eq!(icon, None);
    }

    #[test]
    fn update_category_color_resolution() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: serde_json::Value::Null,
            color: serde_json::Value::String("#FF00FF".to_string()),
        };
        let color = resolve_optional_json_field(req.color, Some("#000000".to_string()));
        assert_eq!(color, Some("#FF00FF".to_string()));
    }

    #[test]
    fn category_response_serialization() {
        let resp = CategoryResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test Category".to_string(),
            icon: Some("💰".to_string()),
            color: Some("#123456".to_string()),
            is_system: false,
            is_default: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Test Category"));
        assert!(json.contains("is_default"));
    }

    #[test]
    fn category_response_without_optional_fields() {
        let resp = CategoryResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Basic".to_string(),
            icon: None,
            color: None,
            is_system: false,
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Basic"));
    }

    #[test]
    fn category_with_counts_serialization() {
        let resp = CategoryWithCounts {
            category: CategoryResponse {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                name: "Counts Test".to_string(),
                icon: Some("📊".to_string()),
                color: Some("#ABCDEF".to_string()),
                is_system: false,
                is_default: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            subscription_count: 5,
            transaction_count: 10,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("subscription_count"));
        assert!(json.contains("transaction_count"));
        assert!(json.contains("Counts Test"));
    }

    #[test]
    fn set_default_request_zero_count() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000000"}"#;
        let req: SetDefaultRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id, Uuid::nil());
    }

    #[test]
    fn create_category_request_long_name() {
        let long_name = "A".repeat(100);
        let json = serde_json::json!({"name": long_name}).to_string();
        let req: CreateCategoryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.name.len(), 100);
    }

    #[test]
    fn create_category_request_with_icon() {
        let json = r##"{"name":"Test","icon":"Star","color":"#FF0000"}"##;
        let req: CreateCategoryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.icon.as_deref(), Some("Star"));
    }

    #[test]
    fn create_category_request_without_optional() {
        let json = r#"{"name":"Basic"}"#;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Basic");
        assert!(req.icon.is_none());
        assert!(req.color.is_none());
    }

    #[test]
    fn update_category_request_all_null() {
        let json = r#"{}"#;
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.icon.is_null());
        assert!(req.color.is_null());
    }

    #[test]
    fn category_response_system_flag() {
        let resp = CategoryResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "System".to_string(),
            icon: Some("Tag".to_string()),
            color: Some("#000000".to_string()),
            is_system: true,
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("is_system"));
    }

    #[test]
    fn category_response_default_flag() {
        let resp = CategoryResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Default".to_string(),
            icon: None,
            color: None,
            is_system: false,
            is_default: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("is_default"));
    }

    #[test]
    fn category_with_counts_zero() {
        let resp = CategoryWithCounts {
            category: CategoryResponse {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                name: "Empty".to_string(),
                icon: None,
                color: None,
                is_system: false,
                is_default: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            subscription_count: 0,
            transaction_count: 0,
        };
        assert_eq!(resp.subscription_count, 0);
        assert_eq!(resp.transaction_count, 0);
    }

    #[test]
    fn category_with_counts_large_numbers() {
        let resp = CategoryWithCounts {
            category: CategoryResponse {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                name: "Popular".to_string(),
                icon: Some("Star".to_string()),
                color: Some("#FFD700".to_string()),
                is_system: false,
                is_default: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            subscription_count: 999999,
            transaction_count: 999999,
        };
        assert_eq!(resp.subscription_count, 999999);
        assert_eq!(resp.transaction_count, 999999);
    }

    #[test]
    fn set_default_request_uuid() {
        let id = Uuid::new_v4();
        let json = serde_json::json!({"id": id.to_string()}).to_string();
        let req: SetDefaultRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.id, id);
    }

    // ── Additional model edge case tests ──
    #[test]
    fn create_category_request_name_with_special_chars() {
        let json = r##"{"name":"Home & Garden 🚗","icon":"Home","color":"#00FF00"}"##;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Home & Garden 🚗");
    }

    #[test]
    fn create_category_request_unicode_color() {
        let json = r##"{"name":"Neon","icon":"Star","color":"#FF00FF"}"##;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.color.as_deref(), Some("#FF00FF"));
    }

    #[test]
    fn update_category_request_partial_name() {
        let json = r#"{"name":"New Name"}"#;
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("New Name"));
        assert!(req.icon.is_null());
        assert!(req.color.is_null());
    }

    #[test]
    fn update_category_request_partial_icon() {
        let json = r#"{"icon":"Star"}"#;
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(!req.icon.is_null());
        assert!(req.color.is_null());
    }

    #[test]
    fn update_category_request_partial_color() {
        let json = r##"{"color":"#ABCDEF"}"##;
        let req: UpdateCategoryRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.icon.is_null());
        assert!(!req.color.is_null());
    }

    #[test]
    fn category_response_system_and_default_both_true() {
        let resp = CategoryResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Special".to_string(),
            icon: Some("Star".to_string()),
            color: Some("#123456".to_string()),
            is_system: true,
            is_default: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(resp.is_system);
        assert!(resp.is_default);
    }

    #[test]
    fn category_with_counts_mixed() {
        let resp = CategoryWithCounts {
            category: CategoryResponse {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                name: "Mixed".to_string(),
                icon: None,
                color: None,
                is_system: false,
                is_default: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            subscription_count: 100,
            transaction_count: 200,
        };
        assert_eq!(resp.subscription_count + resp.transaction_count, 300);
    }

    #[test]
    fn set_default_request_invalid_uuid_edge() {
        let json = r#"{"id":"not-a-uuid"}"#;
        let result: Result<SetDefaultRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn set_default_request_empty_uuid_edge() {
        let json = r#"{"id":""}"#;
        let result: Result<SetDefaultRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn category_response_serialize_preserves_all_fields() {
        let cat = CategoryResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test".to_string(),
            icon: Some("Star".to_string()),
            color: Some("#ABCDEF".to_string()),
            is_system: true,
            is_default: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"user_id\""));
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"icon\""));
        assert!(json.contains("\"color\""));
        assert!(json.contains("\"is_system\""));
        assert!(json.contains("\"is_default\""));
    }
}
