use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::categories::models::{CreateCategoryRequest, CategoryResponse, UpdateCategoryRequest, SetDefaultRequest};
use crate::categories::service::{CategoryService, CategoryWithCounts};

use crate::shared::{AuthQuery, OptionalQuery};
use crate::shared::auth_extractor::extract_token_from_request;

pub async fn create(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateCategoryRequest>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CategoryResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let categories = CategoryService::get_all(&pool, claims.sub).await?;

    Ok(Json(categories.into_iter().map(Into::into).collect()))
}

pub async fn list_with_counts(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CategoryWithCounts>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let categories = CategoryService::get_with_counts(&pool, claims.sub).await?;

    Ok(Json(categories))
}

pub async fn get(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(category.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateCategoryRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(category.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    CategoryService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default(
    State(pool): State<PgPool>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<SetDefaultRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(query.as_ref().and_then(|q| q.token.as_ref()), &headers)?).await?;

    let category = CategoryService::set_default(&pool, claims.sub, req.id).await?;

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
    fn create_category_request_deserialize() {
        let json = "{\"name\":\"Streaming\",\"icon\":\"tv\",\"color\":\"#ff0000\"}";
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Streaming");
        assert_eq!(req.icon.as_deref(), Some("tv"));
        assert_eq!(req.color.as_deref(), Some("#ff0000"));
    }

    #[test]
    fn update_category_request_deserialize_all_fields() {
        let json = "{\"name\":\"Updated\",\"icon\":\"star\",\"color\":\"#00ff00\"}";
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
    fn update_category_icon_resolution() {
        let req = UpdateCategoryRequest {
            name: None,
            icon: serde_json::Value::String("new-icon".to_string()),
            color: serde_json::Value::Null,
        };
        let icon = resolve_optional_json_field(req.icon, Some("old".to_string()));
        assert_eq!(icon, Some("new-icon".to_string()));
    }
}
