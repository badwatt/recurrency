use axum::{
    extract::{Path, State, Query},
    http::{HeaderMap, StatusCode},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::categories::models::{CreateCategoryRequest, CategoryResponse, UpdateCategoryRequest, SetDefaultRequest};
use crate::categories::service::{CategoryService, CategoryWithCounts};

use crate::shared::AuthQuery;
use crate::shared::auth_extractor::extract_token_from_request;

pub async fn create(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateCategoryRequest>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let claims = AuthService::verify_token(&extract_token_from_request(auth.token.as_ref(), &headers)?).await?;

    let category = CategoryService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CategoryResponse>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(auth.token.as_ref(), &headers)?).await?;

    let categories = CategoryService::get_all(&pool, claims.sub).await?;

    Ok(Json(categories.into_iter().map(Into::into).collect()))
}

pub async fn list_with_counts(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<CategoryWithCounts>>> {
    let claims = AuthService::verify_token(&extract_token_from_request(auth.token.as_ref(), &headers)?).await?;

    let categories = CategoryService::get_with_counts(&pool, claims.sub).await?;

    Ok(Json(categories))
}

pub async fn get(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(auth.token.as_ref(), &headers)?).await?;

    let category = CategoryService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(category.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateCategoryRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(auth.token.as_ref(), &headers)?).await?;

    let category = CategoryService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(category.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&extract_token_from_request(auth.token.as_ref(), &headers)?).await?;

    CategoryService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<SetDefaultRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&extract_token_from_request(auth.token.as_ref(), &headers)?).await?;

    let category = CategoryService::set_default(&pool, claims.sub, req.id).await?;

    Ok(Json(category.into()))
}
