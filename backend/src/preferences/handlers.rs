use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;

use crate::auth::service::AuthService;
use crate::preferences::models::{PreferencesResponse, UpdatePreferencesRequest};
use crate::preferences::service::PreferencesService;
use crate::shared::error::AppResult;

#[derive(serde::Deserialize)]
pub struct PreferencesQuery {
    pub token: String,
}

pub async fn get_preferences(
    State(pool): State<PgPool>,
    Query(query): Query<PreferencesQuery>,
) -> AppResult<Json<PreferencesResponse>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let preferences = PreferencesService::get_preferences(&pool, claims.sub).await?;
    Ok(Json(preferences))
}

pub async fn update_preferences(
    State(pool): State<PgPool>,
    Query(query): Query<PreferencesQuery>,
    Json(req): Json<UpdatePreferencesRequest>,
) -> AppResult<Json<PreferencesResponse>> {
    let claims = AuthService::verify_token(&query.token).await?;
    let preferences = PreferencesService::update_preferences(&pool, claims.sub, req).await?;
    Ok(Json(preferences))
}
