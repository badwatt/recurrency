use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::service::AuthService;
use crate::shared::error::AppResult;
use crate::categories::models::{CreateCategoryRequest, CategoryResponse, UpdateCategoryRequest, SetDefaultRequest};
use crate::categories::service::{CategoryService, CategoryWithCounts};

#[derive(serde::Deserialize)]
pub struct AuthQuery {
    pub token: String,
}

pub async fn create(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Json(req): Json<CreateCategoryRequest>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let category = CategoryService::create(&pool, claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn list(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
) -> AppResult<Json<Vec<CategoryResponse>>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let categories = CategoryService::get_all(&pool, claims.sub).await?;

    Ok(Json(categories.into_iter().map(Into::into).collect()))
}

pub async fn list_with_counts(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
) -> AppResult<Json<Vec<CategoryWithCounts>>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let categories = CategoryService::get_with_counts(&pool, claims.sub).await?;

    Ok(Json(categories))
}

pub async fn get(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let category = CategoryService::get_by_id(&pool, claims.sub, id).await?;

    Ok(Json(category.into()))
}

pub async fn update(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCategoryRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let category = CategoryService::update(&pool, claims.sub, id, req).await?;

    Ok(Json(category.into()))
}

pub async fn delete(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let claims = AuthService::verify_token(&auth.token).await?;

    CategoryService::delete(&pool, claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default(
    State(pool): State<PgPool>,
    Query(auth): Query<AuthQuery>,
    Json(req): Json<SetDefaultRequest>,
) -> AppResult<Json<CategoryResponse>> {
    let claims = AuthService::verify_token(&auth.token).await?;

    let category = CategoryService::set_default(&pool, claims.sub, req.id).await?;

    Ok(Json(category.into()))
}
