use axum::{
    extract::Query,
    response::Json,
};
use serde::Deserialize;

use crate::logos::service;
use crate::shared::error::AppResult;

#[derive(Debug, Deserialize)]
pub struct LogoSearchQuery {
    pub q: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LogoSearchResponse {
    pub results: Vec<service::LogoResult>,
}

pub async fn search(
    Query(params): Query<LogoSearchQuery>,
) -> AppResult<Json<LogoSearchResponse>> {
    let results = service::search_logos(&params.q).await?;
    
    Ok(Json(LogoSearchResponse { results }))
}
