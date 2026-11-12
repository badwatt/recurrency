use axum::{Json, extract::State, http::HeaderMap};

use crate::backup::models::{ImportResult, UserBackup};
use crate::shared::OptionalQuery;
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;

#[derive(serde::Deserialize)]
pub struct BackupQuery {
    pub token: Option<String>,
}

pub async fn export(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<BackupQuery>,
    headers: HeaderMap,
) -> AppResult<Json<UserBackup>> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;
    Ok(Json(state.backup_service.export(claims.sub).await?))
}

pub async fn import(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<BackupQuery>,
    headers: HeaderMap,
    Json(backup): Json<UserBackup>,
) -> AppResult<Json<ImportResult>> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;
    Ok(Json(
        state.backup_service.import(claims.sub, backup).await?,
    ))
}
