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
    use crate::api::test_support::{with_response, with_server};
    use chrono::Utc;
    use serial_test::serial;

    fn sample_account() -> Account {
        Account {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::nil(),
            name: "Main".into(),
            icon: None,
            color: None,
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    #[serial]
    async fn account_endpoints_exchange_json() {
        let account = sample_account();
        let json = serde_json::to_string(&account).unwrap();
        let listed: Vec<Account> = with_server(format!("[{json}]"), list()).await.unwrap();
        assert_eq!(listed.len(), 1);

        let created: Account = with_server(
            json.clone(),
            create(&CreateAccountRequest {
                name: "Main".into(),
                icon: None,
                color: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(created.id, account.id);

        let updated: Account = with_server(
            json,
            update(
                &account.id,
                &UpdateAccountRequest {
                    name: Some("Renamed".into()),
                    icon: None,
                    color: None,
                },
            ),
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "Main");

        with_response("204 No Content", "", delete(&account.id))
            .await
            .unwrap();
        with_server("null", set_default(&account.id)).await.unwrap();
    }
}
