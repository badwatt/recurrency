use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use uuid::Uuid;

use crate::accounts::models::{
    AccountResponse, AccountWithCounts, CreateAccountRequest, SetDefaultAccountRequest,
    UpdateAccountRequest,
};
use crate::shared::app_state::AppState;
use crate::shared::auth_extractor::extract_token_from_request;
use crate::shared::error::AppResult;
use crate::shared::{AuthQuery, OptionalQuery};

pub async fn create(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateAccountRequest>,
) -> AppResult<(StatusCode, Json<AccountResponse>)> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;

    let account = state.account_service.create(claims.sub, req).await?;

    Ok((StatusCode::CREATED, Json(account.into())))
}

pub async fn list(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<AccountResponse>>> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;

    let accounts = state.account_service.get_all(claims.sub).await?;

    Ok(Json(accounts.into_iter().map(Into::into).collect()))
}

pub async fn list_with_counts(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
) -> AppResult<Json<Vec<AccountWithCounts>>> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;

    let accounts = state.account_service.get_with_counts(claims.sub).await?;

    Ok(Json(accounts))
}

pub async fn get(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<AccountResponse>> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;

    let account = state.account_service.get_by_id(claims.sub, id).await?;

    Ok(Json(account.into()))
}

pub async fn update(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdateAccountRequest>,
) -> AppResult<Json<AccountResponse>> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;

    let account = state.account_service.update(claims.sub, id, req).await?;

    Ok(Json(account.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;

    state.account_service.delete(claims.sub, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<AuthQuery>,
    headers: HeaderMap,
    Json(req): Json<SetDefaultAccountRequest>,
) -> AppResult<Json<AccountResponse>> {
    let claims = state
        .auth_service
        .verify_token(&extract_token_from_request(
            query.as_ref().and_then(|q| q.token.as_ref()),
            &headers,
        )?)
        .await?;

    let account = state
        .account_service
        .set_default(claims.sub, req.id)
        .await?;

    Ok(Json(account.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::models::AccountResponse;

    #[test]
    fn account_response_serialization() {
        let response = AccountResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Checking".into(),
            icon: Some("Wallet".into()),
            color: Some("#6366f1".into()),
            is_default: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Checking"));
        assert!(json.contains("is_default"));
    }

    #[test]
    fn set_default_account_request_deserializes() {
        let id = Uuid::new_v4();
        let req: SetDefaultAccountRequest =
            serde_json::from_str(&format!(r#"{{"id":"{id}"}}"#)).unwrap();
        assert_eq!(req.id, id);
    }

    #[test]
    fn create_account_request_deserializes() {
        let req: CreateAccountRequest =
            serde_json::from_str(r#"{"name":"Broker","icon":"Wallet"}"#).unwrap();
        assert_eq!(req.name, "Broker");
        assert_eq!(req.icon.as_deref(), Some("Wallet"));
    }
}
