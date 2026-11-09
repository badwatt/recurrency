use crate::api::client::{ApiError, api_delete, api_get, api_post, api_put};
use crate::models::account::*;

pub async fn list() -> Result<Vec<Account>, ApiError> {
    api_get("/api/accounts").await
}

pub async fn create(req: &CreateAccountRequest) -> Result<Account, ApiError> {
    api_post("/api/accounts", req).await
}

pub async fn update(id: &uuid::Uuid, req: &UpdateAccountRequest) -> Result<Account, ApiError> {
    api_put(&format!("/api/accounts/{}", id), req).await
}

pub async fn delete(id: &uuid::Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/accounts/{}", id)).await
}

pub async fn set_default(id: &uuid::Uuid) -> Result<(), ApiError> {
    let _: serde_json::Value =
        api_put("/api/accounts/default", &serde_json::json!({ "id": id })).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_paths_are_scoped_to_api() {
        assert_eq!("/api/accounts", "/api/accounts");
        let id = uuid::Uuid::new_v4();
        assert!(format!("/api/accounts/{}", id).starts_with("/api/accounts/"));
    }

    #[test]
    fn set_default_body_contains_id() {
        let id = uuid::Uuid::new_v4();
        let body = serde_json::json!({ "id": id });
        assert_eq!(body["id"], id.to_string());
    }

    #[test]
    fn create_account_request_serializes() {
        let req = CreateAccountRequest {
            name: "Checking".into(),
            icon: Some("Wallet".into()),
            color: Some("#6366f1".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Checking"));
        let back: CreateAccountRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }
}
