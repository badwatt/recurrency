use axum::{Json, extract::State, http::HeaderMap};

use crate::admin::models::{AdminSettings, AdminUser, FullBackup, UpdateAdminSettingsRequest};
use crate::auth::models::Claims;
use crate::shared::OptionalQuery;
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::{AppError, AppResult};

#[derive(serde::Deserialize)]
pub struct AdminQuery {
    pub token: Option<String>,
}

async fn require_admin(
    state: &AppState,
    query: Option<&AdminQuery>,
    headers: &HeaderMap,
) -> AppResult<Claims> {
    let token = extract_token_from_request(query.and_then(|q| q.token.as_ref()), headers)?;
    let claims = state.auth_service.verify_token(&token).await?;
    if !state.admin_service.is_admin(claims.sub).await? {
        return Err(AppError::Forbidden(
            "Administrator access required".to_string(),
        ));
    }
    Ok(claims)
}

pub async fn list_users(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AdminQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<AdminUser>>> {
    require_admin(&state, query.as_ref(), &headers).await?;
    Ok(Json(state.admin_service.list_users().await?))
}

pub async fn get_settings(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AdminQuery>,
    headers: HeaderMap,
) -> AppResult<Json<AdminSettings>> {
    require_admin(&state, query.as_ref(), &headers).await?;
    Ok(Json(state.admin_service.settings().await?))
}

pub async fn update_settings(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AdminQuery>,
    headers: HeaderMap,
    Json(req): Json<UpdateAdminSettingsRequest>,
) -> AppResult<Json<AdminSettings>> {
    require_admin(&state, query.as_ref(), &headers).await?;
    Ok(Json(
        state
            .admin_service
            .set_registration_enabled(req.registration_enabled)
            .await?,
    ))
}

pub async fn full_backup(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AdminQuery>,
    headers: HeaderMap,
) -> AppResult<Json<FullBackup>> {
    require_admin(&state, query.as_ref(), &headers).await?;
    Ok(Json(state.admin_service.full_backup().await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_query_deserializes_token() {
        let query: AdminQuery = serde_urlencoded::from_str("token=admin-token").unwrap();
        assert_eq!(query.token.as_deref(), Some("admin-token"));
    }

    #[test]
    fn update_settings_deserializes_registration_flag() {
        let req: UpdateAdminSettingsRequest =
            serde_json::from_str(r#"{"registration_enabled":false}"#).unwrap();
        assert!(!req.registration_enabled);
    }
}
